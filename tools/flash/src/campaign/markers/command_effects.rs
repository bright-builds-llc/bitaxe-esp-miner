use super::{assess_mining_terminal, CampaignStatusMarker, CampaignTerminalReasonMarker};
use crate::campaign::{CampaignFailure, CampaignTerminalCategory};

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
