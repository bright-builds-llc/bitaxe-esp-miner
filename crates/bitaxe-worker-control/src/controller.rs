mod wire;

use std::fmt;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroize;

use self::wire::{classify_json_error, ControllerRequest, FrameDiscriminator, RestorePayload};

use crate::codec::{base64_url, canonical_json, digest_text, strict_json_frame};
use crate::possession::{PossessionError, PossessionRequest};
use crate::session::{LeaseAuthorizationVerifier, RestorationReason, WorkerSession};
use crate::{
    DeviceIdentity, LeaseDeadlines, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal,
};

const PROTOCOL_VERSION: &str = "bwg-worker-controller/0.3";
const MAXIMUM_SEEN_NONCES: usize = 256;

#[derive(Debug, Error)]
pub enum WorkerControlError {
    #[error("Worker control frame is invalid")]
    InvalidFrame,
    #[error("Worker control request is invalid")]
    InvalidRequest,
    #[error("Worker possession admission is required")]
    AdmissionRequired,
    #[error("Worker possession proof is invalid")]
    InvalidProof,
    #[error("Work Lease authentication failed")]
    AuthenticationFailed,
    #[error("Work Lease state is invalid")]
    InvalidTransition,
    #[error("Worker monotonic continuity was lost")]
    MonotonicReset,
    #[error("Worker session effect failed")]
    SessionFailed,
    #[error("Worker restoration is pending")]
    RestorationPending,
    #[error("Worker response confirmation is stale")]
    StaleResponse,
    #[error("Worker response encoding failed")]
    Encoding,
}

impl WorkerControlError {
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::InvalidFrame => "invalid_frame",
            Self::InvalidRequest => "invalid_request",
            Self::AdmissionRequired => "admission_required",
            Self::InvalidProof => "invalid_proof",
            Self::AuthenticationFailed => "authentication_failed",
            Self::InvalidTransition => "invalid_transition",
            Self::MonotonicReset => "monotonic_reset",
            Self::SessionFailed => "session_failed",
            Self::RestorationPending => "restoration_pending",
            Self::StaleResponse => "stale_response",
            Self::Encoding => "encoding_failed",
        }
    }
}

impl From<PossessionError> for WorkerControlError {
    fn from(error: PossessionError) -> Self {
        match error {
            PossessionError::InvalidFrame => Self::InvalidFrame,
            PossessionError::InvalidRequest => Self::InvalidRequest,
            PossessionError::Encoding(_) => Self::Encoding,
        }
    }
}

#[derive(Clone, Debug)]
enum PreparedEffect {
    Admit {
        generation: u64,
        token: u64,
        established_at_monotonic_milliseconds: u64,
        control_session_binding_sha256: String,
    },
}

/// Bounded response plus a send-confirmation effect; Debug never includes frame bytes.
pub struct PreparedResponse {
    frame: Vec<u8>,
    maybe_effect: Option<PreparedEffect>,
}

impl PreparedResponse {
    #[must_use]
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
}

impl fmt::Debug for PreparedResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedResponse")
            .field("frame", &"[redacted]")
            .field("has_effect", &self.maybe_effect.is_some())
            .finish()
    }
}

impl Drop for PreparedResponse {
    fn drop(&mut self) {
        self.frame.zeroize();
    }
}

enum RestorationState {
    NotRequired,
    Pending,
    Confirmed(RestorationReason),
}

struct ActiveLease {
    grant: WorkerLeaseGrant,
    deadlines: LeaseDeadlines,
}

struct EnumerationAdmission {
    generation: u64,
    established_at_monotonic_milliseconds: u64,
    context: WorkerLeaseAuthorizationContext,
}

