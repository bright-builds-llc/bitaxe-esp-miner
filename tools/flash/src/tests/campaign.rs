use super::*;

mod command_effects;
mod evidence;
mod failure_diagnostics;
mod job_transition;
mod live_share;
mod stop_predicate;
mod terminal_boundary;

const CAMPAIGN_MARKER_PREFIX: &str = "mining_campaign_status=";

fn campaign_command(
    dir: &TempDir,
    stage: MiningCampaignStage,
    profile: Option<MiningCampaignProfile>,
) -> MiningCampaignCommand {
    let root = dir_path(dir);
    set_private_directory_mode(&root).expect("private test parent");
    let wifi_credentials = write_wifi_credentials(dir, "CampaignNet", "wifi-private");
    let pool_credentials =
        (stage != MiningCampaignStage::Observation).then(|| write_pool_credentials(dir));
    MiningCampaignCommand {
        stage,
        profile,
        board: BoardId::Ultra205,
        port: Some("/dev/cu.usbmodem101".to_owned()),
        manifest: Some(write_manifest_v3(dir, DEFAULT_ELF_NAME)),
        wifi_credentials,
        pool_credentials,
        evidence_dir: root.join("attempt-001"),
        duration_seconds: match stage {
            MiningCampaignStage::Observation => 360,
            MiningCampaignStage::LiveShare
            | MiningCampaignStage::Soak
            | MiningCampaignStage::CommandEffects => 600,
            MiningCampaignStage::JobTransition => 1_800,
        },
        redact_evidence: true,
    }
}

fn write_pool_credentials(dir: &TempDir) -> Utf8PathBuf {
    let path = dir_path(dir).join("pool.json");
    std::fs::write(
        path.as_std_path(),
        serde_json::json!({
            "poolURL": "pool.private.test",
            "poolPort": 3333,
            "poolUser": "owner.worker",
            "poolPassword": "pool-private",
        })
        .to_string(),
    )
    .expect("write pool credentials");
    path
}

struct CampaignMarkerFixture<'a> {
    stage: &'a str,
    lease_id: serde_json::Value,
    state: &'a str,
    profile: &'a str,
    active_ms: u64,
    submit_outcome: &'a str,
    terminal_reason: &'a str,
    safety: &'a str,
    pool_config: &'a str,
    actuation: &'a str,
    safe_stop: &'a str,
}

fn campaign_marker(fixture: CampaignMarkerFixture<'_>) -> String {
    campaign_marker_with_failure(
        fixture,
        serde_json::json!({
            "phase": "none",
            "step": "none",
            "detail": "none",
            "rollback_step": "none",
            "rollback_detail": "none",
        }),
    )
}

fn campaign_marker_with_failure(
    fixture: CampaignMarkerFixture<'_>,
    failure: serde_json::Value,
) -> String {
    let safety_fresh = fixture.safety == "fresh";
    format!(
        "mining_campaign_status={}",
        serde_json::json!({
            "schema": "mining-campaign-status-v13",
            "stage": fixture.stage,
            "lease_id": fixture.lease_id,
            "campaign_state": fixture.state,
            "profile": fixture.profile,
            "active_ms": fixture.active_ms,
            "submit_outcome": fixture.submit_outcome,
            "qualified_candidate_count": if fixture.submit_outcome == "none" { 0 } else { 1 },
            "below_pool_target_count": 0,
            "duplicate_candidate_count": 0,
            "accepted_share_count": if fixture.submit_outcome == "accepted" { 1 } else { 0 },
            "rejected_share_count": if fixture.submit_outcome == "rejected" { 1 } else { 0 },
            "job_transition": {
                "pool_notify_count": 0,
                "clean_jobs_notify_count": 0,
                "previous_block_change_count": 0,
                "new_block_generation_count": 0,
                "replacement_dispatch_count": 0,
                "post_transition_correlated_result_count": 0,
                "completed_transition_count": 0,
                "stale_generation_result_discard_count": 0,
                "stale_generation_submit_count": 0,
                "reconnect_count": 0,
                "latest_state": "not_observed",
            },
            "asic_bridge": {
                "poll_request_count": 0,
                "idle_completion_count": 0,
                "nonce_completion_count": 0,
                "register_read_count": 0,
                "discards": {
                    "invalid_length": 0,
                    "invalid_preamble": 0,
                    "invalid_crc": 0,
                    "job_lookup": 0,
                    "core": 0,
                    "address_interval": 0,
                    "register_response": 0,
                    "parser_invariant": 0,
                },
                "generation_invalidation_count": 0,
                "stale_completion_count": 0,
                "post_transition_poll_request_count": 0,
                "post_transition_completion_count": 0,
                "post_transition_nonce_emission_count": 0,
                "post_transition_correlation_count": 0,
                "blocked_correlation_count": 0,
                "blocked_correlations": {
                    "wrong_session": 0,
                    "job_lookup": 0,
                    "work_stale": 0,
                    "target_mismatch": 0,
                    "other": 0,
                },
                "changed_block_to_replacement_dispatch_ms": null,
                "changed_block_to_first_poll_ms": null,
                "changed_block_to_first_nonce_ms": null,
                "changed_block_to_first_correlation_ms": null,
                "final_poll_state": "idle",
                "latest_event": null,
            },
            "terminal_reason": fixture.terminal_reason,
            "protocol_gate": "ready",
            "readiness_transition": {
                "wakeup": "observations_changed",
                "previous_blocker": "safety_prerequisites_stale",
                "current_blocker": fixture.terminal_reason,
                "session_phase": "waiting_for_readiness",
                "campaign_state": fixture.state,
                "hardware_state": if fixture.state == "consumed" { "stopped" } else { "unprepared" },
                "safety_sample": fixture.safety,
                "observation_epoch": "advanced",
                "pending_observation_recovered": true,
            },
            "operator_sensor": {
                "available": false,
                "boot_session": 0,
                "revision": 0,
                "stage": "none",
                "outcome": "none",
                "duration_bucket": "none",
            },
            "resumable_pause_safe_stop": "not_required", "safety": fixture.safety,
            "fresh_observation_count": if safety_fresh { 5 } else { 4 },
            "observation_freshness": {
                "power_watts": true,
                "bus_voltage_volts": true,
                "current_amps": true,
                "chip_temp_celsius": safety_fresh,
                "vr_temp_celsius": false,
                "fan_rpm": true,
            },
            "observation_requirements": {
                "power_watts": true,
                "bus_voltage_volts": true,
                "current_amps": true,
                "chip_temp_celsius": true,
                "vr_temp_celsius": false,
                "fan_rpm": true,
            },
            "pool_config": fixture.pool_config,
            "pool_config_persisted": fixture.state == "consumed"
                && fixture.pool_config == "local_owner_supplied",
            "actuation": fixture.actuation,
            "mineonboot": false,
            "safe_stop": fixture.safe_stop,
            "failure": failure,
        })
    )
}

