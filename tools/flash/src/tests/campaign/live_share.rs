use super::*;

#[test]
fn rejected_submit_is_a_completed_campaign() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::LiveShare,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[live_terminal("rejected")]));

    // Act
    run_campaign_observation_fixture(&command, &environment)
        .expect("rejected submit is terminal evidence");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "submit_response_observed");
    assert_eq!(result["submit_outcome"], "rejected");
    assert_eq!(
        environment.campaign_observations(),
        vec![(
            MiningCampaignStage::LiveShare,
            CampaignCaptureLimit::Bounded(780)
        )]
    );
}
