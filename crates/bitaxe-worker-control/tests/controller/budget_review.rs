use super::*;

fn frame(campaign: &str) -> String {
    format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_budget_review","command":"acceptance_budget_review","payload":{"campaignId":campaign}})
    )
}

#[test]
fn review_requires_confirmed_fresh_possession() {
    // Arrange
    let mut worker = worker();
    let request = frame(&URL_SAFE_NO_PAD.encode([7_u8; 16]));
    worker
        .begin_serial_session(fixture_binding())
        .expect("session");
    // Act
    let result = worker.prepare_frame(request.as_bytes(), 1000);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn review_returns_closed_campaign_bound_facts_without_side_effects() {
    // Arrange
    let mut worker = admitted_worker();
    let campaign = URL_SAFE_NO_PAD.encode([7_u8; 16]);
    // Act
    let prepared = worker
        .prepare_frame(frame(&campaign).as_bytes(), 1001)
        .expect("review");
    let response: serde_json::Value = serde_json::from_slice(prepared.frame()).expect("JSON");
    // Assert
    assert_eq!(
        response["result"],
        json!({"schema":"worker-budget-review-v1","campaign_match":true,"reserved_mask":1,"completed_mask":1,"charged_ms":180000,"pending":false})
    );
    assert!(!String::from_utf8_lossy(prepared.frame()).contains(&campaign));
    assert!(worker.session().events.is_empty());
}

#[test]
fn wrong_campaign_is_reported_without_exposing_the_expected_identifier() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let prepared = worker
        .prepare_frame(frame(&URL_SAFE_NO_PAD.encode([8_u8; 16])).as_bytes(), 1001)
        .expect("review");
    let response: serde_json::Value = serde_json::from_slice(prepared.frame()).expect("JSON");
    // Assert
    assert_eq!(response["result"]["campaign_match"], false);
    assert!(worker.session().events.is_empty());
}

#[test]
fn expired_possession_cannot_review_a_budget() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let result = worker.prepare_frame(frame(&URL_SAFE_NO_PAD.encode([7_u8; 16])).as_bytes(), 61001);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn active_work_lease_cannot_review_a_budget() {
    // Arrange
    let mut worker = admitted_worker();
    let prepared = worker
        .prepare_frame(start_frame().as_bytes(), 1001)
        .expect("start");
    worker.confirm_sent(prepared).expect("sent");
    // Act
    let result = worker.prepare_frame(frame(&URL_SAFE_NO_PAD.encode([7_u8; 16])).as_bytes(), 1002);
    // Assert
    assert!(result.is_err());
    assert_eq!(worker.session().events, vec!["start"]);
}

#[test]
fn review_rejects_noncanonical_or_extra_input_fields() {
    // Arrange
    let mut worker = admitted_worker();
    let mut payload: serde_json::Value =
        serde_json::from_str(&frame(&URL_SAFE_NO_PAD.encode([7_u8; 16]))).expect("request");
    payload["payload"]["extra"] = true.into();
    // Act
    let extra = worker.prepare_frame(format!("{payload}\n").as_bytes(), 1001);
    let malformed = worker.prepare_frame(frame("not-a-campaign").as_bytes(), 1002);
    // Assert
    assert!(extra.is_err());
    assert!(malformed.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn authentication_failure_has_a_correlated_closed_response_and_never_starts_work() {
    // Arrange
    let mut worker = admitted_worker();
    let frame = start_frame().replace(
        "fixture-authentication-not-a-production-secret",
        "invalid-signature",
    );
    // Act
    let error = worker
        .prepare_frame(frame.as_bytes(), 1001)
        .expect_err("invalid authentication");
    let reply = worker
        .prepare_rejection(frame.as_bytes(), &error)
        .expect("closed rejection");
    let response: serde_json::Value = serde_json::from_slice(reply.frame()).expect("response");
    // Assert
    assert_eq!(response["ok"], false);
    assert_eq!(
        response["error"],
        json!({"code":"command_rejected","message":"authentication_failed"})
    );
    assert!(worker.session().events.is_empty());
    assert!(!String::from_utf8_lossy(reply.frame()).contains("invalid-signature"));
}

#[test]
fn malformed_requests_cannot_obtain_a_correlated_rejection() {
    // Arrange
    let worker = worker();
    // Act
    let result = worker.prepare_rejection(
        b"bad\n",
        &bitaxe_worker_control::WorkerControlError::InvalidFrame,
    );
    // Assert
    assert!(result.is_err());
}
