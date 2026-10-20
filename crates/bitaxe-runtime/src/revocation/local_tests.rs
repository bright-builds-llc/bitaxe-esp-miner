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

#[test]
fn a_link_holds_the_gate() {
    // Arrange
    let gate = GenerationGate::new();
    gate.begin_link(0).expect("link");
    // Act / Assert
    assert!(gate.holds_link());
}

#[test]
fn a_local_generation_is_not_a_link() {
    // Arrange / Act
    let (gate, _generation) = active_local();
    // Assert
    assert!(!gate.holds_link());
}

#[test]
fn a_finished_generation_holds_nothing() {
    // Arrange
    let gate = GenerationGate::new();
    let link = gate.begin_link(0).expect("link");
    gate.revoke_at(link, 1);
    // Act
    gate.finish_shutdown(link);
    // Assert
    assert!(!gate.holds_link());
}

#[test]
fn a_link_waits_until_a_stopping_local_generation_is_retired() {
    // Arrange: the handover revoked the local generation; its safe-stop runs.
    let (gate, local) = active_local();
    gate.revoke_at(local, 100);
    let while_stopping = gate.begin_link(200);
    // Act
    gate.finish_shutdown(local);
    let after_retirement = gate.begin_link(300);
    // Assert
    assert!(while_stopping.is_none() && after_retirement.is_some());
}

/// The local overheat trip in milli-degrees, as the sensor path notes readings.
fn trip_milli() -> i32 {
    (crate::local_mining::LOCAL_OVERHEAT_TRIP_C * 1_000.0).round() as i32
}

#[cfg(feature = "overheat-trip-test")]
#[test]
fn a_local_generation_stops_at_the_local_overheat_trip() {
    // Arrange
    let (gate, generation) = active_local();
    gate.note_chip_temperature(Some(trip_milli()));
    // Act
    gate.check_safety(true, true, 500);
    // Assert
    assert!(!gate.permits(Some(generation)));
    assert_eq!(
        gate.maybe_unsafe_detail()
            .and_then(|detail| detail.maybe_chip_temp_milli),
        Some(trip_milli())
    );
}

#[test]
fn a_local_generation_below_the_trip_keeps_mining() {
    // Arrange
    let (gate, generation) = active_local();
    gate.note_chip_temperature(Some(trip_milli() - 1));
    // Act
    gate.check_safety(true, true, 500);
    // Assert
    assert!(gate.permits(Some(generation)));
}

#[test]
fn a_local_generation_without_a_fresh_chip_reading_keeps_mining() {
    // Arrange
    let (gate, generation) = active_local();
    gate.note_chip_temperature(None);
    // Act
    gate.check_safety(true, true, 500);
    // Assert
    assert!(gate.permits(Some(generation)));
}

#[test]
fn a_linked_generation_ignores_the_local_overheat_trip() {
    // Arrange: a lease stops only on the shared verdict, which this safe sample passed.
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    gate.note_chip_temperature(Some(trip_milli()));
    // Act
    gate.check_safety(true, true, 500);
    // Assert
    assert!(gate.permits(Some(generation)));
}

#[cfg(feature = "overheat-trip-test")]
#[test]
fn a_local_overheat_stop_is_labelled_an_overheat() {
    // Arrange
    let (gate, _generation) = active_local();
    gate.note_chip_temperature(Some(trip_milli()));
    gate.check_safety(true, true, 500);
    let detail = gate.maybe_unsafe_detail().expect("detail");
    // Act
    let overheat = crate::local_mining::is_overheat(
        RevocationReason::UnsafeObservation,
        detail
            .maybe_chip_temp_milli
            .map(|milli| f64::from(milli) / 1_000.0),
    );
    // Assert
    assert!(overheat);
}
