use super::readiness::{ReadinessHardwareStateMarker, ReadinessSafetySampleMarker};
use super::{
    assess_mining_terminal, ActuationMarker, CampaignFailurePhaseMarker, CampaignStateMarker,
    CampaignStatusMarker, CampaignTerminalReasonMarker, ResumablePauseSafeStopMarker,
    SafeStopMarker,
};
use crate::campaign::{
    CampaignAdmission, CampaignFailure, CampaignTerminalCategory, MiningCampaignStage,
};

pub(super) fn is_recoverable_command_effects_stopped_readiness(
    marker: &CampaignStatusMarker,
    admission: CampaignAdmission,
) -> bool {
    // Command effects keeps hardware stopped between commands. A pause or
    // resume transition can publish a stale sensor sample before the next
    // observation wakeup; only an exact stopped, non-failed state may recover.
    let stopped = admission.stage == MiningCampaignStage::CommandEffects
        && marker.campaign_state == CampaignStateMarker::Armed
        && marker.readiness_transition.campaign_state == CampaignStateMarker::Armed
        && marker.readiness_transition.hardware_state == ReadinessHardwareStateMarker::Stopped
        && marker.readiness_transition.safety_sample == ReadinessSafetySampleMarker::Stale
        && marker.actuation == ActuationMarker::Qualified
        && marker.safe_stop == SafeStopMarker::Pending
        && marker.failure.phase == CampaignFailurePhaseMarker::None;
    if !stopped {
        return false;
    }
    let paused = marker.terminal_reason == CampaignTerminalReasonMarker::OperatorPaused
        && marker.readiness_transition.current_blocker
            == CampaignTerminalReasonMarker::OperatorPaused
        && marker.resumable_pause_safe_stop == ResumablePauseSafeStopMarker::Confirmed;
    let resuming = marker.terminal_reason == CampaignTerminalReasonMarker::SafetyPrerequisitesStale
        && marker.readiness_transition.current_blocker
            == CampaignTerminalReasonMarker::SafetyPrerequisitesStale
        && marker.resumable_pause_safe_stop == ResumablePauseSafeStopMarker::NotRequired;
    paused || resuming
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
