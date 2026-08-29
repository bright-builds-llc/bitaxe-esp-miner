use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    LeaseAuthorizationError, LeaseDeadlines, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal,
};

/// Deployment-owned verification over the complete parsed authorization input.
pub trait LeaseAuthorizationVerifier {
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
