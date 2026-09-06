use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("gate-campaign.json")).expect("public Gate campaign fixture")
}

fn verifier(fixture: &Value) -> WorkLeaseAuthorizationVerifier<MemorySequenceStore> {
    let mut trust: Value = serde_json::from_str(TRUST).expect("public trust fixture");
    trust["workLeaseAuthority"] = fixture["authority"].clone();
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&trust.to_string())
        .expect("public campaign authority");
    WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default())
}

fn grant(vector: &Value) -> WorkerLeaseGrant {
    let mut request = vector["input"]["request"].clone();
    request["authorization"] = vector["authorization"].clone();
    serde_json::from_value(request).expect("public campaign grant")
}

fn context(vector: &Value) -> WorkerLeaseAuthorizationContext {
    WorkerLeaseAuthorizationContext::parse(
        vector["input"]["controlSessionBindingSha256"]
            .as_str()
            .expect("public fixture binding"),
    )
    .expect("public campaign context")
}

#[test]
fn verifies_gate_signed_acceptance_campaign_windows() {
    // Arrange
    let fixture = fixture();
    let mut verifier = verifier(&fixture);
    for vector in fixture["vectors"].as_array().expect("campaign vectors") {
        let grant = grant(vector);
        let context = context(vector);

        // Act
        let result = verifier.verify_start(&grant, &context);

        // Assert
        assert_eq!(result, Ok(()));
    }
}

#[test]
fn rejects_a_changed_signed_campaign_window() {
    // Arrange
    let fixture = fixture();
    let mut verifier = verifier(&fixture);
    let mut vector = fixture["vectors"][0].clone();
    vector["input"]["request"]["acceptanceCampaign"]["window"] = 1.into();
    vector["input"]["request"]["acceptanceCampaign"]["maximumActiveMilliseconds"] = 30000.into();
    let grant = grant(&vector);
    let context = context(&vector);

    // Act
    let result = verifier.verify_start(&grant, &context);

    // Assert
    assert_eq!(result, Err(LeaseAuthorizationError::InvalidAuthorization));
}
