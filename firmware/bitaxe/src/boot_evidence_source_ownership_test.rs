const BOOT_EVIDENCE_SOURCE: &str = include_str!("boot_evidence.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

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

#[test]
fn platform_readiness_avoids_the_full_operator_snapshot_stack() {
    // Arrange
    let function = "fn publish_platform_readiness(";
    let start = STARTUP_SOURCE
        .find(function)
        .expect("platform-readiness publisher must exist");
    let end = STARTUP_SOURCE[start..]
        .find("const fn readiness_label(")
        .map(|offset| start + offset)
        .expect("platform-readiness publisher boundary must exist");

    // Act
    let publisher = &STARTUP_SOURCE[start..end];

    // Assert
    assert!(publisher.contains("runtime_snapshot::collect_platform_readiness_snapshot()"));
    assert!(!publisher.contains("collect_api_snapshot()"));
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains(
        "collect_platform_snapshot(PlatformSnapshot::safe_ultra_205(), &platform_identity)",
    ));
}
