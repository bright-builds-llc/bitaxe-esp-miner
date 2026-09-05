use super::*;

mod command_effects;
mod evidence;
mod failure_diagnostics;
mod job_transition;
mod live_share;
mod stop_predicate;
mod terminal_boundary;

include!("campaign/fixtures.rs");

#[test]
fn observation_fixture_validates_paused_seed_and_sealed_evidence_without_device_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[observation_marker("fresh")]));

    // Act
    run_campaign_observation_fixture(&command, &environment).expect("observation campaign");

    // Assert
    assert_eq!(
        environment.campaign_observations(),
        vec![(
            MiningCampaignStage::Observation,
            CampaignCaptureLimit::Bounded(360)
        )]
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(environment.executed_commands().is_empty());
    let csv = environment
        .written_files()
        .iter()
        .find(|(path, _)| path.file_name() == Some("campaign-nvs.csv"))
        .map(|(_, contents)| contents.clone())
        .expect("campaign CSV");
    assert!(csv.contains("mineonboot,data,u16,0"));
    assert!(csv.contains("campstage,data,string,observation"));
    for forbidden in [
        "camplease",
        "campprofile",
        "campdurms",
        "stratumurl",
        "stratumuser",
        "stratumpass",
    ] {
        assert!(!csv.contains(forbidden), "unexpected key {forbidden}");
    }
    let result = read_campaign_result(&command);
    assert_eq!(result["schema"], "mining-campaign-result-v16");
    assert_eq!(
        result["readiness_transition"],
        serde_json::json!({
            "wakeup": "observations_changed",
            "previous_blocker": "safety_prerequisites_stale",
            "current_blocker": "none",
            "session_phase": "waiting_for_readiness",
            "campaign_state": "unavailable",
            "hardware_state": "unprepared",
            "safety_sample": "fresh",
            "observation_epoch": "advanced",
            "pending_observation_recovered": true,
        })
    );
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["terminal_category"], "observation_complete");
    assert_eq!(result["runtime_identity"], "trusted");
    assert_eq!(result["runtime_attestation_status"], "trusted");
    assert_eq!(result["runtime_attestation_parse_failure"], "none");
    assert_eq!(
        result["runtime_attestation_parse_failure_counts"],
        serde_json::json!({
            "missing_marker": 0,
            "malformed_token": 0,
            "duplicate_field": 0,
            "unknown_field": 0,
            "missing_field": 0,
            "invalid_field": 0,
            "incomplete_readiness": 0,
        })
    );
    assert_eq!(result["serial_outcome_detail"], "clean");
    assert_eq!(
        result["observation_freshness"],
        serde_json::json!({
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        })
    );
    assert_eq!(
        result["observation_requirements"],
        serde_json::json!({
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        })
    );
    assert!(result["failure_observation_freshness"].is_null());
    assert_eq!(result["usb_cleanup"], "ready");
    assert_eq!(result["parity_promotion"], false);
    let observations = read_campaign_observations(&command);
    assert_eq!(observations["schema"], "mining-campaign-observations-v4");
    assert_eq!(observations["marker_count"], 1);
    assert!(observations.get("markers").is_none());
    assert!(observations["terminal_marker"].is_object());
    assert_private_campaign_artifacts(&command.evidence_dir);
}

#[test]
fn non_utf8_boot_noise_does_not_invalidate_valid_observation_markers() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let mut campaign_bytes = vec![0xff, 0xfe, b'\n'];
    campaign_bytes.extend_from_slice(campaign_log(&[observation_marker("fresh")]).as_bytes());
    let environment = FakeFlashEnvironment::default().with_campaign_bytes(campaign_bytes);

    // Act
    run_campaign_observation_fixture(&command, &environment)
        .expect("non-marker binary boot noise must not invalidate the campaign");

    // Assert
    let result = read_campaign_result(&command);
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["terminal_category"], "observation_complete");
    assert_eq!(result["marker_count"], 1);
    assert_eq!(result["serial_outcome_detail"], "clean");
    let diagnostics = read_campaign_diagnostics(&command);
    assert_eq!(
        diagnostics["schema"],
        "mining-campaign-serial-diagnostics-v4"
    );
    assert_eq!(diagnostics["non_utf8_line_count"], 1);
    assert_eq!(diagnostics["accepted_marker_count"], 1);
}

#[test]
fn malformed_marker_preserves_independent_runtime_attestation_status() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let campaign_bytes = format!(
        "{}\nmining_campaign_status={{]\n",
        runtime_attestation_log()
    )
    .into_bytes();
    let environment = FakeFlashEnvironment::default().with_campaign_bytes(campaign_bytes);

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("malformed marker must fail closed");

    // Assert
    assert!(format!("{error:#}").contains("category=marker_invalid"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "marker_invalid");
    assert_eq!(result["serial_outcome_detail"], "marker_json_invalid");
    assert_eq!(result["runtime_attestation_status"], "trusted");
    assert_eq!(result["runtime_identity"], "trusted");
    assert_eq!(result["marker_count"], 0);
}

