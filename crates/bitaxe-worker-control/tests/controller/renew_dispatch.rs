use super::*;

fn renewal_frame(authorization: &str) -> String {
    renewal_frame_for(authorization, "lease_fixture_03", 60_000, 20_000)
}

fn renewal_frame_for(
    authorization: &str,
    lease_id: &str,
    duration: u64,
    renew_after: u64,
) -> String {
    format!(
        "{}\n",
        json!({
            "protocolVersion": "bwg-worker-controller/0.4",
            "requestId": "serial_renew_dispatch",
            "command": "renew_lease",
            "payload": {
                "protocolVersion": "bwg-worker-controller/0.4",
                "leaseId": lease_id,
                "authorization": authorization,
                "durationMilliseconds": duration,
                "renewAfterMilliseconds": renew_after
            }
        })
    )
}

/// A signed V2 Start on the fixture worker, as the STR-005 Worker issues it.
fn v2_started_worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = admitted_worker();
    // A V2 Start is admitted only after the current share scope was observed idle and delivered.
    let mut query = serde_json::to_vec(&json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_renew_query",
        "command":"stratum_v2_status","payload":{"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":null}}))
    .expect("query frame");
    query.push(b'\n');
    let observation = worker.prepare_frame(&query, 990).expect("idle share");
    worker.confirm_sent(observation).expect("delivered");
    let mut grant: serde_json::Value = serde_json::from_str(&start_frame()).expect("grant fixture");
    grant["payload"]["stratum"] = json!({"profile":"bwg-worker-stratum-v2-standard/0.1","endpoint":"stratum+tcp://192.168.1.3:12345/","authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"});
    grant["payload"]["qualificationAttempt"] = json!({"schema":"worker-qualification-attempt-v1","id":URL_SAFE_NO_PAD.encode([1;16]),"ordinal":18,"purpose":"normal","maximumActiveMilliseconds":180000});
    let mut bytes = serde_json::to_vec(&grant).expect("grant frame");
    bytes.push(b'\n');
    worker.prepare_frame(&bytes, 1000).expect("signed V2 Start");
    worker
}

#[test]
fn renew_dispatch_preserves_request_correlation_and_qualification_evidence() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1000)
        .expect("Start");
    let evidence = json!({"schema":"fixture-qualification-v1","generation":7});
    worker.session_mut().maybe_status_evidence = Some(evidence.clone());

    // Act
    let prepared = worker
        .prepare_frame(
            renewal_frame("fixture-renewal-authentication").as_bytes(),
            2000,
        )
        .expect("Renew");
    let response: serde_json::Value = serde_json::from_slice(prepared.frame()).expect("response");

    // Assert
    assert_eq!(response["requestId"], "serial_renew_dispatch");
    assert_eq!(response["ok"], true);
    assert_eq!(response["result"]["state"], "mining");
    assert_eq!(response["result"]["qualification"], evidence);
    assert_eq!(worker.session().events, ["start", "renew"]);
}

#[test]
fn renew_dispatch_authentication_failure_still_restores_before_returning_error() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1000)
        .expect("Start");

    // Act
    let result = worker.prepare_frame(
        renewal_frame("invalid-renewal-authentication").as_bytes(),
        2000,
    );

    // Assert
    assert_eq!(
        result.expect_err("invalid signature").category(),
        "authentication_failed"
    );
    assert_eq!(worker.session().events, ["start", "control_failed"]);
}

#[test]
fn renew_dispatch_without_active_lease_cannot_call_session_renew() {
    // Arrange
    let mut worker = admitted_worker();

    // Act
    let result = worker.prepare_frame(
        renewal_frame("fixture-renewal-authentication").as_bytes(),
        2000,
    );

    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn renewal_for_another_lease_fails_authentication_and_restores() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1000)
        .expect("Start");

    // Act
    let result = worker.prepare_frame(
        renewal_frame_for(
            "fixture-renewal-authentication",
            "lease_other",
            60_000,
            20_000,
        )
        .as_bytes(),
        2000,
    );

    // Assert
    assert_eq!(
        result.expect_err("foreign lease").category(),
        "authentication_failed"
    );
    assert_eq!(worker.session().events, ["start", "control_failed"]);
}

#[test]
fn v2_renewal_with_a_non_canonical_window_is_refused_before_session_renew() {
    // Arrange
    let mut worker = v2_started_worker();

    // Act: both windows pass general renewal validation; only the V2 rule refuses them.
    let shorter = worker.prepare_frame(
        renewal_frame_for(
            "fixture-renewal-authentication",
            "lease_fixture_03",
            30_000,
            10_000,
        )
        .as_bytes(),
        2000,
    );
    let faster = worker.prepare_frame(
        renewal_frame_for(
            "fixture-renewal-authentication",
            "lease_fixture_03",
            60_000,
            10_000,
        )
        .as_bytes(),
        2001,
    );

    // Assert
    assert_eq!(
        shorter.expect_err("30 s window").category(),
        "invalid_request"
    );
    assert_eq!(
        faster.expect_err("10 s cadence").category(),
        "invalid_request"
    );
    assert!(!worker.session().events.contains(&"renew"));
}

#[test]
fn renewal_after_the_lease_expired_cannot_reach_session_renew() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1000)
        .expect("Start");
    let tick = worker.tick(70_000);
    assert!(
        worker.session().events.contains(&"lease_expired"),
        "{tick:?}"
    );

    // Act
    let result = worker.prepare_frame(
        renewal_frame("fixture-renewal-authentication").as_bytes(),
        70_001,
    );

    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.contains(&"lease_expired"));
    assert!(!worker.session().events.contains(&"renew"));
}
