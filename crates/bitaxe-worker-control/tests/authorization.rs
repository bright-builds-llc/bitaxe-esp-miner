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
  "profile": "bwg-worker-deployment-trust/0.2",
  "updateAuthority": {
    "issuer": "development-update-authority",
    "audience": "bwg-reference-firmware-capability/0.2",
    "role": "update_authority",
    "keys": [
      {
        "kid": "fixture-serial-update",
        "kty": "OKP",
        "crv": "Ed25519",
        "x": "FqC19_WMEfE5IvmS3TRRv5YT7MszPYsGSGUqoK0Doks",
        "alg": "Ed25519",
        "use": "sig",
        "key_ops": [
          "verify"
        ]
      }
    ]
  },
  "workLeaseAuthority": {
    "profile": "bwg-worker-deployment-trust/0.2",
    "issuer": "development-worker-lease-authority",
    "audience": "bwg-worker-controller/0.4",
    "role": "work_lease_authority",
    "keys": [
      {
        "kid": "fixture-serial-lease",
        "kty": "OKP",
        "crv": "Ed25519",
        "x": "7yfVEFCiaXA5UJ31PkJcdJlDsqTbiKeXTJiuN3F1QVs",
        "alg": "Ed25519",
        "use": "sig",
        "key_ops": [
          "verify"
        ]
      }
    ]
  }
}"#;

const START_AUTHORIZATION: &str = "eyJhbGciOiJFZDI1NTE5Iiwia2lkIjoiZml4dHVyZS1zZXJpYWwtbGVhc2UiLCJ0eXAiOiJid2ctd29ya2VyLWxlYXNlLWF1dGhvcml6YXRpb24randzIn0.eyJjb250cm9sU2Vzc2lvbkJpbmRpbmdTaGEyNTYiOiJEQXlreGh3ckxpNmNldzlmYnVibkhvYXRsNFlnVXVZTWlzVWpsM054RHpFIiwib3BlcmF0aW9uIjoic3RhcnQiLCJyZXF1ZXN0U2hhMjU2IjoidFRtSkNTOXhsLVVOWTVvQ0lKNmN5d29jWnM2cVpRRjcxd1dFRjBGRkhuZyIsInNlcXVlbmNlIjoiMSJ9.yfWvVCXNm7lOuma2kyli2JdGBUyDi0060ZfOp2W8OOvdvqyNFKH1wKDhIsE87n5Hi5mT9qEHmYlEaT0k5qF9Bg";
const RENEW_AUTHORIZATION: &str = "eyJhbGciOiJFZDI1NTE5Iiwia2lkIjoiZml4dHVyZS1zZXJpYWwtbGVhc2UiLCJ0eXAiOiJid2ctd29ya2VyLWxlYXNlLWF1dGhvcml6YXRpb24randzIn0.eyJjb250cm9sU2Vzc2lvbkJpbmRpbmdTaGEyNTYiOiJEQXlreGh3ckxpNmNldzlmYnVibkhvYXRsNFlnVXVZTWlzVWpsM054RHpFIiwib3BlcmF0aW9uIjoicmVuZXciLCJyZXF1ZXN0U2hhMjU2IjoiSmJYbFpTSENYZDhwQUpabWtsRXZRZGRHRkpjeUlJcnZWd2dvYjZYYjNyUSIsInNlcXVlbmNlIjoiMiJ9.EZvX5QojqTCGRss-bk7mdrhq0C0sf5Mv_gOoF43cKIg7QSkY0SEASygbJG1fBCtKfNh5DiEckOyUKuc0J-pDDg";

#[derive(Default)]
struct MemorySequenceStore {
    accepted: BTreeMap<String, u64>,
}

impl AcceptedSequenceStore for MemorySequenceStore {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

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
        WorkerLeaseAuthorizationContext::parse("DAykxhwrLi6cew9fbubnHoatl4YgUuYMisUjl3NxDzE")
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
        WorkerLeaseAuthorizationContext::parse("DAykxhwrLi6cew9fbubnHoatl4YgUuYMisUjl3NxDzE")
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
        WorkerLeaseAuthorizationContext::parse("DAykxhwrLi6cew9fbubnHoatl4YgUuYMisUjl3NxDzE")
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
        "protocolVersion": "bwg-worker-controller/0.4",
        "leaseId": "lease_fixture_01",
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
        "protocolVersion": "bwg-worker-controller/0.4",
        "leaseId": "lease_fixture_01",
        "authorization": authorization,
        "durationMilliseconds": 60_000,
        "renewAfterMilliseconds": 20_000,
    }))
    .expect("Renew fixture should parse")
}

#[test]
fn adding_an_unsigned_acceptance_campaign_invalidates_start_authorization() {
    // Arrange
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(TRUST).expect("fixture trust");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let context =
        WorkerLeaseAuthorizationContext::parse("DAykxhwrLi6cew9fbubnHoatl4YgUuYMisUjl3NxDzE")
            .expect("fixture context");
    let grant: WorkerLeaseGrant = serde_json::from_value(serde_json::json!({
        "protocolVersion":"bwg-worker-controller/0.4", "leaseId":"lease_fixture_01",
        "challengeId":"challenge_00000000000000000000000000000001",
        "authorization":START_AUTHORIZATION, "durationMilliseconds":60000,
        "renewAfterMilliseconds":20000, "stratum":{"endpoint":"stratum+tcp://127.0.0.1:3333/", "username":"fixture-session-user", "password":"fixture-session-password"},
        "acceptanceCampaign":{"id":"AAAAAAAAAAAAAAAAAAAAAA", "window":0,"maximumActiveMilliseconds":180000}
    })).expect("structured grant");

    // Act
    let result = verifier.verify_start(&grant, &context);

    // Assert
    assert_eq!(
        result.expect_err("campaign must be signed").category(),
        "invalid_authorization"
    );
}
