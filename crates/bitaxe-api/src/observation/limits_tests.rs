use super::*;
use bitaxe_config::catalog::gamma_601_catalog_entry;

#[test]
fn the_ultra_205_limits_are_unchanged() {
    // Act
    let limits = MiningSafetyLimits::ultra_205();
    // Assert
    assert_eq!(
        (limits.max_power_watts, limits.maybe_vr_temp_max_celsius),
        (15.0, None)
    );
}

#[test]
fn the_gamma_601_limits_follow_its_catalog() {
    // Act
    let limits = MiningSafetyLimits::for_board(gamma_601_catalog_entry());
    // Assert
    assert_eq!(
        (limits.max_power_watts, limits.maybe_vr_temp_max_celsius),
        (22.0, Some(105.0))
    );
}

#[test]
fn the_regulator_temperature_holds_up_to_upstream_s_throttle() {
    // Arrange
    let limits = MiningSafetyLimits::for_board(gamma_601_catalog_entry());
    // Act
    let in_range =
        [40.0, 105.0, 105.1, f64::NAN].map(|celsius| limits.vr_temperature_in_range(celsius));
    // Assert
    assert_eq!(in_range, [true, true, false, false]);
}

#[test]
fn a_board_without_a_tps546_has_no_regulator_limit() {
    // Act
    let in_range = MiningSafetyLimits::ultra_205().vr_temperature_in_range(f64::NAN);
    // Assert
    assert!(in_range);
}

#[test]
fn every_modelled_board_shares_the_5_v_supply_the_verdict_assumes() {
    // Act
    let nominal_volts = [
        bitaxe_config::catalog::ultra_205_catalog_entry(),
        gamma_601_catalog_entry(),
    ]
    .map(|entry| f64::from(entry.family_facts().nominal_voltage_volts));
    // Assert
    assert_eq!(
        nominal_volts,
        [bitaxe_safety::power::INPUT_VOLTAGE_NOMINAL_VOLTS; 2]
    );
}
