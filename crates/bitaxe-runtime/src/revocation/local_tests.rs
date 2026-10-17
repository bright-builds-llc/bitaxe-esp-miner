//! Local generations keep every device-local stop and have no Gate deadlines (ADR-0038).
use super::*;

fn active_local() -> (GenerationGate, WorkerGeneration) {
    let gate = GenerationGate::new();
    let generation = gate.begin_local().expect("local generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    (gate, generation)
}

#[test]
fn a_local_generation_outlives_the_heartbeat_cutoff() {
    // Arrange
    let (gate, generation) = active_local();
    gate.check_safety(true, true, 9_000);
    // Act
    gate.check_deadline(10_000);
    // Assert
    assert!(gate.permits(Some(generation)));
}

#[test]
fn a_linked_generation_still_ends_at_the_heartbeat_cutoff() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    // Act
    gate.check_deadline(u64::from(HEARTBEAT_CUTOFF_MS));
    // Assert
    assert_eq!(gate.maybe_revoked(), Some(generation));
}

#[test]
fn an_unsafe_reading_revokes_a_local_generation() {
    // Arrange
    let (gate, generation) = active_local();
    // Act
    gate.check_safety(false, true, 500);
    // Assert
    assert_eq!(gate.maybe_revoked(), Some(generation));
}

#[test]
fn a_zero_fan_reading_after_the_fan_proof_revokes_a_local_generation() {
    // Arrange: preparation proves the fan before mining, as for a lease.
    let (gate, generation) = active_local();
    gate.note_fan_proof(generation, 400);
    // Act
    gate.check_safety(true, false, 500);
    // Assert
    assert_eq!(gate.maybe_revoked(), Some(generation));
}

#[test]
fn safety_silence_after_the_fan_proof_revokes_a_local_generation() {
    // Arrange
    let (gate, generation) = active_local();
    gate.note_fan_proof(generation, 1_000);
    // Act
    gate.check_deadline(2_001);
    // Assert
    assert_eq!(gate.maybe_revoked(), Some(generation));
}

#[test]
fn a_local_generation_is_refused_beside_a_gate_link() {
    // Arrange: handing over between a link and local mining is ADR-0038 step 4.
    let gate = GenerationGate::new();
    gate.begin_link(0).expect("link");
    // Act
    let maybe_local = gate.begin_local();
    // Assert
    assert!(maybe_local.is_none());
}

#[test]
fn a_gate_link_is_refused_beside_a_local_generation() {
    // Arrange
    let (gate, _generation) = active_local();
    // Act
    let maybe_link = gate.begin_link(0);
    // Assert
    assert!(maybe_link.is_none());
}

#[test]
fn a_later_link_is_not_mistaken_for_the_finished_local_generation() {
    // Arrange
    let (gate, local) = active_local();
    gate.revoke_at(local, 100);
    gate.finish_shutdown(local);
    let link = gate.begin_link(200).expect("link after local mining");
    // Act
    let local_flag = gate.is_local(link);
    // Assert
    assert!(!local_flag);
}
