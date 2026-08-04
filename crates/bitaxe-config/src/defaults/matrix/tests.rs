use serde::Deserialize;

use super::{board_profile_defaults, BoardProfileDefaults, BoardProfileSeedKind};
use crate::{board_catalog, VerificationScope};

#[derive(Debug, Deserialize)]
struct Fixture {
    metadata: FixtureMetadata,
    profiles: Vec<FixtureProfile>,
}

#[derive(Debug, Deserialize)]
struct FixtureMetadata {
    schema_version: String,
    reference_commit: String,
    source_pattern: String,
}

#[derive(Debug, Deserialize)]
struct FixtureProfile {
    seed_id: String,
    source_path: String,
    seed_kind: String,
    board_version: String,
    device_model: String,
    asic_model: String,
    asic_frequency_mhz: u16,
    asic_voltage_mv: u16,
    rotation: u16,
    auto_fan_speed: bool,
    manual_fan_speed: u16,
    self_test: bool,
    overheat_mode: bool,
    primary_pool_port: u16,
}

#[test]
fn matrix_matches_pinned_seed_fixture() {
    // Arrange
    let fixture = fixture();

    // Act
    let defaults = board_profile_defaults();

    // Assert
    assert_eq!(
        fixture.metadata.schema_version,
        "bitaxe-board-profile-defaults-v1"
    );
    assert_eq!(
        fixture.metadata.reference_commit,
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(
        fixture.metadata.source_pattern,
        "reference/esp-miner/config-*.cvs"
    );
    assert_eq!(defaults.len(), 21);
    assert_eq!(defaults.len(), fixture.profiles.len());
    for (actual, expected) in defaults.iter().zip(fixture.profiles.iter()) {
        assert_matches_fixture(*actual, expected);
    }
}

#[test]
fn numbered_seeds_match_catalog_discriminators_and_defaults() {
    // Arrange
    let numbered = board_profile_defaults()
        .iter()
        .filter(|profile| profile.seed_kind() == BoardProfileSeedKind::Numbered)
        .collect::<Vec<_>>();

    // Act and assert
    assert_eq!(numbered.len(), 20);
    for defaults in numbered {
        let entry = board_catalog()
            .iter()
            .find(|entry| entry.board_version() == defaults.board_version())
            .expect("numbered defaults seed must have a catalog entry");
        let asic = entry.asic();

        assert_eq!(entry.family().to_ascii_lowercase(), defaults.device_model());
        assert_eq!(asic.model(), defaults.asic_model());
        assert_eq!(asic.default_frequency_mhz(), defaults.asic_frequency_mhz());
        assert_eq!(asic.default_voltage_mv(), defaults.asic_voltage_mv());
        let expected_scope = if defaults.board_version() == "205" {
            VerificationScope::ActiveUltra205
        } else {
            VerificationScope::NotHardwareVerified
        };
        assert_eq!(entry.verification_scope(), expected_scope);
    }
}

#[test]
fn custom_seed_is_explicit_and_not_selectable() {
    // Arrange
    let custom_seeds = board_profile_defaults()
        .iter()
        .filter(|profile| profile.seed_kind() == BoardProfileSeedKind::CustomOverride)
        .collect::<Vec<_>>();

    // Act
    let custom = custom_seeds
        .first()
        .expect("matrix must include the upstream custom seed");
    let numbered_207 = board_profile_defaults()
        .iter()
        .find(|profile| profile.seed_id() == "207")
        .expect("matrix must include numbered seed 207");

    // Assert
    assert_eq!(custom_seeds.len(), 1);
    assert!(!custom.is_selectable());
    assert!(numbered_207.is_selectable());
    assert_eq!(custom.board_version(), numbered_207.board_version());
    assert_eq!(custom.primary_pool_port(), 21496);
    assert_eq!(numbered_207.primary_pool_port(), 3333);
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!(
        "../../../fixtures/board-profile-defaults.json"
    ))
    .expect("board profile defaults fixture must parse")
}

fn assert_matches_fixture(actual: BoardProfileDefaults, expected: &FixtureProfile) {
    assert_eq!(actual.seed_id(), expected.seed_id);
    assert_eq!(actual.source_path(), expected.source_path);
    assert_eq!(
        actual.seed_kind(),
        match expected.seed_kind.as_str() {
            "numbered" => BoardProfileSeedKind::Numbered,
            "custom_override" => BoardProfileSeedKind::CustomOverride,
            other => panic!("unknown fixture seed kind {other}"),
        }
    );
    assert_eq!(actual.board_version(), expected.board_version);
    assert_eq!(actual.device_model(), expected.device_model);
    assert_eq!(actual.asic_model(), expected.asic_model);
    assert_eq!(actual.asic_frequency_mhz(), expected.asic_frequency_mhz);
    assert_eq!(actual.asic_voltage_mv(), expected.asic_voltage_mv);
    assert_eq!(actual.rotation(), expected.rotation);
    assert_eq!(actual.auto_fan_speed(), expected.auto_fan_speed);
    assert_eq!(actual.manual_fan_speed(), expected.manual_fan_speed);
    assert_eq!(actual.self_test(), expected.self_test);
    assert_eq!(actual.overheat_mode(), expected.overheat_mode);
    assert_eq!(actual.primary_pool_port(), expected.primary_pool_port);
}
