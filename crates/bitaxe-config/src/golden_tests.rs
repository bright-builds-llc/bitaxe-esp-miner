use std::collections::BTreeMap;

use serde::Deserialize;

use crate::nvs::StoredValueKind;
use crate::{
    all_settings_schema, board_catalog, compatibility_writes_for_active, load_setting_value,
    migration_decisions, ultra_205_defaults, AsicProfile, BoardCatalogEntry, LoadedValue,
    MigrationDecision, NvsErase, NvsKeyName, NvsWrite, SettingDefault, SettingSchema, StoredType,
    StoredValue, VerificationScope, NVS_NAMESPACE,
};

#[derive(Debug, Deserialize)]
struct CsvDefaultRecord {
    key: String,
    #[serde(rename = "type")]
    kind: String,
    encoding: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct CatalogFixture {
    asic_profiles: Vec<CatalogAsicProfileFixture>,
    boards: Vec<CatalogBoardFixture>,
}

#[derive(Debug, Deserialize)]
struct CatalogAsicProfileFixture {
    profile_id: String,
    asic_model: String,
    chip_id: u16,
    default_frequency_mhz: u16,
    frequency_options: Vec<u16>,
    default_voltage_mv: u16,
    voltage_options: Vec<u16>,
    core_count: u16,
    small_core_count: u16,
    hash_domains: u8,
    default_asic_timeout: u16,
}

#[derive(Debug, Deserialize)]
struct CatalogBoardFixture {
    board_version: String,
    family: String,
    asic_profile_id: Option<String>,
    asic_model: String,
    asic_count: u8,
    power_consumption_target: u16,
    #[serde(rename = "DS4432U")]
    ds4432u: bool,
    #[serde(rename = "INA260")]
    ina260: bool,
    #[serde(rename = "TPS546")]
    tps546: bool,
    plug_sense: bool,
    asic_enable: bool,
    verification_scope: String,
    frequency_options: Option<Vec<u16>>,
    voltage_options: Option<Vec<u16>>,
    default_frequency_mhz: Option<u16>,
    default_voltage_mv: Option<u16>,
    core_count: Option<u16>,
    small_core_count: Option<u16>,
    hash_domains: Option<u8>,
    default_asic_timeout: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct NvsSchemaFixture {
    metadata: NvsSchemaMetadataFixture,
    settings: Vec<NvsSettingFixture>,
}

#[derive(Debug, Deserialize)]
struct NvsSchemaMetadataFixture {
    namespace: String,
}

#[derive(Debug, Deserialize)]
struct NvsSettingFixture {
    nvs_key_name: String,
    #[serde(rename = "type")]
    kind: String,
    rest_name: Option<String>,
    default: Option<serde_json::Value>,
    min: Option<i32>,
    max: Option<i32>,
    array_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct NvsMigrationsFixture {
    cases: Vec<NvsMigrationCaseFixture>,
}

#[derive(Debug, Deserialize)]
struct NvsMigrationCaseFixture {
    name: String,
    legacy_key: Option<String>,
    active_key: Option<String>,
    key: Option<String>,
    input_type: String,
    input: serde_json::Value,
    #[serde(default)]
    operations: Vec<NvsOperationFixture>,
    loaded: Option<NvsLoadedFixture>,
}

#[derive(Debug, Deserialize)]
struct NvsOperationFixture {
    operation: String,
    nvs_key_name: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct NvsLoadedFixture {
    #[serde(rename = "type")]
    kind: String,
    value: serde_json::Value,
}

#[test]
fn ultra_205_defaults_match_golden_fixture() {
    // Arrange
    let fixture = ultra_205_default_fixture();
    let defaults = ultra_205_defaults();
    let primary_pool = defaults.primary_pool();
    let fallback_pool = defaults.fallback_pool();

    // Act
    let actual_defaults = [
        ("hostname", "string", defaults.hostname().to_owned()),
        ("stratumurl", "string", primary_pool.url().to_owned()),
        ("stratumport", "u16", primary_pool.port().to_string()),
        ("stratumtls", "u16", primary_pool.tls().to_string()),
        ("stratumcert", "string", primary_pool.cert().to_owned()),
        ("stratumuser", "string", primary_pool.user().to_owned()),
        ("stratumpass", "string", primary_pool.password().to_owned()),
        ("stratumdiff", "u16", primary_pool.difficulty().to_string()),
        (
            "stratumxnsub",
            "u16",
            primary_pool.extranonce_subscribe().to_string(),
        ),
        ("fbstratumurl", "string", fallback_pool.url().to_owned()),
        ("fbstratumport", "u16", fallback_pool.port().to_string()),
        ("fbstratumtls", "u16", fallback_pool.tls().to_string()),
        ("fbstratumcert", "string", fallback_pool.cert().to_owned()),
        ("fbstratumuser", "string", fallback_pool.user().to_owned()),
        (
            "fbstratumpass",
            "string",
            fallback_pool.password().to_owned(),
        ),
        (
            "fbstratumdiff",
            "u16",
            fallback_pool.difficulty().to_string(),
        ),
        (
            "fbstratumxnsum",
            "u16",
            fallback_pool.extranonce_subscribe().to_string(),
        ),
        (
            "asicfrequency",
            "u16",
            defaults.asic_frequency_mhz().to_string(),
        ),
        ("asicvoltage", "u16", defaults.asic_voltage_mv().to_string()),
        ("asicmodel", "string", defaults.asic_model().to_owned()),
        ("devicemodel", "string", defaults.device_model().to_owned()),
        (
            "boardversion",
            "string",
            defaults.board_version().to_owned(),
        ),
        ("rotation", "u16", defaults.rotation().to_string()),
        (
            "autofanspeed",
            "u16",
            bool_as_fixture_u16(defaults.auto_fan_speed()).to_owned(),
        ),
        ("fanspeed", "u16", defaults.manual_fan_speed().to_string()),
        (
            "selftest",
            "u16",
            bool_as_fixture_u16(defaults.self_test()).to_owned(),
        ),
        (
            "overheat_mode",
            "u16",
            bool_as_fixture_u16(defaults.overheat_mode()).to_owned(),
        ),
    ];

    // Assert
    assert_eq!(
        fixture
            .get("main")
            .expect("fixture must include main namespace row")
            .kind,
        "namespace"
    );
    for (key, encoding, value) in actual_defaults {
        assert_csv_fixture_value(&fixture, key, encoding, &value);
    }
}

#[test]
fn board_catalog_matches_golden_fixture() {
    // Arrange
    let fixture = catalog_fixture();
    let profile_by_id = fixture
        .asic_profiles
        .iter()
        .map(|profile| (profile.profile_id.as_str(), profile))
        .collect::<BTreeMap<_, _>>();

    // Act
    let catalog = board_catalog();

    // Assert
    assert_eq!(catalog.len(), fixture.boards.len());
    for (entry, fixture_board) in catalog.iter().zip(fixture.boards.iter()) {
        assert_board_matches_fixture(*entry, fixture_board, &profile_by_id);
    }
}

#[test]
fn nvs_schema_matches_golden_fixture() {
    // Arrange
    let fixture = nvs_schema_fixture();

    // Act
    let schema = all_settings_schema();

    // Assert
    assert_eq!(fixture.metadata.namespace, NVS_NAMESPACE);
    assert_eq!(schema.len(), fixture.settings.len());
    for (actual, expected) in schema.iter().zip(fixture.settings.iter()) {
        assert_schema_row_matches_fixture(actual, expected);
    }
}

#[test]
fn nvs_migrations_match_golden_fixture() {
    // Arrange
    let fixture = nvs_migrations_fixture();

    for case in fixture.cases {
        // Act / Assert
        let is_active_compatibility_case =
            case.active_key.is_some() && case.legacy_key.is_none() && case.key.is_none();
        if is_active_compatibility_case {
            let active_key = case
                .active_key
                .as_deref()
                .expect("active compatibility case must name active key");
            assert_active_compatibility_case_matches_fixture(active_key, &case);
            continue;
        }

        if case.loaded.is_some() {
            assert_loaded_case_matches_fixture(&case);
            continue;
        }

        assert_migration_case_matches_fixture(&case);
    }
}

fn ultra_205_default_fixture() -> BTreeMap<String, CsvDefaultRecord> {
    let mut reader = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_reader(include_str!("../fixtures/ultra-205-defaults.csv").as_bytes());

    reader
        .deserialize::<CsvDefaultRecord>()
        .map(|row| {
            let row = row.expect("ultra 205 defaults fixture row must parse");
            (row.key.clone(), row)
        })
        .collect()
}

fn catalog_fixture() -> CatalogFixture {
    serde_json::from_str(include_str!("../fixtures/catalog.json"))
        .expect("catalog fixture must parse")
}

fn nvs_schema_fixture() -> NvsSchemaFixture {
    serde_json::from_str(include_str!("../fixtures/nvs-schema.json"))
        .expect("nvs schema fixture must parse")
}

fn nvs_migrations_fixture() -> NvsMigrationsFixture {
    serde_json::from_str(include_str!("../fixtures/nvs-migrations.json"))
        .expect("nvs migrations fixture must parse")
}

fn assert_csv_fixture_value(
    fixture: &BTreeMap<String, CsvDefaultRecord>,
    key: &str,
    encoding: &str,
    value: &str,
) {
    let record = fixture
        .get(key)
        .unwrap_or_else(|| panic!("fixture must include default key {key}"));
    assert_eq!(record.kind, "data", "fixture key {key} kind mismatch");
    assert_eq!(
        record.encoding, encoding,
        "fixture key {key} encoding mismatch"
    );
    assert_eq!(record.value, value, "fixture key {key} value mismatch");
}

fn bool_as_fixture_u16(value: bool) -> &'static str {
    if value {
        return "1";
    }

