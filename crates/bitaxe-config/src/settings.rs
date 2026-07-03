//! Pure settings update decisions for future API and firmware adapters.
//!
//! Reference: `reference/esp-miner/main/http_server/http_server.c`
//! Reference: `reference/esp-miner/main/nvs_config.c`

use std::collections::BTreeMap;

use crate::{
    all_settings_schema, compatibility_writes_for_active, BoolLike, ConfigValidationError,
    FanDutyPercent, Hostname, MinFanDutyPercent, NvsWrite, PortNumber, SettingSchema, StoredType,
    StratumProtocol, Sv2ChannelType, TemperatureCelsius, TlsMode, WifiPassword, WifiSsid,
};

/// Raw setting value accepted at the pure update boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum RawSettingValue {
    /// String value.
    String(String),
    /// Integral JSON/API number.
    Number(i64),
    /// Floating-point JSON/API number.
    Float(f64),
    /// Boolean value.
    Bool(bool),
}

impl RawSettingValue {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
        }
    }
}

/// Collection of REST-named settings to update.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettingsPatch {
    values: BTreeMap<String, RawSettingValue>,
}

impl SettingsPatch {
    /// Creates an empty settings patch.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Creates a settings patch from REST-name/value pairs.
    pub fn from_pairs<I, K>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, RawSettingValue)>,
        K: Into<String>,
    {
        let mut patch = Self::new();
        for (rest_name, value) in pairs {
            patch.insert(rest_name, value);
        }
        patch
    }

    /// Inserts or replaces a REST-named setting value.
    pub fn insert(&mut self, rest_name: impl Into<String>, value: RawSettingValue) {
        self.values.insert(rest_name.into(), value);
    }

    fn get(&self, rest_name: &str) -> Option<&RawSettingValue> {
        self.values.get(rest_name)
    }
}

/// Settings update result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsUpdateDecision {
    /// Every supplied known field was valid; adapters may apply these writes.
    Accepted { writes: Vec<NvsWrite> },
    /// At least one supplied known field was invalid; no writes are emitted.
    Rejected { errors: Vec<ConfigValidationError> },
}

/// Alias retained for callers that want an error type name at the settings boundary.
pub type SettingsUpdateError = ConfigValidationError;

/// Applies an upstream-schema-driven settings patch without performing I/O.
#[must_use]
pub fn apply_settings_patch(patch: &SettingsPatch) -> SettingsUpdateDecision {
    let schema = all_settings_schema();
    let mut active_writes = Vec::new();
    let mut errors = Vec::new();

    for setting in &schema {
        let Some(rest_name) = &setting.rest_name else {
            continue;
        };

        let Some(raw_value) = patch.get(rest_name.as_str()) else {
            continue;
        };

        match validate_setting(setting, raw_value) {
            Ok(write) => active_writes.push(write),
            Err(error) => errors.push(error),
        }
    }

    if !errors.is_empty() {
        return SettingsUpdateDecision::Rejected { errors };
    }

    let mut writes = Vec::new();
    for write in active_writes {
        let compatibility_writes = compatibility_writes_for_active(&write);
        writes.push(write);
        writes.extend(compatibility_writes);
    }

    SettingsUpdateDecision::Accepted { writes }
}

fn validate_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    match schema.stored_type {
        StoredType::Str => validate_string_setting(schema, raw_value),
        StoredType::U16 => validate_u16_setting(schema, raw_value),
        StoredType::I32 => validate_i32_setting(schema, raw_value),
        StoredType::U64 => validate_u64_setting(schema, raw_value),
        StoredType::FloatString => validate_float_string_setting(schema, raw_value),
        StoredType::BoolAsU16 => validate_bool_setting(schema, raw_value),
    }
}

fn validate_string_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let RawSettingValue::String(value) = raw_value else {
        return Err(invalid_type(schema, raw_value));
    };

    validate_schema_length(schema, value)?;

    match schema.key.as_str() {
        "wifissid" => {
            WifiSsid::parse(value.clone())?;
        }
        "wifipass" => {
            WifiPassword::parse(value.clone())?;
        }
        "hostname" => {
            Hostname::parse(value.clone())?;
        }
        "stratumprot" | "fbstratumprot" => {
            StratumProtocol::parse(value)?;
        }
        "sv2chantype" | "fbsv2chantype" => {
            Sv2ChannelType::parse(value)?;
        }
        _ => {}
    }

    Ok(NvsWrite::String {
        key: schema.key.clone(),
        value: value.clone(),
    })
}

