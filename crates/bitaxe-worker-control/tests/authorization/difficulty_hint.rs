use super::*;
use serde_json::{json, Value};
fn fixture() -> Value {
    serde_json::from_str(include_str!("gate-difficulty-hint.json")).expect("public RFC fixture")
}
fn verifier(fixture: &Value) -> WorkLeaseAuthorizationVerifier<MemorySequenceStore> {
    let mut trust: Value = serde_json::from_str(TRUST).expect("public trust");
    trust["workLeaseAuthority"] = fixture["trust"].clone();
    WorkLeaseAuthorizationVerifier::new(
        WorkLeaseAuthorityTrust::from_deployment_json(&trust.to_string()).expect("fixture trust"),
        MemorySequenceStore::default(),
    )
}
fn grant_value(vector: &Value) -> Value {
    let mut grant = vector["input"]["request"].clone();
    grant["authorization"] = vector["authorization"].clone();
    grant
}
fn context(vector: &Value) -> WorkerLeaseAuthorizationContext {
    WorkerLeaseAuthorizationContext::parse(
        vector["input"]["controlSessionBindingSha256"]
            .as_str()
            .expect("binding"),
    )
    .expect("context")
}
#[test]
fn gate_signed_absent_zero_positive_and_maximum_hints_verify() {
    // Arrange
    let fixture = fixture();
    for vector in fixture["vectors"].as_array().expect("vectors") {
        let grant: WorkerLeaseGrant = serde_json::from_value(grant_value(vector)).expect("grant");
        let expected = vector["input"]["request"]["stratum"]
            .get("suggestedDifficulty")
            .map(|value| u16::try_from(value.as_u64().expect("hint")).expect("u16"));
        // Act / Assert
        assert_eq!(grant.maybe_suggested_difficulty(), expected);
        assert_eq!(
            verifier(&fixture).verify_start(&grant, &context(vector)),
            Ok(())
        );
    }
}
#[test]
fn adding_removing_or_changing_a_hint_invalidates_signature_without_spending_sequence() {
    // Arrange
    let fixture = fixture();
    for vector in fixture["vectors"].as_array().expect("vectors") {
        let original = grant_value(vector);
        for replacement in [None, Some(0), Some(1), Some(1000), Some(65535)] {
            let mut changed = original.clone();
            match replacement {
                Some(value) => changed["stratum"]["suggestedDifficulty"] = json!(value),
                None => {
                    changed["stratum"]
                        .as_object_mut()
                        .expect("stratum")
                        .remove("suggestedDifficulty");
                }
            }
            if changed == original {
                continue;
            }
            let changed: WorkerLeaseGrant =
                serde_json::from_value(changed).expect("valid altered hint");
            let correct: WorkerLeaseGrant =
                serde_json::from_value(original.clone()).expect("original");
            let mut verifier = verifier(&fixture);
            // Act / Assert
            assert!(verifier.verify_start(&changed, &context(vector)).is_err());
            assert_eq!(verifier.verify_start(&correct, &context(vector)), Ok(()));
            assert_eq!(
                verifier
                    .verify_start(&correct, &context(vector))
                    .expect_err("replay")
                    .category(),
                "replay"
            );
        }
    }
}
