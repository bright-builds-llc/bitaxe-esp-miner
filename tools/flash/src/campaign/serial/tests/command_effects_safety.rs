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
