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
    value["hardwareProfile"] = json!("upstream-default");
    value["soakAllowance"] = json!({"id": URL_SAFE_NO_PAD.encode([3; 16]), "maximumActiveMilliseconds": 615_550,
        "ordinal": 1, "schema": "worker-soak-allowance-v1"});
    value
}

fn valid(value: serde_json::Value) -> bool {
    serde_json::from_value::<WorkerLeaseGrant>(value).is_ok_and(|grant| grant.validate())
}

#[test]
fn a_grant_without_the_new_fields_keeps_its_signed_bytes_and_conservative_profile() {
    // Arrange
    let grant: WorkerLeaseGrant = serde_json::from_value(v1()).expect("grant");
    // Act
    let canonical = serde_json::to_string(&grant.authorizationless()).expect("canonical");
    // Assert
    assert!(!canonical.contains("hardwareProfile") && !canonical.contains("soakAllowance"));
    assert_eq!(
        grant.hardware_profile(),
        crate::HardwareProfile::Conservative
    );
}

#[test]
fn an_upstream_default_soak_grant_is_valid_and_signs_both_fields_in_canonical_order() {
    // Arrange
    let grant: WorkerLeaseGrant = serde_json::from_value(soak()).expect("grant");
    // Act
    let canonical = serde_json::to_string(&grant.authorizationless()).expect("canonical");
    // Assert
    assert!(grant.validate());
    assert_eq!(
        grant.hardware_profile(),
        crate::HardwareProfile::UpstreamDefault
    );
    let value: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    assert_eq!(
        canonical,
        crate::codec::canonical_json(&value).expect("sorted")
    );
    assert_eq!(value["hardwareProfile"], json!("upstream-default"));
    assert_eq!(
        value["soakAllowance"]["maximumActiveMilliseconds"],
        json!(615_550)
    );
}

#[test]
fn changing_the_profile_changes_the_signed_canonical_value() {
    // Arrange
    let mut conservative = soak();
    conservative["hardwareProfile"] = json!("conservative");
    // Act
    let left = serde_json::to_string(
        &serde_json::from_value::<WorkerLeaseGrant>(soak())
            .expect("grant")
            .authorizationless(),
    )
    .expect("json");
    let right = serde_json::to_string(
        &serde_json::from_value::<WorkerLeaseGrant>(conservative)
            .expect("grant")
            .authorizationless(),
    )
    .expect("json");
    // Assert
    assert_ne!(left, right);
}

#[test]
fn upstream_default_without_a_soak_is_rejected() {
    // Arrange
    let mut value = v1();
    value["hardwareProfile"] = json!("upstream-default");
    // Act / Assert
    assert!(!valid(value));
}

#[test]
fn a_soak_must_state_its_profile_and_use_the_exact_v1_window() {
    // Arrange
    let mut unstated = soak();
    unstated
        .as_object_mut()
        .expect("object")
        .remove("hardwareProfile");
    let mut short_lease = soak();
    short_lease["durationMilliseconds"] = json!(30_000);
    let mut fast_renewal = soak();
    fast_renewal["renewAfterMilliseconds"] = json!(5_000);
    // Act / Assert
    for value in [unstated, short_lease, fast_renewal] {
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

#[test]
fn an_unknown_profile_is_refused_at_parse() {
    // Arrange
    let mut value = soak();
    value["hardwareProfile"] = json!("overclock");
    // Act / Assert
    assert!(serde_json::from_value::<WorkerLeaseGrant>(value).is_err());
}

#[test]
fn a_conservative_soak_is_allowed_for_rehearsal() {
    // Arrange
    let mut value = soak();
    value["hardwareProfile"] = json!("conservative");
    // Act / Assert
    assert!(valid(value));
}
