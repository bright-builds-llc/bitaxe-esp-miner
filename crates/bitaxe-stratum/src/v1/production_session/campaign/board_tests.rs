//! Profiles for boards other than the Ultra 205, from their catalog entries.

use bitaxe_config::catalog::{gamma_601_catalog_entry, ultra_205_catalog_entry};

use super::{DeviceProfileRefusal, MiningHardwareProfile, MiningHardwareProfilePreset};

fn pair(profile: MiningHardwareProfile) -> (u16, u16) {
    (
        profile.frequency().mhz(),
        profile.core_voltage().millivolts(),
    )
}

#[test]
fn the_ultra_205_presets_are_unchanged_by_the_board_seam() {
    // Act
    let presets = [
        MiningHardwareProfilePreset::Conservative,
        MiningHardwareProfilePreset::UpstreamDefault,
    ]
    .map(|preset| {
        preset
            .profile_for_board(ultra_205_catalog_entry())
            .map(pair)
    });
    // Assert
    assert_eq!(presets, [Some((400, 1_100)), Some((485, 1_200))]);
}

#[test]
fn the_gamma_601_presets_come_from_its_catalog() {
    // Act
    let presets = [
        MiningHardwareProfilePreset::Conservative,
        MiningHardwareProfilePreset::UpstreamDefault,
    ]
    .map(|preset| {
        preset
            .profile_for_board(gamma_601_catalog_entry())
            .map(pair)
    });
    // Assert
    assert_eq!(presets, [Some((400, 1_000)), Some((525, 1_150))]);
}

#[test]
fn the_gamma_601_mines_upstream_s_overheat_pair() {
    // Arrange: 525/1150 lowered by 100/100, as upstream recovers from an overheat.
    let entry = gamma_601_catalog_entry();
    // Act
    let profile =
        MiningHardwareProfile::from_device_settings_for_board(entry, Some(425.0), Some(1_050));
    // Assert
    assert_eq!(profile.map(pair), Ok((425, 1_050)));
}

#[test]
fn the_gamma_601_refuses_a_voltage_above_its_tps546_ceiling() {
    // Act
    let refusal = MiningHardwareProfile::from_device_settings_for_board(
        gamma_601_catalog_entry(),
        Some(525.0),
        Some(1_250),
    );
    // Assert
    assert_eq!(refusal, Err(DeviceProfileRefusal::VoltageOutOfRange));
}

#[test]
fn the_ultra_205_device_settings_parse_as_before() {
    // Act
    let same = [
        (485.0, 1_200),
        (575.0, 1_300),
        (399.0, 1_200),
        (485.0, 1_301),
    ]
    .iter()
    .all(|&(mhz, mv)| {
        MiningHardwareProfile::from_device_settings(Some(mhz), Some(mv))
            == MiningHardwareProfile::from_device_settings_for_board(
                ultra_205_catalog_entry(),
                Some(mhz),
                Some(mv),
            )
    });
    // Assert
    assert!(same);
}
