use super::*;

fn command_effects_terminal() -> String {
    campaign_marker(CampaignMarkerFixture {
        stage: "command-effects",
        lease_id: serde_json::Value::Null,
        state: "consumed",
        profile: "conservative",
        active_ms: 600_000,
        submit_outcome: "accepted",
        terminal_reason: "campaign_lease_consumed",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "safe_stopped",
        safe_stop: "confirmed",
    })
}

#[test]
fn command_effects_requires_the_typed_network_quorum_and_safe_stop() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::CommandEffects,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[command_effects_terminal()]));

    // Act
    run_mining_campaign(&command, &environment).expect("command effects campaign");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "command_effects_complete");
    assert_eq!(result["network_status"], "accepted");
    assert_eq!(result["safe_stop"], "confirmed");
    assert_eq!(
        environment.campaign_observations(),
        vec![(MiningCampaignStage::CommandEffects, 780)]
    );
    let network: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            command
                .evidence_dir
                .join("campaign-network.private.json")
                .as_std_path(),
        )
        .expect("network evidence"),
    )
    .expect("network JSON");
    assert_eq!(network["command_effects"]["pause_request_count"], 1);
    assert_eq!(network["command_effects"]["identify_request_count"], 2);
}
