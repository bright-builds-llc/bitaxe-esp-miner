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

/// The synthetic deployment trust whose lease key only this module signs with.
#[inline(never)]
pub(crate) fn synthetic_trust_json() -> Result<String, SimulationError> {
    let trust = json!({"profile":"bwg-worker-deployment-trust/0.2",
        "updateAuthority":{"issuer":"simulation-update","audience":"bwg-reference-firmware-capability/0.2","role":"update_authority","keys":[key([12;32])]},
        "workLeaseAuthority":{"profile":"bwg-worker-deployment-trust/0.2","issuer":"simulation-lease","audience":"bwg-worker-controller/0.4","role":"work_lease_authority","keys":[key([11;32])]}});
    Ok(serde_json::to_string(&trust)?)
}

/// Verifies leases against a deployment trust document, as firmware does with the
/// one it compiles in; the board's storage holds the accepted sequence.
#[inline(never)]
pub(crate) fn verifier(
    board: Rc<RefCell<VirtualBoard>>,
    trust_json: &str,
) -> Result<Verifier, SimulationError> {
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(trust_json)
        .map_err(|_| SimulationError::Boundary("deployment_trust"))?;
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
    request["authorization"] = authorize("start", &request, binding, 1)?.into();
    Ok(request)
}

/// A renewal of the synthetic Start's lease under the next accepted sequence.
#[cfg(test)]
pub(crate) fn signed_renewal(binding: &str, sequence: u64) -> Result<Value, SimulationError> {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":"simulation_lease",
        "durationMilliseconds":60000,"renewAfterMilliseconds":20000});
    request["authorization"] = authorize("renew", &request, binding, sequence)?.into();
    Ok(request)
}

/// The synthetic lease key's compact JWS over one request bound to the control session.
fn authorize(
    operation: &str,
    request: &Value,
    binding: &str,
    sequence: u64,
) -> Result<String, SimulationError> {
    let digest = json!({"activeChallengeId":"simulation_challenge","audience":"bwg-worker-controller/0.4",
        "issuer":"simulation-lease","operation":operation,"profile":"bwg-worker-lease-authorization/0.2","request":request});
    let protected = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"alg":"Ed25519","kid":"simulation-lease","typ":"bwg-worker-lease-authorization+jws"}))?);
    let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"controlSessionBindingSha256":binding,
        "operation":operation,"requestSha256":URL_SAFE_NO_PAD.encode(Sha256::digest(serde_json::to_vec(&digest)?)),"sequence":sequence.to_string()}))?);
    let input = format!("{protected}.{claims}");
    let signature = SigningKey::from_bytes(&[11; 32]).sign(input.as_bytes());
    Ok(format!(
        "{input}.{}",
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
}