/// Pure Worker-control owner for one boot lifetime and one current USB enumeration.
pub struct WorkerControl<V, S> {
    identity: DeviceIdentity,
    verifier: V,
    session: S,
    capability: Value,
    capability_sha256: String,
    descriptor_sha256: String,
    generation: u64,
    maybe_admission: Option<EnumerationAdmission>,
    maybe_pending_admission_token: Option<u64>,
    next_response_token: u64,
    seen_nonce_digests: Vec<[u8; 32]>,
    maybe_active: Option<ActiveLease>,
    maybe_cleanup_reason: Option<RestorationReason>,
    restoration: RestorationState,
    maybe_last_monotonic_milliseconds: Option<u64>,
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub fn new(
        identity: DeviceIdentity,
        verifier: V,
        session: S,
        capability: Value,
        descriptor_sha256: &str,
    ) -> Result<Self, WorkerControlError> {
        if !digest_text(descriptor_sha256) || !capability.is_object() {
            return Err(WorkerControlError::InvalidRequest);
        }
        let capability_sha256 = base64_url(Sha256::digest(
            canonical_json(&capability)
                .map_err(|_| WorkerControlError::Encoding)?
                .as_bytes(),
        ));
        Ok(Self {
            identity,
            verifier,
            session,
            capability,
            capability_sha256,
            descriptor_sha256: descriptor_sha256.to_owned(),
            generation: 0,
            maybe_admission: None,
            maybe_pending_admission_token: None,
            next_response_token: 0,
            seen_nonce_digests: Vec::new(),
            maybe_active: None,
            maybe_cleanup_reason: None,
            restoration: RestorationState::NotRequired,
            maybe_last_monotonic_milliseconds: None,
        })
    }

    pub fn begin_enumeration(&mut self) {
        self.generation = self.generation.saturating_add(1);
        self.maybe_admission = None;
        self.maybe_pending_admission_token = None;
        self.seen_nonce_digests.clear();
    }

    #[must_use]
    pub fn capability_sha256(&self) -> &str {
        &self.capability_sha256
    }

    #[must_use]
    pub const fn session(&self) -> &S {
        &self.session
    }

    #[must_use]
    pub const fn has_active_lease(&self) -> bool {
        self.maybe_active.is_some()
    }

    pub fn prepare_frame(
        &mut self,
        frame: &[u8],
        monotonic_milliseconds: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        self.enforce_clock(monotonic_milliseconds)?;
        let json = strict_json_frame(frame).map_err(|_| WorkerControlError::InvalidFrame)?;
        let discriminator: FrameDiscriminator =
            serde_json::from_str(json).map_err(classify_json_error)?;
        if discriminator.profile.is_some() {
            return self.prepare_possession(frame, monotonic_milliseconds);
        }
        let request: ControllerRequest =
            serde_json::from_str(json).map_err(|_| WorkerControlError::InvalidRequest)?;
        request.validate()?;
        self.prepare_controller(request, monotonic_milliseconds)
    }

    pub fn confirm_sent(
        &mut self,
        mut response: PreparedResponse,
    ) -> Result<(), WorkerControlError> {
        let Some(PreparedEffect::Admit {
            generation,
            token,
            established_at_monotonic_milliseconds,
            control_session_binding_sha256,
        }) = response.maybe_effect.take()
        else {
            return Ok(());
        };
        if generation != self.generation || self.maybe_pending_admission_token != Some(token) {
            return Err(WorkerControlError::StaleResponse);
        }
        self.maybe_pending_admission_token = None;
        self.maybe_admission = Some(EnumerationAdmission {
            generation,
            established_at_monotonic_milliseconds,
            context: WorkerLeaseAuthorizationContext::parse(&control_session_binding_sha256)
                .map_err(|_| WorkerControlError::InvalidProof)?,
        });
        Ok(())
    }

