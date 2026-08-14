use super::*;

fn terminal_marker(completed: bool) -> String {
    let marker = campaign_marker(CampaignMarkerFixture {
        stage: "job-transition",
        lease_id: serde_json::Value::Null,
        state: "consumed",
        profile: "conservative",
        active_ms: 1_800_000,
        submit_outcome: "none",
        terminal_reason: "campaign_lease_consumed",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "safe_stopped",
        safe_stop: "confirmed",
    });
    let payload = marker
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("fixture marker prefix");
    let mut value: serde_json::Value = serde_json::from_str(payload).expect("fixture JSON");
    if completed {
        value["job_transition"] = serde_json::json!({
            "pool_notify_count": 3,
            "clean_jobs_notify_count": 2,
            "previous_block_change_count": 1,
            "new_block_generation_count": 1,
            "replacement_dispatch_count": 1,
            "post_transition_correlated_result_count": 1,
            "completed_transition_count": 1,
            "stale_generation_result_discard_count": 0,
            "stale_generation_submit_count": 0,
            "reconnect_count": 0,
            "latest_state": "replacement_result_correlated",
        });
        value["asic_bridge"]["generation_invalidation_count"] = serde_json::json!(2);
        value["asic_bridge"]["poll_request_count"] = serde_json::json!(10);
        value["asic_bridge"]["idle_completion_count"] = serde_json::json!(8);
        value["asic_bridge"]["nonce_completion_count"] = serde_json::json!(2);
        value["asic_bridge"]["post_transition_poll_request_count"] = serde_json::json!(2);
        value["asic_bridge"]["post_transition_completion_count"] = serde_json::json!(2);
        value["asic_bridge"]["post_transition_nonce_emission_count"] = serde_json::json!(1);
        value["asic_bridge"]["post_transition_correlation_count"] = serde_json::json!(1);
        value["asic_bridge"]["changed_block_to_replacement_dispatch_ms"] = serde_json::json!(1);
        value["asic_bridge"]["changed_block_to_first_poll_ms"] = serde_json::json!(2);
        value["asic_bridge"]["changed_block_to_first_nonce_ms"] = serde_json::json!(3);
        value["asic_bridge"]["changed_block_to_first_correlation_ms"] = serde_json::json!(4);
        value["asic_bridge"]["final_poll_state"] = serde_json::json!("invalidated");
    }
    format!("{CAMPAIGN_MARKER_PREFIX}{value}")
}

#[test]
fn campaign_uses_exact_duration_and_accepts_complete_chain() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::JobTransition,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment =
        FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[terminal_marker(true)]));

    // Act
    run_mining_campaign(&command, &environment).expect("complete transition campaign");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "job_transition_complete");
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["job_transition"]["completed_transition_count"], 1);
    assert_eq!(
        result["asic_bridge"]["post_transition_poll_request_count"],
        2
    );
    let diagnostics: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            command
                .evidence_dir
                .join("campaign-mining-diagnostics.private.json")
                .as_std_path(),
        )
        .expect("mining diagnostics"),
    )
    .expect("mining diagnostics JSON");
    assert_eq!(diagnostics["schema"], "mining-campaign-asic-diagnostics-v1");
    assert_eq!(
        diagnostics["asic_bridge"]["post_transition_correlation_count"],
        1
    );
    assert_eq!(
        environment.campaign_observations(),
        vec![(
            MiningCampaignStage::JobTransition,
            CampaignCaptureLimit::Bounded(1_980)
        )]
    );
    let csv = environment
        .written_files()
        .iter()
        .find(|(path, _)| path.file_name() == Some("campaign-nvs.csv"))
        .map(|(_, contents)| contents.clone())
        .expect("campaign CSV");
    assert!(csv.contains("campstage,data,string,job-transition"));
    assert!(csv.contains("campdurms,data,u64,1800000"));
}

#[test]
fn nonexact_duration_is_rejected_before_device_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let mut command = campaign_command(
        &dir,
        MiningCampaignStage::JobTransition,
        Some(MiningCampaignProfile::Conservative),
    );
    command.duration_seconds = 1_799;
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("nonexact transition duration must fail admission");

    // Assert
    assert!(format!("{error:#}").contains("category=admission_failed"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn clean_full_duration_without_transition_is_inconclusive() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::JobTransition,
        Some(MiningCampaignProfile::Conservative),
    );
    let environment =
        FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[terminal_marker(false)]));

    // Act
    run_mining_campaign(&command, &environment).expect("clean no-transition campaign");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "job_transition_not_observed");
    assert_eq!(result["status"], "inconclusive");
}

#[test]
fn any_rejected_share_fails_the_campaign() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::JobTransition,
        Some(MiningCampaignProfile::Conservative),
    );
    let marker = terminal_marker(true);
    let payload = marker
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("fixture marker prefix");
    let mut value: serde_json::Value = serde_json::from_str(payload).expect("fixture JSON");
    value["rejected_share_count"] = serde_json::json!(1);
    let rejected = format!("{CAMPAIGN_MARKER_PREFIX}{value}");
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[rejected]));

    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("rejected share must fail transition campaign");

    // Assert
    assert!(format!("{error:#}").contains("category=rejected_share_observed"));
}
