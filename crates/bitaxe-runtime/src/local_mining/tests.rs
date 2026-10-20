use super::*;

fn profile(frequency_mhz: f32, core_voltage_mv: u16) -> MiningHardwareProfile {
    MiningHardwareProfile::from_device_settings(Some(frequency_mhz), Some(core_voltage_mv))
        .expect("catalog pair")
}

fn reading(celsius: f64, acquired_at_ms: u64) -> Option<ChipReading> {
    Some(ChipReading {
        celsius,
        acquired_at_ms,
    })
}

/// Cool readings every 500 ms from `from_ms` through `to_ms`, the last step returned.
fn cool_until(overheat: &mut OverheatRecovery, from_ms: u64, to_ms: u64) -> OverheatStep {
    let mut step = OverheatStep::Cooling;
    for at in (from_ms..=to_ms).step_by(500) {
        step = overheat.observe(reading(44.0, at), at);
    }
    step
}

fn entered(frequency_mhz: f32, core_voltage_mv: u16) -> OverheatRecovery {
    let mut overheat = OverheatRecovery::default();
    overheat.enter(profile(frequency_mhz, core_voltage_mv), 0);
    overheat
}

#[test]
fn a_hot_chip_at_an_unsafe_revocation_is_an_overheat() {
    // Arrange / Act / Assert
    assert!(is_overheat(
        RevocationReason::UnsafeObservation,
        Some(LOCAL_OVERHEAT_TRIP_C)
    ));
}

#[test]
fn a_cool_chip_at_an_unsafe_revocation_is_not_an_overheat() {
    // Arrange: power or voltage revoked the generation, not the chip.
    // Act / Assert
    assert!(!is_overheat(
        RevocationReason::UnsafeObservation,
        Some(LOCAL_OVERHEAT_TRIP_C - 1.0)
    ));
}

#[test]
fn a_hot_chip_without_an_unsafe_revocation_is_not_an_overheat() {
    // Arrange / Act / Assert
    assert!(!is_overheat(
        RevocationReason::LeaseOrBudgetExpired,
        Some(80.0)
    ));
}

#[test]
fn a_revocation_without_a_chip_reading_is_not_an_overheat() {
    // Arrange: a stale reading is a sensor fault.
    // Act / Assert
    assert!(!is_overheat(RevocationReason::UnsafeObservation, None));
}

#[test]
fn an_overheat_holds_the_asic_down_while_cooling() {
    // Arrange / Act
    let overheat = entered(575.0, 1300);
    // Assert
    assert!(overheat.holds_power_down());
}

#[test]
fn recovery_waits_for_thirty_seconds_of_cool_readings() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    // Act
    let step = cool_until(&mut overheat, 1_000, 30_500);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn thirty_seconds_of_cool_readings_recover_at_the_lowered_pair() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    // Act
    let step = cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Recover(profile(475.0, 1200)));
}

#[test]
fn a_recovered_overheat_no_longer_holds_the_asic_down() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    // Act
    cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert!(!overheat.holds_power_down());
}

#[test]
fn a_reading_at_exactly_forty_five_counts_as_cool() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    overheat.observe(reading(45.0, 1_000), 1_000);
    // Act
    let mut step = OverheatStep::Cooling;
    for at in (1_500..=31_000).step_by(500) {
        step = overheat.observe(reading(45.0, at), at);
    }
    // Assert
    assert_eq!(step, OverheatStep::Recover(profile(475.0, 1200)));
}