    pub fn disconnect(&mut self, monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::ConnectivityLost, monotonic_milliseconds);
        self.begin_enumeration();
        result
    }

    pub fn reboot(&mut self, monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::Reboot, monotonic_milliseconds);
        self.begin_enumeration();
        result
    }

    pub fn control_failed(
        &mut self,
        monotonic_milliseconds: u64,
    ) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::ControlFailed, monotonic_milliseconds);
        self.begin_enumeration();
        result
    }

    pub fn tick(&mut self, monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        self.enforce_clock(monotonic_milliseconds)
    }

    fn prepare_possession(
        &mut self,
        frame: &[u8],
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let request = PossessionRequest::from_frame(frame)?;
        let nonce_digest: [u8; 32] = Sha256::digest(request.nonce().as_bytes()).into();
        if !request.matches_bindings(&self.capability_sha256, &self.descriptor_sha256)
            || self.seen_nonce_digests.contains(&nonce_digest)
            || self.seen_nonce_digests.len() >= MAXIMUM_SEEN_NONCES
            || self.maybe_pending_admission_token.is_some()
            || self.maybe_active.is_some()
        {
            return Err(WorkerControlError::InvalidProof);
        }
        self.seen_nonce_digests.push(nonce_digest);
        let response = self.identity.prove(&request)?;
        let control_session_binding_sha256 = request.control_session_binding(&response)?;
        self.next_response_token = self.next_response_token.saturating_add(1);
        let token = self.next_response_token;
        self.maybe_pending_admission_token = Some(token);
        Ok(PreparedResponse {
            frame: response.to_frame()?,
            maybe_effect: Some(PreparedEffect::Admit {
                generation: self.generation,
                token,
                established_at_monotonic_milliseconds: now,
                control_session_binding_sha256,
            }),
        })
    }

    fn prepare_controller(
        &mut self,
        request: ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let result = match request.command.as_str() {
            "discover" => {
                request.require_no_payload()?;
                self.capability.clone()
            }
            "start_lease" => self.start(request.required_payload()?, now)?,
            "renew_lease" => self.renew(request.required_payload()?, now)?,
            "status" => {
                request.require_no_payload()?;
                self.status(now)?
            }
            "pause" => {
                request.require_no_payload()?;
                self.safe_stop(RestorationReason::Paused, now)?;
                self.status(now)?
            }
            "cancel" => {
                request.require_no_payload()?;
                self.safe_stop(RestorationReason::Cancelled, now)?;
                self.status(now)?
            }
            "restore" => {
                let payload: RestorePayload = request.required_payload()?;
                self.safe_stop(payload.reason, now)?;
                self.status(now)?
            }
            _ => return Err(WorkerControlError::InvalidRequest),
        };
        response(&request.request_id, result)
    }

    fn start(&mut self, grant: WorkerLeaseGrant, now: u64) -> Result<Value, WorkerControlError> {
        let context = self.required_start_context(now)?.clone();
        if self.maybe_active.is_some()
            || self.maybe_cleanup_reason.is_some()
            || matches!(self.restoration, RestorationState::Pending)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        if !grant.validate() {
            return Err(WorkerControlError::InvalidRequest);
        }
        self.verifier
            .verify_start(&grant, &context)
            .map_err(|_| WorkerControlError::AuthenticationFailed)?;
        let deadlines = LeaseDeadlines::from_window(
            now,
            grant.duration_milliseconds(),
            grant.renew_after_milliseconds(),
        )
        .ok_or(WorkerControlError::InvalidRequest)?;
        self.maybe_active = Some(ActiveLease { grant, deadlines });
        self.restoration = RestorationState::Pending;
        let start_result = self
            .maybe_active
            .as_ref()
            .ok_or(WorkerControlError::InvalidTransition)
            .and_then(|active| {
                self.session
                    .start(&active.grant, active.deadlines)
                    .map_err(|_| WorkerControlError::SessionFailed)
            });
        if start_result.is_err() {
            self.safe_stop(RestorationReason::ControlFailed, now)?;
            return Err(WorkerControlError::SessionFailed);
        }
        self.status(now)
    }

    fn renew(
        &mut self,
        renewal: WorkerLeaseRenewal,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        let context = self.required_active_context()?.clone();
        if !renewal.validate() {
            return Err(WorkerControlError::InvalidRequest);
        }
        let active = self
            .maybe_active
            .as_ref()
            .ok_or(WorkerControlError::InvalidTransition)?;
        let challenge_id = active.grant.challenge_id().to_owned();
        let authentication_failed = renewal.lease_id() != active.grant.lease_id()
            || self
                .verifier
                .verify_renewal(&renewal, &challenge_id, &context)
                .is_err();
        if authentication_failed {
            self.safe_stop(RestorationReason::ControlFailed, now)?;
            return Err(WorkerControlError::AuthenticationFailed);
        }
        let deadlines = LeaseDeadlines::from_window(
            now,
            renewal.duration_milliseconds(),
            renewal.renew_after_milliseconds(),
        )
        .ok_or(WorkerControlError::InvalidRequest)?;
        if self.session.renew(&renewal, deadlines).is_err() {
            self.safe_stop(RestorationReason::ControlFailed, now)?;
            return Err(WorkerControlError::SessionFailed);
        }
        let active = self
            .maybe_active
            .as_mut()
            .ok_or(WorkerControlError::InvalidTransition)?;
        active.deadlines = deadlines;
        self.status(now)
    }

    fn safe_stop(&mut self, reason: RestorationReason, now: u64) -> Result<(), WorkerControlError> {
        self.maybe_admission = None;
        self.maybe_pending_admission_token = None;
        self.maybe_cleanup_reason = Some(reason);
        self.restoration = RestorationState::Pending;
        if self.maybe_active.is_some() {
            self.session
                .safe_stop(reason)
                .map_err(|_| WorkerControlError::SessionFailed)?;
        }
        drop(self.maybe_active.take());
        self.maybe_cleanup_reason = None;
        self.restoration = RestorationState::Confirmed(reason);
        self.maybe_last_monotonic_milliseconds = Some(now);
        Ok(())
    }

    fn enforce_clock(&mut self, now: u64) -> Result<(), WorkerControlError> {
        if let Some(reason) = self.maybe_cleanup_reason {
            self.safe_stop(reason, now)?;
        }
        if self
            .maybe_last_monotonic_milliseconds
            .is_some_and(|last| now < last)
        {
            self.safe_stop(RestorationReason::MonotonicReset, now)?;
            return Err(WorkerControlError::MonotonicReset);
        }
        self.maybe_last_monotonic_milliseconds = Some(now);
        if self
            .maybe_active
            .as_ref()
            .is_some_and(|active| now >= active.deadlines.expires_at_monotonic_milliseconds())
        {
            self.safe_stop(RestorationReason::LeaseExpired, now)?;
        }
        Ok(())
    }

    fn required_start_context(
        &self,
        now: u64,
    ) -> Result<&WorkerLeaseAuthorizationContext, WorkerControlError> {
        let admission = self
            .maybe_admission
            .as_ref()
            .filter(|admission| admission.generation == self.generation)
            .ok_or(WorkerControlError::AdmissionRequired)?;
        if now.saturating_sub(admission.established_at_monotonic_milliseconds) >= 60_000 {
            return Err(WorkerControlError::AdmissionRequired);
        }
        Ok(&admission.context)
    }

    fn required_active_context(
        &self,
    ) -> Result<&WorkerLeaseAuthorizationContext, WorkerControlError> {
        self.maybe_admission
            .as_ref()
            .filter(|admission| admission.generation == self.generation)
            .map(|admission| &admission.context)
            .ok_or(WorkerControlError::AdmissionRequired)
    }

    fn status(&self, now: u64) -> Result<Value, WorkerControlError> {
        if let Some(active) = self.maybe_active.as_ref() {
            return Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "state": "mining",
                "monotonicMilliseconds": now,
                "lease": {
                    "leaseId": active.grant.lease_id(),
                    "challengeId": active.grant.challenge_id(),
                    "renewAtMonotonicMilliseconds": active.deadlines.renew_at_monotonic_milliseconds(),
                    "expiresAtMonotonicMilliseconds": active.deadlines.expires_at_monotonic_milliseconds(),
                },
                "restoration": { "status": "pending" },
            }));
        }
        let restoration = match self.restoration {
            RestorationState::NotRequired => json!({ "status": "not_required" }),
            RestorationState::Confirmed(reason) => {
                json!({ "status": "confirmed", "reason": reason })
            }
            RestorationState::Pending => return Err(WorkerControlError::RestorationPending),
        };
        Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "state": "baseline",
            "monotonicMilliseconds": now,
            "restoration": restoration,
        }))
    }
}

impl<V, S> fmt::Debug for WorkerControl<V, S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerControl")
            .field("generation", &self.generation)
            .field("admitted", &self.maybe_admission.is_some())
            .field("active", &self.maybe_active.is_some())
            .field("private_material", &"[redacted]")
            .finish()
    }
}

fn response(request_id: &str, result: Value) -> Result<PreparedResponse, WorkerControlError> {
    let mut frame = serde_json::to_vec(&json!({
        "protocolVersion": PROTOCOL_VERSION,
        "requestId": request_id,
        "ok": true,
        "result": result,
    }))
    .map_err(|_| WorkerControlError::Encoding)?;
    frame.push(b'\n');
    Ok(PreparedResponse {
        frame,
        maybe_effect: None,
    })
}
