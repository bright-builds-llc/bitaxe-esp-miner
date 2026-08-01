use super::*;

pub(super) fn assess_soak_terminal(
    marker: &CampaignStatusMarker,
    duration_seconds: u64,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    assess_mining_terminal(marker)?;
    if !marker.pool_config_persisted {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::PoolPersistenceUnconfirmed,
        ));
    }
    if marker.submit_outcome != SubmitOutcomeMarker::Accepted
        || marker.qualified_candidate_count == 0
        || marker.accepted_share_count == 0
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ));
    }
    if marker.active_ms < duration_seconds.saturating_mul(1_000) {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SoakDurationShort,
        ));
    }
    Ok(CampaignTerminalCategory::SoakDurationComplete)
}
