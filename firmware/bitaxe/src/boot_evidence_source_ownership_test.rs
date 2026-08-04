const BOOT_EVIDENCE_SOURCE: &str = include_str!("boot_evidence.rs");

#[test]
fn boot_lifetime_observer_owns_one_explicit_complex_runtime_stack() {
    // Arrange
    let expected_budget = "const OBSERVER_THREAD_STACK_BYTES: usize = 16 * 1024;";

    // Act
    let budget_declarations = BOOT_EVIDENCE_SOURCE
        .matches("const OBSERVER_THREAD_STACK_BYTES: usize")
        .count();
    let budget_uses = BOOT_EVIDENCE_SOURCE
        .matches(".stack_size(OBSERVER_THREAD_STACK_BYTES)")
        .count();

    // Assert
    assert!(BOOT_EVIDENCE_SOURCE.contains(expected_budget));
    assert_eq!(budget_declarations, 1);
    assert_eq!(budget_uses, 1);
    assert_eq!(BOOT_EVIDENCE_SOURCE.matches(".spawn(observe_boot_lifetime)").count(), 1);
}

#[test]
fn boot_identity_replay_keeps_the_ten_second_contract() {
    // Arrange
    let observer_start = BOOT_EVIDENCE_SOURCE
        .find("fn observe_boot_lifetime()")
        .expect("boot-lifetime observer must exist");
    let observer_end = BOOT_EVIDENCE_SOURCE[observer_start..]
        .find("fn runtime_attestation(")
        .map(|offset| observer_start + offset)
        .expect("observer boundary must exist");

    // Act
    let observer = &BOOT_EVIDENCE_SOURCE[observer_start..observer_end];

    // Assert
    assert!(observer.contains(
        "let mut identity_deadline_ms = started_at_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);",
    ));
    assert!(observer.contains(
        "identity_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);",
    ));
    assert_eq!(observer.matches("emit_boot_identity(").count(), 1);
}
