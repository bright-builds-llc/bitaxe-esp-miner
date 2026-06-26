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

const SETTINGS_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c settings table";
const MIGRATION_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c fallback migration";

fn key(value: &'static str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("static upstream NVS key names must fit ESP-IDF limits")
}

fn rest(value: &'static str) -> RestFieldName {
    RestFieldName::parse(value).expect("static upstream REST field names must be non-empty")
}

/// Returns the upstream legacy migration rules captured by this model.
#[must_use]
pub fn migration_rules() -> Vec<MigrationRule> {
    vec![
        MigrationRule {
            source_key: key("asicfrequency"),
            target_key: key("asicfrequency_f"),
            description: "legacy u16 ASIC frequency to active float string key",
        },
        MigrationRule {
            source_key: key("fanspeed"),
            target_key: key("manualfanspeed"),
            description: "legacy manual fan speed key to active u16 key",
        },
        MigrationRule {
            source_key: key("stratumprot"),
            target_key: key("stratumprot"),
            description: "stratum protocol u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbstratumprot"),
            target_key: key("fbstratumprot"),
            description: "fallback stratum protocol u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("sv2chantype"),
            target_key: key("sv2chantype"),
            description: "SV2 channel type u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbsv2chantype"),
            target_key: key("fbsv2chantype"),
            description: "fallback SV2 channel type u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbSv2ChanType"),
            target_key: key("fbsv2chantype"),
            description: "legacy mixed-case fallback SV2 channel type key to active key",
        },
    ]
}

/// Returns ordered migration decisions for raw values already read from NVS.
#[must_use]
pub fn migration_decisions(stored_values: &[StoredValue]) -> Vec<MigrationDecision> {
    let mut decisions = Vec::new();

    if !has_key(stored_values, "asicfrequency_f") {
        if let Some(value) = maybe_u16(stored_values, "asicfrequency") {
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                "asicfrequency_f",
                value.to_string(),
            )));
        }
    }

    if !has_key(stored_values, "manualfanspeed") {
        if let Some(value) = maybe_u16(stored_values, "fanspeed") {
            decisions.push(MigrationDecision::Write(NvsWrite::u16(
                "manualfanspeed",
                value,
            )));
        }
    }

    for protocol_key in ["stratumprot", "fbstratumprot"] {
        if let Some(value) = maybe_u16(stored_values, protocol_key) {
            decisions.push(MigrationDecision::Erase(NvsErase::key(protocol_key)));
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                protocol_key,
                stratum_protocol_name(value),
            )));
        }
    }

    for channel_type_key in ["sv2chantype", "fbsv2chantype"] {
        if let Some(value) = maybe_u16(stored_values, channel_type_key) {
            decisions.push(MigrationDecision::Erase(NvsErase::key(channel_type_key)));
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                channel_type_key,
                sv2_channel_type_name(value),
            )));
        }
    }

    if !has_key(stored_values, "fbsv2chantype") {
        if let Some(value) = maybe_u16(stored_values, "fbSv2ChanType") {
            decisions.push(MigrationDecision::Erase(NvsErase::key("fbSv2ChanType")));
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                "fbsv2chantype",
                sv2_channel_type_name(value),
            )));
        }
    }

    decisions
}

/// Returns legacy compatibility writes for an active write decision.
#[must_use]
pub fn compatibility_writes_for_active(write: &NvsWrite) -> Vec<NvsWrite> {
    match write {
        NvsWrite::String { key, value } if key.as_str() == "asicfrequency_f" => value
            .parse::<f32>()
            .ok()
            .map(|frequency| NvsWrite::u16("asicfrequency", frequency as u16))
            .into_iter()
            .collect(),
        NvsWrite::U16 { key, value } if key.as_str() == "manualfanspeed" => {
            vec![NvsWrite::u16("fanspeed", *value)]
        }
        _ => Vec::new(),
    }
}

