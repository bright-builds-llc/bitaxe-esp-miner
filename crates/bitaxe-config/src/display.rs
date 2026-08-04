//! Strict projection of the confirmed NVS snapshot into Ultra 205 display settings.

use bitaxe_core::display::{
    DisplayConfigurationError, Ultra205DisplayConfiguration, ULTRA205_DISPLAY_NAME,
};
use thiserror::Error;

use crate::{NvsSnapshot, StoredValueKind};

const DEFAULT_ROTATION_DEGREES: u16 = 0;
const DEFAULT_INVERTED: bool = false;
const DEFAULT_TIMEOUT_MINUTES: i32 = -1;

/// Closed failure categories for corrupt or unsupported stored display settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DisplaySettingsError {
    #[error("display setting has the wrong stored type")]
    WrongStoredType,
    #[error("display inversion is outside the boolean vocabulary")]
    InvalidInversion,
    #[error("display configuration is unsupported")]
    UnsupportedConfiguration,
}

/// Loads exact display settings without accepting corrupt storage as a default.
pub fn load_ultra205_display_configuration(
    snapshot: &NvsSnapshot,
) -> Result<Ultra205DisplayConfiguration, DisplaySettingsError> {
    let panel_name = match snapshot.maybe_stored_value("display") {
        None => ULTRA205_DISPLAY_NAME,
        Some(stored) => match &stored.value {
            StoredValueKind::String(value) if value.is_empty() => ULTRA205_DISPLAY_NAME,
            StoredValueKind::String(value) => value,
            _ => return Err(DisplaySettingsError::WrongStoredType),
        },
    };
    let rotation_degrees = match snapshot.maybe_stored_value("rotation") {
        None => DEFAULT_ROTATION_DEGREES,
        Some(stored) => match stored.value {
            StoredValueKind::U16(value) => value,
            _ => return Err(DisplaySettingsError::WrongStoredType),
        },
    };
    let inverted = match snapshot.maybe_stored_value("invertscreen") {
        None => DEFAULT_INVERTED,
        Some(stored) => match stored.value {
            StoredValueKind::U16(0) => false,
            StoredValueKind::U16(1) => true,
            StoredValueKind::U16(_) => return Err(DisplaySettingsError::InvalidInversion),
            _ => return Err(DisplaySettingsError::WrongStoredType),
        },
    };
    let timeout_minutes = match snapshot.maybe_stored_value("displayTimeout") {
        None => DEFAULT_TIMEOUT_MINUTES,
        Some(stored) => match stored.value {
            StoredValueKind::I32(value) => value,
            _ => return Err(DisplaySettingsError::WrongStoredType),
        },
    };

    Ultra205DisplayConfiguration::new(panel_name, rotation_degrees, inverted, timeout_minutes)
        .map_err(map_configuration_error)
}

fn map_configuration_error(_error: DisplayConfigurationError) -> DisplaySettingsError {
    DisplaySettingsError::UnsupportedConfiguration
}

#[cfg(test)]
mod tests {
    use bitaxe_core::display::{DisplayRotation, DisplayTimeout};

    use super::*;
    use crate::StoredValue;

    #[test]
    fn missing_and_empty_values_use_upstream_ultra205_defaults() {
        // Arrange
        let missing = NvsSnapshot::new();
        let empty = NvsSnapshot::from_values([StoredValue::string("display", "")]);

        // Act
        let missing_config =
            load_ultra205_display_configuration(&missing).expect("missing defaults");
        let empty_config = load_ultra205_display_configuration(&empty).expect("empty defaults");

        // Assert
        for configuration in [missing_config, empty_config] {
            assert_eq!(configuration.rotation(), DisplayRotation::Rotate0);
            assert!(!configuration.inverted());
            assert_eq!(configuration.timeout(), DisplayTimeout::AlwaysOn);
        }
    }

    #[test]
    fn exact_stored_values_project_without_schema_coercion() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string("display", ULTRA205_DISPLAY_NAME),
            StoredValue::u16("rotation", 270),
            StoredValue::u16("invertscreen", 1),
            StoredValue::i32("displayTimeout", 15),
        ]);

        // Act
        let configuration =
            load_ultra205_display_configuration(&snapshot).expect("valid display settings");

        // Assert
        assert_eq!(configuration.rotation(), DisplayRotation::Rotate270);
        assert!(configuration.inverted());
        assert_eq!(
            configuration.timeout(),
            DisplayTimeout::InactivityMillis(900_000)
        );
    }

    #[test]
    fn wrong_storage_types_fail_closed_instead_of_defaulting() {
        // Arrange
        let fixtures = [
            StoredValue::u16("display", 0),
            StoredValue::string("rotation", "0"),
            StoredValue::i32("invertscreen", 0),
            StoredValue::u16("displayTimeout", 1),
        ];

        // Act / Assert
        for stored in fixtures {
            assert_eq!(
                load_ultra205_display_configuration(&NvsSnapshot::from_values([stored])),
                Err(DisplaySettingsError::WrongStoredType)
            );
        }
    }

    #[test]
    fn unsupported_panel_rotation_timeout_and_boolean_fail_closed() {
        // Arrange / Act / Assert
        for stored in [
            StoredValue::string("display", "NONE"),
            StoredValue::u16("rotation", 45),
            StoredValue::i32("displayTimeout", -2),
        ] {
            assert_eq!(
                load_ultra205_display_configuration(&NvsSnapshot::from_values([stored])),
                Err(DisplaySettingsError::UnsupportedConfiguration)
            );
        }
        assert_eq!(
            load_ultra205_display_configuration(&NvsSnapshot::from_values([StoredValue::u16(
                "invertscreen",
                2
            ),])),
            Err(DisplaySettingsError::InvalidInversion)
        );
    }
}
