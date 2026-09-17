use super::*;
use bitaxe_worker_control::v2::*;
pub(super) fn observation() -> CurrentObservation {
    CurrentObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        serial_transport_epoch: 4,
        maybe_observed_at_us: Some(1_000_000),
        clock_valid: true,
        maybe_station_ipv4: Some("192.168.1.2".into()),
        wifi_connected: true,
        maybe_socket: None,
    }
}
fn frame(command: &str, payload: serde_json::Value) -> Vec<u8> {
    let mut f=serde_json::to_vec(&json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_v2","command":command,"payload":payload})).expect("frame");
    f.push(b'\n');
    f
}
fn query() -> Vec<u8> {
    frame(
        "stratum_v2_status",
        json!({"schema":"worker-stratum-v2-query-v1","scope":"channel","attemptId":null}),
    )
}
fn start() -> Vec<u8> {
    frame(
        "stratum_v2_channel_start",
        json!({"schema":"worker-stratum-v2-channel-start-v1","attemptId":URL_SAFE_NO_PAD.encode([1;16]),"expectedBootOrdinal":2,"networkObservedAtUs":1_000_000,"stratum":{"profile":"bwg-worker-stratum-v2-standard/0.1","endpoint":"stratum+tcp://192.168.1.3:12345/","authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"}}),
    )
}
fn observed() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut w = admitted_worker();
    let response = w.prepare_frame(&query(), 1001).expect("observation");
    w.confirm_sent_at(response, 1002).expect("delivered");
    w
}
#[test]
fn delivered_observation_and_delivered_ack_are_distinct_admission_boundaries() {
    // Arrange
    let mut w = admitted_worker();
    let _unsent = w.prepare_frame(&query(), 1001).expect("unsent observation");
    assert!(w.prepare_frame(&start(), 1002).is_err());
    let mut w = observed();
    // Act
    let reply = w.prepare_frame(&start(), 1003).expect("admission");
    // Assert
    assert_eq!(w.session().events, ["v2_admitted"]);
    assert!(!w.has_active_lease());
    w.confirm_sent_at(reply, 1004).expect("actual delivery");
    assert_eq!(w.session().events, ["v2_admitted", "v2_dispatched"]);
}
#[test]
fn lost_ack_and_replaced_session_cannot_dispatch() {
    // Arrange
    let mut w = observed();
    let reply = w.prepare_frame(&start(), 1003).expect("admitted");
    // Act
    w.disconnect(1004).expect("disconnect");
    // Assert
    assert!(w.confirm_sent_at(reply, 1005).is_err());
    assert_eq!(w.session().events, ["v2_admitted"]);
}
#[test]
fn active_channel_keeps_its_admitted_binding_past_sixty_seconds() {
    // Arrange
    let mut w = observed();
    let reply = w.prepare_frame(&start(), 1003).expect("admitted");
    w.confirm_sent_at(reply, 1004).expect("sent");
    // Act
    let q = frame(
        "stratum_v2_status",
        json!({"schema":"worker-stratum-v2-query-v1","scope":"channel","attemptId":URL_SAFE_NO_PAD.encode([1;16])}),
    );
    // Assert
    assert!(w.prepare_frame(&q, 70_000).is_ok());
    assert!(!w.has_active_lease());
}
#[test]
fn wrong_attempt_and_effectful_reentry_fail_while_channel_is_fenced() {
    // Arrange
    let mut w = observed();
    let _reply = w.prepare_frame(&start(), 1003).expect("admitted");
    // Act / Assert
    assert!(w.prepare_frame(&frame("stratum_v2_status",json!({"schema":"worker-stratum-v2-query-v1","scope":"channel","attemptId":URL_SAFE_NO_PAD.encode([9;16])})),1004).is_err());
    assert!(w.prepare_frame(&start(), 1005).is_err());
    assert!(w.prepare_frame(start_frame().as_bytes(), 1006).is_err());
}

#[test]
fn blocked_network_cleanup_preserves_read_only_reconnect_but_denies_new_effects() {
    // Arrange: the fake native stop executes, but its opaque network job has not returned.
    let mut w = admitted_worker();
    let observation = w
        .prepare_frame(
            &frame(
                "stratum_v2_status",
                json!({"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":null}),
            ),
            1001,
        )
        .expect("idle share");
    w.confirm_sent(observation).expect("delivered");
    let mut grant: serde_json::Value = serde_json::from_str(&start_frame()).expect("grant fixture");
    grant["payload"]["stratum"] = json!({"profile":"bwg-worker-stratum-v2-standard/0.1","endpoint":"stratum+tcp://192.168.1.3:12345/","authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"});
    grant["payload"]["qualificationAttempt"] = json!({"schema":"worker-qualification-attempt-v1","id":URL_SAFE_NO_PAD.encode([1;16]),"ordinal":18,"purpose":"normal","maximumActiveMilliseconds":180000});
    let mut bytes = serde_json::to_vec(&grant).expect("grant frame");
    bytes.push(b'\n');
    w.prepare_frame(&bytes, 1002).expect("signed start");
    w.session_mut().v2_cleanup_blocked = true;
    let query = frame(
        "stratum_v2_status",
        json!({"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":URL_SAFE_NO_PAD.encode([1;16])}),
    );
    // Act / Assert
    assert!(w.tick(70_000).is_err());
    assert!(w.session().events.contains(&"lease_expired"));
    assert!(w.prepare_frame(&query, 70_001).is_ok());
    assert!(w.prepare_frame(start_frame().as_bytes(), 70_002).is_err());
    assert!(w
        .prepare_frame(&frame("qualification_restart", json!({})), 70_003)
        .is_err());
    assert!(w.disconnect(70_004).is_err());
    admit(&mut w, 70_005);
    assert!(
        w.prepare_frame(&query, 140_000).is_ok(),
        "new authenticated read binding remains valid past60s"
    );
    assert!(w.prepare_frame(&bytes, 140_001).is_err());
    w.session_mut().v2_cleanup_blocked = false;
    w.tick(140_002).expect("actual completion permits cleanup");
    assert!(!w.has_active_lease());
    assert!(w.prepare_frame(&query, 140_003).is_ok());
    assert!(
        w.prepare_frame(start_frame().as_bytes(), 140_004).is_err(),
        "observation is not new work admission"
    );
}
