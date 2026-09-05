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
        manifest: Some(write_manifest_v4(dir, DEFAULT_ELF_NAME)),
        wifi_credentials,
        pool_credentials,
        evidence_dir: root.join("attempt-001"),
        duration_seconds: match stage {
            MiningCampaignStage::Observation => 360,
            MiningCampaignStage::LiveShare
            | MiningCampaignStage::Soak
            | MiningCampaignStage::CommandEffects => 600,
            MiningCampaignStage::JobTransition => 1_800,
            MiningCampaignStage::StratumV2 => 180,
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
