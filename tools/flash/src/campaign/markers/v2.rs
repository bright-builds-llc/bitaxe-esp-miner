use super::*;

pub(super) fn v2_marker_not_supported(
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    Err(CampaignFailure::new(
        CampaignTerminalCategory::MarkerMissing,
    ))
}
