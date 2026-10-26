//! The diagnostic fields a soak sample copies from system info: send counts and
//! durations on the HTTP server task, the Wi-Fi RSSI, and the hardware settings and readings.

use super::super::sample::SoakSample;
use super::sample;

fn wire() -> bitaxe_api::SystemInfoWire {
    bitaxe_api::system_info_from_snapshot(&bitaxe_api::ApiSnapshot::safe_ultra_205())
}

#[test]
fn a_sample_records_the_server_task_send_durations_and_rssi() {
    // Arrange
    let mut wire = wire();
    wire.runtime_health
        .http_server_sends
        .http_responses
        .recent_worst_millis = 4_600;
    wire.runtime_health
        .http_server_sends
        .websocket_sends
        .failures = 2;
    wire.wifi_rssi = -71;
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(
        (
            sample.http_response_recent_worst_ms,
            sample.websocket_send_failures,
            sample.wifi_rssi
        ),
        (4_600, 2, -71)
    );
}

#[test]
fn a_sample_records_send_counts_and_the_worst_send_since_boot() {
    // Arrange: a 5.2 s write long ago has aged out of the recent window.
    let mut wire = wire();
    let sends = &mut wire.runtime_health.http_server_sends;
    sends.http_responses.sends = 412;
    sends.http_responses.worst_millis = 5_200;
    sends.http_responses.recent_worst_millis = 40;
    sends.websocket_sends.sends = 1_337;
    sends.websocket_sends.worst_millis = 3_100;
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(
        (
            sample.http_response_sends,
            sample.http_response_worst_ms,
            sample.http_response_recent_worst_ms,
            sample.websocket_sends,
            sample.websocket_send_worst_ms,
        ),
        (412, 5_200, 40, 1_337, 3_100)
    );
}

#[test]
fn a_journal_sample_from_before_the_new_fields_still_parses() {
    // Arrange: soak 002's sample shape, without the best-difficulty or send fields.
    let mut value = serde_json::to_value(sample(0, 1)).expect("sample json");
    let object = value.as_object_mut().expect("object");
    for key in [
        "bestDiff",
        "bestSessionDiff",
        "httpResponseSends",
        "httpResponseWorstMs",
        "httpResponseRecentWorstMs",
        "httpResponseFailures",
        "websocketSends",
        "websocketSendWorstMs",
        "websocketSendRecentWorstMs",
        "websocketSendFailures",
        "wifiRssi",
        "frequencyMhz",
        "coreVoltageMv",
        "maybeCoreVoltageActualMv",
        "powerWatts",
        "hashRateGhs",
        "autoFanSpeed",
        "manualFanSpeed",
        "fanRpm",
    ] {
        object.remove(key).expect("field present");
    }
    // Act
    let parsed: SoakSample = serde_json::from_value(value).expect("older sample parses");
    // Assert
    assert_eq!(
        (
            parsed.best_diff,
            parsed.http_response_sends,
            parsed.websocket_send_worst_ms
        ),
        (0, 0, 0)
    );
}

#[test]
fn a_sample_records_a_safe_stop_in_progress_as_neither_active_nor_stopped() {
    // Arrange
    let mut wire = wire();
    wire.mining_paused = false;
    wire.mining_activity = "stopping".to_owned();
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(
        (sample.mining_active, sample.mining_stopping),
        (false, true)
    );
}

#[test]
fn a_journal_sample_from_before_the_stopping_state_parses_as_not_stopping() {
    // Arrange
    let mut value = serde_json::to_value(sample(0, 1)).expect("sample json");
    let object = value.as_object_mut().expect("object");
    object.remove("miningStopping").expect("field present");
    // Act
    let parsed: SoakSample = serde_json::from_value(value).expect("older sample parses");
    // Assert
    assert!(!parsed.mining_stopping);
}