fn validate_u16_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let value = number_value(schema, raw_value)?;
    validate_schema_range(schema, value)?;

    match schema.key.as_str() {
        "manualfanspeed" => {
            FanDutyPercent::parse(value)?;
        }
        "minfanspeed" => {
            MinFanDutyPercent::parse(value)?;
        }
        "temptarget" => {
            TemperatureCelsius::parse(value)?;
        }
        "stratumport" | "fbstratumport" => {
            PortNumber::parse(value)?;
        }
        "stratumtls" | "fbstratumtls" => {
            TlsMode::parse(value)?;
        }
        "rotation" if ![0, 90, 180, 270].contains(&value) => {
            return Err(ConfigValidationError::InvalidEnum {
                field: field_name(schema),
                value: value.to_string(),
            });
        }
        _ => {}
    }

    Ok(NvsWrite::U16 {
        key: schema.key.clone(),
        value: value as u16,
    })
}

fn validate_i32_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let value = number_value(schema, raw_value)?;
    validate_schema_range(schema, value)?;

    Ok(NvsWrite::I32 {
        key: schema.key.clone(),
        value: value as i32,
    })
}

fn validate_u64_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let value = number_value(schema, raw_value)?;
    if value < 0 {
        return Err(ConfigValidationError::OutOfRange {
            field: field_name(schema),
            min: 0,
            max: i64::MAX,
            actual: value,
        });
    }

    validate_schema_range(schema, value)?;

    Ok(NvsWrite::U64 {
        key: schema.key.clone(),
        value: value as u64,
    })
}

fn validate_float_string_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let value = float_value(schema, raw_value)?;
    validate_float_schema_range(schema, value)?;

    Ok(NvsWrite::String {
        key: schema.key.clone(),
        value: format_active_float_string(value),
    })
}

