//! Synthetic signer, persisted counting store and possession driver for restoration tests.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bitaxe_worker_control::{
    AcceptedSequenceStore, DeviceIdentity, FirmwareSourceCommit, LeaseAuthorizationError,
    LeaseDeadlines, PossessionRequest, RestorationReason, SequenceStoreResult, StateFingerprint,
    WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier, WorkerControl, WorkerControlError,
    WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const LEASE_KEY_ID: &str = "restoration-lease";
pub const CHALLENGE_ID: &str = "challenge_00000000000000000000000000000007";
pub const LEASE_ID: &str = "lease_restoration_07";
pub const POOL_PASSWORD: &str = "restoration-pool-password";
const DEVICE_SEED: [u8; 32] = [7; 32];
const LEASE_SEED: [u8; 32] = [21; 32];
const UPDATE_SEED: [u8; 32] = [22; 32];
const MANIFEST: &str = "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StoreCounts {
    pub loads: usize,
    pub compare_and_store: usize,
    pub mark_effect_pending: usize,
    pub clear_effect_pending: usize,
}

/// Durable high-water shared across simulated reboots; counts every access.
#[derive(Clone, Default)]
pub struct PersistedStore {
    accepted: Rc<RefCell<BTreeMap<String, u64>>>,
    counts: Rc<RefCell<StoreCounts>>,
}

impl PersistedStore {
    pub fn counts(&self) -> StoreCounts {
        *self.counts.borrow()
    }
}

