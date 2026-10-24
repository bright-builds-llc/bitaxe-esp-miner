//! Why a preparation or safe-stop step failed, and how the session classifies it.

use bitaxe_asic::bm1366::production::ProductionAsicBlocker;
use bitaxe_stratum::v1::production_session::HardwarePreparationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningActuationAdapterError {
    WorkerGenerationRevoked,
    SafetyObservationsUnavailable,
    SafetyOwnerUnavailable,
    SafetyQueueFull,
    SafetyReplyTimedOut,
    SafetyHardwareWriteFailed,
    FanRpmProofTimedOut,
    UnsupportedProfile,
    Asic(ProductionAsicBlocker),
    AsicPlanInvalid,
    CoolingProofTimedOut,
    CoolingProofRequired,
}

impl MiningActuationAdapterError {
    #[must_use]
    pub const fn hardware_preparation_failure(self) -> HardwarePreparationFailure {
        match self {
            Self::WorkerGenerationRevoked
            | Self::SafetyObservationsUnavailable
            | Self::SafetyOwnerUnavailable
            | Self::SafetyQueueFull
            | Self::UnsupportedProfile => HardwarePreparationFailure::Rejected,
            Self::SafetyReplyTimedOut | Self::FanRpmProofTimedOut | Self::CoolingProofTimedOut => {
                HardwarePreparationFailure::TimedOut
            }
            Self::SafetyHardwareWriteFailed
            | Self::Asic(_)
            | Self::AsicPlanInvalid
            | Self::CoolingProofRequired => HardwarePreparationFailure::DeviceFault,
        }
    }

    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::WorkerGenerationRevoked => "worker_generation_revoked",
            Self::SafetyObservationsUnavailable => "safety_observations_unavailable",
            Self::SafetyOwnerUnavailable => "safety_owner_unavailable",
            Self::SafetyQueueFull => "safety_queue_full",
            Self::SafetyReplyTimedOut => "safety_reply_timed_out",
            Self::SafetyHardwareWriteFailed => "safety_hardware_write_failed",
            Self::FanRpmProofTimedOut => "fan_rpm_proof_timed_out",
            Self::UnsupportedProfile => "unsupported_profile",
            Self::Asic(_) => "asic_actuation_failed",
            Self::AsicPlanInvalid => "asic_plan_invalid",
            Self::CoolingProofTimedOut => "cooling_proof_timed_out",
            Self::CoolingProofRequired => "cooling_proof_required",
        }
    }
}