fn campaign_log(markers: &[String]) -> String {
    format!(
        "{}\n{}\nraw poolURL=https://pool.private.test poolUser=owner.worker poolPassword=pool-private\n",
        runtime_attestation_log(),
        markers.join("\n")
    )
}

fn observation_marker(safety: &str) -> String {
    campaign_marker(CampaignMarkerFixture {
        stage: "observation",
        lease_id: serde_json::Value::Null,
        state: "unavailable",
        profile: "none",
        active_ms: 0,
        submit_outcome: "none",
        terminal_reason: "none",
        safety,
        pool_config: "not_read",
        actuation: "none",
        safe_stop: "not_required",
    })
}

fn live_terminal(submit_outcome: &str) -> String {
    campaign_marker(CampaignMarkerFixture {
        stage: "live-share",
        lease_id: serde_json::Value::Null,
        state: "consumed",
        profile: "conservative",
        active_ms: 2_000,
        submit_outcome,
        terminal_reason: "campaign_lease_consumed",
        safety: "fresh",
        pool_config: "local_owner_supplied",
        actuation: "safe_stopped",
        safe_stop: "confirmed",
    })
}

fn read_campaign_result(command: &MiningCampaignCommand) -> serde_json::Value {
    let result = command.evidence_dir.join("campaign-result.json");
    serde_json::from_str(&std::fs::read_to_string(result.as_std_path()).expect("campaign result"))
        .expect("campaign result JSON")
}

fn read_campaign_diagnostics(command: &MiningCampaignCommand) -> serde_json::Value {
    let diagnostics = command
        .evidence_dir
        .join("campaign-diagnostics.private.json");
    serde_json::from_str(
        &std::fs::read_to_string(diagnostics.as_std_path()).expect("campaign diagnostics"),
    )
    .expect("campaign diagnostics JSON")
}

fn read_campaign_observations(command: &MiningCampaignCommand) -> serde_json::Value {
    let observations = command
        .evidence_dir
        .join("campaign-observations.private.json");
    serde_json::from_str(
        &std::fs::read_to_string(observations.as_std_path()).expect("campaign observations"),
    )
    .expect("campaign observations JSON")
}

#[test]
fn observation_campaign_uses_exact_package_combined_paused_seed_and_sealed_evidence() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(&dir, MiningCampaignStage::Observation, None);
    let environment = FakeFlashEnvironment::default()
        .with_log_contents(&campaign_log(&[observation_marker("fresh")]));

    // Act
    run_mining_campaign(&command, &environment).expect("observation campaign");

    // Assert
    assert_eq!(
        environment.campaign_observations(),
        vec![(
            MiningCampaignStage::Observation,
            CampaignCaptureLimit::Bounded(360)
        )]
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert_eq!(environment.executed_commands().len(), 3);
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
    assert_eq!(result["schema"], "mining-campaign-result-v13");
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
    run_mining_campaign(&command, &environment)
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
        "mining-campaign-serial-diagnostics-v2"
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
    let error =
        run_mining_campaign(&command, &environment).expect_err("malformed marker must fail closed");

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
    let error =
        run_mining_campaign(&command, &environment).expect_err("missing marker must fail closed");

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
    let error = run_mining_campaign(&command, &environment)
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
    let error = run_mining_campaign(&command, &environment)
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
    let error =
        run_mining_campaign(&command, &environment).expect_err("existing attempt must fail closed");

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
    let error =
        run_mining_campaign(&command, &environment).expect_err("short soak must fail closed");

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
    let error = run_mining_campaign(&command, &environment)
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
    let error = run_mining_campaign(&command, &environment)
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
