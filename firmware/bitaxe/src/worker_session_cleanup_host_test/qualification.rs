use super::*;
fn exhausted_original() -> AcceptanceBudget {
    let mut ledger = AcceptanceBudget::new("AAAAAAAAAAAAAAAAAAAAAA").expect("fixture");
    for (index, ms) in [180000, 30000, 30000].into_iter().enumerate() {
        ledger = ledger
            .reserve("AAAAAAAAAAAAAAAAAAAAAA", index as u8, ms)
            .expect("reserve")
            .finish()
            .expect("finish");
    }
    ledger
}
fn qualified_grant() -> WorkerLeaseGrant {
    let mut value = serde_json::json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"fixture","challengeId":"fixture","authorization":"synthetic","durationMilliseconds":10000,"renewAfterMilliseconds":5000,"stratum":{"endpoint":"stratum+tcp://example.invalid:3333/","username":"fixture","password":"fixture"}});
    value["qualificationAttempt"] = serde_json::json!({"schema":"worker-qualification-attempt-v1","id":"AAAAAAAAAAAAAAAAAAAAAA","ordinal":1,"purpose":"diagnostic","maximumActiveMilliseconds":30000});
    serde_json::from_value(value).expect("fixture")
}
#[test]
fn separate_attempt_never_changes_original_campaign_and_cannot_replay() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    // Act
    worker_acceptance_budget::admit(scope.generation, &qualified_grant()).expect("admit");
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Assert
    assert_eq!(ledger(), original);
    let report = worker_qualification_budget::review().expect("review");
    assert_eq!(report["total_charged_ms"], 30000);
    assert_eq!(report["next_ordinal"], 2);
    assert_eq!(report["pending"], false);
    assert!(worker_acceptance_budget::admit(scope.generation, &qualified_grant()).is_err());
}
#[test]
fn qualification_failed_write_still_charges_full_allowance_during_cleanup() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    FAIL_WRITE.store(true, Ordering::SeqCst);
    // Act
    assert!(worker_acceptance_budget::admit(scope.generation, &qualified_grant()).is_err());
    FAIL_WRITE.store(false, Ordering::SeqCst);
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Assert
    assert_eq!(ledger(), original);
    assert_eq!(
        worker_qualification_budget::review().expect("review")["total_charged_ms"],
        30000
    );
}
#[test]
fn qualification_revocation_during_write_rejects_activation_and_retains_charge() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    INTERRUPT_WRITE.store(true, Ordering::SeqCst);
    // Act
    assert!(worker_acceptance_budget::admit(scope.generation, &qualified_grant()).is_err());
    // Assert
    assert!(CLEANUP_WAS_BUSY.load(Ordering::SeqCst));
    assert!(!revocation::permits(Some(scope.generation)));
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    assert_eq!(ledger(), original);
    assert_eq!(
        worker_qualification_budget::review().expect("review")["total_charged_ms"],
        30000
    );
}
#[test]
fn qualification_boot_recovery_completes_pending_without_touching_original() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    worker_acceptance_budget::admit(scope.generation, &qualified_grant()).expect("admit");
    // Act
    worker_acceptance_budget::recover_after_boot(&startup::BootMiningBaselineConfirmed)
        .expect("recover");
    // Assert
    assert_eq!(ledger(), original);
    let report = worker_qualification_budget::review().expect("review");
    assert_eq!(report["total_charged_ms"], 30000);
    assert_eq!(report["pending"], false);
}
#[test]
fn original_campaign_must_be_exhausted_before_separate_attempt() {
    // Arrange
    let scope = Scope::new();
    // Act / Assert
    assert!(worker_acceptance_budget::admit(scope.generation, &qualified_grant()).is_err());
    assert_eq!(
        worker_qualification_budget::review().expect("review")["total_charged_ms"],
        0
    );
}

#[test]
fn qualification_ambiguous_readback_keeps_charge_and_blocks_early_ack() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    FAIL_AFTER_WRITE.store(true, Ordering::SeqCst);
    // Act
    assert!(worker_acceptance_budget::admit(scope.generation, &qualified_grant()).is_err());
    assert!(worker_acceptance_budget::finish(scope.generation).is_err());
    FAIL_AFTER_WRITE.store(false, Ordering::SeqCst);
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Assert
    assert_eq!(ledger(), original);
    let report = worker_qualification_budget::review().expect("review");
    assert_eq!(report["total_charged_ms"], 30000);
    assert_eq!(report["pending"], false);
}
