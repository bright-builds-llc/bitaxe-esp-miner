use super::{
    all_settings_schema, compatibility_writes_for_active, load_setting_value, migration_decisions,
    migration_rules, LoadedValue, MigrationDecision, NvsErase, NvsKeyName, NvsWrite, SettingSchema,
    StoredType, StoredValue, NVS_NAMESPACE,
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
                    MigrationDecision::Write(NvsWrite::string("sv2chantype", expected_value)),
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
fn nvs_schema_migrates_mixed_case_sv2_channel_type_after_primary_exists() {
    // Arrange
    let stored_values = [
        StoredValue::string("sv2chantype", "extended"),
        StoredValue::u16("fbSv2ChanType", 1),
    ];

    // Act
    let decisions = migration_decisions(&stored_values);

    // Assert
    assert_eq!(
        decisions,
        vec![
            MigrationDecision::Erase(NvsErase::key("fbSv2ChanType")),
            MigrationDecision::Write(NvsWrite::string("fbsv2chantype", "standard")),
        ]
    );
}

#[test]
fn nvs_schema_mixed_case_sv2_rule_names_primary_target() {
    // Arrange
    let rules = migration_rules();

    // Act
    let rule = rules
        .iter()
        .find(|rule| rule.source_key.as_str() == "fbSv2ChanType")
        .expect("mixed-case SV2 migration rule must exist");

    // Assert
    assert_eq!(rule.target_key.as_str(), "sv2chantype");
    assert!(rule.description.contains("first missing SV2 channel key"));
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
