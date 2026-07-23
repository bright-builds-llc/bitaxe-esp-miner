use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, UNIX_EPOCH};

use camino::Utf8PathBuf;
use serde_json::Value;

use super::super::effects::{
    classify_independent_effect_document, classify_independent_effect_root,
    IndependentEffectAdmission, IndependentEffectEvidenceError, IndependentEffectObservationSource,
};
use super::*;

const ELIGIBLE: &str = include_str!("../../../fixtures/phase36/independent-effects-eligible.json");

fn ledger() -> Value {
    serde_json::from_str(ELIGIBLE).expect("eligible effect fixture should be JSON")
}

fn classify(value: &Value) -> Result<IndependentEffectAdmission, IndependentEffectEvidenceError> {
    let document = serde_json::to_string(value).expect("effect ledger should serialize");
    classify_independent_effect_document(Some(&document), None)
}

fn assert_insufficient(value: &Value) {
    assert_eq!(
        classify(value),
        Ok(IndependentEffectAdmission::Insufficient {
            category: ComponentInsufficiency::IndependentEffectObservation,
        })
    );
}

#[test]
fn phase36_effects_admits_complete_independent_interval() {
    // Arrange
    let value = ledger();

    // Act
    let admission = classify(&value).expect("complete ledger should classify");

    // Assert
    let IndependentEffectAdmission::Validated { interval } = admission else {
        panic!("complete ledger should be validated");
    };
    assert_eq!(
        interval.observation_source,
        IndependentEffectObservationSource::IndependentLedger
    );
    assert_eq!(interval.effect_count, 8);
    assert_eq!(interval.duration_millis, 900);
    assert_eq!(interval.ledger_digest.len(), 64);
    assert_eq!(interval.claim_fact_digest.len(), 64);
}

#[test]
fn phase36_effects_supervisor_boolean_has_zero_authority() {
    // Arrange
    let supervisor_attestation = r#"{"no_actuation_verified":true,"cleanup_verified":true}"#;

    // Act
    let admission = classify_independent_effect_document(None, Some(supervisor_attestation))
        .expect("supervisor-only evidence should classify");

    // Assert
    assert_eq!(
        admission,
        IndependentEffectAdmission::Insufficient {
            category: ComponentInsufficiency::IndependentEffectObservation,
        }
    );
}

#[test]
fn phase36_effects_each_missing_allowed_effect_is_insufficient() {
    // Arrange
    let effect_count = ledger()["records"]
        .as_array()
        .expect("records should be an array")
        .len();

    // Act and Assert
    for missing_index in 0..effect_count {
        let mut value = ledger();
        value["records"]
            .as_array_mut()
            .expect("records should be an array")
            .remove(missing_index);
        assert_insufficient(&value);
    }
}

#[test]
fn phase36_effects_duplicate_effect_is_insufficient() {
    // Arrange
    let mut value = ledger();
    value["records"][7]["effect"] = value["records"][6]["effect"].clone();

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_out_of_order_effect_is_insufficient() {
    // Arrange
    let mut value = ledger();
    value["records"]
        .as_array_mut()
        .expect("records should be an array")
        .swap(2, 3);

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_unclosed_record_is_insufficient() {
    // Arrange
    let mut value = ledger();
    value["records"][4]["closed"] = Value::Bool(false);

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_unledgered_direct_path_is_insufficient() {
    // Arrange
    let mut value = ledger();
    value["unledgered_effect_paths"] = Value::from(1);

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_ambiguous_ownership_is_insufficient() {
    // Arrange
    let mut value = ledger();
    value["records"][3]["owner"] = Value::String("ambiguous".to_owned());

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_incomplete_interval_bounds_are_insufficient() {
    // Arrange
    let mut value = ledger();
    value["interval_closed"] = Value::Bool(false);

    // Act and Assert
    assert_insufficient(&value);
}

#[test]
fn phase36_effects_each_prohibited_category_is_rejected() {
    // Arrange
    let prohibited = [
        "active_control",
        "self_test",
        "watchdog",
        "mining",
        "credential_mutation",
        "ota",
        "other_board",
    ];

    // Act and Assert
    for category in prohibited {
        let mut value = ledger();
        value["records"][0]["effect"] = Value::String(category.to_owned());
        assert_eq!(
            classify(&value),
            Err(IndependentEffectEvidenceError::ProhibitedEffect),
            "{category} was not rejected"
        );
    }
}

#[test]
fn phase36_effects_unknown_category_fails_closed() {
    // Arrange
    let mut value = ledger();
    value["records"][0]["effect"] = Value::String("unknown_effect".to_owned());

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(result, Err(IndependentEffectEvidenceError::DocumentInvalid));
}

#[test]
fn phase36_effects_missing_explicit_root_document_is_exactly_insufficient() {
    // Arrange
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should follow Unix epoch")
        .as_nanos();
    let root = Utf8PathBuf::from(format!(
        "{}/phase36-effects-{nonce}",
        std::env::temp_dir().display()
    ));
    fs::create_dir(&root).expect("protected root should be created");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .expect("protected root permissions should be set");

    // Act
    let result = classify_independent_effect_root(&root);
    fs::remove_dir(&root).expect("protected root should be removed");

    // Assert
    assert_eq!(
        result,
        Ok(IndependentEffectAdmission::Insufficient {
            category: ComponentInsufficiency::IndependentEffectObservation,
        })
    );
}
