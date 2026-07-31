use super::*;

#[test]
fn live_share_accepts_repeated_markers_and_cleared_terminal_lease() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::LiveShare,
        Some(MiningCampaignProfile::Conservative),
    );
    let active = campaign_marker(CampaignMarkerFixture {
        stage: "live-share",
        lease_id: serde_json::json!(42),
        state: "active",
        profile: "conservative",
        active_ms: 1_000,
        submit_outcome: "none",
        terminal_reason: "none",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "qualified",
        safe_stop: "pending",
    });
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[
        active.clone(),
        active,
        live_terminal("accepted"),
    ]));

    // Act
    run_mining_campaign(&command, &environment).expect("live-share campaign");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "submit_response_observed");
    assert_eq!(result["submit_outcome"], "accepted");
    assert_eq!(result["terminal_reason"], "campaign_lease_consumed");
    for (field, expected) in [
        ("qualified_candidate_count", 1),
        ("below_pool_target_count", 0),
        ("duplicate_candidate_count", 0),
    ] {
        assert_eq!(result[field], expected);
    }
    assert_eq!(result["marker_count"], 3);
    assert_eq!(result["safe_stop"], "confirmed");
    assert_eq!(result["observation_freshness"]["fan_rpm"], true);
    let csv = environment
        .written_files()
        .iter()
        .find(|(path, _)| path.file_name() == Some("campaign-nvs.csv"))
        .map(|(_, contents)| contents.clone())
        .expect("campaign CSV");
    for expected in [
        "mineonboot,data,u16,0",
        "campstage,data,string,live-share",
        "campprofile,data,string,conservative",
        "camplease,data,u64,42",
        "campdurms,data,u64,600000",
        "stratumprot,data,string,SV1",
        "stratumtls,data,u16,0",
    ] {
        assert!(csv.contains(expected), "missing row {expected}");
    }
}

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
