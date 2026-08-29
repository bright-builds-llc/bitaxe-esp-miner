use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bitaxe_worker_control::{
    AcceptedSequenceStore, LeaseAuthorizationError, SequenceStoreResult, WorkLeaseAuthorityTrust,
    WorkLeaseAuthorizationVerifier, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal,
};
use curve25519_dalek::constants::{ED25519_BASEPOINT_POINT, EIGHT_TORSION};

const TRUST: &str = r#"{
  "profile":"bwg-worker-deployment-trust/0.1",
  "updateAuthority":{
    "issuer":"development-update-authority",
    "audience":"bwg-reference-firmware-capability/0.1",
    "role":"update_authority",
    "keys":[{"kid":"dev-update-DwaQYLSvuWqah8oY","kty":"OKP","crv":"Ed25519","x":"xf8DO6ofYrezCboUdY03qe5Wq0zgFp3_k5kjW8ht96o","alg":"Ed25519","use":"sig","key_ops":["verify"]}]
  },
  "workLeaseAuthority":{
    "profile":"bwg-worker-deployment-trust/0.1",
    "issuer":"development-worker-lease-authority",
    "audience":"bwg-worker-controller/0.3",
    "role":"work_lease_authority",
    "keys":[{"kid":"dev-lease-OVBcK5Mlzd6E_zbg","kty":"OKP","crv":"Ed25519","x":"abl7RfBOVNNiVmOIJhpBBuFlscyifz8coOVEks7c9r8","alg":"Ed25519","use":"sig","key_ops":["verify"]}]
  }
}"#;

const START_AUTHORIZATION: &str = "eyJhbGciOiJFZDI1NTE5Iiwia2lkIjoiZGV2LWxlYXNlLU9WQmNLNU1semQ2RV96YmciLCJ0eXAiOiJid2ctd29ya2VyLWxlYXNlLWF1dGhvcml6YXRpb24randzIn0.eyJjb250cm9sU2Vzc2lvbkJpbmRpbmdTaGEyNTYiOiJ6RDV1RERuZEZuSzkxaGZWTFpGZnNQRHI3SFEyaVhPRUltOVZHUFBWQVdJIiwib3BlcmF0aW9uIjoic3RhcnQiLCJyZXF1ZXN0U2hhMjU2IjoiaVpQeVEwMVY0bzhsYUphaEhlZjQySXBBSWg4V2VqeFNPUHIzNTZlQzZMZyIsInNlcXVlbmNlIjoiMSJ9.jdMFNKg72Db3CCMaU1RZR9zGHQvXRe4kGeGnLEqPjzeaJuY974yhiSR5WTCScAZdrYtGq2U9dytuswI5ChKrCw";
const RENEW_AUTHORIZATION: &str = "eyJhbGciOiJFZDI1NTE5Iiwia2lkIjoiZGV2LWxlYXNlLU9WQmNLNU1semQ2RV96YmciLCJ0eXAiOiJid2ctd29ya2VyLWxlYXNlLWF1dGhvcml6YXRpb24randzIn0.eyJjb250cm9sU2Vzc2lvbkJpbmRpbmdTaGEyNTYiOiJ6RDV1RERuZEZuSzkxaGZWTFpGZnNQRHI3SFEyaVhPRUltOVZHUFBWQVdJIiwib3BlcmF0aW9uIjoicmVuZXciLCJyZXF1ZXN0U2hhMjU2IjoiVkhpZ3VfX044TVFDMnB1dkhQckp6NmlGY0wwck9LaU1lMWJZMkZSUjd3MCIsInNlcXVlbmNlIjoiMiJ9.tFC46zNC0wUlUcRhojV6aFilKx-BjjMAKecv7kGuW5eUuPfb3Yw1fU-_xszjf5XCuVHHo-erIuxGcE69Zky5Bg";

#[derive(Default)]
struct MemorySequenceStore {
    accepted: BTreeMap<String, u64>,
}