impl AcceptedSequenceStore for PersistedStore {
    fn authorization_high_water_fingerprint(
        &self,
    ) -> Result<Option<StateFingerprint>, LeaseAuthorizationError> {
        let bytes = serde_json::to_vec(&*self.accepted.borrow())
            .map_err(|_| LeaseAuthorizationError::Persistence)?;
        Ok(Some(StateFingerprint::of_public_state(&bytes)))
    }

    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.counts.borrow_mut().mark_effect_pending += 1;
        Ok(())
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.counts.borrow_mut().clear_effect_pending += 1;
        Ok(())
    }

    fn load(&self, key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError> {
        self.counts.borrow_mut().loads += 1;
        Ok(self.accepted.borrow().get(key_id).copied())
    }

    fn compare_and_store(
        &mut self,
        key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError> {
        self.counts.borrow_mut().compare_and_store += 1;
        let mut accepted = self.accepted.borrow_mut();
        if accepted.get(key_id).copied() != expected {
            return Ok(SequenceStoreResult::Stale);
        }
        accepted.insert(key_id.to_owned(), next);
        Ok(SequenceStoreResult::Committed)
    }
}

#[derive(Default)]
pub struct CountingSession {
    pub events: Vec<&'static str>,
}

impl WorkerSession for CountingSession {
    fn start(&mut self, _: &WorkerLeaseGrant, _: LeaseDeadlines) -> Result<(), WorkerSessionError> {
        self.events.push("start");
        Ok(())
    }

    fn renew(
        &mut self,
        _: &WorkerLeaseRenewal,
        _: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("renew");
        Ok(())
    }

    fn safe_stop(&mut self, reason: RestorationReason) -> Result<(), WorkerSessionError> {
        self.events.push(reason.category());
        Ok(())
    }
}

pub type Worker = WorkerControl<WorkLeaseAuthorizationVerifier<PersistedStore>, CountingSession>;

fn public_key(seed: [u8; 32], kid: &str) -> Value {
    json!({"kid":kid,"kty":"OKP","crv":"Ed25519",
        "x":URL_SAFE_NO_PAD.encode(SigningKey::from_bytes(&seed).verifying_key().as_bytes()),
        "alg":"Ed25519","use":"sig","key_ops":["verify"]})
}

pub fn trust() -> WorkLeaseAuthorityTrust {
    let document = json!({"profile":"bwg-worker-deployment-trust/0.2",
        "updateAuthority":{"issuer":"restoration-update","audience":"bwg-reference-firmware-capability/0.2",
            "role":"update_authority","keys":[public_key(UPDATE_SEED, "restoration-update")]},
        "workLeaseAuthority":{"profile":"bwg-worker-deployment-trust/0.2","issuer":"restoration-lease",
            "audience":"bwg-worker-controller/0.4","role":"work_lease_authority",
            "keys":[public_key(LEASE_SEED, LEASE_KEY_ID)]}});
    WorkLeaseAuthorityTrust::from_deployment_json(&document.to_string()).expect("synthetic trust")
}

/// One boot: a fresh verifier and controller over the persisted store.
pub fn boot(store: &PersistedStore, maybe_restoration: Option<RestorationReason>) -> Worker {
    WorkerControl::new(
        DeviceIdentity::from_seed(DEVICE_SEED),
        WorkLeaseAuthorizationVerifier::new(trust(), store.clone()),
        CountingSession::default(),
        maybe_restoration,
        bitaxe_worker_control::FirmwareIdentity::new(source_commit(), &"b".repeat(64))
            .expect("firmware identity"),
        json!({"protocolVersion":"bwg-worker-controller/0.4","transportProfile":"bwg-worker-serial/0.2"}),
        MANIFEST,
    )
    .expect("synthetic Worker")
}

fn source_commit() -> FirmwareSourceCommit {
    FirmwareSourceCommit::parse(&"a".repeat(40)).expect("source commit")
}

fn binding() -> bitaxe_worker_control::serial::SerialSessionBinding {
    let session = "AAAAAAAAAAAAAAAAAAAAAA";
    let nonce = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    bitaxe_worker_control::serial::SerialSessionBinding::parse(session, nonce, nonce)
        .expect("binding")
}

/// Opens a fresh logical session, proves possession and returns the new context binding.
pub fn possess(worker: &mut Worker, nonce_byte: u8, now: u64) -> String {
    worker
        .begin_serial_session(binding())
        .expect("fresh session");
    let frame = json!({"profile":"bwg-worker-possession/0.2","requestId":format!("pos_{nonce_byte}"),
        "command":"prove_possession","payload":{"purpose":"initial_admission",
        "possessionNonce":URL_SAFE_NO_PAD.encode([nonce_byte; 32]),
        "challengeBindingSha256":"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
        "controllerCapabilitySha256":worker.capability_sha256(),
        "sessionId":"AAAAAAAAAAAAAAAAAAAAAA","hostNonce":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "deviceNonce":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","serialManifestSha256":MANIFEST}});
    let line = format!("{frame}\n");
    let proof = worker
        .prepare_frame(line.as_bytes(), now)
        .expect("possession proof");
    worker.confirm_sent(proof).expect("admission");
    let request = PossessionRequest::from_frame(line.as_bytes()).expect("request");
    let response = DeviceIdentity::from_seed(DEVICE_SEED)
        .prove(&request, &source_commit(), &"b".repeat(64))
        .expect("independent proof");
    request.control_session_binding(&response).expect("binding")
}

fn sign(operation: &str, request: &Value, binding: &str, sequence: u64) -> String {
    let digest = json!({"activeChallengeId":CHALLENGE_ID,"audience":"bwg-worker-controller/0.4",
        "issuer":"restoration-lease","operation":operation,
        "profile":"bwg-worker-lease-authorization/0.2","request":request});
    let digest_bytes = serde_json::to_vec(&digest).expect("digest input");
    let protected = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({"alg":"Ed25519","kid":LEASE_KEY_ID,
            "typ":"bwg-worker-lease-authorization+jws"}))
        .expect("header"),
    );
    let claims = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(
            &json!({"controlSessionBindingSha256":binding,"operation":operation,
            "requestSha256":URL_SAFE_NO_PAD.encode(Sha256::digest(digest_bytes)),
            "sequence":sequence.to_string()}),
        )
        .expect("claims"),
    );
    let input = format!("{protected}.{claims}");
    let signature = SigningKey::from_bytes(&LEASE_SEED).sign(input.as_bytes());
    format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}

pub fn start_payload(binding: &str, sequence: u64) -> Value {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":LEASE_ID,
        "challengeId":CHALLENGE_ID,"durationMilliseconds":60000,"renewAfterMilliseconds":20000,
        "stratum":{"endpoint":"stratum+tcp://127.0.0.1:3333/","username":"restoration-user",
        "password":POOL_PASSWORD}});
    request["authorization"] = sign("start", &request, binding, sequence).into();
    request
}

pub fn renew_payload(binding: &str, sequence: u64) -> Value {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4","leaseId":LEASE_ID,
        "durationMilliseconds":60000,"renewAfterMilliseconds":20000});
    request["authorization"] = sign("renew", &request, binding, sequence).into();
    request
}

pub fn command(command: &str, maybe_payload: Option<&Value>) -> Vec<u8> {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4",
        "requestId":"serial_restoration","command":command});
    if let Some(payload) = maybe_payload {
        request["payload"] = payload.clone();
    }
    let mut frame = serde_json::to_vec(&request).expect("frame");
    frame.push(b'\n');
    frame
}

/// Prepares and confirms one frame, returning the parsed response.
pub fn send(worker: &mut Worker, frame: &[u8], now: u64) -> Result<Value, WorkerControlError> {
    let prepared = worker.prepare_frame(frame, now)?;
    let response = serde_json::from_slice(prepared.frame()).expect("response JSON");
    worker.confirm_sent_at(prepared, now)?;
    Ok(response)
}

pub fn review(worker: &mut Worker, now: u64) -> Value {
    let prepared = worker
        .prepare_frame(
            &command("authorization_rejection_review", Some(&json!({}))),
            now,
        )
        .expect("idle review");
    let response: Value = serde_json::from_slice(prepared.frame()).expect("review JSON");
    response["result"].clone()
}
