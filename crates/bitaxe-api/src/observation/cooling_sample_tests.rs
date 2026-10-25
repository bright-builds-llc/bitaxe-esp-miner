use bitaxe_config::catalog::gamma_601_catalog_entry;
use bitaxe_safety::observation::{
    BootSessionId, MonotonicMillis, Observation, ObservationSequence, StaleReason,
    UnavailableReason,
};

use super::*;

const ACQUIRED_AT_MS: u64 = 4_000;

fn fresh(value: f64) -> Observation<f64> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(9),
        MonotonicMillis::new(ACQUIRED_AT_MS),
    )
    .expect("fixture sequence should advance")
    .0
}

fn observations(chip: Observation<f64>, vr: Observation<f64>) -> TelemetryObservations {
    TelemetryObservations {
        chip_temp_celsius: chip,
        vr_temp_celsius: vr,
        ..TelemetryObservations::default()
    }
}

fn unpowered() -> Observation<f64> {
    Observation::unavailable(UnavailableReason::AsicUnpowered)
}

fn sample(cool: bool) -> Option<CoolingSample> {
    Some(CoolingSample {
        acquired_at_ms: ACQUIRED_AT_MS,
        cool,
    })
}

fn gamma_601() -> MiningSafetyLimits {
    MiningSafetyLimits::for_board(gamma_601_catalog_entry())
}

#[test]
fn the_ultra_205_samples_its_chip_temperature_at_45_c() {
    // Arrange
    let chips = [45.0, 45.1, f64::NAN].map(fresh);
    // Act
    let samples = chips.map(|chip| {
        observations(chip, fresh(20.0)).restart_cooling_sample(&MiningSafetyLimits::ultra_205())
    });
    // Assert
    assert_eq!(samples, [sample(true), sample(false), sample(false)]);
}

#[test]
fn the_ultra_205_has_no_sample_without_a_fresh_chip_reading() {
    // Arrange
    let stale = fresh(30.0)
        .mark_stale(StaleReason::ProducerCadenceExpired)
        .expect("last good");
    let chips = [stale, unpowered()];
    // Act
    let samples = chips.map(|chip| {
        observations(chip, fresh(20.0)).restart_cooling_sample(&MiningSafetyLimits::ultra_205())
    });
    // Assert
    assert_eq!(samples, [None, None]);
}

#[test]
fn the_gamma_601_samples_its_regulator_at_95_c_with_the_chip_unpowered() {
    // Arrange
    let regulators = [95.0, 95.1, f64::NAN].map(fresh);
    // Act
    let samples =
        regulators.map(|vr| observations(unpowered(), vr).restart_cooling_sample(&gamma_601()));
    // Assert
    assert_eq!(samples, [sample(true), sample(false), sample(false)]);
}

#[test]
fn a_gamma_601_chip_not_reported_unpowered_is_never_cool() {
    // Arrange
    let chips = [
        Observation::unavailable(UnavailableReason::AsicNotInitialized),
        Observation::unavailable(UnavailableReason::NotYetObserved),
        fresh(30.0),
    ];
    // Act
    let samples =
        chips.map(|chip| observations(chip, fresh(40.0)).restart_cooling_sample(&gamma_601()));
    // Assert
    assert_eq!(samples, [sample(false); 3]);
}

#[test]
fn the_gamma_601_has_no_sample_without_a_fresh_regulator_reading() {
    // Arrange
    let stale = fresh(40.0)
        .mark_stale(StaleReason::ProducerCadenceExpired)
        .expect("last good");
    // Act
    let maybe_sample = observations(unpowered(), stale).restart_cooling_sample(&gamma_601());
    // Assert
    assert_eq!(maybe_sample, None);
}
