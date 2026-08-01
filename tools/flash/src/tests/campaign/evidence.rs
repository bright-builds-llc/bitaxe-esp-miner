use super::*;

#[test]
fn campaign_evidence_never_projects_raw_serial_or_credentials() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[observation_marker("fresh")]));

    // Act
    run_mining_campaign(&command, &environment).expect("observation campaign");

    // Assert
    for name in [
        "campaign-diagnostics.private.json",
        "campaign-mining-diagnostics.private.json",
        "campaign-network.private.json",
        "campaign-observations.private.json",
        "campaign-result.json",
        "campaign-result.sha256",
    ] {
        let contents = std::fs::read_to_string(command.evidence_dir.join(name).as_std_path())
            .expect("evidence");
        for forbidden in [
            "pool.private.test",
            "owner.worker",
            "pool-private",
            "wifi-private",
            "/dev/cu",
        ] {
            assert!(!contents.contains(forbidden), "{name} leaked {forbidden}");
        }
    }
    let result_bytes = std::fs::read(
        command
            .evidence_dir
            .join("campaign-result.json")
            .as_std_path(),
    )
    .expect("result bytes");
    let diagnostic_bytes = std::fs::read(
        command
            .evidence_dir
            .join("campaign-diagnostics.private.json")
            .as_std_path(),
    )
    .expect("diagnostic bytes");
    let mining_diagnostic_bytes = std::fs::read(
        command
            .evidence_dir
            .join("campaign-mining-diagnostics.private.json")
            .as_std_path(),
    )
    .expect("mining diagnostic bytes");
    let network_bytes = std::fs::read(
        command
            .evidence_dir
            .join("campaign-network.private.json")
            .as_std_path(),
    )
    .expect("network continuity bytes");
    let result = read_campaign_result(&command);
    assert_eq!(
        result["diagnostics_sha256"],
        sha256_bytes(&diagnostic_bytes)
    );
    assert_eq!(
        result["mining_diagnostics_sha256"],
        sha256_bytes(&mining_diagnostic_bytes)
    );
    assert_eq!(
        result["network_continuity_sha256"],
        sha256_bytes(&network_bytes)
    );
    let network: serde_json::Value =
        serde_json::from_slice(&network_bytes).expect("network continuity JSON");
    assert_eq!(network["schema"], "mining-campaign-network-continuity-v2");
    assert_eq!(network["http_startup_transition_count"], 0);
    assert_eq!(network["websocket_startup_transition_count"], 0);
    assert_eq!(network["http_initial_active_observed"], false);
    assert_eq!(network["websocket_initial_active_observed"], false);
    let seal = std::fs::read_to_string(
        command
            .evidence_dir
            .join("campaign-result.sha256")
            .as_std_path(),
    )
    .expect("seal");
    assert_eq!(seal.trim(), sha256_bytes(&result_bytes));
}
