use crate::{LoadedValue, NvsSnapshot, PersistenceDecision, StoredValue};

use super::ultra_205_defaults;

/// Number of configured fields in the pinned Ultra 205 seed.
pub const ULTRA_205_DEFAULT_FIELD_COUNT: u16 = 27;

/// Closed comparison of loaded settings against the pinned Ultra 205 seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ultra205DefaultsAttestation {
    matching_field_count: u16,
}

impl Ultra205DefaultsAttestation {
    /// Compares one loaded snapshot with every configured field in the seed.
    #[must_use]
    pub fn from_snapshot(snapshot: &NvsSnapshot) -> Self {
        Self::from_persistence(&crate::reload_snapshot(snapshot))
    }

    /// Compares one already-loaded persistence decision with the seed.
    #[must_use]
    pub fn from_persistence(loaded: &PersistenceDecision) -> Self {
        let defaults = ultra_205_defaults();
        let primary = defaults.primary_pool();
        let fallback = defaults.fallback_pool();
        let matches = [
            string_matches(loaded, "hostname", defaults.hostname()),
            string_matches(loaded, "stratumurl", primary.url()),
            u16_matches(loaded, "stratumport", primary.port()),
            u16_matches(loaded, "stratumtls", primary.tls()),
            string_matches(loaded, "stratumcert", primary.cert()),
            string_matches(loaded, "stratumuser", primary.user()),
            string_matches(loaded, "stratumpass", primary.password()),
            u16_matches(loaded, "stratumdiff", primary.difficulty()),
            bool_matches(loaded, "stratumxnsub", primary.extranonce_subscribe() != 0),
            string_matches(loaded, "fbstratumurl", fallback.url()),
            u16_matches(loaded, "fbstratumport", fallback.port()),
            u16_matches(loaded, "fbstratumtls", fallback.tls()),
            string_matches(loaded, "fbstratumcert", fallback.cert()),
            string_matches(loaded, "fbstratumuser", fallback.user()),
            string_matches(loaded, "fbstratumpass", fallback.password()),
            u16_matches(loaded, "fbstratumdiff", fallback.difficulty()),
            bool_matches(
                loaded,
                "stratumfbxnsub",
                fallback.extranonce_subscribe() != 0,
            ),
            float_matches(
                loaded,
                "asicfrequency_f",
                f32::from(defaults.asic_frequency_mhz()),
            ),
            u16_matches(loaded, "asicvoltage", defaults.asic_voltage_mv()),
            string_matches(loaded, "asicmodel", defaults.asic_model()),
            string_matches(loaded, "devicemodel", defaults.device_model()),
            string_matches(loaded, "boardversion", defaults.board_version()),
            u16_matches(loaded, "rotation", defaults.rotation()),
            bool_matches(loaded, "autofanspeed", defaults.auto_fan_speed()),
            u16_matches(loaded, "manualfanspeed", defaults.manual_fan_speed()),
            bool_matches(loaded, "selftest", defaults.self_test()),
            bool_matches(loaded, "overheat_mode", defaults.overheat_mode()),
        ];
        let matching_field_count = matches
            .into_iter()
            .filter(|matches_default| *matches_default)
            .count() as u16;

        Self {
            matching_field_count,
        }
    }

    /// Returns the number of fields that exactly matched.
    #[must_use]
    pub const fn matching_field_count(self) -> u16 {
        self.matching_field_count
    }

    /// Returns whether every configured field exactly matched.
    #[must_use]
    pub const fn all_defaults_match(self) -> bool {
        self.matching_field_count == ULTRA_205_DEFAULT_FIELD_COUNT
    }

    /// Renders the closed retained marker without configured values.
    #[must_use]
    pub fn retained_marker(self, mining_on_boot_disabled: bool) -> String {
        format!(
            "ultra205_config_defaults schema_version=1 matching_fields={} total_fields={} all_match={} mineonboot_disabled={} redacted=true",
            self.matching_field_count,
            ULTRA_205_DEFAULT_FIELD_COUNT,
            self.all_defaults_match(),
            mining_on_boot_disabled,
        )
    }
}

