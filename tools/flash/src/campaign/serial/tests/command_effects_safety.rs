use super::*;

fn command_effects_admission() -> CampaignAdmission {
    CampaignAdmission {
        stage: MiningCampaignStage::CommandEffects,
        maybe_profile: Some(MiningCampaignProfile::Conservative),
        duration_seconds: 600,
        maybe_lease_id: Some(7),
    }
}

fn command_effects_paused_stale_marker() -> Vec<u8> {
    let template = live_share_preparing_marker();
    let document = std::str::from_utf8(&template).expect("marker template");
    let payload = document
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("marker prefix")
        .trim_end();
    let mut marker: serde_json::Value = serde_json::from_str(payload).expect("marker json");
    marker["stage"] = serde_json::json!("command-effects");
    marker["campaign_state"] = serde_json::json!("armed");
    marker["terminal_reason"] = serde_json::json!("operator_paused");
    marker["readiness_transition"]["current_blocker"] = serde_json::json!("operator_paused");
    marker["readiness_transition"]["campaign_state"] = serde_json::json!("armed");
    marker["readiness_transition"]["hardware_state"] = serde_json::json!("stopped");
    marker["readiness_transition"]["safety_sample"] = serde_json::json!("stale");
    marker["resumable_pause_safe_stop"] = serde_json::json!("confirmed");
    marker["safety"] = serde_json::json!("stale");
    marker["fresh_observation_count"] = serde_json::json!(0);
    for field in [
        "power_watts",
        "bus_voltage_volts",
        "current_amps",
        "chip_temp_celsius",
        "fan_rpm",
    ] {
        marker["observation_freshness"][field] = serde_json::json!(false);
    }
    marker["pool_config"] = serde_json::json!("local_owner_supplied");
    marker["actuation"] = serde_json::json!("qualified");
    marker["safe_stop"] = serde_json::json!("pending");
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

fn command_effects_consumed_marker() -> Vec<u8> {
    let paused = command_effects_paused_stale_marker();
    let document = std::str::from_utf8(&paused).expect("marker template");
    let payload = document
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("marker prefix")
        .trim_end();
    let mut marker: serde_json::Value = serde_json::from_str(payload).expect("marker json");
    marker["lease_id"] = serde_json::Value::Null;
    marker["campaign_state"] = serde_json::json!("consumed");
    marker["active_ms"] = serde_json::json!(600_000);
    marker["terminal_reason"] = serde_json::json!("campaign_lease_consumed");
    marker["readiness_transition"]["current_blocker"] =
        serde_json::json!("campaign_lease_consumed");
    marker["readiness_transition"]["campaign_state"] = serde_json::json!("consumed");
    marker["readiness_transition"]["hardware_state"] = serde_json::json!("stopped");
    marker["readiness_transition"]["safety_sample"] = serde_json::json!("fresh");
    marker["resumable_pause_safe_stop"] = serde_json::json!("not_required");
    marker["safety"] = serde_json::json!("fresh");
    marker["fresh_observation_count"] = serde_json::json!(5);
    for field in [
        "power_watts",
        "bus_voltage_volts",
        "current_amps",
        "chip_temp_celsius",
        "fan_rpm",
    ] {
        marker["observation_freshness"][field] = serde_json::json!(true);
    }
    marker["actuation"] = serde_json::json!("safe_stopped");
    marker["safe_stop"] = serde_json::json!("confirmed");
    marker["pool_config_persisted"] = serde_json::json!(true);
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

fn command_effects_armed_consumed_reason_marker() -> Vec<u8> {
    let paused = command_effects_paused_stale_marker();
    let document = std::str::from_utf8(&paused).expect("marker template");
    let payload = document
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("marker prefix")
        .trim_end();
    let mut marker: serde_json::Value = serde_json::from_str(payload).expect("marker json");
    marker["active_ms"] = serde_json::json!(600_000);
    marker["terminal_reason"] = serde_json::json!("campaign_lease_consumed");
    marker["readiness_transition"]["current_blocker"] =
        serde_json::json!("campaign_lease_consumed");
    marker["resumable_pause_safe_stop"] = serde_json::json!("not_required");
    marker["safety"] = serde_json::json!("fresh");
    marker["fresh_observation_count"] = serde_json::json!(5);
    for field in [
        "power_watts",
        "bus_voltage_volts",
        "current_amps",
        "chip_temp_celsius",
        "fan_rpm",
    ] {
        marker["observation_freshness"][field] = serde_json::json!(true);
    }
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

#[test]
fn confirmed_stopped_command_pause_does_not_fail_on_transient_stale_sensors() {
    // Arrange
    let marker = command_effects_paused_stale_marker();

    // Act
    let capture = analyze_campaign_serial_bytes(&marker, command_effects_admission());

    // Assert
    assert_eq!(capture.diagnostics.accepted_marker_count, 1);
    assert_eq!(capture.maybe_failure, None);
}

#[test]
fn later_valid_consumed_marker_recovers_transient_serial_json_damage() {
    // Arrange
    let damaged = format!("{CAMPAIGN_MARKER_PREFIX}{{]\n").into_bytes();
    let terminal = command_effects_consumed_marker();
    let mut analyzer = CampaignSerialAnalyzer::new(command_effects_admission());

    // Act
    let damaged_stop = analyzer.observe_chunk(&damaged);
    let terminal_stop = analyzer.observe_chunk(&terminal);
    let capture = analyzer.finish();
    let maybe_terminal = crate::campaign::network::terminal_capture_handoff(&capture);

    // Assert
    assert!(!damaged_stop);
    assert!(terminal_stop);
    assert_eq!(capture.maybe_failure, None);
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(capture.diagnostics.marker_invalid_json_count, 1);
    assert_eq!(capture.diagnostics.accepted_marker_count, 1);
    assert!(maybe_terminal.is_some());
}

#[test]
fn accepted_terminal_handoff_does_not_hide_an_independent_schema_failure() {
    // Arrange
    let valid_terminal = command_effects_consumed_marker();
    let document = std::str::from_utf8(&valid_terminal).expect("terminal marker");
    let payload = document
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("marker prefix")
        .trim_end();
    let mut invalid: serde_json::Value = serde_json::from_str(payload).expect("marker json");
    invalid["schema"] = serde_json::json!("mining-campaign-status-v0");
    let mut stream = format!("{CAMPAIGN_MARKER_PREFIX}{invalid}\n").into_bytes();
    stream.extend_from_slice(&valid_terminal);

    // Act
    let capture = analyze_campaign_serial_bytes(&stream, command_effects_admission());
    let maybe_terminal = crate::campaign::network::terminal_capture_handoff(&capture);

    // Assert
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::MarkerInvalid)
    );
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerSchemaInvalid
    );
    assert!(maybe_terminal.is_some());
}

#[test]
fn consumed_reason_with_armed_state_hands_off_the_terminal_failure() {
    // Arrange
    let stream = command_effects_armed_consumed_reason_marker();

    // Act
    let capture = analyze_campaign_serial_bytes(&stream, command_effects_admission());
    let terminal = crate::campaign::network::terminal_capture_handoff(&capture)
        .expect("consumed terminal reason should be handed off");

    // Assert
    assert_eq!(capture.maybe_failure, None);
    assert!(!terminal.terminal_consumed);
    assert_eq!(
        terminal.maybe_failure,
        Some(CampaignTerminalCategory::TerminalStateUnconfirmed)
    );
}
