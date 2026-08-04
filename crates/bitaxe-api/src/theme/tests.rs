use bitaxe_config::{NvsSnapshot, StoredValue};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{
    plan_theme_post, theme_settings_from_snapshot, ThemePostFailure, ThemePostResponse,
    MAX_THEME_POST_BODY_BYTES,
};

#[derive(Debug, Deserialize)]
struct Fixture {
    metadata: FixtureMetadata,
    default_get: Value,
    post_success: Value,
}

#[derive(Debug, Deserialize)]
struct FixtureMetadata {
    reference_commit: String,
    get_source: String,
    post_source: String,
}

#[test]
fn default_get_and_post_success_match_golden_fixture() {
    // Arrange
    let fixture = fixture();

    // Act
    let projected = serde_json::to_value(theme_settings_from_snapshot(&NvsSnapshot::new()))
        .expect("theme projection must serialize");
    let success =
        serde_json::to_value(ThemePostResponse::ok()).expect("theme success must serialize");

    // Assert
    assert_eq!(
        fixture.metadata.reference_commit,
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(
        fixture.metadata.get_source,
        "reference/esp-miner/main/http_server/theme_api.c:theme_get_handler"
    );
    assert_eq!(
        fixture.metadata.post_source,
        "reference/esp-miner/main/http_server/theme_api.c:theme_post_handler"
    );
    assert_eq!(projected, fixture.default_get);
    assert_eq!(success, fixture.post_success);
}

#[test]
fn get_projects_stored_values_and_omits_malformed_colors() {
    // Arrange
    let valid = NvsSnapshot::from_values([
        StoredValue::string("themescheme", "light"),
        StoredValue::string("themecolors", r##"{"--primary-color":"#123456"}"##),
    ]);
    let malformed = NvsSnapshot::from_values([
        StoredValue::string("themescheme", "custom"),
        StoredValue::string("themecolors", "not-json"),
    ]);

    // Act
    let valid_projection = theme_settings_from_snapshot(&valid);
    let malformed_projection = theme_settings_from_snapshot(&malformed);

    // Assert
    assert_eq!(valid_projection.color_scheme(), "light");
    assert_eq!(
        valid_projection.maybe_accent_colors(),
        Some(&json!({"--primary-color": "#123456"}))
    );
    assert_eq!(malformed_projection.color_scheme(), "custom");
    assert_eq!(malformed_projection.maybe_accent_colors(), None);
}

#[test]
fn post_plans_only_correctly_typed_present_fields_in_handler_order() {
    // Arrange
    let body = r##"{"unknown":true,"colorScheme":"light","accentColors":{"--x":"#fff"}}"##;

    // Act
    let plan = plan_theme_post(body).expect("valid theme JSON must plan");

    // Assert
    assert!(plan.has_writes());
    assert_eq!(
        plan.writes(),
        [
            bitaxe_config::NvsWrite::string("themescheme", "light"),
            bitaxe_config::NvsWrite::string("themecolors", r##"{"--x":"#fff"}"##),
        ]
    );
}

#[test]
fn post_ignores_wrong_types_unknown_fields_and_non_object_json() {
    // Arrange
    let bodies = [
        r#"{"colorScheme":false,"unknown":"value"}"#,
        r#"["not","an","object"]"#,
        "null",
    ];

    // Act and assert
    for body in bodies {
        let plan = plan_theme_post(body).expect("valid JSON must retain upstream success behavior");
        assert!(!plan.has_writes());
        assert!(plan.writes().is_empty());
        assert!(plan.reconciles(&NvsSnapshot::new()));
    }
}

#[test]
fn post_rejects_malformed_and_oversized_bodies_with_generic_error() {
    // Arrange
    let oversized = "x".repeat(MAX_THEME_POST_BODY_BYTES + 1);

    // Act
    let malformed = plan_theme_post("{").expect_err("malformed JSON must reject");
    let too_large = plan_theme_post(&oversized).expect_err("oversized body must reject");

    // Assert
    assert_eq!(malformed, ThemePostFailure::InvalidJson);
    assert_eq!(too_large, ThemePostFailure::BodyTooLarge);
    for failure in [malformed, too_large] {
        assert_eq!(failure.status(), 400);
        assert_eq!(failure.body(), "Invalid JSON");
    }
}

#[test]
fn post_reconciliation_requires_every_requested_exact_value() {
    // Arrange
    let plan = plan_theme_post(
        r##"{"colorScheme":"light","accentColors":{"--primary-color":"#123456"}}"##,
    )
    .expect("valid theme JSON must plan");
    let exact = NvsSnapshot::from_values(plan.writes().into_iter().map(stored_from_write));
    let wrong = NvsSnapshot::from_values([
        StoredValue::string("themescheme", "dark"),
        StoredValue::string("themecolors", r##"{"--primary-color":"#123456"}"##),
    ]);

    // Act and assert
    assert!(plan.reconciles(&exact));
    assert!(!plan.reconciles(&wrong));
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../fixtures/api/theme-cases.json"))
        .expect("theme fixture must parse")
}

fn stored_from_write(write: bitaxe_config::NvsWrite) -> StoredValue {
    match write {
        bitaxe_config::NvsWrite::String { key, value } => StoredValue {
            key,
            value: bitaxe_config::nvs::StoredValueKind::String(value),
        },
        other => panic!("unexpected theme write {other:?}"),
    }
}
