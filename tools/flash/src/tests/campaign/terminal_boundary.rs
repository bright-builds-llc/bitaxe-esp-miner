use super::*;

#[test]
fn live_terminal_boundary_ignores_a_partial_following_marker() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::LiveShare,
        Some(MiningCampaignProfile::Conservative),
    );
    let campaign_bytes = format!(
        "{}\n{}\nmining_campaign_status={{\"schema\":",
        runtime_attestation_log(),
        live_terminal("rejected"),
    )
    .into_bytes();
    let environment = FakeFlashEnvironment::default().with_campaign_bytes(campaign_bytes);

    // Act
    run_mining_campaign(&command, &environment)
        .expect("bytes after a complete terminal marker are outside the campaign boundary");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["terminal_category"], "submit_response_observed");
    assert_eq!(result["submit_outcome"], "rejected");
    assert_eq!(result["serial_outcome_detail"], "clean");
    assert_eq!(result["marker_count"], 1);
    let diagnostics = read_campaign_diagnostics(&command);
    assert_eq!(diagnostics["trailing_partial_count"], 0);
    assert!(diagnostics["post_terminal_ignored_byte_count"]
        .as_u64()
        .is_some_and(|count| count > 0));
}
