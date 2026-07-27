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
    serde_json::from_str(include_str!("../../fixtures/settings-updates.json"))
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
                .expect("expected u16 write value must be numeric") as u16,
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
