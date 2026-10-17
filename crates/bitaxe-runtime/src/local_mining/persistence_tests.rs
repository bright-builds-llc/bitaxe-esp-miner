use super::*;
use bitaxe_config::StoredValue;

fn profile(frequency_mhz: f32, voltage_mv: u16) -> MiningHardwareProfile {
    MiningHardwareProfile::from_device_settings(Some(frequency_mhz), Some(voltage_mv))
        .expect("catalog pair")
}

#[test]
fn an_overheat_saves_the_fan_as_manual_at_full_speed_with_overheat_mode() {
    // Arrange / Act
    let writes = overheat_entry_writes(false).expect("valid settings");
    // Assert: the marker first, overheat_mode last.
    assert_eq!(
        writes,
        vec![
            NvsWrite::u16(OVERHEAT_STOP_MARKER_KEY, 0),
            NvsWrite::u16("autofanspeed", 0),
            NvsWrite::u16("manualfanspeed", 100),
            NvsWrite::u16("fanspeed", 100),
            NvsWrite::u16("overheat_mode", 1),
        ]
    );
}

#[test]
fn an_overheat_that_cannot_lower_saves_the_stop_marker() {
    // Arrange / Act
    let writes = overheat_entry_writes(true).expect("valid settings");
    // Assert
    assert_eq!(
        writes.first(),
        Some(&NvsWrite::u16(OVERHEAT_STOP_MARKER_KEY, 1))
    );
    assert!(!writes.contains(&NvsWrite::u16(OVERHEAT_STOP_MARKER_KEY, 0)));
}

#[test]
fn the_lowered_pair_is_stored_as_a_patch_stores_it() {
    // Arrange
    let lowered = profile(425.0, 1_100);
    // Act
    let writes = recovery_pair_writes(lowered).expect("valid settings");
    // Assert
    assert_eq!(
        writes,
        vec![
            NvsWrite::string("asicfrequency_f", "425.000000"),
            NvsWrite::u16("asicfrequency", 425),
            NvsWrite::u16("asicvoltage", 1_100),
        ]
    );
}

#[test]
fn clearing_overheat_mode_clears_the_marker_and_leaves_the_fan() {
    // Arrange / Act / Assert
    assert_eq!(
        overheat_cleared_writes(),
        vec![
            NvsWrite::u16("overheat_mode", 0),
            NvsWrite::u16(OVERHEAT_STOP_MARKER_KEY, 0)
        ]
    );
}

#[test]
fn every_overheat_write_set_is_device_owned() {
    // Arrange
    let lowered = profile(425.0, 1_100);
    let sets = [
        overheat_entry_writes(true).expect("valid settings"),
        recovery_pair_writes(lowered).expect("valid settings"),
        overheat_cleared_writes(),
        vec![stop_marker_write(false)],
    ];
    // Act / Assert
    for writes in sets {
        assert!(bitaxe_api::SettingsPersistencePlan::device_owned(writes).is_ok());
    }
}

#[test]
fn boot_reads_overheat_mode_and_the_marker() {
    // Arrange
    let settings = NvsSnapshot::from_values([StoredValue::u16("overheat_mode", 1)]);
    // Act
    let state = OverheatBootState::read(&settings, Some(1));
    // Assert
    assert_eq!(
        state,
        OverheatBootState {
            overheat_mode: true,
            stopped_marker: true
        }
    );
}

#[test]
fn a_missing_marker_reads_as_clear() {
    // Arrange
    let settings = NvsSnapshot::from_values([StoredValue::u16("overheat_mode", 1)]);
    // Act / Assert
    assert!(!OverheatBootState::read(&settings, None).stopped_marker);
}

#[test]
fn missing_settings_read_as_no_overheat() {
    // Arrange / Act / Assert
    assert_eq!(
        OverheatBootState::read(&NvsSnapshot::from_values([]), None),
        OverheatBootState::default()
    );
}
