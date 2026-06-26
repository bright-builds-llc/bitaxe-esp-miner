//! Pure in-memory NVS persistence model.
//!
//! Breadcrumbs:
//! - `reference/esp-miner/main/nvs_config.c` loads settings from namespace
//!   `main`, applies legacy fallback migrations, and falls back to defaults
//!   for missing or corrupt values.
//! - This module models those decisions as inert data only. ESP-IDF NVS,
//!   HTTP, flash, reboot, mining, ASIC, voltage, fan, thermal, and power
//!   effects remain outside this crate.

use std::collections::BTreeMap;

use crate::nvs::StoredValueKind;
use crate::{
    all_settings_schema, apply_settings_patch, load_setting_value, migration_decisions,
    ConfigValidationError, LoadedValue, MigrationDecision, NvsErase, NvsWrite, SettingsPatch,
    SettingsUpdateDecision, StoredValue,
};

/// Deterministic in-memory snapshot of raw NVS key/value pairs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NvsSnapshot {
    values: BTreeMap<String, StoredValue>,
}

impl NvsSnapshot {
    /// Creates an empty in-memory snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a snapshot from raw stored values.
    #[must_use]
    pub fn from_values(values: impl IntoIterator<Item = StoredValue>) -> Self {
        let mut snapshot = Self::new();
        for value in values {
            snapshot.values.insert(value.key.as_str().to_owned(), value);
        }
        snapshot
    }

    /// Returns a stored value by exact NVS key name when present.
    #[must_use]
    pub fn maybe_stored_value(&self, key: &str) -> Option<&StoredValue> {
        self.values.get(key)
    }

    /// Returns all stored values in deterministic key order.
    #[must_use]
    pub fn stored_values(&self) -> Vec<StoredValue> {
        self.values.values().cloned().collect()
    }

    fn apply_write(&mut self, write: &NvsWrite) {
        let stored_value = stored_value_from_write(write);
        self.values
            .insert(stored_value.key.as_str().to_owned(), stored_value);
    }

    fn apply_erase(&mut self, erase: &NvsErase) {
        self.values.remove(erase.key.as_str());
    }
}

/// Pure persistence decision with typed adapter commands and loaded values.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistenceDecision {
    snapshot: NvsSnapshot,
    values: BTreeMap<String, LoadedValue>,
    writes: Vec<NvsWrite>,
    erases: Vec<NvsErase>,
    errors: Vec<ConfigValidationError>,
}

impl PersistenceDecision {
    /// Returns the snapshot after pure load/update/reload decisions.
    #[must_use]
    pub const fn snapshot(&self) -> &NvsSnapshot {
        &self.snapshot
    }

    /// Returns a loaded value by exact NVS key name when present.
    #[must_use]
    pub fn loaded_value(&self, key: &str) -> Option<&LoadedValue> {
        self.values.get(key)
    }

    /// Returns inert write commands emitted by the pure decision.
    #[must_use]
    pub fn writes(&self) -> &[NvsWrite] {
        &self.writes
    }

    /// Returns inert erase commands emitted by the pure decision.
    #[must_use]
    pub fn erases(&self) -> &[NvsErase] {
        &self.erases
    }

    /// Returns typed validation errors when an update was rejected.
    #[must_use]
    pub fn errors(&self) -> &[ConfigValidationError] {
        &self.errors
    }
}

/// Loads a snapshot through schema defaults without side effects.
#[must_use]
pub fn load_snapshot(snapshot: &NvsSnapshot) -> PersistenceDecision {
    let mut migrated_snapshot = snapshot.clone();
    let (writes, erases) = apply_migrations(&mut migrated_snapshot);
    let values = load_values(&migrated_snapshot);

    PersistenceDecision {
        snapshot: migrated_snapshot,
        values,
        writes,
        erases,
        errors: Vec::new(),
    }
}

