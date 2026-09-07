use super::*;
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!("gate-qualification.json"))
        .expect("public qualification fixture")
}
fn verifier(f: &Value) -> WorkLeaseAuthorizationVerifier<MemorySequenceStore> {
    let mut trust: Value = serde_json::from_str(TRUST).expect("fixture");
    trust["workLeaseAuthority"] = f["trust"].clone();
    WorkLeaseAuthorizationVerifier::new(
        WorkLeaseAuthorityTrust::from_deployment_json(&trust.to_string()).expect("trust"),
        MemorySequenceStore::default(),
    )
}
#[test]
fn all_gate_signed_qualification_purposes_verify_in_rust() {
    // Arrange
    let f = fixture();
    let mut v = verifier(&f);
    for vector in f["vectors"].as_array().expect("vectors") {
        let grant: WorkerLeaseGrant =
            serde_json::from_value(vector["grant"].clone()).expect("grant");
        let context = WorkerLeaseAuthorizationContext::parse(
            vector["input"]["controlSessionBindingSha256"]
                .as_str()
                .expect("binding"),
        )
        .expect("context");
        // Act / Assert
        assert_eq!(v.verify_start(&grant, &context), Ok(()));
    }
}
#[test]
fn changing_any_attempt_identity_or_limit_invalidates_authorization() {
    // Arrange
    let f = fixture();
    let vector = &f["vectors"][0];
    let context = WorkerLeaseAuthorizationContext::parse(
        vector["input"]["controlSessionBindingSha256"]
            .as_str()
            .expect("binding"),
    )
    .expect("context");
    for (field, value) in [
        ("ordinal", serde_json::json!(2)),
        ("id", serde_json::json!("AQEBAQEBAQEBAQEBAQEBAQ")),
        ("purpose", serde_json::json!("heartbeat_loss")),
        ("maximumActiveMilliseconds", serde_json::json!(180000)),
    ] {
        let mut grant = vector["grant"].clone();
        grant["qualificationAttempt"][field] = value;
        let grant: WorkerLeaseGrant = serde_json::from_value(grant).expect("grant");
        // Act / Assert
        assert!(verifier(&f).verify_start(&grant, &context).is_err());
    }
}
