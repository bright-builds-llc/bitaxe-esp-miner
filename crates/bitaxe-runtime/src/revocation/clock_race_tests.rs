//! Stamps written by another thread after a check read its clock are fresh, never
//! wrapped into a ~49.7-day silence (the 2026-10-14 soak start revocation).
use super::*;

#[test]
fn a_heartbeat_stamped_after_the_checks_clock_reading_is_fresh() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(100).expect("fresh generation");
    assert!(gate.admit_budget(generation, 180_100));
    assert!(gate.activate(generation));
    assert!(gate.heartbeat(generation, 1_002));
    // Act
    gate.check_deadline(1_000);
    // Assert
    assert_eq!(gate.maybe_revoked(), None);
}