/// Applies a settings patch to a new snapshot without side effects.
#[must_use]
pub fn apply_patch_to_snapshot(
    snapshot: &NvsSnapshot,
    patch: &SettingsPatch,
) -> PersistenceDecision {
    match apply_settings_patch(patch) {
        SettingsUpdateDecision::Accepted { writes } => {
            let mut updated_snapshot = snapshot.clone();
            for write in &writes {
                updated_snapshot.apply_write(write);
            }

            let mut decision = reload_snapshot(&updated_snapshot);
            let mut all_writes = writes;
            all_writes.extend(decision.writes);
            decision.writes = all_writes;
            decision
        }
        SettingsUpdateDecision::Rejected { errors } => PersistenceDecision {
            snapshot: snapshot.clone(),
            values: load_values(snapshot),
            writes: Vec::new(),
            erases: Vec::new(),
            errors,
        },
    }
}

/// Reloads a snapshot through migration and load rules without side effects.
#[must_use]
pub fn reload_snapshot(snapshot: &NvsSnapshot) -> PersistenceDecision {
    load_snapshot(snapshot)
}

fn apply_migrations(snapshot: &mut NvsSnapshot) -> (Vec<NvsWrite>, Vec<NvsErase>) {
    let decisions = migration_decisions(&snapshot.stored_values());
    let mut writes = Vec::new();
    let mut erases = Vec::new();

    for decision in decisions {
        match decision {
            MigrationDecision::Write(write) => {
                snapshot.apply_write(&write);
                writes.push(write);
            }
            MigrationDecision::Erase(erase) => {
                snapshot.apply_erase(&erase);
                erases.push(erase);
            }
        }
    }

    (writes, erases)
}

fn load_values(snapshot: &NvsSnapshot) -> BTreeMap<String, LoadedValue> {
    let mut values = BTreeMap::new();

    for schema in all_settings_schema() {
        let key = schema.key.as_str().to_owned();
        let loaded = load_setting_value(&schema, snapshot.maybe_stored_value(&key));
        values.insert(key, loaded);
    }

    values
}

