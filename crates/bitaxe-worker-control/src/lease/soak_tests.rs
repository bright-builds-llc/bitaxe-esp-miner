use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::json;

fn v1() -> serde_json::Value {
    json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"lease","challengeId":"challenge","authorization":"synthetic",
    "durationMilliseconds":60000,"renewAfterMilliseconds":20000,
    "stratum":{"endpoint":"stratum+tcp://192.168.1.3:3333/","username":"synthetic","password":"x"}})
}

fn soak() -> serde_json::Value {
    let mut value = v1();
    value["soakAllowance"] = json!({"id": URL_SAFE_NO_PAD.encode([3; 16]), "maximumActiveMilliseconds": 600_000,
        "ordinal": 1, "schema": "worker-soak-allowance-v2"});
    value
}

fn valid(value: serde_json::Value) -> bool {
    serde_json::from_value::<WorkerLeaseGrant>(value).is_ok_and(|grant| grant.validate())
}

#[test]
fn a_grant_without_a_soak_keeps_its_signed_bytes() {
    // Arrange
    let grant: WorkerLeaseGrant = serde_json::from_value(v1()).expect("grant");
    // Act
    let canonical = serde_json::to_string(&grant.authorizationless()).expect("canonical");
    // Assert
    assert!(!canonical.contains("hardwareProfile") && !canonical.contains("soakAllowance"));
}

#[test]
fn a_soak_grant_is_valid_and_signs_its_allowance_in_canonical_order() {
    // Arrange
    let grant: WorkerLeaseGrant = serde_json::from_value(soak()).expect("grant");
    // Act
    let canonical = serde_json::to_string(&grant.authorizationless()).expect("canonical");
    // Assert
    assert!(grant.validate());
    let value: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    assert_eq!(
        canonical,
        crate::codec::canonical_json(&value).expect("sorted")
    );
    assert_eq!(
        value["soakAllowance"]["maximumActiveMilliseconds"],
        json!(600_000)
    );
}

#[test]
fn a_grant_carrying_a_hardware_profile_is_refused_at_parse() {
    // Arrange: ADR-0037 retired the signed profile; the device's settings choose it.
    let mut value = soak();
    value["hardwareProfile"] = json!("upstream-default");
    // Act / Assert
    assert!(serde_json::from_value::<WorkerLeaseGrant>(value).is_err());
}

#[test]
fn a_v1_soak_allowance_is_refused() {
    // Arrange: v1 allowances counted the shutdown tail inside 619,050 ms.
    let mut value = soak();
    value["soakAllowance"]["schema"] = json!("worker-soak-allowance-v1");
    value["soakAllowance"]["maximumActiveMilliseconds"] = json!(619_050);
    // Act / Assert
    assert!(!valid(value));
}

#[test]
fn a_soak_must_use_the_exact_v1_window() {
    // Arrange
    let mut short_lease = soak();
    short_lease["durationMilliseconds"] = json!(30_000);
    let mut fast_renewal = soak();
    fast_renewal["renewAfterMilliseconds"] = json!(5_000);
    // Act / Assert
    for value in [short_lease, fast_renewal] {
        assert!(!valid(value));
    }
}

#[test]
fn a_soak_cannot_be_combined_with_another_allowance() {
    // Arrange
    let mut both = soak();
    both["qualificationAttempt"] = json!({"schema":"worker-qualification-attempt-v1","id":URL_SAFE_NO_PAD.encode([1;16]),
        "ordinal":18,"purpose":"normal","maximumActiveMilliseconds":180000});
    // Act / Assert
    assert!(!valid(both));
}
