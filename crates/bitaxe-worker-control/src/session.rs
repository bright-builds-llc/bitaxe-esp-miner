use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    LeaseAuthorizationError, LeaseDeadlines, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal,
};

/// Deployment-owned verification over the complete parsed authorization input.
pub trait LeaseAuthorizationVerifier {
    /// Fingerprints validated persistent replay marks through their existing owner.
    fn authorization_high_water_fingerprint(
        &self,
    ) -> Result<Option<crate::StateFingerprint>, LeaseAuthorizationError> {
        Ok(None)
    }

    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError>;
    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError>;
    fn verify_start(
        &mut self,
        grant: &WorkerLeaseGrant,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError>;
    fn verify_renewal(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        challenge_id: &str,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError>;
}

/// Sole mining-owner adapter; implementations must keep supplied credentials volatile.
pub trait WorkerSession {
    /// Current authenticated native observation; no endpoint discovery or effects.
    fn noise_observation(
        &self,
    ) -> Result<Option<crate::noise::NoiseObservation>, WorkerSessionError> {
        Ok(None)
    }
    /// Reads the retained boot-local job, including after logical-session recovery.
    fn noise_status(&self) -> Result<Option<crate::noise::NoiseStatus>, WorkerSessionError> {
        Ok(None)
    }
    /// Consumes the boot slot and native idle fence before reply preparation.
    fn noise_admit(
        &mut self,
        _input: crate::noise::NoiseStart,
    ) -> Result<crate::noise::NoiseStatus, WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }
    /// Starts only after true response delivery and a fresh logical/native recheck.
    fn noise_dispatch(&mut self) -> Result<(), WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }
    /// Cancellation is nonblocking and never establishes worker completion.
    fn noise_cancel(
        &mut self,
        _detail: crate::noise::NoiseDetail,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }
    /// Supervises deadlines and only joins an actually completed worker.
    fn noise_poll(&mut self) {}
    /// Remains true through failed cleanup until native resources actually release.
    fn noise_busy(&self) -> bool {
        false
    }

    /// Supplies only an exact idle native restart binding; unsupported adapters reject by default.
    fn qualification_restart_context(
        &self,
    ) -> Result<Option<crate::QualificationRestartContext>, WorkerSessionError> {
        Ok(None)
    }

    /// Rechecks native ownership/readiness and immediately restarts after confirmed reply completion.
    fn qualification_restart(
        &mut self,
        _context: crate::QualificationRestartContext,
        _expires_at_ms: u64,
    ) -> Result<(), WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }

    /// Returns only an allowlisted nonsecret-settings fingerprint and boot preference.
    fn settings_preservation(
        &self,
    ) -> Result<Option<crate::SettingsPreservation>, WorkerSessionError> {
        Ok(None)
    }

    /// Proves full cooling without ASIC or voltage effects or a mining reservation.
    fn qualify_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }

    /// Restores only this session's fan-only effects after fresh cooling proof.
    fn restore_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }

    /// Reads campaign-bound reservation facts after fresh possession; never mutates the ledger.
    fn acceptance_budget_review(
        &self,
        _expected_campaign: &str,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        Ok(None)
    }

    fn qualification_attempt_review(
        &self,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        Ok(None)
    }

    /// Explicit read-only export of boot-local serial observations after fresh possession.
    fn serial_trace_review(
        &self,
    ) -> Result<Option<crate::serial::trace::SerialTraceSnapshot>, WorkerSessionError> {
        Ok(None)
    }

    /// Nonblocking diagnostic witness after successful Controller probe preparation, never delivery.
    fn telemetry_cadence_probe_prepared(&self, _request_bytes: usize, _response_bytes: usize) {}

    /// Arms one bounded diagnostic phase without granting work or extending liveness.
    fn telemetry_cadence_arm(
        &mut self,
        _phase: crate::cadence::CadencePhase,
    ) -> Result<Option<crate::cadence::CadenceArmReceipt>, WorkerSessionError> {
        Ok(None)
    }

    /// Copies immutable phase summaries after fresh idle possession.
    fn telemetry_cadence_review(
        &self,
    ) -> Result<Option<crate::cadence::CadenceSnapshot>, WorkerSessionError> {
        Ok(None)
    }

    /// Private, freshly observed station endpoint; never part of public status evidence.
    fn telemetry_cadence_endpoint(
        &self,
    ) -> Result<Option<crate::cadence::CadenceEndpoint>, WorkerSessionError> {
        Ok(None)
    }

    /// Returns only bounded, non-secret qualification observations; never authority.
    fn status_evidence(&self) -> Option<serde_json::Value> {
        None
    }

    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError>;
    fn renew(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError>;
    fn safe_stop(&mut self, reason: RestorationReason) -> Result<(), WorkerSessionError>;
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WorkerSessionError {
    #[error("Worker session rejected the request")]
    Rejected,
    #[error("Worker session safe stop failed")]
    SafeStopFailed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestorationReason {
    Paused,
    Cancelled,
    LeaseExpired,
    LostContinuity,
    MonotonicReset,
    Reboot,
    ChallengeSatisfied,
    ChallengeExpired,
    TabClosed,
    ConnectivityLost,
    ControlFailed,
}

impl RestorationReason {
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::Paused => "paused",
            Self::Cancelled => "cancelled",
            Self::LeaseExpired => "lease_expired",
            Self::LostContinuity => "lost_continuity",
            Self::MonotonicReset => "monotonic_reset",
            Self::Reboot => "reboot",
            Self::ChallengeSatisfied => "challenge_satisfied",
            Self::ChallengeExpired => "challenge_expired",
            Self::TabClosed => "tab_closed",
            Self::ConnectivityLost => "connectivity_lost",
            Self::ControlFailed => "control_failed",
        }
    }
}
