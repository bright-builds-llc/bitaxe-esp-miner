use super::*;
use serde_json::{json, Value};
fn input() -> Value {
    json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"synthetic_lease","challengeId":"synthetic_challenge","authorization":"synthetic_authorization","durationMilliseconds":60000,"renewAfterMilliseconds":20000,"stratum":{"endpoint":"stratum+tcp://127.0.0.1:3333/","username":"synthetic_user","password":"synthetic_password"}})
}
#[test]
fn omitted_hint_remains_absent_from_authorization() {
    // Arrange / Act
    let grant: WorkerLeaseGrant = serde_json::from_value(input()).expect("fixture");
    let encoded = serde_json::to_value(grant.authorizationless()).expect("canonical fixture");
    // Assert
    assert!(grant.validate());
    assert_eq!(grant.maybe_suggested_difficulty(), None);
    assert!(encoded["stratum"].get("suggestedDifficulty").is_none());
}
#[test]
fn zero_and_positive_hint_are_preserved_in_canonical_key_order() {
    for hint in [0, 1, 100, 65535] {
        // Arrange
        let mut value = input();
        value["stratum"]["suggestedDifficulty"] = json!(hint);
        // Act
        let grant: WorkerLeaseGrant = serde_json::from_value(value).expect("fixture");
        let encoded = serde_json::to_string(&grant.authorizationless()).expect("canonical fixture");
        // Assert
        assert!(grant.validate());
        assert_eq!(
            encoded,
            crate::codec::canonical_json(&serde_json::from_str::<Value>(&encoded).expect("JSON"))
                .expect("canonical JSON")
        );
        assert_eq!(grant.maybe_suggested_difficulty(), Some(hint));
        assert!(encoded.contains(&format!("\"stratum\":{{\"endpoint\":\"stratum+tcp://127.0.0.1:3333/\",\"password\":\"synthetic_password\",\"suggestedDifficulty\":{hint},\"username\":\"synthetic_user\"}}")));
    }
}
#[test]
fn invalid_hint_types_and_ranges_are_rejected() {
    for invalid in [
        Value::Null,
        json!(-1),
        json!(0.5),
        json!(65536),
        json!("1"),
        json!(true),
    ] {
        // Arrange
        let mut value = input();
        value["stratum"]["suggestedDifficulty"] = invalid;
        // Act / Assert
        assert!(serde_json::from_value::<WorkerLeaseGrant>(value).is_err());
    }
}
