//! Independent synthetic signer driving the production verifier.
use std::cell::RefCell;
use std::rc::Rc;

use crate::SimulationError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_virtual_board::VirtualBoard;
use bitaxe_worker_control::{
    AcceptedSequenceStore, LeaseAuthorizationError, SequenceStoreResult, WorkLeaseAuthorityTrust,
    WorkLeaseAuthorizationVerifier,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub(crate) type Verifier = WorkLeaseAuthorizationVerifier<SequenceStore>;

pub(crate) struct SequenceStore(pub Rc<RefCell<VirtualBoard>>);
impl AcceptedSequenceStore for SequenceStore {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.0
            .borrow_mut()
            .storage
            .transaction(&[("effect_pending".into(), vec![1])])
            .map_err(|_| LeaseAuthorizationError::Persistence)
    }
    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.0
            .borrow_mut()
            .storage
            .transaction(&[("effect_pending".into(), vec![0])])
            .map_err(|_| LeaseAuthorizationError::Persistence)
    }
    fn load(&self, _key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError> {
        let high = self.0.borrow().storage.replay_high_water;
        Ok((high != 0).then_some(high))
    }
    fn compare_and_store(
        &mut self,
        _key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError> {
        if self.load("synthetic")? != expected {
            return Ok(SequenceStoreResult::Stale);
        }
        self.0
            .borrow_mut()
            .storage
            .admit_replay(next)
            .map_err(|_| LeaseAuthorizationError::Persistence)?;
        Ok(SequenceStoreResult::Committed)
    }
}

#[inline(never)]
fn key(seed: [u8; 32]) -> Value {
    json!({"kid":if seed[0] == 11 {"simulation-lease"} else {"simulation-update"},
        "kty":"OKP","crv":"Ed25519","x":URL_SAFE_NO_PAD.encode(SigningKey::from_bytes(&seed).verifying_key().as_bytes()),
        "alg":"Ed25519","use":"sig","key_ops":["verify"]})
}

#[inline(never)]
pub(crate) fn verifier(board: Rc<RefCell<VirtualBoard>>) -> Result<Verifier, SimulationError> {
    let trust = json!({"profile":"bwg-worker-deployment-trust/0.2",
        "updateAuthority":{"issuer":"simulation-update","audience":"bwg-reference-firmware-capability/0.2","role":"update_authority","keys":[key([12;32])]},
        "workLeaseAuthority":{"profile":"bwg-worker-deployment-trust/0.2","issuer":"simulation-lease","audience":"bwg-worker-controller/0.4","role":"work_lease_authority","keys":[key([11;32])]}});
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&serde_json::to_string(&trust)?)
        .map_err(|_| SimulationError::Boundary("synthetic_trust"))?;
    Ok(WorkLeaseAuthorizationVerifier::new(
        trust,
        SequenceStore(board),
    ))
}

/// Canonical Value objects use serde_json's sorted BTreeMap, independently of
/// the verifier's streamed grant representation; production verification is the oracle.
pub(crate) fn signed_start(binding: &str, ordinal: u64) -> Result<Value, SimulationError> {
    signed_start_for_profile(binding, ordinal, false)
}

pub(crate) fn signed_start_for_profile(
    binding: &str,
    ordinal: u64,
    use_v2: bool,
) -> Result<Value, SimulationError> {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4",
        "leaseId":"simulation_lease","challengeId":"simulation_challenge",
        "durationMilliseconds":60000,"renewAfterMilliseconds":20000,
        "stratum":{"endpoint":"stratum+tcp://127.0.0.1:3333/","username":"simulation","password":"simulation"},
        "qualificationAttempt":{"schema":"worker-qualification-attempt-v1","id":URL_SAFE_NO_PAD.encode([9;16]),
        "maximumActiveMilliseconds":180000,"ordinal":ordinal,"purpose":"normal"}});
    if use_v2 {
        request["stratum"] = json!({"profile":"bwg-worker-stratum-v2-standard/0.1",
            "endpoint":format!("stratum+tcp://{}:{}/",crate::v2::SYNTHETIC_ENDPOINT_HOST,crate::v2::SYNTHETIC_ENDPOINT_PORT),
            "authorityPublicKey":URL_SAFE_NO_PAD.encode(crate::v2::synthetic_authority_public_key()),"userIdentity":crate::v2::SYNTHETIC_USER_IDENTITY});
    }
    let digest = json!({"activeChallengeId":"simulation_challenge","audience":"bwg-worker-controller/0.4",
        "issuer":"simulation-lease","operation":"start","profile":"bwg-worker-lease-authorization/0.2","request":request});
    let protected = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"alg":"Ed25519","kid":"simulation-lease","typ":"bwg-worker-lease-authorization+jws"}))?);
    let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"controlSessionBindingSha256":binding,
        "operation":"start","requestSha256":URL_SAFE_NO_PAD.encode(Sha256::digest(serde_json::to_vec(&digest)?)),"sequence":"1"}))?);
    let input = format!("{protected}.{claims}");
    let signature = SigningKey::from_bytes(&[11; 32]).sign(input.as_bytes());
    request["authorization"] =
        format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes())).into();
    Ok(request)
}
