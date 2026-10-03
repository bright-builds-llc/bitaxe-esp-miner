use bitaxe_safety::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
    UnavailableReason,
};

use super::*;

const NOW: MonotonicMillis = MonotonicMillis::new(1_250);

fn fresh<T: Copy>(value: T) -> Observation<T> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(9),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance")
    .0
}

fn safe() -> TelemetryObservations {
    TelemetryObservations {
        power_watts: fresh(4.0),
        bus_voltage_volts: fresh(5.2),
        current_amps: fresh(0.8),
        core_voltage_actual_mv: fresh(1_100.0),
        chip_temp_celsius: fresh(40.0),
        vr_temp_celsius: Observation::unavailable(UnavailableReason::ThermalReadingUnavailable),
        fan_rpm: fresh(3_200_u16),
    }
}

#[test]
fn safe_observations_have_no_verdict() {
    // Arrange / Act
    let maybe_verdict = safe().ultra_205_mining_safety_verdict_at(NOW);
    // Assert
    assert_eq!(maybe_verdict, None);
}

#[test]
fn bus_voltage_over_the_ceiling_reports_its_value_and_age() {
    // Arrange
    let observations = TelemetryObservations {
        bus_voltage_volts: fresh(5.501),
        ..safe()
    };
    // Act
    let maybe_verdict = observations.ultra_205_mining_safety_verdict_at(NOW);
    // Assert
    assert_eq!(
        maybe_verdict,
        Some(SafetyVerdict {
            fact: SafetyFact::BusVoltage,
            state: SafetyFactState::OutOfRange,
            maybe_value_milli: Some(5_501),
            maybe_age_ms: Some(1_000),
        })
    );
}

#[test]
fn freshness_failures_win_over_earlier_range_failures() {
    // Arrange: the predicate checks every fact's freshness before any range.
    let observations = TelemetryObservations {
        power_watts: fresh(20.0),
        fan_rpm: Observation::unavailable(UnavailableReason::NotYetObserved),
        ..safe()
    };
    // Act
    let verdict = observations
        .ultra_205_mining_safety_verdict_at(NOW)
        .expect("unsafe");
    // Assert
    assert_eq!(
        (verdict.fact, verdict.state, verdict.maybe_value_milli),
        (SafetyFact::FanRpm, SafetyFactState::Unavailable, None)
    );
}

#[test]
fn a_fresh_sample_beyond_the_window_is_expired_with_its_age() {
    // Arrange / Act
    let verdict = safe()
        .ultra_205_mining_safety_verdict_at(MonotonicMillis::new(1_251))
        .expect("unsafe");
    // Assert
    assert_eq!(
        (verdict.fact, verdict.state, verdict.maybe_age_ms),
        (SafetyFact::Power, SafetyFactState::Expired, Some(1_001))
    );
}

#[test]
fn a_stale_fact_reports_its_last_good_value() {
    // Arrange
    let observations = TelemetryObservations {
        current_amps: fresh(0.8)
            .mark_stale(StaleReason::PowerSampleStale)
            .expect("last good"),
        ..safe()
    };
    // Act
    let verdict = observations
        .ultra_205_mining_safety_verdict_at(NOW)
        .expect("unsafe");
    // Assert
    assert_eq!(
        verdict,
        SafetyVerdict {
            fact: SafetyFact::Current,
            state: SafetyFactState::Stale,
            maybe_value_milli: Some(800),
            maybe_age_ms: Some(1_000),
        }
    );
}

#[test]
fn a_fault_without_a_last_good_sample_has_no_value() {
    // Arrange
    let observations = TelemetryObservations {
        chip_temp_celsius: Observation::<f64>::unavailable(UnavailableReason::NotYetObserved)
            .record_fault(FaultReason::ReadFailed),
        ..safe()
    };
    // Act
    let verdict = observations
        .ultra_205_mining_safety_verdict_at(NOW)
        .expect("unsafe");
    // Assert
    assert_eq!(
        (
            verdict.fact,
            verdict.state,
            verdict.maybe_value_milli,
            verdict.maybe_age_ms
        ),
        (
            SafetyFact::ChipTemperature,
            SafetyFactState::Fault,
            None,
            None
        )
    );
}

#[test]
fn a_non_finite_value_is_out_of_range_without_a_projected_value() {
    // Arrange
    let observations = TelemetryObservations {
        current_amps: fresh(f64::NAN),
        ..safe()
    };
    // Act
    let verdict = observations
        .ultra_205_mining_safety_verdict_at(NOW)
        .expect("unsafe");
    // Assert
    assert_eq!(
        (verdict.fact, verdict.state, verdict.maybe_value_milli),
        (SafetyFact::Current, SafetyFactState::OutOfRange, None)
    );
}

#[test]
fn codes_round_trip_through_their_closed_labels() {
    // Arrange
    let facts = [1, 2, 3, 4, 5].map(SafetyFact::maybe_from_code);
    let states = [1, 2, 3, 4, 5].map(SafetyFactState::maybe_from_code);
    // Act / Assert
    assert!(facts
        .iter()
        .zip(1..)
        .all(|(fact, code)| fact.map(|f| f as u8) == Some(code)));
    assert!(states
        .iter()
        .zip(1..)
        .all(|(state, code)| state.map(|s| s as u8) == Some(code)));
    assert_eq!(
        (
            SafetyFact::maybe_from_code(0),
            SafetyFactState::maybe_from_code(6)
        ),
        (None, None)
    );
}
