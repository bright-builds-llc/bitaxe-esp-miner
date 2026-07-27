//! Pure NVS schema model.
//!
//! Breadcrumbs:
//! - `reference/esp-miner/main/nvs_config.h` defines upstream stored types and
//!   the settings table shape.
//! - `reference/esp-miner/main/nvs_config.c` defines namespace, key names,
//!   REST names, defaults, ranges, indexed behavior, and legacy migrations.

use thiserror::Error;

/// Upstream ESP-Miner NVS namespace for settings.
pub const NVS_NAMESPACE: &str = "main";

/// ESP-IDF NVS key names are limited to 15 bytes, excluding the terminator.
pub const NVS_KEY_NAME_MAX_BYTES: usize = 15;

/// Validation errors for NVS schema identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NvsSchemaError {
    /// NVS keys must name a real setting.
    #[error("NVS key name must not be empty")]
    EmptyKeyName,
    /// ESP-IDF NVS keys are byte-sized ASCII names in the upstream table.
    #[error("NVS key name must be ASCII: {value}")]
    NonAsciiKeyName { value: String },
    /// ESP-IDF rejects key names longer than 15 bytes.
    #[error("NVS key name exceeds {max_bytes} bytes: {value}")]
    KeyNameTooLong { value: String, max_bytes: usize },
    /// REST/API names are separate from NVS keys but still must be present.
    #[error("REST field name must not be empty")]
    EmptyRestFieldName,
}

/// Exact upstream NVS key name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NvsKeyName(String);

impl NvsKeyName {
    /// Parses an NVS key name without truncating invalid values.
    pub fn parse(value: impl Into<String>) -> Result<Self, NvsSchemaError> {
        let value = value.into();
        if value.is_empty() {
            return Err(NvsSchemaError::EmptyKeyName);
        }

        if !value.is_ascii() {
            return Err(NvsSchemaError::NonAsciiKeyName { value });
        }

        if value.len() > NVS_KEY_NAME_MAX_BYTES {
            return Err(NvsSchemaError::KeyNameTooLong {
                value,
                max_bytes: NVS_KEY_NAME_MAX_BYTES,
            });
        }

        Ok(Self(value))
    }

    /// Returns the NVS key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// REST/API field name paired with a setting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestFieldName(String);

impl RestFieldName {
    /// Parses a REST/API field name separately from an NVS key name.
    pub fn parse(value: impl Into<String>) -> Result<Self, NvsSchemaError> {
        let value = value.into();
        if value.is_empty() {
            return Err(NvsSchemaError::EmptyRestFieldName);
        }

        Ok(Self(value))
    }

    /// Returns the REST/API field name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Upstream storage encoding used for an NVS setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredType {
    /// Upstream `TYPE_STR`.
    Str,
    /// Upstream `TYPE_U16`.
    U16,
    /// Upstream `TYPE_I32`.
    I32,
    /// Upstream `TYPE_U64`.
    U64,
    /// Upstream `TYPE_FLOAT`, persisted as an NVS string.
    FloatString,
    /// Upstream `TYPE_BOOL`, persisted as an NVS `u16`.
    BoolAsU16,
}

/// Default value from the upstream settings table or Ultra 205 seed defaults.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingDefault {
    /// String default.
    Str(&'static str),
    /// Unsigned 16-bit integer default.
    U16(u16),
    /// Signed 32-bit integer default.
    I32(i32),
    /// Unsigned 64-bit integer default.
    U64(u64),
    /// Float default for values stored as strings.
    Float(f32),
    /// Boolean default stored as a `u16` in NVS.
    Bool(bool),
}

/// Typed NVS schema row.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingSchema {
    /// Exact NVS key name.
    pub key: NvsKeyName,
    /// Upstream storage encoding.
    pub stored_type: StoredType,
    /// Upstream or Ultra 205 default value.
    pub default_value: Option<SettingDefault>,
    /// REST/API field name when one exists.
    pub rest_name: Option<RestFieldName>,
    /// Minimum accepted value or length from upstream metadata.
    pub min: Option<i32>,
    /// Maximum accepted value or length from upstream metadata.
    pub max: Option<i32>,
    /// Number of indexed entries for array-like settings.
    pub array_size: Option<usize>,
    /// Reference breadcrumb for this row.
    pub provenance: &'static str,
}

