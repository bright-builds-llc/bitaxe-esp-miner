use super::super::*;

fn active_gate() -> (GenerationGate, WorkerGeneration) {
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    (gate, generation)
}

fn bus_voltage_verdict(value_milli: i64) -> bitaxe_api::SafetyVerdict {
    bitaxe_api::SafetyVerdict {
        fact: bitaxe_api::SafetyFact::BusVoltage,
        state: bitaxe_api::SafetyFactState::OutOfRange,
        maybe_value_milli: Some(value_milli),
        maybe_age_ms: Some(40),
    }
}

#[test]
fn an_unsafe_sample_records_its_first_failing_fact() {
    // Arrange
    let (gate, generation) = active_gate();
    gate.check_safety(true, true, 400);
    // Act
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Assert
    assert_eq!(
        gate.maybe_unsafe_detail(),
        Some(UnsafeObservationDetail {
            generation: generation.raw(),
            trigger: UnsafeTrigger::UnsafeSample,
            maybe_verdict: Some(bus_voltage_verdict(5_512)),
            since_safe_ms: 100,
            closed_ms: 500,
            maybe_chip_temp_milli: None,
        })
    );
}

#[test]
fn a_zero_fan_after_fan_proof_records_the_zero_fan_trigger() {
    // Arrange
    let (gate, generation) = active_gate();
    gate.note_fan_proof(generation, 200);
    // Act
    gate.check_safety_verdict(None, false, 300);
    // Assert
    let detail = gate.maybe_unsafe_detail().expect("detail");
    assert_eq!(
        (detail.trigger, detail.maybe_verdict, detail.since_safe_ms),
        (UnsafeTrigger::ZeroFan, None, 100)
    );
}

#[test]
fn a_missing_safe_sample_records_its_silence() {
    // Arrange
    let (gate, generation) = active_gate();
    assert!(gate.heartbeat(generation, 1_000));
    gate.note_fan_proof(generation, 200);
    // Act
    gate.check_deadline(1_201);
    // Assert
    let detail = gate.maybe_unsafe_detail().expect("detail");
    assert_eq!(
        (detail.trigger, detail.since_safe_ms, detail.closed_ms),
        (UnsafeTrigger::NoSafeSample, 1_001, 1_201)
    );
}

#[test]
fn a_safe_sample_stamped_after_the_checks_clock_reading_is_fresh() {
    // Arrange: the 2026-10-14 soak race. A safety sample stamped 2 ms after the
    // deadline check read its clock once looked about 49.7 days old.
    let (gate, generation) = active_gate();
    assert!(gate.heartbeat(generation, 1_000));
    gate.note_fan_proof(generation, 1_202);
    // Act
    gate.check_deadline(1_200);
    // Assert
    assert_eq!(
        (gate.maybe_revoked(), gate.maybe_unsafe_detail()),
        (None, None)
    );
}

#[test]
fn a_later_unsafe_sample_cannot_replace_the_first_detail() {
    // Arrange
    let (gate, _generation) = active_gate();
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Act
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_600)), true, 600);
    // Assert
    let detail = gate.maybe_unsafe_detail().expect("detail");
    assert_eq!(detail.maybe_verdict, Some(bus_voltage_verdict(5_512)));
}

#[test]
fn another_revocation_reason_records_no_unsafe_detail() {
    // Arrange
    let (gate, _generation) = active_gate();
    // Act
    gate.check_deadline(2_800);
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 2_900);
    // Assert
    assert_eq!(gate.maybe_unsafe_detail(), None);
}

#[test]
fn the_detail_marker_is_a_closed_single_line() {
    // Arrange
    let (gate, _generation) = active_gate();
    gate.check_safety(true, true, 400);
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Act
    let marker = gate.maybe_unsafe_detail().expect("detail").marker();
    // Assert
    assert_eq!(marker, "worker_revocation_detail schema=v1 generation=1 reason=unsafe_observation trigger=unsafe_sample fact=bus_voltage state=out_of_range value_milli=5512 age_ms=40 since_safe_ms=100 closed_ms=500 redacted=true");
}

#[test]
fn a_detail_without_a_verdict_marks_its_fact_none() {
    // Arrange
    let (gate, generation) = active_gate();
    gate.note_fan_proof(generation, 200);
    gate.check_safety_verdict(None, false, 300);
    // Act
    let marker = gate.maybe_unsafe_detail().expect("detail").marker();
    // Assert
    assert!(marker.contains(
        " trigger=zero_fan fact=none state=none value_milli=unavailable age_ms=unavailable "
    ));
}

#[test]
fn a_revocation_records_the_chip_temperature_of_the_sample_that_revoked() {
    // Arrange
    let (gate, _generation) = active_gate();
    gate.check_safety(true, true, 400);
    gate.note_chip_temperature(Some(76_000));
    // Act
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Assert
    assert_eq!(
        gate.maybe_unsafe_detail()
            .and_then(|detail| detail.maybe_chip_temp_milli),
        Some(76_000)
    );
}

#[test]
fn a_sample_without_a_fresh_chip_reading_records_none() {
    // Arrange
    let (gate, _generation) = active_gate();
    gate.note_chip_temperature(Some(76_000));
    gate.note_chip_temperature(None);
    // Act
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Assert
    assert_eq!(
        gate.maybe_unsafe_detail()
            .and_then(|detail| detail.maybe_chip_temp_milli),
        None
    );
}

#[test]
fn a_later_reading_never_replaces_the_revoking_samples_temperature() {
    // Arrange: first wins, as for the rest of the detail.
    let (gate, _generation) = active_gate();
    gate.note_chip_temperature(Some(76_000));
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Act
    gate.note_chip_temperature(Some(40_000));
    // Assert
    assert_eq!(
        gate.maybe_unsafe_detail()
            .and_then(|detail| detail.maybe_chip_temp_milli),
        Some(76_000)
    );
}

#[test]
fn a_safety_silence_revocation_records_no_chip_temperature() {
    // Arrange: a hot reading noted earlier must not pass as the revoking sample's.
    let (gate, generation) = active_gate();
    gate.note_chip_temperature(Some(80_000));
    gate.note_fan_proof(generation, 1_000);
    // Act
    gate.check_deadline(2_001);
    // Assert
    assert_eq!(
        gate.maybe_unsafe_detail()
            .and_then(|detail| detail.maybe_chip_temp_milli),
        None
    );
}

#[test]
fn another_generations_detail_is_not_returned_for_this_one() {
    // Arrange
    let (gate, generation) = active_gate();
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    gate.finish_shutdown(generation);
    let next = gate.begin_link(600).expect("next link");
    // Act / Assert
    assert_eq!(gate.maybe_unsafe_detail_for(next), None);
}

#[test]
fn a_generations_own_detail_is_returned_for_it() {
    // Arrange
    let (gate, generation) = active_gate();
    // Act
    gate.check_safety_verdict(Some(bus_voltage_verdict(5_512)), true, 500);
    // Assert
    assert!(gate.maybe_unsafe_detail_for(generation).is_some());
}