#[test]
fn a_soak_005_journal_sample_without_the_counts_still_parses() {
    // Arrange: soak 005 and 006 samples carry recent worst and failures only.
    let mut value = serde_json::to_value(sample(0, 1)).expect("sample json");
    let object = value.as_object_mut().expect("object");
    for key in [
        "httpResponseSends",
        "httpResponseWorstMs",
        "websocketSends",
        "websocketSendWorstMs",
    ] {
        object.remove(key).expect("field present");
    }
    // Act
    let parsed: Result<SoakSample, _> = serde_json::from_value(value);
    // Assert
    assert!(parsed.is_ok());
}

#[test]
fn a_sample_records_the_settings_and_the_fresh_measured_core_voltage() {
    // Arrange
    let mut wire = wire();
    wire.frequency = 400.0;
    wire.core_voltage = 1_100;
    wire.core_voltage_actual = 1_096.0;
    wire.core_voltage_actual_status.state = bitaxe_api::ObservationStateWire::Fresh;
    wire.power = 8.25;
    wire.hash_rate = 410.5;
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(
        (
            sample.frequency_mhz,
            sample.core_voltage_mv,
            sample.maybe_core_voltage_actual_mv,
            sample.power_watts,
            sample.hash_rate_ghs
        ),
        (400.0, 1_100, Some(1_096.0), 8.25, 410.5)
    );
}

#[test]
fn a_stale_core_voltage_reading_is_not_recorded() {
    // Arrange
    let mut wire = wire();
    wire.core_voltage_actual = 1_196.0;
    wire.core_voltage_actual_status.state = bitaxe_api::ObservationStateWire::Stale;
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(sample.maybe_core_voltage_actual_mv, None);
}

#[test]
fn a_sample_records_the_fan_settings_and_reading() {
    // Arrange
    let mut wire = wire();
    wire.auto_fan_speed = 0;
    wire.manual_fan_speed = 80;
    wire.fan_rpm = 5_900;
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert_eq!(
        (
            sample.auto_fan_speed,
            sample.manual_fan_speed,
            sample.fan_rpm
        ),
        (0, 80, 5_900)
    );
}

/// A running sample with every reading fresh and inside the limits, drawing `power` watts.
fn fresh_running(power: f64) -> bitaxe_api::SystemInfoWire {
    use bitaxe_api::ObservationStateWire::Fresh;
    let mut wire = wire();
    for status in [
        &mut wire.power_status,
        &mut wire.voltage_status,
        &mut wire.current_status,
        &mut wire.chip_temp_status,
        &mut wire.fan_rpm_status,
    ] {
        status.state = Fresh;
    }
    wire.power = power;
    wire.voltage_millivolts = 5_100.0;
    wire.current_milliamps = 3_000.0;
    wire.temp = 60.0;
    wire.fan_rpm = 4_000;
    wire
}

#[test]
fn sixteen_watts_is_unsafe_on_the_ultra_205() {
    // Arrange
    let wire = fresh_running(16.0);
    // Act
    let valid = sample::safety_valid(&wire, sample::SoakBoard::Ultra205);
    // Assert
    assert!(!valid);
}

#[test]
fn sixteen_watts_is_safe_on_the_gamma_601() {
    // Arrange
    let wire = fresh_running(16.0);
    // Act
    let valid = sample::safety_valid(&wire, sample::SoakBoard::Gamma601);
    // Assert
    assert!(valid);
}

#[test]
fn twenty_three_watts_is_unsafe_on_either_board() {
    // Arrange
    let wire = fresh_running(23.0);
    // Act
    let valid = [sample::SoakBoard::Ultra205, sample::SoakBoard::Gamma601]
        .map(|board| sample::safety_valid(&wire, board));
    // Assert
    assert_eq!(valid, [false, false]);
}

#[test]
fn a_sample_without_a_board_judges_the_ultra_205() {
    // Arrange
    let wire = fresh_running(16.0);
    // Act
    let sample = SoakSample::from_wire(&wire, None);
    // Assert
    assert!(!sample.safety_valid);
}

#[test]
fn a_sample_on_the_gamma_601_judges_its_power_stop() {
    // Arrange
    let wire = fresh_running(16.0);
    // Act
    let sample = SoakSample::from_wire_on(&wire, None, sample::SoakBoard::Gamma601);
    // Assert
    assert!(sample.safety_valid);
}
