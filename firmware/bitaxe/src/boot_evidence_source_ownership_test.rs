const BOOT_EVIDENCE_SOURCE: &str = include_str!("boot_evidence.rs");

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
