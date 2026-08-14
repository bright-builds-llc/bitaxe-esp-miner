use super::readiness::{ReadinessHardwareStateMarker, ReadinessSafetySampleMarker};
use super::{
    assess_mining_terminal, ActuationMarker, CampaignFailurePhaseMarker, CampaignStateMarker,
    CampaignStatusMarker, CampaignTerminalReasonMarker, ResumablePauseSafeStopMarker,
    SafeStopMarker,
};
use crate::campaign::{
    CampaignAdmission, CampaignFailure, CampaignTerminalCategory, MiningCampaignStage,
};

pub(super) fn is_recoverable_command_effects_resume_readiness(
    marker: &CampaignStatusMarker,
    admission: CampaignAdmission,
) -> bool {
    // Command effects keeps hardware stopped across human checkpoints. Resume
    // can synchronously publish a stale readiness sample before the next
    // observation wakeup; only that exact non-actuating state may recover.
    admission.stage == MiningCampaignStage::CommandEffects
        && marker.campaign_state == CampaignStateMarker::Armed
        && marker.terminal_reason == CampaignTerminalReasonMarker::SafetyPrerequisitesStale
        && marker.readiness_transition.current_blocker
            == CampaignTerminalReasonMarker::SafetyPrerequisitesStale
        && marker.readiness_transition.campaign_state == CampaignStateMarker::Armed
        && marker.readiness_transition.hardware_state == ReadinessHardwareStateMarker::Stopped
        && marker.readiness_transition.safety_sample == ReadinessSafetySampleMarker::Stale
        && marker.resumable_pause_safe_stop == ResumablePauseSafeStopMarker::NotRequired
        && marker.actuation == ActuationMarker::Qualified
        && marker.safe_stop == SafeStopMarker::Pending
        && marker.failure.phase == CampaignFailurePhaseMarker::None
}

pub(super) fn assess_command_effects_terminal(
    marker: &CampaignStatusMarker,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    assess_mining_terminal(marker)?;
    if marker.terminal_reason == CampaignTerminalReasonMarker::CampaignActivationTimedOut {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::CampaignActivationTimedOut,
        ));
    }
    Ok(CampaignTerminalCategory::CommandEffectsComplete)
}