fn validate_bool_setting(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<NvsWrite, ConfigValidationError> {
    let bool_like = match raw_value {
        RawSettingValue::Bool(value) => BoolLike::from_bool(*value),
        RawSettingValue::Number(value) => BoolLike::from_number(*value, field_name(schema))?,
        _ => return Err(invalid_type(schema, raw_value)),
    };

    let value = i64::from(bool_like.as_u16());
    validate_schema_range(schema, value)?;

    Ok(NvsWrite::U16 {
        key: schema.key.clone(),
        value: bool_like.as_u16(),
    })
}

fn number_value(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<i64, ConfigValidationError> {
    let RawSettingValue::Number(value) = raw_value else {
        return Err(invalid_type(schema, raw_value));
    };

    Ok(*value)
}

fn float_value(
    schema: &SettingSchema,
    raw_value: &RawSettingValue,
) -> Result<f64, ConfigValidationError> {
    match raw_value {
        RawSettingValue::Number(value) => Ok(*value as f64),
        RawSettingValue::Float(value) if value.is_finite() => Ok(*value),
        RawSettingValue::Float(value) => Err(ConfigValidationError::InvalidEnum {
            field: field_name(schema),
            value: value.to_string(),
        }),
        _ => Err(invalid_type(schema, raw_value)),
    }
}

fn validate_schema_length(
    schema: &SettingSchema,
    value: &str,
) -> Result<(), ConfigValidationError> {
    let min = schema.min.unwrap_or(0).max(0) as usize;
    let max = schema.max.unwrap_or(i32::MAX).max(0) as usize;
    let actual = value.len();
    if actual < min || actual > max {
        return Err(ConfigValidationError::InvalidLength {
            field: field_name(schema),
            min,
            max,
            actual,
        });
    }

    Ok(())
}

fn validate_schema_range(schema: &SettingSchema, value: i64) -> Result<(), ConfigValidationError> {
    let min = i64::from(schema.min.unwrap_or(i32::MIN));
    let max = i64::from(schema.max.unwrap_or(i32::MAX));
    if value < min || value > max {
        return Err(ConfigValidationError::OutOfRange {
            field: field_name(schema),
            min,
            max,
            actual: value,
        });
    }

    Ok(())
}

fn validate_float_schema_range(
    schema: &SettingSchema,
    value: f64,
) -> Result<(), ConfigValidationError> {
    let min = f64::from(schema.min.unwrap_or(i32::MIN));
    let max = f64::from(schema.max.unwrap_or(i32::MAX));
    if value < min || value > max {
        return Err(ConfigValidationError::OutOfRange {
            field: field_name(schema),
            min: min as i64,
            max: max as i64,
            actual: value as i64,
        });
    }

    Ok(())
}

fn format_active_float_string(value: f64) -> String {
    format!("{value:.6}")
}

fn invalid_type(schema: &SettingSchema, raw_value: &RawSettingValue) -> ConfigValidationError {
    ConfigValidationError::InvalidEnum {
        field: field_name(schema),
        value: raw_value.kind_name().to_owned(),
    }
}

fn field_name(schema: &SettingSchema) -> &'static str {
    match schema.key.as_str() {
        "wifissid" => "ssid",
        "wifipass" => "wifiPass",
        "hostname" => "hostname",
        "stratumprot" => "stratumProtocol",
        "stratumport" => "stratumPort",
        "stratumtls" => "stratumTLS",
        "sv2chantype" => "stratumV2ChannelType",
        "fbstratumprot" => "fallbackStratumProtocol",
        "fbstratumport" => "fallbackStratumPort",
        "fbstratumtls" => "fallbackStratumTLS",
        "fbsv2chantype" => "fallbackStratumV2ChannelType",
        "asicfrequency_f" => "frequency",
        "asicvoltage" => "coreVoltage",
        "autofanspeed" => "autofanspeed",
        "manualfanspeed" => "manualFanSpeed",
        "minfanspeed" => "minFanSpeed",
        "temptarget" => "temptarget",
        "overheat_mode" => "overheat_mode",
        "rotation" => "rotation",
        _ => "setting",
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{
        apply_settings_patch, ConfigValidationError, NvsKeyName, NvsWrite, RawSettingValue,
        SettingsPatch, SettingsUpdateDecision,
    };

    #[derive(Debug, Deserialize)]
    struct Fixture {
        valid: Vec<ValidFixtureCase>,
        invalid: Vec<InvalidFixtureCase>,
    }

    #[derive(Debug, Deserialize)]
    struct ValidFixtureCase {
        field: String,
        value: serde_json::Value,
        nvs_key_name: String,
        expected_writes: Vec<ExpectedWriteFixture>,
    }

    #[derive(Debug, Deserialize)]
    struct InvalidFixtureCase {
        field: String,
        value: serde_json::Value,
    }

    #[derive(Debug, Deserialize)]
    struct ExpectedWriteFixture {
        #[serde(rename = "type")]
        kind: String,
        nvs_key_name: String,
        value: serde_json::Value,
    }

    fn fixture() -> Fixture {
        serde_json::from_str(include_str!("../fixtures/settings-updates.json"))
            .expect("settings update fixture must be valid JSON")
    }

    fn raw_setting_value(value: &serde_json::Value) -> RawSettingValue {
        match value {
            serde_json::Value::String(value) => RawSettingValue::String(value.clone()),
            serde_json::Value::Number(value) => {
                if let Some(integer) = value.as_i64() {
                    return RawSettingValue::Number(integer);
                }

                RawSettingValue::Float(
                    value
                        .as_f64()
                        .expect("fixture numeric value must fit f64 for settings tests"),
                )
            }
            serde_json::Value::Bool(value) => RawSettingValue::Bool(*value),
            other => panic!("unsupported fixture value: {other:?}"),
        }
    }

    fn one_field_patch(field: &str, value: RawSettingValue) -> SettingsPatch {
        SettingsPatch::from_pairs([(field, value)])
    }

    fn writes_contain_key(writes: &[NvsWrite], key: &str) -> bool {
        writes.iter().any(|write| match write {
            NvsWrite::String { key: write_key, .. }
            | NvsWrite::U16 { key: write_key, .. }
            | NvsWrite::I32 { key: write_key, .. }
            | NvsWrite::U64 { key: write_key, .. } => write_key.as_str() == key,
        })
    }

    fn expected_write(fixture: ExpectedWriteFixture) -> NvsWrite {
        match fixture.kind.as_str() {
            "string" => NvsWrite::String {
                key: expected_key(&fixture.nvs_key_name),
                value: fixture
                    .value
                    .as_str()
                    .expect("expected string write value must be a string")
                    .to_owned(),
            },
            "u16" => NvsWrite::U16 {
                key: expected_key(&fixture.nvs_key_name),
                value: fixture
                    .value
                    .as_u64()
                    .expect("expected u16 write value must be numeric")
                    as u16,
            },
            other => panic!("unsupported expected write kind: {other}"),
        }
    }

    fn expected_key(value: &str) -> NvsKeyName {
        NvsKeyName::parse(value).expect("fixture expected write key must be a valid NVS key")
    }

    #[test]
    fn validation_accepts_valid_settings_update_fixture() {
        // Arrange
        let fixture = fixture();

        for case in fixture.valid {
            let patch = one_field_patch(&case.field, raw_setting_value(&case.value));

            // Act
            let decision = apply_settings_patch(&patch);

            // Assert
            let SettingsUpdateDecision::Accepted { writes } = decision else {
                panic!("valid fixture case rejected: {}", case.field);
            };
            let expected_writes = case
                .expected_writes
                .into_iter()
                .map(expected_write)
                .collect::<Vec<_>>();
            assert!(writes_contain_key(&writes, &case.nvs_key_name));
            assert_eq!(writes, expected_writes);
        }
    }

    #[test]
    fn validation_rejects_invalid_settings_update_fixture() {
        // Arrange
        let fixture = fixture();

        for case in fixture.invalid {
            let patch = one_field_patch(&case.field, raw_setting_value(&case.value));

            // Act
            let decision = apply_settings_patch(&patch);

            // Assert
            let SettingsUpdateDecision::Rejected { errors } = decision else {
                panic!("invalid fixture case accepted: {}", case.field);
            };
            assert!(!errors.is_empty());
        }
    }

    #[test]
    fn validation_accepts_schema_valid_custom_frequency_and_voltage() {
        // Arrange
        let patch = SettingsPatch::from_pairs([
            ("frequency", RawSettingValue::Number(486)),
            ("coreVoltage", RawSettingValue::Number(1199)),
        ]);

        // Act
        let decision = apply_settings_patch(&patch);

        // Assert
        let SettingsUpdateDecision::Accepted { writes } = decision else {
            panic!("schema-valid custom frequency and voltage should be accepted");
        };
        assert!(writes_contain_key(&writes, "asicfrequency_f"));
        assert!(writes.contains(&NvsWrite::u16("asicvoltage", 1199)));
    }

    #[test]
    fn validation_frequency_and_manual_fan_updates_emit_legacy_mirror_writes() {
        // Arrange
        let patch = SettingsPatch::from_pairs([
            ("frequency", RawSettingValue::Number(485)),
            ("manualFanSpeed", RawSettingValue::Number(42)),
        ]);

        // Act
        let decision = apply_settings_patch(&patch);

        // Assert
        assert_eq!(
            decision,
            SettingsUpdateDecision::Accepted {
                writes: vec![
                    NvsWrite::string("asicfrequency_f", "485.000000"),
                    NvsWrite::u16("asicfrequency", 485),
                    NvsWrite::u16("manualfanspeed", 42),
                    NvsWrite::u16("fanspeed", 42),
                ],
            }
        );
    }

    #[test]
    fn validation_bool_values_store_as_u16() {
        // Arrange
        let patch = one_field_patch("autofanspeed", RawSettingValue::Bool(true));

        // Act
        let decision = apply_settings_patch(&patch);

        // Assert
        assert_eq!(
            decision,
            SettingsUpdateDecision::Accepted {
                writes: vec![NvsWrite::u16("autofanspeed", 1)],
            }
        );
    }

    #[test]
    fn validation_wifi_credentials_write_upstream_nvs_keys() {
        // Arrange
        let patch = SettingsPatch::from_pairs([
            ("ssid", RawSettingValue::String("lab-network".to_owned())),
            (
                "wifiPass",
                RawSettingValue::String("lab-password".to_owned()),
            ),
        ]);

        // Act
        let decision = apply_settings_patch(&patch);

        // Assert
        assert_eq!(
            decision,
            SettingsUpdateDecision::Accepted {
                writes: vec![
                    NvsWrite::string("wifissid", "lab-network"),
                    NvsWrite::string("wifipass", "lab-password"),
                ],
            }
        );
    }

    #[test]
    fn validation_wifi_credentials_report_public_field_names() {
        // Arrange
        let patch = SettingsPatch::from_pairs([
            ("ssid", RawSettingValue::String(String::new())),
            ("wifiPass", RawSettingValue::String("p".repeat(64))),
        ]);

        // Act
        let decision = apply_settings_patch(&patch);

        // Assert
        let SettingsUpdateDecision::Rejected { errors } = decision else {
            panic!("invalid Wi-Fi credentials should be rejected");
        };
        assert!(errors.iter().any(|error| {
            matches!(
                error,
                ConfigValidationError::InvalidLength { field: "ssid", .. }
            )
        }));
        assert!(errors.iter().any(|error| {
            matches!(
                error,
                ConfigValidationError::InvalidLength {
                    field: "wifiPass",
                    ..
                }
            )
        }));
    }
}