    "0"
}

fn assert_board_matches_fixture(
    entry: BoardCatalogEntry,
    fixture: &CatalogBoardFixture,
    profile_by_id: &BTreeMap<&str, &CatalogAsicProfileFixture>,
) {
    let asic = entry.asic();
    let capabilities = entry.capabilities();
    let profile_id = fixture
        .asic_profile_id
        .as_deref()
        .unwrap_or(fixture.asic_model.as_str());
    let fixture_profile = profile_by_id
        .get(profile_id)
        .unwrap_or_else(|| panic!("fixture profile missing for {profile_id}"));

    assert_eq!(entry.board_version(), fixture.board_version);
    assert_eq!(entry.family(), fixture.family);
    assert_eq!(asic.model(), fixture.asic_model);
    assert_eq!(entry.asic_count(), fixture.asic_count);
    assert_eq!(
        entry.power_consumption_target(),
        fixture.power_consumption_target
    );
    assert_eq!(capabilities.ds4432u(), fixture.ds4432u);
    assert_eq!(capabilities.ina260(), fixture.ina260);
    assert_eq!(capabilities.tps546(), fixture.tps546);
    assert_eq!(capabilities.plug_sense(), fixture.plug_sense);
    assert_eq!(capabilities.asic_enable(), fixture.asic_enable);
    assert_eq!(
        verification_scope_name(entry.verification_scope()),
        fixture.verification_scope
    );

    assert_asic_profile_matches_fixture(asic, fixture_profile);
    assert_optional_board_profile_fields_match(asic, fixture);
}

fn assert_asic_profile_matches_fixture(asic: AsicProfile, fixture: &CatalogAsicProfileFixture) {
    assert_eq!(asic.profile_id(), fixture.profile_id);
    assert_eq!(asic.model(), fixture.asic_model);
    assert_eq!(asic.chip_id(), fixture.chip_id);
    assert_eq!(asic.default_frequency_mhz(), fixture.default_frequency_mhz);
    assert_eq!(asic.frequency_options(), fixture.frequency_options);
    assert_eq!(asic.default_voltage_mv(), fixture.default_voltage_mv);
    assert_eq!(asic.voltage_options(), fixture.voltage_options);
    assert_eq!(asic.core_count(), fixture.core_count);
    assert_eq!(asic.small_core_count(), fixture.small_core_count);
    assert_eq!(asic.hash_domains(), fixture.hash_domains);
    assert_eq!(asic.default_asic_timeout(), fixture.default_asic_timeout);
}

fn assert_optional_board_profile_fields_match(asic: AsicProfile, fixture: &CatalogBoardFixture) {
    if let Some(frequency_options) = &fixture.frequency_options {
        assert_eq!(asic.frequency_options(), frequency_options);
    }
    if let Some(voltage_options) = &fixture.voltage_options {
        assert_eq!(asic.voltage_options(), voltage_options);
    }
    if let Some(default_frequency_mhz) = fixture.default_frequency_mhz {
        assert_eq!(asic.default_frequency_mhz(), default_frequency_mhz);
    }
    if let Some(default_voltage_mv) = fixture.default_voltage_mv {
        assert_eq!(asic.default_voltage_mv(), default_voltage_mv);
    }
    if let Some(core_count) = fixture.core_count {
        assert_eq!(asic.core_count(), core_count);
    }
    if let Some(small_core_count) = fixture.small_core_count {
        assert_eq!(asic.small_core_count(), small_core_count);
    }
    if let Some(hash_domains) = fixture.hash_domains {
        assert_eq!(asic.hash_domains(), hash_domains);
    }
    if let Some(default_asic_timeout) = fixture.default_asic_timeout {
        assert_eq!(asic.default_asic_timeout(), default_asic_timeout);
    }
}

fn verification_scope_name(scope: VerificationScope) -> &'static str {
    match scope {
        VerificationScope::ActiveUltra205 => "active_ultra_205",
        VerificationScope::NotHardwareVerified => "not_hardware_verified",
    }
}

fn assert_schema_row_matches_fixture(actual: &SettingSchema, expected: &NvsSettingFixture) {
    assert_eq!(actual.key.as_str(), expected.nvs_key_name);
    assert_eq!(stored_type_name(actual.stored_type), expected.kind);
    assert_eq!(
        actual.rest_name.as_ref().map(|name| name.as_str()),
        expected.rest_name.as_deref()
    );
    assert_setting_default_matches_fixture(
        actual.default_value.as_ref(),
        expected.default.as_ref(),
    );
    assert_eq!(
        actual.min, expected.min,
        "schema min mismatch for {}",
        expected.nvs_key_name
    );
    assert_eq!(
        actual.max, expected.max,
        "schema max mismatch for {}",
        expected.nvs_key_name
    );
    assert_eq!(
        actual.array_size, expected.array_size,
        "schema array_size mismatch for {}",
        expected.nvs_key_name
    );
}

fn stored_type_name(stored_type: StoredType) -> &'static str {
    match stored_type {
        StoredType::Str => "str",
        StoredType::U16 => "u16",
        StoredType::I32 => "i32",
        StoredType::U64 => "u64",
        StoredType::FloatString => "float_string",
        StoredType::BoolAsU16 => "bool_as_u16",
    }
}

fn assert_setting_default_matches_fixture(
    actual: Option<&SettingDefault>,
    expected: Option<&serde_json::Value>,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(SettingDefault::Str(actual)), Some(expected)) => {
            assert_eq!(expected.as_str(), Some(*actual));
        }
        (Some(SettingDefault::U16(actual)), Some(expected)) => {
            assert_eq!(expected.as_u64(), Some(u64::from(*actual)));
        }
        (Some(SettingDefault::I32(actual)), Some(expected)) => {
            assert_eq!(expected.as_i64(), Some(i64::from(*actual)));
        }
        (Some(SettingDefault::U64(actual)), Some(expected)) => {
            assert_eq!(expected.as_u64(), Some(*actual));
        }
        (Some(SettingDefault::Float(actual)), Some(expected)) => {
            assert_eq!(expected.as_f64(), Some(f64::from(*actual)));
        }
        (Some(SettingDefault::Bool(actual)), Some(expected)) => {
            assert_eq!(expected.as_bool(), Some(*actual));
        }
        (actual, expected) => {
            panic!("default mismatch: actual {actual:?}, fixture {expected:?}");
        }
    }
}

