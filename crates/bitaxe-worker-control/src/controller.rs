mod status;
mod wire;
use status::response;

use std::fmt;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroize;

use self::wire::{
    classify_json_error, ControllerRequest, FrameDiscriminator, ProbePayload, RestorePayload,
};

use crate::codec::{base64_url, canonical_json, digest_text, strict_json_frame};
use crate::possession::{FirmwareIdentity, PossessionError, PossessionRequest};
use crate::session::{LeaseAuthorizationVerifier, RestorationReason, WorkerSession};
use crate::{
    DeviceIdentity, LeaseDeadlines, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal,
};

const PROTOCOL_VERSION: &str = "bwg-worker-controller/0.4";
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
    #[error("Worker effect state persistence failed")]
    PersistenceFailed,
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
            Self::PersistenceFailed => "persistence_failed",
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
    BootRestorationReported {
        generation: u64,
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

struct LogicalSessionAdmission {
    generation: u64,
    established_at_monotonic_milliseconds: u64,
    context: WorkerLeaseAuthorizationContext,
}

/// Pure Worker-control owner for one boot lifetime and one current logical serial session.
pub struct WorkerControl<V, S> {
    identity: DeviceIdentity,
    verifier: V,
    session: S,
    capability: Value,
    capability_sha256: String,
    manifest_sha256: String,
    firmware_identity: FirmwareIdentity,
    maybe_serial_binding: Option<crate::serial::SerialSessionBinding>,
    generation: u64,
    maybe_admission: Option<LogicalSessionAdmission>,
    authenticated_logical_session: bool,
    maybe_pending_admission_token: Option<u64>,
    next_response_token: u64,
    seen_nonce_digests: Vec<[u8; 32]>,
    maybe_active: Option<ActiveLease>,
    effect_cleanup_required: bool,
    boot_restoration_clear_required: bool,
    maybe_boot_restoration_report_generation: Option<u64>,
    maybe_cleanup_reason: Option<RestorationReason>,
    restoration: RestorationState,
    maybe_last_monotonic_milliseconds: Option<u64>,
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub fn new(
        identity: DeviceIdentity,
        verifier: V,
        session: S,
        initial_restoration: Option<RestorationReason>,
        firmware_identity: FirmwareIdentity,
        capability: Value,
        manifest_sha256: &str,
    ) -> Result<Self, WorkerControlError> {
        if !digest_text(manifest_sha256) || !capability.is_object() {
            return Err(WorkerControlError::InvalidRequest);
        }
        let capability_sha256 = base64_url(Sha256::digest(
            canonical_json(&capability)
                .map_err(|_| WorkerControlError::Encoding)?
                .as_bytes(),
        ));
        let boot_restoration_clear_required = initial_restoration.is_some();
        Ok(Self {
            identity,
            verifier,
            session,
            capability,
            capability_sha256,
            manifest_sha256: manifest_sha256.to_owned(),
            firmware_identity,
            maybe_serial_binding: None,
            generation: 0,
            maybe_admission: None,
            authenticated_logical_session: false,
            maybe_pending_admission_token: None,
            next_response_token: 0,
            seen_nonce_digests: Vec::new(),
            maybe_active: None,
            effect_cleanup_required: false,
            boot_restoration_clear_required,
            maybe_boot_restoration_report_generation: None,
            maybe_cleanup_reason: None,
            restoration: initial_restoration
                .map_or(RestorationState::NotRequired, RestorationState::Confirmed),
            maybe_last_monotonic_milliseconds: None,
        })
    }

    /// Installs a fresh transport identity only after prior work has stopped.
    pub fn begin_serial_session(
        &mut self,
        binding: crate::serial::SerialSessionBinding,
    ) -> Result<(), WorkerControlError> {
        if self.maybe_active.is_some() || self.effect_cleanup_required {
            return Err(WorkerControlError::InvalidTransition);
        }
        self.invalidate_session();
        self.maybe_serial_binding = Some(binding);
        Ok(())
    }

    /// Configures the firmware effect adapter when a fresh logical session begins.
    pub fn session_mut(&mut self) -> &mut S {
        &mut self.session
    }

    #[must_use]
    pub fn is_admitted(&self) -> bool {
        self.maybe_admission.is_some()
    }

    fn invalidate_session(&mut self) {
        self.authenticated_logical_session = false;
        self.maybe_serial_binding = None;
        self.generation = self.generation.saturating_add(1);
        self.maybe_admission = None;
        self.maybe_pending_admission_token = None;
        self.maybe_boot_restoration_report_generation = None;
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
        self.acknowledge_boot_restoration()?;
        self.prepare_controller(request, monotonic_milliseconds)
    }

    pub fn confirm_sent(
        &mut self,
        mut response: PreparedResponse,
    ) -> Result<(), WorkerControlError> {
        let Some(effect) = response.maybe_effect.take() else {
            return Ok(());
        };
        match effect {
            PreparedEffect::Admit {
                generation,
                token,
                established_at_monotonic_milliseconds,
                control_session_binding_sha256,
            } => {
                if generation != self.generation
                    || self.maybe_pending_admission_token != Some(token)
                {
                    return Err(WorkerControlError::StaleResponse);
                }
                self.maybe_pending_admission_token = None;
                self.authenticated_logical_session = true;
                self.maybe_admission = Some(LogicalSessionAdmission {
                    generation,
                    established_at_monotonic_milliseconds,
                    context: WorkerLeaseAuthorizationContext::parse(
                        &control_session_binding_sha256,
                    )
                    .map_err(|_| WorkerControlError::InvalidProof)?,
                });
            }
            PreparedEffect::BootRestorationReported { generation } => {
                if generation != self.generation || !self.boot_restoration_clear_required {
                    return Err(WorkerControlError::StaleResponse);
                }
                self.maybe_boot_restoration_report_generation = Some(generation);
            }
        }
        Ok(())
    }

    pub fn disconnect(&mut self, monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::ConnectivityLost, monotonic_milliseconds);
        self.invalidate_session();
        result
    }

    pub fn reboot(&mut self, monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::Reboot, monotonic_milliseconds);
        self.invalidate_session();
        result
    }

    pub fn control_failed(
        &mut self,
        monotonic_milliseconds: u64,
    ) -> Result<(), WorkerControlError> {
        let result = self.safe_stop(RestorationReason::ControlFailed, monotonic_milliseconds);
        self.invalidate_session();
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
        let binding = self
            .maybe_serial_binding
            .as_ref()
            .ok_or(WorkerControlError::AdmissionRequired)?;
        let nonce_digest: [u8; 32] = Sha256::digest(request.nonce().as_bytes()).into();
        if !request.matches_bindings(&self.capability_sha256, &self.manifest_sha256, binding)
            || self.seen_nonce_digests.contains(&nonce_digest)
            || self.seen_nonce_digests.len() >= MAXIMUM_SEEN_NONCES
            || self.maybe_pending_admission_token.is_some()
            || self.maybe_active.is_some()
        {
            return Err(WorkerControlError::InvalidProof);
        }
        self.acknowledge_boot_restoration()?;
        self.seen_nonce_digests.push(nonce_digest);
        let response = self.identity.prove(
            &request,
            &self.firmware_identity.source_commit,
            &self.firmware_identity.app_elf_sha256,
        )?;
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
        let mut result = match request.command.as_str() {
            "discover" => {
                request.require_no_payload()?;
                self.capability.clone()
            }
            "transport_probe" => self.probe(request.required_payload()?, now)?,
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
        if request.command != "discover" && request.command != "transport_probe" {
            result = self.with_status_evidence(result)?;
        }
        let reports_boot_restoration = request.command == "status"
            && self.boot_restoration_clear_required
            && result
                .pointer("/restoration/reason")
                .and_then(Value::as_str)
                == Some("reboot");
        response(
            &request.request_id,
            result,
            reports_boot_restoration.then_some(PreparedEffect::BootRestorationReported {
                generation: self.generation,
            }),
        )
    }

    fn probe(&self, payload: ProbePayload, now: u64) -> Result<Value, WorkerControlError> {
        self.required_start_context(now)?;
        if self.maybe_active.is_some() || self.effect_cleanup_required {
            return Err(WorkerControlError::InvalidTransition);
        }
        if !payload.padding.bytes().all(|byte| byte == b'x') {
            return Err(WorkerControlError::InvalidRequest);
        }
        Ok(json!({"padding": payload.padding}))
    }

    fn start(&mut self, grant: WorkerLeaseGrant, now: u64) -> Result<Value, WorkerControlError> {
        let context = self.required_start_context(now)?.clone();
        if self.maybe_active.is_some()
            || self.maybe_cleanup_reason.is_some()
            || self.boot_restoration_clear_required
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
        self.verifier
            .mark_effect_pending()
            .map_err(|_| WorkerControlError::PersistenceFailed)?;
        self.effect_cleanup_required = true;
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
        if self.maybe_active.is_some() || self.effect_cleanup_required {
            self.session
                .safe_stop(reason)
                .map_err(|_| WorkerControlError::SessionFailed)?;
            self.verifier
                .clear_effect_pending()
                .map_err(|_| WorkerControlError::PersistenceFailed)?;
        }
        drop(self.maybe_active.take());
        self.effect_cleanup_required = false;
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

    fn acknowledge_boot_restoration(&mut self) -> Result<(), WorkerControlError> {
        if self.maybe_boot_restoration_report_generation != Some(self.generation) {
            return Ok(());
        }
        self.verifier
            .clear_effect_pending()
            .map_err(|_| WorkerControlError::PersistenceFailed)?;
        self.boot_restoration_clear_required = false;
        self.maybe_boot_restoration_report_generation = None;
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