/// Loads a raw stored value through schema defaults without any NVS side effect.
#[must_use]
pub fn load_setting_value(
    schema: &SettingSchema,
    maybe_stored: Option<&StoredValue>,
) -> LoadedValue {
    match schema.stored_type {
        StoredType::Str => load_string(schema, maybe_stored),
        StoredType::U16 => load_u16(schema, maybe_stored),
        StoredType::I32 => load_i32(schema, maybe_stored),
        StoredType::U64 => load_u64(schema, maybe_stored),
        StoredType::FloatString => load_float_string(schema, maybe_stored),
        StoredType::BoolAsU16 => load_bool_as_u16(schema, maybe_stored),
    }
}

fn maybe_stored_value<'a>(stored_values: &'a [StoredValue], key: &str) -> Option<&'a StoredValue> {
    stored_values
        .iter()
        .find(|stored| stored.key.as_str() == key)
}

fn has_key(stored_values: &[StoredValue], key: &str) -> bool {
    maybe_stored_value(stored_values, key).is_some()
}

fn maybe_u16(stored_values: &[StoredValue], key: &str) -> Option<u16> {
    let stored = maybe_stored_value(stored_values, key)?;

    match stored.value {
        StoredValueKind::U16(value) => Some(value),
        _ => None,
    }
}

fn stratum_protocol_name(value: u16) -> &'static str {
    if value == 1 {
        return "SV2";
    }

    "SV1"
}

fn sv2_channel_type_name(value: u16) -> &'static str {
    if value == 1 {
        return "standard";
    }

    "extended"
}

fn load_string(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::String(value),
        ..
    }) = maybe_stored
    {
        if !value.is_empty() {
            return LoadedValue::Str(value.clone());
        }
    }

    LoadedValue::Str(default_string(schema))
}

fn load_u16(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U16(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::U16(*value);
    }

    LoadedValue::U16(default_u16(schema))
}

fn load_i32(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::I32(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::I32(*value);
    }

    LoadedValue::I32(default_i32(schema))
}

fn load_u64(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U64(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::U64(*value);
    }

    LoadedValue::U64(default_u64(schema))
}

fn load_float_string(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::String(value),
        ..
    }) = maybe_stored
    {
        if let Ok(parsed) = value.parse::<f32>() {
            return LoadedValue::Float(parsed);
        }
    }

    LoadedValue::Float(default_float(schema))
}

fn load_bool_as_u16(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U16(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::Bool(*value != 0);
    }

    LoadedValue::Bool(default_bool(schema))
}

fn default_string(schema: &SettingSchema) -> String {
    match schema.default_value {
        Some(SettingDefault::Str(value)) => value.to_owned(),
        _ => String::new(),
    }
}

fn default_u16(schema: &SettingSchema) -> u16 {
    match schema.default_value {
        Some(SettingDefault::U16(value)) => value,
        _ => 0,
    }
}

fn default_i32(schema: &SettingSchema) -> i32 {
    match schema.default_value {
        Some(SettingDefault::I32(value)) => value,
        _ => 0,
    }
}

fn default_u64(schema: &SettingSchema) -> u64 {
    match schema.default_value {
        Some(SettingDefault::U64(value)) => value,
        _ => 0,
    }
}

fn default_float(schema: &SettingSchema) -> f32 {
    match schema.default_value {
        Some(SettingDefault::Float(value)) => value,
        _ => 0.0,
    }
}

fn default_bool(schema: &SettingSchema) -> bool {
    match schema.default_value {
        Some(SettingDefault::Bool(value)) => value,
        _ => false,
    }
}