fn assert_active_compatibility_case_matches_fixture(
    active_key: &str,
    case: &NvsMigrationCaseFixture,
) {
    let write = nvs_write_from_fixture(active_key, &case.input_type, &case.input);
    let expected_writes = case
        .operations
        .iter()
        .map(nvs_write_from_operation)
        .collect::<Vec<_>>();

    assert_eq!(
        compatibility_writes_for_active(&write),
        expected_writes,
        "active compatibility case {} mismatch",
        case.name
    );
}

fn assert_loaded_case_matches_fixture(case: &NvsMigrationCaseFixture) {
    let key = case.key.as_deref().expect("loaded case must name key");
    let stored = stored_value_from_fixture(key, &case.input_type, &case.input);
    let schema = all_settings_schema()
        .into_iter()
        .find(|schema| schema.key.as_str() == key)
        .unwrap_or_else(|| panic!("schema missing for loaded case key {key}"));
    let expected = loaded_value_from_fixture(
        case.loaded
            .as_ref()
            .expect("loaded case must include expected loaded value"),
    );

    assert_eq!(
        load_setting_value(&schema, Some(&stored)),
        expected,
        "loaded case {} mismatch",
        case.name
    );
}

fn assert_migration_case_matches_fixture(case: &NvsMigrationCaseFixture) {
    let key = case
        .legacy_key
        .as_deref()
        .or(case.key.as_deref())
        .expect("migration case must name source key");
    let stored = stored_value_from_fixture(key, &case.input_type, &case.input);
    let expected_decisions = case
        .operations
        .iter()
        .map(migration_decision_from_operation)
        .collect::<Vec<_>>();

    assert_eq!(
        migration_decisions(&[stored]),
        expected_decisions,
        "migration case {} mismatch",
        case.name
    );
}