/// Returns the exact configured values from `config-205.cvs` as private NVS data.
///
/// Callers must keep the returned pool identity fields out of logs and public
/// evidence.
#[must_use]
pub fn ultra_205_default_seed_values() -> Vec<StoredValue> {
    let defaults = ultra_205_defaults();
    let primary = defaults.primary_pool();
    let fallback = defaults.fallback_pool();
    vec![
        StoredValue::string("hostname", defaults.hostname()),
        StoredValue::string("stratumurl", primary.url()),
        StoredValue::u16("stratumport", primary.port()),
        StoredValue::u16("stratumtls", primary.tls()),
        StoredValue::string("stratumcert", primary.cert()),
        StoredValue::string("stratumuser", primary.user()),
        StoredValue::string("stratumpass", primary.password()),
        StoredValue::u16("stratumdiff", primary.difficulty()),
        StoredValue::u16("stratumxnsub", primary.extranonce_subscribe()),
        StoredValue::string("fbstratumurl", fallback.url()),
        StoredValue::u16("fbstratumport", fallback.port()),
        StoredValue::u16("fbstratumtls", fallback.tls()),
        StoredValue::string("fbstratumcert", fallback.cert()),
        StoredValue::string("fbstratumuser", fallback.user()),
        StoredValue::string("fbstratumpass", fallback.password()),
        StoredValue::u16("fbstratumdiff", fallback.difficulty()),
        StoredValue::u16("fbstratumxnsum", fallback.extranonce_subscribe()),
        StoredValue::u16("asicfrequency", defaults.asic_frequency_mhz()),
        StoredValue::u16("asicvoltage", defaults.asic_voltage_mv()),
        StoredValue::string("asicmodel", defaults.asic_model()),
        StoredValue::string("devicemodel", defaults.device_model()),
        StoredValue::string("boardversion", defaults.board_version()),
        StoredValue::u16("rotation", defaults.rotation()),
        StoredValue::u16("autofanspeed", u16::from(defaults.auto_fan_speed())),
        StoredValue::u16("fanspeed", defaults.manual_fan_speed()),
        StoredValue::u16("selftest", u16::from(defaults.self_test())),
        StoredValue::u16("overheat_mode", u16::from(defaults.overheat_mode())),
    ]
}

fn string_matches(loaded: &PersistenceDecision, key: &str, expected: &str) -> bool {
    matches!(loaded.maybe_loaded_value(key), Some(LoadedValue::Str(value)) if value == expected)
}

fn u16_matches(loaded: &PersistenceDecision, key: &str, expected: u16) -> bool {
    matches!(loaded.maybe_loaded_value(key), Some(LoadedValue::U16(value)) if *value == expected)
}

fn float_matches(loaded: &PersistenceDecision, key: &str, expected: f32) -> bool {
    matches!(loaded.maybe_loaded_value(key), Some(LoadedValue::Float(value)) if *value == expected)
}

fn bool_matches(loaded: &PersistenceDecision, key: &str, expected: bool) -> bool {
    matches!(loaded.maybe_loaded_value(key), Some(LoadedValue::Bool(value)) if *value == expected)
}

#[cfg(test)]
mod tests {
    use crate::{NvsSnapshot, StoredValue};

    use super::{
        ultra_205_default_seed_values, Ultra205DefaultsAttestation, ULTRA_205_DEFAULT_FIELD_COUNT,
    };

    #[test]
    fn exact_seed_attests_all_fields_without_rendering_values() {
        // Arrange
        let snapshot = NvsSnapshot::from_values(ultra_205_default_seed_values());

        // Act
        let attestation = Ultra205DefaultsAttestation::from_snapshot(&snapshot);
        let marker = attestation.retained_marker(true);

        // Assert
        assert_eq!(
            attestation.matching_field_count(),
            ULTRA_205_DEFAULT_FIELD_COUNT
        );
        assert!(attestation.all_defaults_match());
        assert_eq!(marker, "ultra205_config_defaults schema_version=1 matching_fields=27 total_fields=27 all_match=true mineonboot_disabled=true redacted=true");
        assert!(!marker.contains("stratum"));
    }

    #[test]
    fn one_changed_field_prevents_complete_attestation() {
        // Arrange
        let mut values = ultra_205_default_seed_values();
        values.push(StoredValue::u16("asicvoltage", 1));
        let snapshot = NvsSnapshot::from_values(values);

        // Act
        let attestation = Ultra205DefaultsAttestation::from_snapshot(&snapshot);

        // Assert
        assert_eq!(
            attestation.matching_field_count(),
            ULTRA_205_DEFAULT_FIELD_COUNT - 1
        );
        assert!(!attestation.all_defaults_match());
    }
}
