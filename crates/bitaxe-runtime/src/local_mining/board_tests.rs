//! Overheat recovery on the Gamma 601: regulator samples with the chip unpowered, and the
//! lowered pair checked against the built board's catalog.

use super::*;
use bitaxe_api::{MiningSafetyLimits, TelemetryObservations};
use bitaxe_config::catalog::{gamma_601_catalog_entry, ultra_205_catalog_entry};
use bitaxe_safety::observation::{
    BootSessionId, MonotonicMillis, Observation, ObservationSequence, UnavailableReason,
};

fn gamma_601_profile(frequency_mhz: f32, core_voltage_mv: u16) -> MiningHardwareProfile {
    MiningHardwareProfile::from_device_settings_for_board(
        gamma_601_catalog_entry(),
        Some(frequency_mhz),
        Some(core_voltage_mv),
    )
    .expect("catalog pair")
}

/// The Gamma 601's sample with its chip unpowered and its regulator at `vr_celsius`.
fn regulator_sample(vr_celsius: f64, acquired_at_ms: u64) -> Option<CoolingSample> {
    let (vr_temp_celsius, _) = Observation::record_success(
        vr_celsius,
        BootSessionId::new(7),
        ObservationSequence::ZERO,
        MonotonicMillis::new(acquired_at_ms),
    )
    .expect("fixture sequence should advance");
    let observations = TelemetryObservations {
        chip_temp_celsius: Observation::unavailable(UnavailableReason::AsicUnpowered),
        vr_temp_celsius,
        ..TelemetryObservations::default()
    };
    observations.restart_cooling_sample(&MiningSafetyLimits::for_board(gamma_601_catalog_entry()))
}

/// Regulator samples every 500 ms from `from_ms` through `to_ms`, the last step returned.
fn regulator_until(
    overheat: &mut OverheatRecovery,
    vr_celsius: f64,
    from_ms: u64,
    to_ms: u64,
) -> OverheatStep {
    let mut step = OverheatStep::Cooling;
    for at in (from_ms..=to_ms).step_by(500) {
        step = overheat.observe(regulator_sample(vr_celsius, at), at);
    }
    step
}

fn entered_gamma_601() -> OverheatRecovery {
    let mut overheat = OverheatRecovery::default();
    overheat.enter(gamma_601_catalog_entry(), gamma_601_profile(600.0, 1150), 0);
    overheat
}

#[test]
fn thirty_seconds_of_a_cool_regulator_recover_a_gamma_601() {
    // Arrange
    let mut overheat = entered_gamma_601();
    // Act
    let step = regulator_until(&mut overheat, 95.0, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Recover(gamma_601_profile(500.0, 1050)));
}

#[test]
fn a_gamma_601_recovery_still_waits_the_thirty_seconds() {
    // Arrange
    let mut overheat = entered_gamma_601();
    // Act
    let step = regulator_until(&mut overheat, 40.0, 1_000, 30_500);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_hot_regulator_restarts_the_gamma_601_cooling_time() {
    // Arrange
    let mut overheat = entered_gamma_601();
    regulator_until(&mut overheat, 60.0, 1_000, 20_000);
    overheat.observe(regulator_sample(95.1, 20_500), 20_500);
    // Act
    let step = regulator_until(&mut overheat, 60.0, 21_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_gap_between_regulator_samples_restarts_the_gamma_601_cooling_time() {
    // Arrange: nothing was observed for 2.5 s at 20 s.
    let mut overheat = entered_gamma_601();
    regulator_until(&mut overheat, 60.0, 1_000, 20_000);
    // Act
    let step = regulator_until(&mut overheat, 60.0, 22_500, 32_500);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn the_lowered_pair_is_checked_against_the_built_board_s_catalog() {
    // Arrange: 1050 mV is inside the Gamma 601's catalog but below the Ultra 205's.
    let overheated = gamma_601_profile(600.0, 1150);
    // Act
    let lowered = [gamma_601_catalog_entry(), ultra_205_catalog_entry()]
        .map(|entry| reduced_profile(entry, overheated));
    // Assert
    assert_eq!(lowered, [Some(gamma_601_profile(500.0, 1050)), None]);
}