#[test]
fn a_reading_above_forty_five_restarts_the_cooling_time() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    cool_until(&mut overheat, 1_000, 20_000);
    overheat.observe(reading(45.1, 20_500), 20_500);
    // Act
    let step = cool_until(&mut overheat, 21_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_missing_reading_restarts_the_cooling_time() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    cool_until(&mut overheat, 1_000, 20_000);
    overheat.observe(None, 20_500);
    // Act
    let step = cool_until(&mut overheat, 21_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_stale_reading_does_not_count_as_cool() {
    // Arrange: the sensor's last value is 1.5 s old.
    let mut overheat = entered(575.0, 1300);
    // Act
    let mut step = OverheatStep::Cooling;
    for at in (2_000..=40_000).step_by(500) {
        step = overheat.observe(reading(44.0, at - 1_500), at);
    }
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_reading_taken_before_the_overheat_does_not_count() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    overheat.enter(profile(575.0, 1300), 10_000);
    // Act
    let step = overheat.observe(reading(30.0, 9_900), 10_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn two_cool_readings_with_a_long_gap_are_not_continuous() {
    // Arrange: nothing was observed for 29 s between them.
    let mut overheat = entered(575.0, 1300);
    overheat.observe(reading(44.0, 1_000), 1_000);
    // Act
    let step = overheat.observe(reading(44.0, 31_000), 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn a_frequency_below_the_catalog_floor_stays_stopped() {
    // Arrange: 485 MHz less 100 is below the Ultra 205's 400 MHz floor.
    let mut overheat = entered(485.0, 1200);
    // Act
    let step = cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::StayStopped);
}

#[test]
fn a_voltage_below_the_catalog_floor_stays_stopped() {
    // Arrange: 1150 mV less 100 is below the 1100 mV floor.
    let mut overheat = entered(575.0, 1150);
    // Act
    let step = cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::StayStopped);
}

#[test]
fn a_stop_holds_the_asic_down_until_the_operator_resumes() {
    // Arrange
    let mut overheat = entered(485.0, 1200);
    cool_until(&mut overheat, 1_000, 31_000);
    // Act
    let held = overheat.holds_power_down();
    overheat.resume();
    // Assert
    assert!(held && !overheat.holds_power_down());
}

#[test]
fn resuming_never_cancels_an_overheat_still_cooling() {
    // Arrange
    let mut overheat = entered(575.0, 1300);
    // Act
    overheat.resume();
    // Assert
    assert!(overheat.holds_power_down());
}

#[test]
fn the_lowest_recoverable_pair_lands_on_the_catalog_floor() {
    // Arrange / Act
    let lowered = reduced_profile(profile(500.0, 1200));
    // Assert
    assert_eq!(lowered, Some(profile(400.0, 1100)));
}

#[test]
fn observing_with_no_overheat_reports_clear() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    // Act
    let step = overheat.observe(reading(44.0, 1_000), 1_000);
    // Assert
    assert_eq!(step, OverheatStep::Clear);
}

#[test]
fn after_a_reboot_recovery_restarts_at_the_saved_pair_without_lowering_it() {
    // Arrange: the pair may already be the lowered one.
    let mut overheat = OverheatRecovery::default();
    overheat.enter_after_boot(Some(profile(475.0, 1200)), false, 0);
    // Act
    let step = cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::Recover(profile(475.0, 1200)));
}

#[test]
fn after_a_reboot_recovery_still_waits_for_cooling() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    overheat.enter_after_boot(Some(profile(485.0, 1200)), false, 0);
    // Act
    let step = overheat.observe(reading(60.0, 1_000), 1_000);
    // Assert
    assert_eq!(step, OverheatStep::Cooling);
}

#[test]
fn after_a_reboot_a_saved_stop_keeps_holding() {
    // Arrange: the last recovery stayed stopped at the pair that overheated.
    let mut overheat = OverheatRecovery::default();
    overheat.enter_after_boot(Some(profile(485.0, 1200)), true, 0);
    // Act
    let step = cool_until(&mut overheat, 1_000, 31_000);
    // Assert
    assert_eq!(step, OverheatStep::StayStopped);
}

#[test]
fn after_a_reboot_a_saved_stop_ends_only_on_resume() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    overheat.enter_after_boot(Some(profile(485.0, 1200)), true, 0);
    // Act
    overheat.resume();
    // Assert
    assert!(!overheat.holds_power_down());
}

#[test]
fn resuming_never_cancels_cooling_after_a_reboot() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    overheat.enter_after_boot(Some(profile(485.0, 1200)), false, 0);
    // Act
    overheat.resume();
    // Assert
    assert!(overheat.holds_power_down());
}

#[test]
fn a_saved_stop_holds_even_with_a_pair_outside_the_catalog() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    // Act
    overheat.enter_after_boot(None, true, 0);
    // Assert
    assert_eq!(overheat.observe(None, 1), OverheatStep::StayStopped);
}

#[test]
fn a_saved_pair_outside_the_catalog_leaves_nothing_to_cool_for() {
    // Arrange
    let mut overheat = OverheatRecovery::default();
    // Act
    overheat.enter_after_boot(None, false, 0);
    // Assert
    assert!(!overheat.holds_power_down());
}

#[cfg(not(feature = "overheat-trip-test"))]
#[test]
fn a_production_build_trips_at_upstreams_seventy_five_degrees() {
    // Arrange / Act / Assert
    assert!((LOCAL_OVERHEAT_TRIP_C - 75.0).abs() < f64::EPSILON);
}