/// Raw NVS value read by a future adapter and passed into the pure model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredValue {
    /// Exact NVS key name for this stored value.
    pub key: NvsKeyName,
    /// Raw storage payload.
    pub value: StoredValueKind,
}

impl StoredValue {
    /// Creates a string stored value for a static upstream key.
    #[must_use]
    pub fn string(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key: self::key(key),
            value: StoredValueKind::String(value.into()),
        }
    }

    /// Creates a `u16` stored value for a static upstream key.
    #[must_use]
    pub fn u16(key: &'static str, value: u16) -> Self {
        Self {
            key: self::key(key),
            value: StoredValueKind::U16(value),
        }
    }

    /// Creates an `i32` stored value for a static upstream key.
    #[must_use]
    pub fn i32(key: &'static str, value: i32) -> Self {
        Self {
            key: self::key(key),
            value: StoredValueKind::I32(value),
        }
    }

    /// Creates a `u64` stored value for a static upstream key.
    #[must_use]
    pub fn u64(key: &'static str, value: u64) -> Self {
        Self {
            key: self::key(key),
            value: StoredValueKind::U64(value),
        }
    }
}

/// Raw NVS value payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredValueKind {
    /// NVS string payload.
    String(String),
    /// NVS `u16` payload.
    U16(u16),
    /// NVS `i32` payload.
    I32(i32),
    /// NVS `u64` payload.
    U64(u64),
}

/// Loaded typed value after applying missing-key and corrupt-value defaults.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedValue {
    /// Loaded string value.
    Str(String),
    /// Loaded `u16` value.
    U16(u16),
    /// Loaded `i32` value.
    I32(i32),
    /// Loaded `u64` value.
    U64(u64),
    /// Loaded float value from a string-backed NVS value.
    Float(f32),
    /// Loaded boolean value from a `u16`-backed NVS value.
    Bool(bool),
}

/// Inert NVS write decision for a future adapter to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvsWrite {
    /// Write an NVS string.
    String { key: NvsKeyName, value: String },
    /// Write an NVS `u16`.
    U16 { key: NvsKeyName, value: u16 },
    /// Write an NVS `i32`.
    I32 { key: NvsKeyName, value: i32 },
    /// Write an NVS `u64`.
    U64 { key: NvsKeyName, value: u64 },
}

impl NvsWrite {
    /// Creates a string write for a static upstream key.
    #[must_use]
    pub fn string(key: &'static str, value: impl Into<String>) -> Self {
        Self::String {
            key: self::key(key),
            value: value.into(),
        }
    }

    /// Creates a `u16` write for a static upstream key.
    #[must_use]
    pub fn u16(key: &'static str, value: u16) -> Self {
        Self::U16 {
            key: self::key(key),
            value,
        }
    }

    /// Creates an `i32` write for a static upstream key.
    #[must_use]
    pub fn i32(key: &'static str, value: i32) -> Self {
        Self::I32 {
            key: self::key(key),
            value,
        }
    }

    /// Creates a `u64` write for a static upstream key.
    #[must_use]
    pub fn u64(key: &'static str, value: u64) -> Self {
        Self::U64 {
            key: self::key(key),
            value,
        }
    }
}

/// Inert NVS erase decision for a future adapter to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvsErase {
    /// Exact NVS key to erase.
    pub key: NvsKeyName,
}

impl NvsErase {
    /// Creates an erase decision for a static upstream key.
    #[must_use]
    pub fn key(key: &'static str) -> Self {
        Self {
            key: self::key(key),
        }
    }
}

/// Metadata describing a pure migration rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRule {
    /// Source key inspected by the migration.
    pub source_key: NvsKeyName,
    /// Target key written by the migration.
    pub target_key: NvsKeyName,
    /// Human-readable rule description.
    pub description: &'static str,
}

/// Ordered pure migration decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationDecision {
    /// Erase a key before writing replacement storage.
    Erase(NvsErase),
    /// Write replacement storage.
    Write(NvsWrite),
}

fn key(value: &'static str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("static upstream NVS key names must fit ESP-IDF limits")
}