/// Returns typed settings schema rows derived from the pinned upstream table.
#[must_use]
pub fn all_settings_schema() -> Vec<SettingSchema> {
    vec![
        SettingSchema {
            key: key("wifissid"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("ssid")),
            min: Some(1),
            max: Some(32),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("wifipass"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("wifiPass")),
            min: Some(0),
            max: Some(63),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("hostname"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("bitaxe")),
            rest_name: Some(rest("hostname")),
            min: Some(1),
            max: Some(32),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumprot"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("SV1")),
            rest_name: Some(rest("stratumProtocol")),
            min: Some(3),
            max: Some(3),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumurl"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("stratumURL")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumport"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("stratumPort")),
            min: Some(0),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumuser"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("stratumUser")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumpass"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("stratumPassword")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumdiff"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("stratumSuggestedDifficulty")),
            min: Some(0),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumxnsub"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: Some(rest("stratumExtranonceSubscribe")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumtls"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("stratumTLS")),
            min: Some(0),
            max: Some(3),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumcert"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("stratumCert")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("sv2chantype"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("extended")),
            rest_name: Some(rest("stratumV2ChannelType")),
            min: Some(8),
            max: Some(8),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("sv2authpubkey"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("")),
            rest_name: Some(rest("stratumV2AuthorityPubkey")),
            min: Some(0),
            max: Some(52),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumdecode"),
            stored_type: StoredType::BoolAsU16,
            default_value: Some(SettingDefault::Bool(true)),
            rest_name: Some(rest("stratumDecodeCoinbase")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumprot"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("SV1")),
            rest_name: Some(rest("fallbackStratumProtocol")),
            min: Some(3),
            max: Some(3),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumurl"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("fallbackStratumURL")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumport"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("fallbackStratumPort")),
            min: Some(0),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumuser"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("fallbackStratumUser")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumpass"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("fallbackStratumPassword")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumdiff"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("fallbackStratumSuggestedDifficulty")),
            min: Some(0),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("stratumfbxnsub"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: Some(rest("fallbackStratumExtranonceSubscribe")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumtls"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("fallbackStratumTLS")),
            min: Some(0),
            max: Some(3),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumcert"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("fallbackStratumCert")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbsv2chantype"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("extended")),
            rest_name: Some(rest("fallbackStratumV2ChannelType")),
            min: Some(8),
            max: Some(8),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbsv2authpubk"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("")),
            rest_name: Some(rest("fallbackStratumV2AuthorityPubkey")),
            min: Some(0),
            max: Some(52),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("fbstratumdecode"),
            stored_type: StoredType::BoolAsU16,
            default_value: Some(SettingDefault::Bool(true)),
            rest_name: Some(rest("fallbackStratumDecodeCoinbase")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("usefbstartum"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: Some(rest("useFallbackStratum")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("asicfrequency_f"),
            stored_type: StoredType::FloatString,
            default_value: Some(SettingDefault::Float(485.0)),
            rest_name: Some(rest("frequency")),
            min: Some(1),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("asicvoltage"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(1200)),
            rest_name: Some(rest("coreVoltage")),
            min: Some(1),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("oc_enabled"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: Some(rest("overclockEnabled")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("display"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("display")),
            min: Some(0),
            max: Some(3999),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("rotation"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(0)),
            rest_name: Some(rest("rotation")),
            min: Some(0),
            max: Some(270),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("invertscreen"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: Some(rest("invertscreen")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("displayOffset"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("displayOffset")),
            min: Some(0),
            max: Some(255),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("displayTimeout"),
            stored_type: StoredType::I32,
            default_value: Some(SettingDefault::I32(-1)),
            rest_name: Some(rest("displayTimeout")),
            min: Some(-1),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("autofanspeed"),
            stored_type: StoredType::BoolAsU16,
            default_value: Some(SettingDefault::Bool(true)),
            rest_name: Some(rest("autofanspeed")),
            min: Some(0),
            max: Some(1),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("manualfanspeed"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(100)),
            rest_name: Some(rest("manualFanSpeed")),
            min: Some(0),
            max: Some(100),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("minfanspeed"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(25)),
            rest_name: Some(rest("minFanSpeed")),
            min: Some(0),
            max: Some(99),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("temptarget"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(60)),
            rest_name: Some(rest("temptarget")),
            min: Some(35),
            max: Some(66),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("overheat_mode"),
            stored_type: StoredType::BoolAsU16,
            default_value: Some(SettingDefault::Bool(false)),
            rest_name: Some(rest("overheat_mode")),
            min: Some(0),
            max: Some(0),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("statsFrequency"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: Some(rest("statsFrequency")),
            min: Some(0),
            max: Some(65535),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("bestdiff"),
            stored_type: StoredType::U64,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest"),
            stored_type: StoredType::BoolAsU16,
            default_value: Some(SettingDefault::Bool(true)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("swarmconfig"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("themescheme"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("themecolors"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("scoreboard"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: Some(20),
            provenance: "reference/esp-miner/main/tasks/scoreboard.h MAX_SCOREBOARD",
        },
        SettingSchema {
            key: key("boardversion"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("000")),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("devicemodel"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("unknown")),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("asicmodel"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("unknown")),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("plug_sense"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("asic_enable"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("EMC2101"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("EMC2103"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("EMC2302"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("emc_int_temp"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("emc_ideality_f"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("emc_beta_comp"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("temp_offset"),
            stored_type: StoredType::I32,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("DS4432U"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("INA260"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("TPS546"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("TMP1075"),
            stored_type: StoredType::BoolAsU16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("power_cons_tgt"),
            stored_type: StoredType::U16,
            default_value: None,
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest_temp"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(65)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest_warm"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(55)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest_max"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(70)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("asicfrequency"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(485)),
            rest_name: None,
            min: Some(1),
            max: Some(65535),
            array_size: None,
            provenance: MIGRATION_PROVENANCE,
        },
        SettingSchema {
            key: key("fanspeed"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(100)),
            rest_name: None,
            min: Some(0),
            max: Some(100),
            array_size: None,
            provenance: MIGRATION_PROVENANCE,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        all_settings_schema, compatibility_writes_for_active, load_setting_value,
        migration_decisions, LoadedValue, MigrationDecision, NvsErase, NvsKeyName, NvsWrite,
        SettingSchema, StoredType, StoredValue, NVS_NAMESPACE,
    };

    fn setting_for_key(key: &str) -> SettingSchema {
        all_settings_schema()
            .into_iter()
            .find(|setting| setting.key.as_str() == key)
            .expect("test key must exist in schema")
    }

    #[test]
    fn nvs_schema_uses_upstream_namespace_main() {
        // Arrange
        let namespace = NVS_NAMESPACE;

        // Act
        let is_main_namespace = namespace == "main";

        // Assert
        assert!(is_main_namespace);
    }

    #[test]
    fn nvs_schema_preserves_active_and_legacy_key_names() {
        // Arrange
        let schema = all_settings_schema();

        // Act
        let keys = schema
            .iter()
            .map(|setting| setting.key.as_str())
            .collect::<Vec<_>>();

        // Assert
        assert!(keys.contains(&"asicfrequency_f"));
        assert!(keys.contains(&"asicfrequency"));
        assert!(keys.contains(&"manualfanspeed"));
        assert!(keys.contains(&"fanspeed"));
        assert!(keys.contains(&"usefbstartum"));
    }

    #[test]
    fn nvs_schema_rejects_keys_longer_than_15_bytes() {
        // Arrange
        let valid_keys = [
            "fbsv2authpubk",
            "emc_ideality_f",
            "emc_beta_comp",
            "power_cons_tgt",
            "selftest_temp",
            "selftest_warm",
            "selftest_max",
        ];

        // Act
        let too_long = NvsKeyName::parse("1234567890123456");
        let valid_results = valid_keys.map(NvsKeyName::parse);

        // Assert
        assert!(too_long.is_err());
        assert!(valid_results.iter().all(Result::is_ok));
    }

    #[test]
    fn nvs_schema_maps_upstream_storage_types() {
        // Arrange
        let schema = all_settings_schema();

        // Act
        let stratum_xnsub = schema
            .iter()
            .find(|setting| setting.key.as_str() == "stratumxnsub")
            .map(|setting| setting.stored_type);
        let frequency = schema
            .iter()
            .find(|setting| setting.key.as_str() == "asicfrequency_f")
            .map(|setting| setting.stored_type);

        // Assert
        assert_eq!(stratum_xnsub, Some(StoredType::BoolAsU16));
        assert_eq!(frequency, Some(StoredType::FloatString));
    }

    #[test]
    fn nvs_schema_migrates_legacy_asicfrequency_to_float_string() {
        // Arrange
        let stored = [StoredValue::u16("asicfrequency", 485)];

        // Act
        let decisions = migration_decisions(&stored);

        // Assert
        assert_eq!(
            decisions,
            vec![MigrationDecision::Write(NvsWrite::string(
                "asicfrequency_f",
                "485"
            ))]
        );
    }

    #[test]
    fn nvs_schema_migrates_legacy_fanspeed_to_manualfanspeed() {
        // Arrange
        let stored = [StoredValue::u16("fanspeed", 42)];

        // Act
        let decisions = migration_decisions(&stored);

        // Assert
        assert_eq!(
            decisions,
            vec![MigrationDecision::Write(NvsWrite::u16(
                "manualfanspeed",
                42
            ))]
        );
    }

    #[test]
    fn nvs_schema_migrates_stratum_protocol_u16_to_string() {
        // Arrange
        let cases = [
            ("stratumprot", 0, "SV1"),
            ("stratumprot", 1, "SV2"),
            ("fbstratumprot", 0, "SV1"),
            ("fbstratumprot", 1, "SV2"),
        ];

        for (key, stored_value, expected_value) in cases {
            // Act
            let decisions = migration_decisions(&[StoredValue::u16(key, stored_value)]);

            // Assert
            assert_eq!(
                decisions,
                vec![
                    MigrationDecision::Erase(NvsErase::key(key)),
                    MigrationDecision::Write(NvsWrite::string(key, expected_value)),
                ]
            );
        }
    }

    #[test]
    fn nvs_schema_migrates_sv2_channel_type_u16_to_string() {
        // Arrange
        let cases = [
            ("sv2chantype", 0, "extended"),
            ("sv2chantype", 1, "standard"),
            ("fbsv2chantype", 0, "extended"),
            ("fbsv2chantype", 1, "standard"),
            ("fbSv2ChanType", 1, "standard"),
        ];

        for (key, stored_value, expected_value) in cases {
            // Act
            let decisions = migration_decisions(&[StoredValue::u16(key, stored_value)]);

            // Assert
            if key == "fbSv2ChanType" {
                assert_eq!(
                    decisions,
                    vec![
                        MigrationDecision::Erase(NvsErase::key("fbSv2ChanType")),
                        MigrationDecision::Write(NvsWrite::string("fbsv2chantype", expected_value)),
                    ]
                );
            } else {
                assert_eq!(
                    decisions,
                    vec![
                        MigrationDecision::Erase(NvsErase::key(key)),
                        MigrationDecision::Write(NvsWrite::string(key, expected_value)),
                    ]
                );
            }
        }
    }

    #[test]
    fn nvs_schema_writes_active_frequency_legacy_compatibility_key() {
        // Arrange
        let write = NvsWrite::string("asicfrequency_f", "485.000000");

        // Act
        let compatibility_writes = compatibility_writes_for_active(&write);

        // Assert
        assert_eq!(
            compatibility_writes,
            vec![NvsWrite::u16("asicfrequency", 485)]
        );
    }

    #[test]
    fn nvs_schema_writes_active_manual_fan_legacy_compatibility_key() {
        // Arrange
        let write = NvsWrite::u16("manualfanspeed", 42);

        // Act
        let compatibility_writes = compatibility_writes_for_active(&write);

        // Assert
        assert_eq!(compatibility_writes, vec![NvsWrite::u16("fanspeed", 42)]);
    }

    #[test]
    fn nvs_schema_corrupt_float_uses_default() {
        // Arrange
        let schema = setting_for_key("asicfrequency_f");
        let stored = StoredValue::string("asicfrequency_f", "bad");

        // Act
        let loaded = load_setting_value(&schema, Some(&stored));

        // Assert
        assert_eq!(loaded, LoadedValue::Float(485.0));
    }
}
