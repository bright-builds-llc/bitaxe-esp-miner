//! The saved settings a lease mines with (ADR-0037): `frequency` (`asicfrequency_f`)
//! and `coreVoltage` (`asicvoltage`). Unlike the general reload, which falls back to a
//! default when a stored value is unreadable, these return `None` for an unreadable or
//! wrongly typed stored value, so the lease is refused rather than mined at a default.

use crate::nvs::StoredValueKind;
use crate::persistence::{reload_snapshot, NvsSnapshot};
use crate::LoadedValue;

/// The stored ASIC frequency in MHz; the default (or a migrated legacy value) when
/// none is stored; `None` when the stored value does not parse.
#[must_use]
pub fn saved_asic_frequency_mhz(snapshot: &NvsSnapshot) -> Option<f32> {
    if let Some(stored) = snapshot.maybe_stored_value("asicfrequency_f") {
        return match &stored.value {
            StoredValueKind::String(raw) => raw.parse::<f32>().ok(),
            _ => None,
        };
    }
    match reload_snapshot(snapshot).maybe_loaded_value("asicfrequency_f") {
        Some(LoadedValue::Float(frequency)) => Some(*frequency),
        _ => None,
    }
}

/// The stored core voltage in mV; the default when none is stored; `None` when the
/// stored value has the wrong type.
#[must_use]
pub fn saved_core_voltage_mv(snapshot: &NvsSnapshot) -> Option<u16> {
    if let Some(stored) = snapshot.maybe_stored_value("asicvoltage") {
        return match &stored.value {
            StoredValueKind::U16(voltage) => Some(*voltage),
            _ => None,
        };
    }
    match reload_snapshot(snapshot).maybe_loaded_value("asicvoltage") {
        Some(LoadedValue::U16(voltage)) => Some(*voltage),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvs::StoredValue;

    fn snapshot(values: Vec<StoredValue>) -> NvsSnapshot {
        NvsSnapshot::from_values(values)
    }

    #[test]
    fn nothing_stored_mines_the_ultra_205_defaults() {
        // Arrange
        let empty = snapshot(Vec::new());
        // Act
        let saved = (
            saved_asic_frequency_mhz(&empty),
            saved_core_voltage_mv(&empty),
        );
        // Assert
        assert_eq!(saved, (Some(485.0), Some(1_200)));
    }

    #[test]
    fn a_stored_frequency_is_read_exactly() {
        // Arrange
        let stored = snapshot(vec![StoredValue::string("asicfrequency_f", "525.5")]);
        // Act
        let frequency = saved_asic_frequency_mhz(&stored);
        // Assert
        assert_eq!(frequency, Some(525.5));
    }

    #[test]
    fn a_corrupt_stored_frequency_is_unreadable_not_the_default() {
        // Arrange
        let stored = snapshot(vec![StoredValue::string("asicfrequency_f", "fast")]);
        // Act
        let frequency = saved_asic_frequency_mhz(&stored);
        // Assert
        assert_eq!(frequency, None);
    }

    #[test]
    fn a_legacy_whole_mhz_frequency_is_migrated() {
        // Arrange: older firmware stored only the u16 `asicfrequency`.
        let stored = snapshot(vec![StoredValue::u16("asicfrequency", 450)]);
        // Act
        let frequency = saved_asic_frequency_mhz(&stored);
        // Assert
        assert_eq!(frequency, Some(450.0));
    }

    #[test]
    fn a_wrongly_typed_stored_voltage_is_unreadable() {
        // Arrange
        let stored = snapshot(vec![StoredValue::string("asicvoltage", "1200")]);
        // Act
        let voltage = saved_core_voltage_mv(&stored);
        // Assert
        assert_eq!(voltage, None);
    }
}
