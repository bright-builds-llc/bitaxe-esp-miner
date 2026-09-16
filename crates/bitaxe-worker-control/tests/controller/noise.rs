use super::*;
use bitaxe_worker_control::noise::*;

pub(super) fn observation() -> NoiseObservation {
    NoiseObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        transport_epoch: 4,
        observed_at_us: 1_000_000,
        station_ipv4: Some("192.168.1.2".into()),
        wifi_connected: true,
    }
}
fn frame(command: &str, payload: serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&json!({"protocolVersion":"bwg-worker-controller/0.4", "requestId":"serial_noise", "command":command, "payload":payload})).expect("frame");
    bytes.push(b'\n');
    bytes
}
fn query() -> Vec<u8> {
    frame(
        "noise_diagnostic_status",
        json!({"schema":"worker-noise-diagnostic-query-v1","attemptId":null}),
    )
}
fn start() -> Vec<u8> {
    frame(
        "noise_diagnostic_start",
        json!({"schema":"worker-noise-diagnostic-start-v2", "attemptId":URL_SAFE_NO_PAD.encode([1;16]),
        "expectedBootOrdinal":2,"networkObservedAtUs":1_000_000,"fixtureIpv4":"192.168.1.3","fixturePort":12345,"authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32])}),
    )
}
fn observed_worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = admitted_worker();
    let response = worker.prepare_frame(&query(), 1001).expect("observation");
    worker.confirm_sent_at(response, 1002).expect("delivered");
    worker
}
#[test]
fn noise_start_requires_delivered_endpoint_observation() {
    // Arrange
    let mut worker = admitted_worker();
    let _undelivered = worker.prepare_frame(&query(), 1001).expect("prepared");
    // Act / Assert
    assert!(worker.prepare_frame(&start(), 1002).is_err());
    assert!(worker.session().events.is_empty());
}
#[test]
fn noise_admission_consumes_without_dispatch_before_confirmed_reply() {
    // Arrange
    let mut worker = observed_worker();
    // Act
    let response = worker.prepare_frame(&start(), 1003).expect("admitted");
    // Assert
    assert_eq!(worker.session().events, ["noise_admitted"]);
    assert!(!worker.has_active_lease());
    worker
        .confirm_sent_at(response, 1004)
        .expect("dispatch after delivery");
    assert_eq!(
        worker.session().events,
        ["noise_admitted", "noise_dispatched"]
    );
}
#[test]
fn cancelled_lost_ack_never_dispatches_and_cannot_rearm() {
    // Arrange
    let mut worker = observed_worker();
    let response = worker.prepare_frame(&start(), 1003).expect("admitted");
    // Act
    worker.disconnect(1004).expect("disconnect");
    let late = worker.confirm_sent_at(response, 1005);
    // Assert
    assert!(late.is_err());
    assert_eq!(worker.session().events, ["noise_admitted"]);
    assert!(worker.session().noise_busy());
}
#[test]
fn a_running_noise_query_does_not_require_renewed_sixty_second_start_admission() {
    // Arrange
    let mut worker = observed_worker();
    let response = worker.prepare_frame(&start(), 1003).expect("admitted");
    worker.confirm_sent_at(response, 1004).expect("dispatch");
    // Act
    let status = worker.prepare_frame(&frame("noise_diagnostic_status", json!({"schema":"worker-noise-diagnostic-query-v1", "attemptId":URL_SAFE_NO_PAD.encode([1;16])})), 70_000);
    // Assert
    assert!(status.is_ok());
    assert!(!worker.has_active_lease());
}
#[test]
fn the_noise_fence_rejects_effectful_controller_commands() {
    // Arrange
    let mut worker = observed_worker();
    let _reply = worker.prepare_frame(&start(), 1003).expect("admitted");
    // Act / Assert
    for command in [
        "start_lease",
        "qualification_restart",
        "qualification_cooling",
        "telemetry_cadence_arm",
    ] {
        assert!(worker
            .prepare_frame(&frame(command, json!({})), 1004)
            .is_err());
    }
    assert_eq!(worker.session().events, ["noise_admitted"]);
}
#[test]
fn missing_query_attempt_field_is_not_accepted_as_explicit_null() {
    // Arrange
    let mut worker = admitted_worker();
    // Act / Assert
    assert!(worker
        .prepare_frame(
            &frame(
                "noise_diagnostic_status",
                json!({"schema":"worker-noise-diagnostic-query-v1"})
            ),
            1001
        )
        .is_err());
}

#[test]
fn same_session_terminal_collection_after_sixty_seconds_does_not_need_a_new_proof() {
    // Arrange
    let mut worker = observed_worker();
    let response = worker.prepare_frame(&start(), 1003).expect("admitted");
    worker.confirm_sent_at(response, 1004).expect("dispatch");
    let record = worker.session_mut().maybe_noise.as_mut().expect("job");
    record.fail(NoiseFailure::new(
        FailureStage::NoisePrepared,
        NoiseCategory::Preparation,
        NoiseDetail::Io,
        Some(61_000_000),
    ));
    record.joined(61_000_001, false);
    assert!(!worker.session().noise_busy());
    // Act
    let response = worker.prepare_frame(&frame("noise_diagnostic_status", json!({"schema":"worker-noise-diagnostic-query-v1", "attemptId":URL_SAFE_NO_PAD.encode([1;16])})), 70_000);
    // Assert
    assert!(response.is_ok());
    worker.disconnect(70_001).expect("disconnect");
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh logical session");
    assert!(worker.prepare_frame(&frame("noise_diagnostic_status", json!({"schema":"worker-noise-diagnostic-query-v1", "attemptId":URL_SAFE_NO_PAD.encode([1;16])})), 70_002).is_err());
}

#[test]
fn ordinary_restore_preserves_pending_noise_authority_until_actual_completion() {
    // Arrange
    let mut worker = observed_worker();
    let _response = worker.prepare_frame(&start(), 1003).expect("admitted");
    // Act
    let pending = worker.prepare_frame(&frame("restore", json!({"reason":"cancelled"})), 1004);
    // Assert: existing Controller rejection, not a new baseline status shape.
    assert_eq!(
        pending.expect_err("pending job").category(),
        "restoration_pending"
    );
    let query = frame(
        "noise_diagnostic_status",
        json!({"schema":"worker-noise-diagnostic-query-v1", "attemptId":URL_SAFE_NO_PAD.encode([1;16])}),
    );
    assert!(worker.prepare_frame(&query, 1005).is_ok());
    assert!(worker.prepare_frame(&frame("noise_diagnostic_cancel", json!({"schema":"worker-noise-diagnostic-query-v1", "attemptId":URL_SAFE_NO_PAD.encode([1;16])})), 1006).is_ok());
    worker
        .session_mut()
        .maybe_noise
        .as_mut()
        .expect("job")
        .joined(1_000_003, false);
    let response = worker
        .prepare_frame(&frame("restore", json!({"reason":"cancelled"})), 1007)
        .expect("normal restored response");
    let confirmed: serde_json::Value = serde_json::from_slice(response.frame()).expect("response");
    assert_eq!(confirmed["result"]["restoration"]["status"], "confirmed");
}
