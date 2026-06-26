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

const SETTINGS_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c settings table";
const MIGRATION_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c fallback migration";

fn key(value: &'static str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("static upstream NVS key names must fit ESP-IDF limits")
}

fn rest(value: &'static str) -> RestFieldName {
    RestFieldName::parse(value).expect("static upstream REST field names must be non-empty")
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
    use super::{all_settings_schema, NvsKeyName, StoredType, NVS_NAMESPACE};

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
}