fn stored_value_from_fixture(key: &str, kind: &str, value: &serde_json::Value) -> StoredValue {
    StoredValue {
        key: nvs_key(key),
        value: match kind {
            "str" => StoredValueKind::String(fixture_string(value).to_owned()),
            "u16" => StoredValueKind::U16(fixture_u16(value)),
            other => panic!("unsupported fixture stored value kind {other} for {key}"),
        },
    }
}

fn nvs_write_from_fixture(key: &str, kind: &str, value: &serde_json::Value) -> NvsWrite {
    match kind {
        "str" => NvsWrite::String {
            key: nvs_key(key),
            value: fixture_string(value).to_owned(),
        },
        "u16" => NvsWrite::U16 {
            key: nvs_key(key),
            value: fixture_u16(value),
        },
        other => panic!("unsupported fixture write kind {other} for {key}"),
    }
}

fn nvs_write_from_operation(operation: &NvsOperationFixture) -> NvsWrite {
    assert_eq!(operation.operation, "write");
    nvs_write_from_fixture(
        &operation.nvs_key_name,
        operation
            .kind
            .as_deref()
            .expect("write operation must name type"),
        operation
            .value
            .as_ref()
            .expect("write operation must include value"),
    )
}

fn migration_decision_from_operation(operation: &NvsOperationFixture) -> MigrationDecision {
    match operation.operation.as_str() {
        "erase" => MigrationDecision::Erase(NvsErase {
            key: nvs_key(&operation.nvs_key_name),
        }),
        "write" => MigrationDecision::Write(nvs_write_from_operation(operation)),
        other => panic!("unsupported fixture migration operation {other}"),
    }
}

fn loaded_value_from_fixture(fixture: &NvsLoadedFixture) -> LoadedValue {
    match fixture.kind.as_str() {
        "float" => LoadedValue::Float(
            fixture
                .value
                .as_f64()
                .expect("float loaded value must be numeric") as f32,
        ),
        other => panic!("unsupported fixture loaded value kind {other}"),
    }
}

fn fixture_string(value: &serde_json::Value) -> &str {
    value.as_str().expect("fixture value must be a string")
}

fn fixture_u16(value: &serde_json::Value) -> u16 {
    let raw_value = value
        .as_u64()
        .expect("fixture value must be an unsigned integer");
    u16::try_from(raw_value).expect("fixture value must fit u16")
}

fn nvs_key(value: &str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("fixture NVS key must be valid")
}
