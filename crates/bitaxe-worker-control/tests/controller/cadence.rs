use super::*;

fn frame(command: &str, payload: serde_json::Value) -> String {
    format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4", "requestId":"serial_cadence",
        "command":command, "payload":payload})
    )
}

#[test]
fn cadence_commands_require_confirmed_possession() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("session");
    // Act
    let arm = worker.prepare_frame(
        frame("telemetry_cadence_arm", json!({"phase":"idle"})).as_bytes(),
        1000,
    );
    let review = worker.prepare_frame(
        frame("telemetry_cadence_review", json!({})).as_bytes(),
        1000,
    );
    let endpoint = worker.prepare_frame(
        frame("telemetry_cadence_endpoint", json!({})).as_bytes(),
        1000,
    );
    // Assert
    assert!(arm.is_err() && review.is_err() && endpoint.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn arm_records_only_diagnostic_effect_without_lease() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let response = worker
        .prepare_frame(
            frame("telemetry_cadence_arm", json!({"phase":"mining"})).as_bytes(),
            1001,
        )
        .expect("arm");
    let value: serde_json::Value = serde_json::from_slice(response.frame()).expect("json");
    // Assert
    assert_eq!(value["result"]["generation"], 7);
    assert_eq!(value["result"]["phase"], "mining");
    assert_eq!(worker.session().events, ["cadence_arm"]);
    assert!(!worker.has_active_lease());
}

#[test]
fn cadence_review_never_exposes_private_endpoint() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let response = worker
        .prepare_frame(
            frame("telemetry_cadence_review", json!({})).as_bytes(),
            1001,
        )
        .expect("review");
    let value: serde_json::Value = serde_json::from_slice(response.frame()).expect("json");
    // Assert
    assert_eq!(value["result"]["schema"], "worker-telemetry-cadence-v2");
    assert!(value["result"].get("ipv4").is_none());
    assert!(worker.session().events.is_empty());
}

#[test]
fn endpoint_is_only_available_through_explicit_private_command() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let response = worker
        .prepare_frame(
            frame("telemetry_cadence_endpoint", json!({})).as_bytes(),
            1001,
        )
        .expect("endpoint");
    let value: serde_json::Value = serde_json::from_slice(response.frame()).expect("json");
    // Assert
    assert_eq!(value["result"]["ipv4"], "192.0.2.1");
    assert_eq!(value["result"]["bootOrdinal"], 2);
    assert!(worker.session().events.is_empty());
}

#[test]
fn cadence_commands_reject_active_work() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1001)
        .expect("start");
    // Act
    for (command, payload) in [
        ("telemetry_cadence_arm", json!({"phase":"idle"})),
        ("telemetry_cadence_review", json!({})),
        ("telemetry_cadence_endpoint", json!({})),
    ] {
        let result = worker.prepare_frame(frame(command, payload).as_bytes(), 1002);
        // Assert
        assert_eq!(
            result.expect_err("active rejected").category(),
            "invalid_transition"
        );
    }
}

#[test]
fn cadence_commands_reject_expired_possession() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let result = worker.prepare_frame(
        frame("telemetry_cadence_arm", json!({"phase":"idle"})).as_bytes(),
        61001,
    );
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn cadence_commands_reject_unrecognized_payload_fields() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let result = worker.prepare_frame(
        frame(
            "telemetry_cadence_arm",
            json!({"phase":"idle", "durationMs":1}),
        )
        .as_bytes(),
        1001,
    );
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn cadence_commands_reject_unfinished_cooling_effects() {
    // Arrange
    let mut worker = admitted_worker();
    let cooling = worker
        .prepare_frame(
            frame("qualification_cooling", json!({"action":"prove_fan"})).as_bytes(),
            1001,
        )
        .expect("fan proof");
    worker.confirm_sent(cooling).expect("delivered");
    // Act
    let result = worker.prepare_frame(
        frame("telemetry_cadence_arm", json!({"phase":"idle"})).as_bytes(),
        1002,
    );
    // Assert
    assert!(result.is_err());
    assert_eq!(worker.session().events, ["cooling"]);
}
