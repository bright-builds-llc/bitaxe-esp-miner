//! The diagnostic fields a soak sample copies from system info: send counts and
//! durations on the HTTP server task, and the Wi-Fi RSSI.

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
