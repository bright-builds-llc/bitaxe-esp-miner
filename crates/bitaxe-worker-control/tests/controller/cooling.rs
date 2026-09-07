use super::*;
fn frame(action: &str) -> String {
    format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_cooling", "command":"qualification_cooling","payload":{"action":action}})
    )
}
#[test]
fn fan_qualification_requires_fresh_possession() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("session");
    // Act
    let result = worker.prepare_frame(frame("prove_fan").as_bytes(), 1000);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}
#[test]
fn fan_only_effect_requires_restoration_and_fresh_possession_before_budget_review() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let proof = worker
        .prepare_frame(frame("prove_fan").as_bytes(), 1001)
        .expect("fan proof");
    worker.confirm_sent(proof).expect("delivered");
    let blocked = worker.prepare_frame(
        budget_review::frame(&URL_SAFE_NO_PAD.encode([7_u8; 16])).as_bytes(),
        1002,
    );
    let restored = worker
        .prepare_frame(frame("restore_baseline").as_bytes(), 1003)
        .expect("fan restored");
    worker.confirm_sent(restored).expect("delivered");
    let stale = worker.prepare_frame(
        budget_review::frame(&URL_SAFE_NO_PAD.encode([7_u8; 16])).as_bytes(),
        1004,
    );
    // Assert
    assert!(blocked.is_err());
    assert!(stale.is_err());
    assert_eq!(worker.session().events, vec!["cooling", "restore_cooling"]);
    assert!(!worker.has_active_lease());
}
#[test]
fn fan_proof_failure_runs_cleanup_without_starting_a_lease() {
    // Arrange
    let mut worker = admitted_worker();
    worker.session_mut().fail_cooling = true;
    // Act
    let result = worker.prepare_frame(frame("prove_fan").as_bytes(), 1001);
    // Assert
    assert!(result.is_err());
    assert_eq!(worker.session().events, vec!["cooling", "control_failed"]);
    assert!(!worker.has_active_lease());
}
#[test]
fn baseline_restore_without_owned_fan_effect_is_rejected() {
    // Arrange
    let mut worker = admitted_worker();
    // Act
    let result = worker.prepare_frame(frame("restore_baseline").as_bytes(), 1001);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn fan_qualification_waits_for_boot_restoration_acknowledgement() {
    // Arrange
    let mut worker = worker_with_restoration(Some(RestorationReason::Reboot));
    admit(&mut worker, 1000);
    // Act
    let result = worker.prepare_frame(frame("prove_fan").as_bytes(), 1001);
    // Assert
    assert!(result.is_err());
    assert!(worker.session().events.is_empty());
}
