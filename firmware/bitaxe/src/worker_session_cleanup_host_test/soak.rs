//! Real soak budget adapter (ADR-0033) through the acceptance dispatcher, with synthetic NVS.
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
fn soak_grant(ordinal: u32) -> WorkerLeaseGrant {
    serde_json::from_value(serde_json::json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"fixture","challengeId":"fixture",
        "authorization":"synthetic","durationMilliseconds":60000,"renewAfterMilliseconds":20000,
        "stratum":{"endpoint":"stratum+tcp://example.invalid:3333/","username":"fixture","password":"fixture"},
        "hardwareProfile":"upstream-default",
        "soakAllowance":{"schema":"worker-soak-allowance-v1","id":"AAAAAAAAAAAAAAAAAAAAAA","ordinal":ordinal,"maximumActiveMilliseconds":619050}}))
    .expect("fixture")
}
fn soak_report() -> serde_json::Value {
    worker_soak_budget::review().expect("review")
}
#[test]
fn a_soak_charges_its_own_ledger_once_and_leaves_the_others_unchanged() {
    // Arrange
    let scope = Scope::new();
    let original = exhausted_original();
    *LEDGER.lock().expect("ledger") = Some(original.clone());
    // Act
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit");
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Assert
    assert_eq!(ledger(), original);
    assert_eq!(worker_qualification_budget::review().expect("review")["total_charged_ms"], 0);
    let report = soak_report();
    assert_eq!((report["total_charged_ms"].clone(), report["next_ordinal"].clone(), report["pending"].clone()),
        (serde_json::json!(619050), serde_json::json!(2), serde_json::json!(false)));
}
#[test]
fn a_replayed_soak_ordinal_is_rejected() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit");
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Act / Assert
    assert!(worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).is_err());
}
#[test]
fn a_failed_soak_write_never_activates_but_still_charges_during_cleanup() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    FAIL_WRITE.store(true, Ordering::SeqCst);
    // Act
    assert!(worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).is_err());
    FAIL_WRITE.store(false, Ordering::SeqCst);
    worker_acceptance_budget::finish(scope.generation).expect("finish");
    // Assert
    assert_eq!(soak_report()["total_charged_ms"], 619050);
    assert!(!revocation::activate(scope.generation, 1_000));
}
#[test]
fn boot_recovery_completes_a_pending_soak_without_refund() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit");
    // Act
    worker_acceptance_budget::recover_after_boot(&startup::BootMiningBaselineConfirmed).expect("recover");
    // Assert
    let report = soak_report();
    assert_eq!((report["total_charged_ms"].clone(), report["pending"].clone()), (serde_json::json!(619050), serde_json::json!(false)));
}
#[test]
fn a_soak_needs_the_exhausted_original_campaign() {
    // Arrange
    let scope = Scope::new();
    // Act / Assert
    assert!(worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).is_err());
    assert_eq!(soak_report()["total_charged_ms"], 0);
}
#[test]
fn the_soak_observation_reports_the_full_reservation_for_its_generation_only() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit");
    // Act
    let observed = worker_soak_budget::observation(scope.generation.raw(), 1_000).expect("observation");
    // Assert
    assert_eq!(observed["reserved_ms"], 619050);
    assert!(worker_soak_budget::observation(scope.generation.raw().wrapping_add(8), 1_000).is_none());
}
fn qualified_grant() -> WorkerLeaseGrant {
    serde_json::from_value(serde_json::json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"fixture","challengeId":"fixture",
        "authorization":"synthetic","durationMilliseconds":10000,"renewAfterMilliseconds":5000,
        "stratum":{"endpoint":"stratum+tcp://example.invalid:3333/","username":"fixture","password":"fixture"},
        "qualificationAttempt":{"schema":"worker-qualification-attempt-v1","id":"AAAAAAAAAAAAAAAAAAAAAA","ordinal":1,"purpose":"diagnostic","maximumActiveMilliseconds":30000}}))
    .expect("fixture")
}
#[test]
fn a_pending_soak_refuses_a_qualification() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit soak");
    let pending = worker_soak_budget::review().expect("review");
    assert_eq!(pending["pending"], true);
    // Act
    let refused = worker_qualification_budget::admit(scope.generation, qualified_grant().maybe_qualification_attempt().expect("attempt"));
    // Assert
    assert!(refused.is_err());
    assert_eq!(worker_qualification_budget::review().expect("review")["total_charged_ms"], 0);
}
#[test]
fn a_failed_soak_recovery_blocks_only_soaks_and_not_boot_recovery() {
    // Arrange
    let scope = Scope::new();
    *LEDGER.lock().expect("ledger") = Some(exhausted_original());
    worker_acceptance_budget::admit(scope.generation, &soak_grant(1)).expect("admit soak");
    FAIL_WRITE.store(true, Ordering::SeqCst);
    // Act
    let recovered = worker_acceptance_budget::recover_after_boot(&startup::BootMiningBaselineConfirmed);
    FAIL_WRITE.store(false, Ordering::SeqCst);
    let soak_refused = worker_soak_budget::admit(scope.generation, soak_grant(2).maybe_soak_allowance().expect("allowance"));
    // Assert
    assert!(recovered.is_ok());
    assert!(soak_refused.is_err());
    worker_soak_budget::recover_after_boot(&startup::BootMiningBaselineConfirmed);
}
