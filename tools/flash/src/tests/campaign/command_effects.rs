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

fn command_effects_resume_marker(safety: &str) -> String {
    let marker = campaign_marker(CampaignMarkerFixture {
        stage: "command-effects",
        lease_id: serde_json::json!(42),
        state: "armed",
        profile: "conservative",
        active_ms: 1_000,
        submit_outcome: "none",
        terminal_reason: if safety == "fresh" {
            "none"
        } else {
            "safety_prerequisites_stale"
        },
        safety,
        pool_config: "local_owner_supplied",
        actuation: "qualified",
        safe_stop: "pending",
    });
    let payload = marker
        .strip_prefix("mining_campaign_status=")
        .expect("campaign marker prefix");
    let mut value: serde_json::Value = serde_json::from_str(payload).expect("campaign marker JSON");
    value["readiness_transition"]["hardware_state"] = serde_json::json!("stopped");
    format!("mining_campaign_status={value}")
}

fn command_effects_active_stale_marker() -> String {
    campaign_marker(CampaignMarkerFixture {
        stage: "command-effects",
        lease_id: serde_json::json!(42),
        state: "active",
        profile: "conservative",
        active_ms: 1_000,
        submit_outcome: "none",
        terminal_reason: "safety_prerequisites_stale",
        safety: "stale",
        pool_config: "local_owner_supplied",
        actuation: "qualified",
        safe_stop: "pending",
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
        vec![(
            MiningCampaignStage::CommandEffects,
            CampaignCaptureLimit::OperatorGated
        )]
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
    assert_eq!(network["command_effects"]["identify_request_count"], 1);
}

#[test]
fn command_effects_preserves_typed_activation_timeout() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::CommandEffects,
        Some(MiningCampaignProfile::Conservative),
    );
    let terminal = campaign_marker(CampaignMarkerFixture {
        stage: "command-effects",
        lease_id: serde_json::Value::Null,
        state: "consumed",
        profile: "conservative",
        active_ms: 0,
        submit_outcome: "none",
        terminal_reason: "campaign_activation_timed_out",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "safe_stopped",
        safe_stop: "confirmed",
    });
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[terminal]));

    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("activation timeout must remain terminal");

    // Assert
    assert!(format!("{error:#}").contains("category=campaign_activation_timed_out"));
    assert_eq!(
        read_campaign_result(&command)["terminal_category"],
        "campaign_activation_timed_out"
    );
}

#[test]
fn command_effects_resume_waits_for_fresh_observation_recovery() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::CommandEffects,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[
        command_effects_resume_marker("stale"),
        command_effects_resume_marker("fresh"),
        command_effects_terminal(),
    ]));

    // Act
    let result = run_mining_campaign(&command, &environment);

    // Assert
    result.expect("fresh observation wakeup must recover a stale resume sample");
    assert_eq!(
        read_campaign_result(&command)["terminal_category"],
        "command_effects_complete"
    );
}

#[test]
fn command_effects_active_stale_marker_remains_terminal() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::CommandEffects,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[
        command_effects_active_stale_marker(),
        command_effects_terminal(),
    ]));

    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("active stale safety must remain terminal");

    // Assert
    assert!(format!("{error:#}").contains("category=safety_stale"));
    assert_eq!(
        read_campaign_result(&command)["terminal_category"],
        "safety_stale"
    );
}