fn stored_value_from_write(write: &NvsWrite) -> StoredValue {
    match write {
        NvsWrite::String { key, value } => StoredValue {
            key: key.clone(),
            value: StoredValueKind::String(value.clone()),
        },
        NvsWrite::U16 { key, value } => StoredValue {
            key: key.clone(),
            value: StoredValueKind::U16(*value),
        },
        NvsWrite::I32 { key, value } => StoredValue {
            key: key.clone(),
            value: StoredValueKind::I32(*value),
        },
        NvsWrite::U64 { key, value } => StoredValue {
            key: key.clone(),
            value: StoredValueKind::U64(*value),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        apply_patch_to_snapshot, load_snapshot, reload_snapshot, LoadedValue, NvsSnapshot,
        NvsWrite, RawSettingValue, SettingsPatch, StoredValue,
    };

    #[test]
    fn persistence_default_load_uses_ultra_205_defaults() {
        // Arrange
        let snapshot = NvsSnapshot::new();

        // Act
        let decision = load_snapshot(&snapshot);

        // Assert
        assert_eq!(
            decision.loaded_value("hostname"),
            Some(&LoadedValue::Str("bitaxe".to_owned()))
        );
        assert_eq!(
            decision.loaded_value("asicfrequency_f"),
            Some(&LoadedValue::Float(485.0))
        );
        assert_eq!(
            decision.loaded_value("asicvoltage"),
            Some(&LoadedValue::U16(1200))
        );
        assert_eq!(
            decision.loaded_value("autofanspeed"),
            Some(&LoadedValue::Bool(true))
        );
        assert_eq!(
            decision.loaded_value("manualfanspeed"),
            Some(&LoadedValue::U16(100))
        );
        assert_eq!(
            decision.loaded_value("selftest"),
            Some(&LoadedValue::Bool(true))
        );
        assert_eq!(
            decision.loaded_value("overheat_mode"),
            Some(&LoadedValue::Bool(false))
        );
    }

    #[test]
    fn persistence_missing_keys_do_not_emit_hardware_effects() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([StoredValue::string("hostname", "bitaxe")]);

        // Act
        let decision = load_snapshot(&snapshot);

        // Assert
        assert_eq!(
            decision.loaded_value("asicfrequency_f"),
            Some(&LoadedValue::Float(485.0))
        );
        assert_eq!(
            decision.loaded_value("manualfanspeed"),
            Some(&LoadedValue::U16(100))
        );
        assert!(decision.writes().is_empty());
        assert!(decision.erases().is_empty());
        assert!(decision.errors().is_empty());
    }

    #[test]
    fn persistence_valid_update_reload_roundtrip() {
        // Arrange
        let snapshot = NvsSnapshot::new();
        let patch = SettingsPatch::from_pairs([
            ("manualFanSpeed", RawSettingValue::Number(42)),
            ("frequency", RawSettingValue::Number(485)),
        ]);

        // Act
        let updated = apply_patch_to_snapshot(&snapshot, &patch);
        let reloaded = reload_snapshot(updated.snapshot());

        // Assert
        assert_eq!(
            updated.writes(),
            [
                NvsWrite::string("asicfrequency_f", "485"),
                NvsWrite::u16("asicfrequency", 485),
                NvsWrite::u16("manualfanspeed", 42),
                NvsWrite::u16("fanspeed", 42),
            ]
        );
        assert_eq!(
            reloaded.loaded_value("manualfanspeed"),
            Some(&LoadedValue::U16(42))
        );
        assert_eq!(
            reloaded.loaded_value("asicfrequency_f"),
            Some(&LoadedValue::Float(485.0))
        );
        assert_eq!(
            updated
                .snapshot()
                .maybe_stored_value("fanspeed")
                .map(|stored| &stored.value),
            Some(&StoredValue::u16("fanspeed", 42).value)
        );
        assert_eq!(
            updated
                .snapshot()
                .maybe_stored_value("asicfrequency")
                .map(|stored| &stored.value),
            Some(&StoredValue::u16("asicfrequency", 485).value)
        );
    }

    #[test]
    fn persistence_invalid_update_leaves_snapshot_unchanged() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([StoredValue::u16("manualfanspeed", 42)]);
        let patch = SettingsPatch::from_pairs([("manualFanSpeed", RawSettingValue::Number(101))]);

        // Act
        let rejected = apply_patch_to_snapshot(&snapshot, &patch);

        // Assert
        assert_eq!(rejected.snapshot(), &snapshot);
        assert!(matches!(
            rejected.errors(),
            [crate::ConfigValidationError::OutOfRange {
                field: "manualFanSpeed",
                min: 0,
                max: 100,
                actual: 101,
            }]
        ));
        assert!(rejected.writes().is_empty());
        assert!(rejected.erases().is_empty());
    }

    #[test]
    fn persistence_legacy_and_corrupt_values_reload_like_upstream() {
        // Arrange
        let legacy_snapshot = NvsSnapshot::from_values([StoredValue::u16("asicfrequency", 485)]);
        let corrupt_snapshot =
            NvsSnapshot::from_values([StoredValue::string("asicfrequency_f", "bad")]);

        // Act
        let migrated = reload_snapshot(&legacy_snapshot);
        let corrupt_loaded = reload_snapshot(&corrupt_snapshot);

        // Assert
        assert_eq!(
            migrated.writes(),
            [NvsWrite::string("asicfrequency_f", "485")]
        );
        assert_eq!(
            migrated.loaded_value("asicfrequency_f"),
            Some(&LoadedValue::Float(485.0))
        );
        assert_eq!(
            corrupt_loaded.loaded_value("asicfrequency_f"),
            Some(&LoadedValue::Float(485.0))
        );
        assert!(corrupt_loaded.writes().is_empty());
        assert!(corrupt_loaded.erases().is_empty());
    }
}
