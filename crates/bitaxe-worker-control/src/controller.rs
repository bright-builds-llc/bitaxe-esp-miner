mod confirmation;
mod response_types;
use response_types::PreparedEffect;
pub use response_types::PreparedResponse;
mod cooling;
mod inspection;
mod noise;
mod probe;
mod restart;
mod status;
mod wire;
use status::response;

use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use self::wire::{classify_json_error, ControllerRequest, FrameDiscriminator, RestorePayload};

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
    restart_consumed: bool,
    maybe_restart_token: Option<u64>,
    seen_nonce_digests: Vec<[u8; 32]>,
    maybe_active: Option<ActiveLease>,
    effect_cleanup_required: bool,
    boot_restoration_clear_required: bool,
    maybe_boot_restoration_report_generation: Option<u64>,
    maybe_cleanup_reason: Option<RestorationReason>,
    restoration: RestorationState,
    maybe_last_monotonic_milliseconds: Option<u64>,
    maybe_noise_observation: Option<crate::noise::NoiseObservation>,
    maybe_noise_generation: Option<u64>,
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
            restart_consumed: false,
            maybe_restart_token: None,
            seen_nonce_digests: Vec::new(),
            maybe_active: None,
            effect_cleanup_required: false,
            boot_restoration_clear_required,
            maybe_boot_restoration_report_generation: None,
            maybe_cleanup_reason: None,
            restoration: initial_restoration
                .map_or(RestorationState::NotRequired, RestorationState::Confirmed),
            maybe_last_monotonic_milliseconds: None,
            maybe_noise_observation: None,
            maybe_noise_generation: None,
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
        self.session
            .noise_cancel(crate::noise::NoiseDetail::SessionReplaced)
            .map_err(|_| WorkerControlError::SessionFailed)?;
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
        self.maybe_noise_observation = None;
        self.authenticated_logical_session = false;
        self.maybe_serial_binding = None;
        self.generation = self.generation.saturating_add(1);
        self.maybe_admission = None;
        self.maybe_pending_admission_token = None;
        self.maybe_restart_token = None;
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
        self.session.noise_poll();
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
        let is_probe = request.command == "transport_probe";
        let prepared = self.prepare_controller(request, monotonic_milliseconds)?;
        if is_probe {
            self.session
                .telemetry_cadence_probe_prepared(json.len(), prepared.frame.len() - 1);
        }
        Ok(prepared)
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
        self.session.noise_poll();
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
        if self.session.noise_busy()
            && matches!(request.command.as_str(), "restore" | "pause" | "cancel")
        {
            if request.command == "restore" {
                let _: RestorePayload = request.required_payload()?;
            } else {
                request.require_no_payload()?;
            }
            self.session
                .noise_cancel(crate::noise::NoiseDetail::CancelRequested)
                .map_err(|_| WorkerControlError::SessionFailed)?;
            return Err(WorkerControlError::RestorationPending);
        }
        if matches!(
            request.command.as_str(),
            "noise_diagnostic_start" | "noise_diagnostic_status" | "noise_diagnostic_cancel"
        ) {
            return self.prepare_noise(&request, now);
        }
        if self.session.noise_busy()
            && matches!(
                request.command.as_str(),
                "start_lease"
                    | "qualification_restart"
                    | "qualification_cooling"
                    | "telemetry_cadence_arm"
            )
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        if request.command == "qualification_restart" {
            return self.prepare_qualification_restart(&request, now);
        }
        if request.command == "serial_trace_review" {
            return self.review_serial_trace(&request, now);
        }
        let mut result = match request.command.as_str() {
            "discover" => {
                request.require_no_payload()?;
                self.capability.clone()
            }
            "transport_probe" => {
                let payload = request.required_payload()?;
                self.required_start_context(now)?;
                if self.maybe_active.is_some() || self.effect_cleanup_required {
                    return Err(WorkerControlError::InvalidTransition);
                }
                probe::response(payload, &request.request_id)?
            }
            "qualification_attempt_review" => self.review_qualification_attempt(&request, now)?,
            "telemetry_cadence_arm" | "telemetry_cadence_review" | "telemetry_cadence_endpoint" => {
                self.telemetry_cadence(&request, now)?
            }
            "acceptance_budget_review" => self.review_acceptance_budget(&request, now)?,
            "qualification_cooling" => self.qualify_cooling(&request, now)?,
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
        if request.includes_status_evidence() {
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
        self.session
            .noise_cancel(match reason {
                RestorationReason::ConnectivityLost => crate::noise::NoiseDetail::SessionReplaced,
                RestorationReason::MonotonicReset => crate::noise::NoiseDetail::ClockDiscontinuity,
                _ => crate::noise::NoiseDetail::CancelRequested,
            })
            .map_err(|_| WorkerControlError::SessionFailed)?;
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
