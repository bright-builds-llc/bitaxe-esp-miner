use super::*;

#[test]
fn hardware_preparation_failure_precedes_missing_pool_configuration() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::LiveShare,
        Some(MiningCampaignProfile::Conservative),
    );
    let terminal = campaign_marker_with_failure(
        CampaignMarkerFixture {
            stage: "live-share",
            lease_id: serde_json::Value::Null,
            state: "consumed",
            profile: "conservative",
            active_ms: 0,
            submit_outcome: "none",
            terminal_reason: "production_asic_unavailable",
            safety: "fresh",
            pool_config: "not_read",
            actuation: "safe_stopped",
            safe_stop: "confirmed",
        },
        serde_json::json!({
            "phase": "hardware_preparation",
            "step": "reset_and_detect_exactly_one_chip",
            "detail": "asic_actuation_failed",
            "rollback_step": "wait_for_fresh_temperature_at_or_below_45_c",
            "rollback_detail": "cooling_proof_timed_out",
        }),
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[terminal]));

    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("typed preparation failure must remain terminal");

    // Assert
    assert!(format!("{error:#}").contains("category=hardware_preparation_failed"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "hardware_preparation_failed");
    assert_eq!(
        result["campaign_failure"],
        serde_json::json!({
            "phase": "hardware_preparation",
            "step": "reset_and_detect_exactly_one_chip",
            "detail": "asic_actuation_failed",
            "rollback_step": "wait_for_fresh_temperature_at_or_below_45_c",
            "rollback_detail": "cooling_proof_timed_out",
        })
    );
}