#[test]
fn absent_campaign_marker_remains_marker_missing() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default().with_log_contents(&runtime_attestation_log());

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("missing marker must fail closed");

    // Assert
    assert!(format!("{error:#}").contains("category=marker_missing"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "marker_missing");
    assert_eq!(result["serial_outcome_detail"], "marker_missing");
    assert_eq!(result["runtime_attestation_status"], "trusted");
}

#[test]
fn valid_marker_preserves_independent_runtime_attestation_failure() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let campaign_bytes = format!("{}\n", observation_marker("fresh")).into_bytes();
    let environment = FakeFlashEnvironment::default().with_campaign_bytes(campaign_bytes);

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("missing runtime attestation must fail independently");

    // Assert
    assert!(format!("{error:#}").contains("category=runtime_identity_untrusted"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "runtime_identity_untrusted");
    assert_eq!(result["serial_outcome_detail"], "clean");
    assert_eq!(result["runtime_attestation_status"], "missing");
    assert_eq!(result["marker_count"], 1);
}

#[test]
fn observation_rejects_pool_input_before_package_or_credential_reads() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let mut command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    command.pool_credentials = Some(write_pool_credentials(&dir));
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("observation pool input must fail closed");

    // Assert
    assert!(format!("{error:#}").contains("category=admission_failed"));
    assert!(environment.read_string_paths().is_empty());
    assert!(environment.executed_commands().is_empty());
    assert_eq!(
        read_campaign_result(&command)["terminal_category"],
        "admission_failed"
    );
}

#[test]
fn existing_attempt_child_is_rejected_before_reads_or_device_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    std::fs::create_dir(command.evidence_dir.as_std_path()).expect("preexisting attempt");
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("existing attempt must fail closed");

    // Assert
    assert!(format!("{error:#}").contains("category=admission_failed"));
    assert!(environment.read_string_paths().is_empty());
    assert!(environment.executed_commands().is_empty());
    assert!(std::fs::read_dir(command.evidence_dir.as_std_path())
        .expect("attempt directory")
        .next()
        .is_none());
}

#[test]
fn soak_requires_full_active_duration() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::Soak,
        Some(MiningCampaignProfile::UpstreamDefault),
    );
    let short = campaign_marker(CampaignMarkerFixture {
        stage: "soak",
        lease_id: serde_json::Value::Null,
        state: "consumed",
        profile: "upstream-default",
        active_ms: 599_999,
        submit_outcome: "accepted",
        terminal_reason: "campaign_lease_consumed",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "safe_stopped",
        safe_stop: "confirmed",
    });
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[short]));

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("short soak must fail closed");

    // Assert
    assert!(format!("{error:#}").contains("category=soak_duration_short"));
    assert_eq!(
        read_campaign_result(&command)["terminal_category"],
        "soak_duration_short"
    );
}

#[test]
fn earliest_safety_failure_survives_a_later_valid_terminal_marker() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[
        observation_marker("stale"),
        observation_marker("fresh"),
    ]));

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("earliest safety failure must remain terminal");

    // Assert
    assert!(format!("{error:#}").contains("category=safety_stale"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "safety_stale");
    assert_eq!(
        result["failure_observation_freshness"]["chip_temp_celsius"],
        false
    );
    assert_eq!(result["observation_freshness"]["chip_temp_celsius"], true);
    assert_eq!(result["fresh_observation_count"], 5);
}

#[test]
fn cleanup_failure_replaces_success_but_not_an_earlier_campaign_failure() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[observation_marker("stale")]))
        .with_cleanup_failure();

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("campaign and cleanup failure must fail");

    // Assert
    assert!(format!("{error:#}").contains("category=safety_stale"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "safety_stale");
    assert_eq!(result["usb_cleanup"], "not_proven");
}

fn assert_private_campaign_artifacts(root: &Utf8Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(root.as_std_path())
                .expect("evidence root")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        for name in [
            "campaign-diagnostics.private.json",
            "campaign-flash.private.json",
            "campaign-mining-diagnostics.private.json",
            "campaign-network.private.json",
            "campaign-observations.private.json",
            "campaign-result.json",
            "campaign-result.sha256",
        ] {
            assert_eq!(
                std::fs::metadata(root.join(name).as_std_path())
                    .expect("evidence file")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }
}

#[test]
fn legacy_mining_campaign_cannot_implicitly_reset_nvs() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_mining_campaign(&command, &environment)
        .expect_err("legacy seed requires explicit successor");
    // Assert
    assert!(error
        .to_string()
        .contains("provisioning_requires_factory_reset"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.generated_nvs_partitions().is_empty());
}
