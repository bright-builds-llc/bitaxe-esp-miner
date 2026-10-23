//! The system-info facts that depend on the board and its configuration.

use bitaxe_config::catalog::gamma_601_catalog_entry;

use crate::{ApiSnapshot, SystemAsicWire, SystemInfoWire};

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

#[test]
fn the_gamma_601_reports_upstream_s_asic_identity() {
    // Arrange
    let snapshot = ApiSnapshot::safe_for_board(gamma_601_catalog_entry());
    // Act
    let asic = SystemAsicWire::from_snapshot(&snapshot);
    // Assert
    assert_eq!(
        (
            asic.asic_model.as_str(),
            asic.device_model.as_str(),
            asic.swarm_color.as_str(),
            asic.asic_count,
            asic.hash_domains,
            asic.default_frequency,
            asic.default_voltage,
        ),
        ("BM1370", "Gamma", "green", 1, 4, 525, 1_150)
    );
    assert_eq!(asic.frequency_options, [400, 490, 525, 550, 600, 625]);
    assert_eq!(
        asic.voltage_options,
        [1_000, 1_060, 1_100, 1_150, 1_200, 1_250]
    );
}

#[test]
fn the_gamma_601_reports_upstream_s_system_identity() {
    // Arrange
    let snapshot = ApiSnapshot::safe_for_board(gamma_601_catalog_entry());
    // Act
    let info = SystemInfoWire::from_snapshot(&snapshot);
    // Assert
    assert_eq!(
        (
            info.asic_model.as_str(),
            info.board_version.as_str(),
            info.small_core_count,
            info.frequency,
            info.core_voltage,
            info.expected_hashrate
        ),
        ("BM1370", "601", 2_040, 525.0, 1_150, 1071.0)
    );
}

#[test]
fn the_platform_vocabulary_names_the_gamma_601_and_its_asic() {
    // Act
    let board = serde_json::to_value(crate::PlatformBoard::Gamma601).expect("board");
    let asic = serde_json::to_value(crate::PlatformAsic::Bm1370).expect("asic");
    // Assert
    assert_eq!(
        (board, asic),
        (serde_json::json!("601"), serde_json::json!("BM1370"))
    );
}