impl AcceptedSequenceStore for MemorySequenceStore {
    fn load(&self, key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError> {
        Ok(self.accepted.get(key_id).copied())
    }

    fn compare_and_store(
        &mut self,
        key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError> {
        let current = self.accepted.get(key_id).copied();
        if current == Some(next) {
            return Ok(SequenceStoreResult::AlreadyCommitted);
        }
        if current != expected {
            return Ok(SequenceStoreResult::Stale);
        }
        self.accepted.insert(key_id.to_owned(), next);
        Ok(SequenceStoreResult::Committed)
    }
}

#[test]
fn verifies_the_exact_pinned_start_and_renew_artifacts_once() {
    // Arrange
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(TRUST)
        .expect("pinned deployment trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let context =
        WorkerLeaseAuthorizationContext::parse("zD5uDDndFnK91hfVLZFfsPDr7HQ2iXOEIm9VGPPVAWI")
            .expect("pinned context should parse");
    let start = start(START_AUTHORIZATION, "fixture-session-password");
    let renewal = renewal(RENEW_AUTHORIZATION);

    // Act
    verifier
        .verify_start(&start, &context)
        .expect("pinned Start should verify");
    verifier
        .verify_renewal(
            &renewal,
            "challenge_00000000000000000000000000000001",
            &context,
        )
        .expect("pinned Renew should verify");
    let replay = verifier.verify_renewal(
        &renewal,
        "challenge_00000000000000000000000000000001",
        &context,
    );

    // Assert
    assert_eq!(
        replay.expect_err("reused sequence must fail").category(),
        "replay"
    );
}

#[test]
fn rejects_changed_complete_request_terms_under_a_valid_signature() {
    // Arrange
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(TRUST)
        .expect("pinned deployment trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let context =
        WorkerLeaseAuthorizationContext::parse("zD5uDDndFnK91hfVLZFfsPDr7HQ2iXOEIm9VGPPVAWI")
            .expect("pinned context should parse");
    let changed = start(START_AUTHORIZATION, "changed-password");

    // Act
    let result = verifier.verify_start(&changed, &context);

    // Assert
    assert_eq!(
        result.expect_err("changed password must fail").category(),
        "invalid_authorization"
    );
}

#[test]
fn rejects_changed_possession_context_under_a_valid_signature() {
    // Arrange
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(TRUST)
        .expect("pinned deployment trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let changed = WorkerLeaseAuthorizationContext::parse(&"T".repeat(43))
        .expect("changed context should parse");
    let start = start(START_AUTHORIZATION, "fixture-session-password");

    // Act
    let result = verifier.verify_start(&start, &changed);

    // Assert
    assert_eq!(
        result.expect_err("changed context must fail").category(),
        "invalid_authorization"
    );
}

#[test]
fn rejects_low_order_deployment_trust() {
    // Arrange
    let mut low_order: serde_json::Value =
        serde_json::from_str(TRUST).expect("trust fixture should be JSON");
    low_order["workLeaseAuthority"]["keys"][0]["x"] =
        serde_json::Value::String("AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned());
    // Act
    let weak = WorkLeaseAuthorityTrust::from_deployment_json(&low_order.to_string());

    // Assert
    assert_eq!(
        weak.expect_err("low-order key must fail").category(),
        "invalid_authorization"
    );
}

#[test]
fn rejects_mixed_torsion_deployment_trust() {
    // Arrange
    let mixed_torsion = (ED25519_BASEPOINT_POINT + EIGHT_TORSION[1])
        .compress()
        .to_bytes();
    let mut trust: serde_json::Value =
        serde_json::from_str(TRUST).expect("trust fixture should be JSON");
    trust["workLeaseAuthority"]["keys"][0]["x"] =
        serde_json::Value::String(URL_SAFE_NO_PAD.encode(mixed_torsion));

    // Act
    let result = WorkLeaseAuthorityTrust::from_deployment_json(&trust.to_string());

    // Assert
    assert_eq!(
        result.expect_err("mixed-torsion key must fail").category(),
        "invalid_authorization"
    );
}

#[test]
fn rejects_wrong_role_deployment_trust() {
    // Arrange
    let mut wrong_role: serde_json::Value =
        serde_json::from_str(TRUST).expect("trust fixture should be JSON");
    wrong_role["workLeaseAuthority"]["role"] =
        serde_json::Value::String("update_authority".to_owned());

    // Act
    let result = WorkLeaseAuthorityTrust::from_deployment_json(&wrong_role.to_string());

    // Assert
    assert_eq!(
        result.expect_err("wrong role must fail").category(),
        "invalid_authorization"
    );
}

#[test]
fn rejects_a_noncanonical_signature_encoding() {
    // Arrange
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(TRUST)
        .expect("pinned deployment trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let context =
        WorkerLeaseAuthorizationContext::parse("zD5uDDndFnK91hfVLZFfsPDr7HQ2iXOEIm9VGPPVAWI")
            .expect("pinned context should parse");
    let mut segments = START_AUTHORIZATION
        .split('.')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let signature = segments.get_mut(2).expect("signature segment should exist");
    let final_index = alphabet
        .find(signature.pop().expect("signature should be nonempty"))
        .expect("signature should use base64url");
    signature.push(
        alphabet
            .chars()
            .nth(final_index | 1)
            .expect("alternate trailing bits should exist"),
    );
    let start = start(&segments.join("."), "fixture-session-password");

    // Act
    let result = verifier.verify_start(&start, &context);

    // Assert
    assert_eq!(
        result
            .expect_err("noncanonical signature must fail")
            .category(),
        "invalid_authorization"
    );
}

fn start(authorization: &str, password: &str) -> WorkerLeaseGrant {
    serde_json::from_value(serde_json::json!({
        "protocolVersion": "bwg-worker-controller/0.3",
        "leaseId": "lease_fixture_03",
        "challengeId": "challenge_00000000000000000000000000000001",
        "authorization": authorization,
        "durationMilliseconds": 60_000,
        "renewAfterMilliseconds": 20_000,
        "stratum": {
            "endpoint": "stratum+tcp://127.0.0.1:3333/",
            "username": "fixture-session-user",
            "password": password,
        },
    }))
    .expect("Start fixture should parse")
}

fn renewal(authorization: &str) -> WorkerLeaseRenewal {
    serde_json::from_value(serde_json::json!({
        "protocolVersion": "bwg-worker-controller/0.3",
        "leaseId": "lease_fixture_03",
        "authorization": authorization,
        "durationMilliseconds": 60_000,
        "renewAfterMilliseconds": 20_000,
    }))
    .expect("Renew fixture should parse")
}
