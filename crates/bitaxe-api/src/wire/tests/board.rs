//! The system-info facts that depend on the board and its configuration.

use crate::{ApiSnapshot, SystemInfoWire};

#[test]
fn the_expected_hashrate_follows_the_configured_frequency() {
    // Arrange: 485 MHz × 894 small cores × 1 chip, as upstream computes it.
    let snapshot = ApiSnapshot::safe_ultra_205();
    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);
    // Assert
    assert!((wire.expected_hashrate - 433.59).abs() < 1e-9);
}

#[test]
fn the_gamma_601_reports_its_family_power_and_hashrate() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.catalog = bitaxe_config::catalog::gamma_601_catalog_entry();
    snapshot.config.asic_frequency_mhz = 525.0;
    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);
    // Assert
    assert_eq!(
        (wire.max_power, wire.nominal_voltage, wire.expected_hashrate),
        (40, 5, 1071.0)
    );
}

#[test]
fn a_paused_device_expects_no_hashrate() {
    // Arrange: upstream zeroes the expected hashrate when mining stops.
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.mining.operator_intent = bitaxe_stratum::v1::state::MiningOperatorIntent::Paused;
    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);
    // Assert
    assert_eq!(wire.expected_hashrate, 0.0);
}
