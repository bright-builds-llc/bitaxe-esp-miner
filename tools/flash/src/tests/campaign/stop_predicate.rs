use super::*;

#[test]
fn valid_observation_marker_does_not_end_the_required_timeout_early() {
    // Arrange
    let marker = format!("{}\n", observation_marker("fresh"));
    let admission = CampaignAdmission {
        stage: MiningCampaignStage::Observation,
        maybe_profile: None,
        duration_seconds: 360,
        maybe_lease_id: None,
    };

    // Act
    let should_stop = campaign_serial_should_stop(marker.as_bytes(), admission);

    // Assert
    assert!(!should_stop);
}

#[test]
fn live_share_terminal_marker_requests_early_stop_after_safe_shutdown() {
    // Arrange
    let marker = format!("{}\n", live_terminal("accepted"));
    let admission = CampaignAdmission {
        stage: MiningCampaignStage::LiveShare,
        maybe_profile: Some(MiningCampaignProfile::Conservative),
        duration_seconds: 600,
        maybe_lease_id: Some(42),
    };

    // Act
    let should_stop = campaign_serial_should_stop(marker.as_bytes(), admission);

    // Assert
    assert!(should_stop);
}
