use bitaxe_worker_control::{QualificationAttempt, QualificationLedger};
fn attempt(ordinal: u32, purpose: &str, duration: u64) -> QualificationAttempt {
    serde_json::from_value(serde_json::json!({"schema":"worker-qualification-attempt-v1","id":"AAAAAAAAAAAAAAAAAAAAAA","ordinal":ordinal,"purpose":purpose,"maximumActiveMilliseconds":duration})).expect("fixture")
}
#[test]
fn pending_and_completed_reservations_never_refund_or_replay() {
    // Arrange
    let original = QualificationLedger::default();
    let grant = attempt(1, "diagnostic", 30000);
    // Act
    let reserved = original.reserve(&grant).expect("reserve");
    let complete = reserved.finish().expect("finish");
    // Assert
    assert!(reserved.reserve(&attempt(2, "diagnostic", 30000)).is_err());
    assert!(complete.reserve(&grant).is_err());
    assert_eq!(complete.total_charged_ms(), 30000);
    assert_eq!(complete.next_ordinal(), 2);
    assert!(!complete.pending());
}
#[test]
fn exact_purpose_limits_and_contiguous_ordinals_are_required() {
    // Arrange
    let ledger = QualificationLedger::default();
    // Act / Assert
    for grant in [
        attempt(1, "diagnostic", 180000),
        attempt(1, "normal", 30000),
        attempt(2, "diagnostic", 30000),
    ] {
        assert!(ledger.reserve(&grant).is_err());
    }
}
#[test]
fn restart_finalization_keeps_the_full_intended_charge() {
    // Arrange
    let reserved = QualificationLedger::default()
        .reserve(&attempt(1, "normal", 180000))
        .expect("reserve");
    let bytes = serde_json::to_vec(&reserved).expect("serialize");
    // Act
    let restored: QualificationLedger = serde_json::from_slice(&bytes).expect("parse");
    let finished = restored.finish().expect("recover");
    // Assert
    assert_eq!(finished.total_charged_ms(), 180000);
    assert_eq!(finished.last_completed_ordinal(), 1);
    assert_eq!(
        finished.finish().expect("idempotent").total_charged_ms(),
        180000
    );
}
#[test]
fn impossible_persisted_totals_fail_closed() {
    // Arrange
    for (ordinal, purpose, total) in [
        (1, "diagnostic", 180000),
        (2, "diagnostic", 90000),
        (2, "diagnostic", 360000),
    ] {
        let mut value = serde_json::to_value(
            QualificationLedger::default()
                .reserve(&attempt(1, purpose, 30000))
                .expect("reserve"),
        )
        .expect("json");
        value["highest_ordinal"] = serde_json::json!(ordinal);
        value["maybe_last"]["allowance"]["ordinal"] = serde_json::json!(ordinal);
        value["total_charged_ms"] = serde_json::json!(total);
        let ledger: QualificationLedger = serde_json::from_value(value).expect("stored shape");
        // Act / Assert
        assert!(ledger.validate().is_err());
    }
}
