//! The chip-temperature rule by source: only an ASIC-diode board skips an unpowered or
//! uninitialized chip, and everything else still revokes.

use bitaxe_config::catalog::gamma_601_catalog_entry;
use bitaxe_safety::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
    UnavailableReason,
};

use super::*;
use crate::observation::ChipTemperatureSource;

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

fn gamma_601() -> MiningSafetyLimits {
    MiningSafetyLimits::for_board(gamma_601_catalog_entry())
}

fn with_chip(chip_temp_celsius: Observation<f64>) -> TelemetryObservations {
    TelemetryObservations {
        power_watts: fresh(6.0),
        bus_voltage_volts: fresh(5.2),
        current_amps: fresh(1.2),
        core_voltage_actual_mv: fresh(0.0),
        chip_temp_celsius,
        vr_temp_celsius: fresh(40.0),
        fan_rpm: fresh(3_200_u16),
    }
}

fn unavailable(reason: UnavailableReason) -> Observation<f64> {
    Observation::unavailable(reason)
}

fn chip_failure(verdict: Option<SafetyVerdict>) -> Option<(SafetyFact, SafetyFactState)> {
    verdict.map(|verdict| (verdict.fact, verdict.state))
}

#[test]
fn the_chip_temperature_source_follows_the_catalog() {
    // Act
    let sources = [
        MiningSafetyLimits::ultra_205().chip_temperature_source(),
        gamma_601().chip_temperature_source(),
    ];
    // Assert
    assert_eq!(
        sources,
        [
            ChipTemperatureSource::BoardSensor,
            ChipTemperatureSource::AsicDiode
        ]
    );
}

#[test]
fn a_gamma_601_with_its_chip_unpowered_is_safe() {
    // Arrange
    let observations = with_chip(unavailable(UnavailableReason::AsicUnpowered));
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(maybe_verdict, None);
}

#[test]
fn a_gamma_601_with_its_chip_not_initialized_is_safe() {
    // Arrange
    let observations = with_chip(unavailable(UnavailableReason::AsicNotInitialized));
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(maybe_verdict, None);
}

#[test]
fn a_gamma_601_chip_not_yet_observed_still_revokes() {
    // Arrange
    let observations = with_chip(unavailable(UnavailableReason::NotYetObserved));
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(
        chip_failure(maybe_verdict),
        Some((SafetyFact::ChipTemperature, SafetyFactState::Unavailable))
    );
}

#[test]
fn a_gamma_601_chip_fault_still_revokes() {
    // Arrange
    let observations = with_chip(fresh(60.0).record_fault(FaultReason::ThermalReadingInvalid));
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(
        chip_failure(maybe_verdict),
        Some((SafetyFact::ChipTemperature, SafetyFactState::Fault))
    );
}

#[test]
fn a_stale_gamma_601_chip_reading_still_revokes() {
    // Arrange
    let stale = fresh(60.0)
        .mark_stale(StaleReason::ProducerCadenceExpired)
        .expect("last good");
    let observations = with_chip(stale);
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(
        chip_failure(maybe_verdict),
        Some((SafetyFact::ChipTemperature, SafetyFactState::Stale))
    );
}

#[test]
fn a_gamma_601_chip_at_75_c_still_revokes() {
    // Arrange
    let observations = with_chip(fresh(75.0));
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(
        chip_failure(maybe_verdict),
        Some((SafetyFact::ChipTemperature, SafetyFactState::OutOfRange))
    );
}

#[test]
fn a_hot_regulator_revokes_a_gamma_601_with_its_chip_unpowered() {
    // Arrange
    let observations = TelemetryObservations {
        vr_temp_celsius: fresh(105.1),
        ..with_chip(unavailable(UnavailableReason::AsicUnpowered))
    };
    // Act
    let maybe_verdict = observations.mining_safety_verdict_at(&gamma_601(), NOW);
    // Assert
    assert_eq!(
        chip_failure(maybe_verdict),
        Some((SafetyFact::VrTemperature, SafetyFactState::OutOfRange))
    );
}

#[test]
fn the_ultra_205_rejects_an_unpowered_or_uninitialized_chip() {
    // Arrange
    let reasons = [
        UnavailableReason::AsicUnpowered,
        UnavailableReason::AsicNotInitialized,
    ];
    // Act
    let failures = reasons.map(|reason| {
        chip_failure(with_chip(unavailable(reason)).ultra_205_mining_safety_verdict_at(NOW))
    });
    // Assert
    assert_eq!(
        failures,
        [Some((SafetyFact::ChipTemperature, SafetyFactState::Unavailable)); 2]
    );
}
