use super::*;

fn frame(payload: serde_json::Value) -> String {
    format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4", "requestId":"serial_trace",
        "command":"serial_trace_review", "payload":payload})
    )
}

#[test]
fn trace_export_requires_fresh_confirmed_possession() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("session");
    // Act
    let result = worker.prepare_frame(frame(json!({})).as_bytes(), 1000);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn trace_export_returns_only_explicit_bounded_snapshot_without_effects() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let response = worker
        .prepare_frame(frame(json!({})).as_bytes(), 1001)
        .expect("explicit trace");
    let value: serde_json::Value = serde_json::from_slice(response.frame()).expect("JSON");
    // Assert
    assert_eq!(value["result"]["schema"], "worker-serial-trace-v1");
    assert_eq!(value["result"]["capacity"], 64);
    assert!(value["result"].get("qualification").is_none());
    assert!(value["result"].get("preservation").is_none());
    assert!(worker.session().events.is_empty());
    assert!(!worker.has_active_lease());
}

#[test]
fn trace_export_rejects_active_work() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1001)
        .expect("start");
    // Act
    let result = worker.prepare_frame(frame(json!({})).as_bytes(), 1002);
    // Assert
    assert_eq!(
        result.expect_err("active trace rejected").category(),
        "invalid_transition"
    );
}

#[test]
fn trace_export_rejects_expired_possession() {
    // Arrange
    let mut expired = admitted_worker();
    // Act
    let expired_result = expired.prepare_frame(frame(json!({})).as_bytes(), 61001);
    // Assert
    assert!(expired_result.is_err());
    assert!(expired.session().events.is_empty());
}

#[test]
fn trace_export_rejects_extra_payload_fields() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let result = worker.prepare_frame(frame(json!({"clear":true})).as_bytes(), 1001);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}
