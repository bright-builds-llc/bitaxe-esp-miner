use super::*;

fn renewal_frame(authorization: &str) -> String {
    format!(
        "{}\n",
        json!({
            "protocolVersion": "bwg-worker-controller/0.4",
            "requestId": "serial_renew_dispatch",
            "command": "renew_lease",
            "payload": {
                "protocolVersion": "bwg-worker-controller/0.4",
                "leaseId": "lease_fixture_03",
                "authorization": authorization,
                "durationMilliseconds": 60000,
                "renewAfterMilliseconds": 20000
            }
        })
    )
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
