//! BAP settings mapped onto the same owners as HTTP: persisted settings become a
//! settings PATCH, and the block flags become command-visible notification state
//! (`bap_handlers.c:BAP_handle_settings`).

use bitaxe_core::bap::BapSettingIntent;
use serde_json::{json, Value};

use crate::commands::BlockFoundNotificationState;

/// The settings PATCH a persisted BAP setting writes, or none for RAM-only
/// settings. A fan speed also turns the automatic fan off, as the reference
/// does.
#[must_use]
pub fn bap_settings_patch(setting: &BapSettingIntent) -> Option<Value> {
    match setting {
        BapSettingIntent::FrequencyMhz(frequency) => {
            Some(json!({ "frequency": f64::from(*frequency) }))
        }
        BapSettingIntent::AsicVoltageMillivolts(voltage) => Some(json!({ "coreVoltage": voltage })),
        BapSettingIntent::WifiSsid(ssid) => Some(json!({ "ssid": ssid })),
        BapSettingIntent::WifiPassword(password) => Some(json!({ "wifiPass": password })),
        BapSettingIntent::ManualFanPercent(speed) => {
            Some(json!({ "autofanspeed": 0, "manualFanSpeed": speed }))
        }
        BapSettingIntent::AutoFan(enabled) => Some(json!({ "autofanspeed": u8::from(*enabled) })),
        BapSettingIntent::FoundBlock(_) | BapSettingIntent::ShowNewBlock(_) => None,
    }
}

/// The notification state after a BAP block flag, or none for a negative
/// `found_block`, which the reference's `int` holds but this count cannot.
#[must_use]
pub fn bap_block_found_state(
    setting: &BapSettingIntent,
    current: BlockFoundNotificationState,
) -> Option<BlockFoundNotificationState> {
    match setting {
        BapSettingIntent::FoundBlock(count) => {
            u64::try_from(*count)
                .ok()
                .map(|block_found| BlockFoundNotificationState {
                    block_found,
                    ..current
                })
        }
        BapSettingIntent::ShowNewBlock(show_new_block) => Some(BlockFoundNotificationState {
            show_new_block: *show_new_block,
            ..current
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::plan_settings_patch_value;
    use bitaxe_config::NvsWrite;

    fn persisted() -> Vec<BapSettingIntent> {
        vec![
            BapSettingIntent::FrequencyMhz(485.5),
            BapSettingIntent::AsicVoltageMillivolts(1_200),
            BapSettingIntent::WifiSsid("synthetic-network".to_owned()),
            BapSettingIntent::WifiPassword("synthetic-private-value".to_owned()),
            BapSettingIntent::ManualFanPercent(80),
            BapSettingIntent::AutoFan(true),
        ]
    }

    fn written_keys(setting: &BapSettingIntent) -> Vec<String> {
        let patch = bap_settings_patch(setting).expect("persisted setting");
        let accepted = plan_settings_patch_value(&patch).expect("the settings planner accepts it");
        accepted
            .writes()
            .iter()
            .map(|write| match write {
                NvsWrite::String { key, .. }
                | NvsWrite::U16 { key, .. }
                | NvsWrite::I32 { key, .. }
                | NvsWrite::U64 { key, .. } => key.as_str().to_owned(),
            })
            .collect()
    }

    #[test]
    fn every_persisted_setting_writes_its_reference_nvs_keys() {
        // Arrange: an unknown REST name would be skipped silently, so check the keys.
        let expected: [&[&str]; 6] = [
            &["asicfrequency", "asicfrequency_f"],
            &["asicvoltage"],
            &["wifissid"],
            &["wifipass"],
            &["autofanspeed", "fanspeed", "manualfanspeed"],
            &["autofanspeed"],
        ];
        // Act
        let written: Vec<Vec<String>> = persisted().iter().map(written_keys).collect();
        // Assert
        for (keys, expected) in written.iter().zip(expected) {
            let mut keys = keys.clone();
            keys.sort();
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn a_fan_speed_also_turns_the_automatic_fan_off() {
        // Arrange / Act
        let patch = bap_settings_patch(&BapSettingIntent::ManualFanPercent(80));
        // Assert
        assert_eq!(
            patch,
            Some(json!({ "autofanspeed": 0, "manualFanSpeed": 80 }))
        );
    }

    #[test]
    fn block_flags_are_not_persisted() {
        assert_eq!(
            bap_settings_patch(&BapSettingIntent::ShowNewBlock(true)),
            None
        );
    }

    #[test]
    fn found_block_replaces_the_count_and_keeps_the_flag() {
        // Arrange
        let current = BlockFoundNotificationState {
            block_found: 2,
            show_new_block: true,
        };
        // Act
        let next = bap_block_found_state(&BapSettingIntent::FoundBlock(7), current);
        // Assert
        assert_eq!(
            next,
            Some(BlockFoundNotificationState {
                block_found: 7,
                show_new_block: true
            })
        );
    }

    #[test]
    fn a_negative_found_block_has_no_state() {
        // Arrange
        let current = BlockFoundNotificationState {
            block_found: 2,
            show_new_block: false,
        };
        // Act / Assert
        assert_eq!(
            bap_block_found_state(&BapSettingIntent::FoundBlock(-1), current),
            None
        );
    }
}
