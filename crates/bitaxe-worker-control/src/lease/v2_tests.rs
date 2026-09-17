use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::json;
fn input() -> serde_json::Value {
    json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"lease","challengeId":"challenge","authorization":"synthetic",
    "durationMilliseconds":60000,"renewAfterMilliseconds":20000,"stratum":{"profile":"bwg-worker-stratum-v2-standard/0.1","endpoint":"stratum+tcp://192.168.1.3:12345/","authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"},
    "qualificationAttempt":{"schema":"worker-qualification-attempt-v1","id":URL_SAFE_NO_PAD.encode([1;16]),"ordinal":18,"purpose":"normal","maximumActiveMilliseconds":180000}})
}
#[test]
fn signed_v2_canonical_value_covers_every_profile_field() {
    // Arrange
    let original = input();
    let grant: WorkerLeaseGrant = serde_json::from_value(original.clone()).expect("grant");
    assert!(grant.validate());
    // Act
    let canonical = serde_json::to_string(&grant.authorizationless()).expect("canonical");
    // Assert
    let value: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    assert_eq!(value["stratum"], original["stratum"]);
    assert_eq!(
        canonical,
        crate::codec::canonical_json(&value).expect("sorted canonical")
    );
    for key in ["authorityPublicKey", "endpoint", "profile", "userIdentity"] {
        let mut changed = original.clone();
        changed["stratum"][key] = json!("changed");
        let parsed = serde_json::from_value::<WorkerLeaseGrant>(changed);
        assert!(
            parsed.is_err()
                || parsed.is_ok_and(|g| !g.validate()
                    || serde_json::to_string(&g.authorizationless()).expect("json") != canonical)
        );
    }
}
#[test]
fn v2_requires_normal_funded_exact_windows_without_v1_fallback() {
    // Arrange
    let original = input();
    // Act / Assert
    for (key, value) in [
        ("durationMilliseconds", json!(30000)),
        ("renewAfterMilliseconds", json!(10000)),
        ("qualificationAttempt", serde_json::Value::Null),
    ] {
        let mut bad = original.clone();
        bad[key] = value;
        assert!(serde_json::from_value::<WorkerLeaseGrant>(bad).map_or(true, |g| !g.validate()));
    }
    for key in ["username", "password", "suggestedDifficulty"] {
        let mut bad = original.clone();
        bad["stratum"][key] = json!(1);
        assert!(serde_json::from_value::<WorkerLeaseGrant>(bad).is_err());
    }
}
