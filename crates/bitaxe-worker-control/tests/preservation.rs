use bitaxe_worker_control::{
    DeviceIdentity, FirmwareIdentity, FirmwareSourceCommit, LeaseAuthorizationError,
    LeaseAuthorizationVerifier, LeaseDeadlines, RestorationReason, SettingsPreservation,
    StateFingerprint, WorkerControl, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
};
use serde_json::{json, Value};

struct PublicStateSession(StateFingerprint);
impl WorkerSession for PublicStateSession {
    fn settings_preservation(&self) -> Result<Option<SettingsPreservation>, WorkerSessionError> {
        Ok(Some(SettingsPreservation::new(self.0, false)))
    }
    fn start(&mut self, _: &WorkerLeaseGrant, _: LeaseDeadlines) -> Result<(), WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }
    fn renew(
        &mut self,
        _: &WorkerLeaseRenewal,
        _: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Err(WorkerSessionError::Rejected)
    }
    fn safe_stop(&mut self, _: RestorationReason) -> Result<(), WorkerSessionError> {
        Ok(())
    }
}
struct PublicSequenceVerifier(StateFingerprint);
impl LeaseAuthorizationVerifier for PublicSequenceVerifier {
    fn authorization_high_water_fingerprint(
        &self,
    ) -> Result<Option<StateFingerprint>, LeaseAuthorizationError> {
        Ok(Some(self.0))
    }
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }
    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }
    fn verify_start(
        &mut self,
        _: &WorkerLeaseGrant,
        _: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        Err(LeaseAuthorizationError::InvalidAuthorization)
    }
    fn verify_renewal(
        &mut self,
        _: &WorkerLeaseRenewal,
        _: &str,
        _: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        Err(LeaseAuthorizationError::InvalidAuthorization)
    }
}
fn status(seed: u8, settings: &[u8], sequences: &[u8]) -> Value {
    status_with_admission(seed, settings, sequences, true)
}
fn status_with_admission(seed: u8, settings: &[u8], sequences: &[u8], admitted: bool) -> Value {
    let firmware = FirmwareIdentity::new(
        FirmwareSourceCommit::parse(&"a".repeat(40)).expect("source"),
        &"b".repeat(64),
    )
    .expect("ELF identity");
    let mut worker = WorkerControl::new(
        DeviceIdentity::from_seed([seed; 32]),
        PublicSequenceVerifier(StateFingerprint::of_public_state(sequences)),
        PublicStateSession(StateFingerprint::of_public_state(settings)),
        None,
        firmware,
        json!({}),
        &"A".repeat(43),
    )
    .expect("controller");
    if admitted {
        worker
            .begin_serial_session(
                bitaxe_worker_control::serial::SerialSessionBinding::parse(
                    "AAAAAAAAAAAAAAAAAAAAAA",
                    &"A".repeat(43),
                    &"A".repeat(43),
                )
                .expect("session"),
            )
            .expect("session begins");
        let mut proof = serde_json::to_vec(&json!({
            "profile":"bwg-worker-possession/0.2", "requestId":"pos_preservation", "command":"prove_possession",
            "payload":{"purpose":"initial_admission", "possessionNonce":"A".repeat(43), "challengeBindingSha256":"A".repeat(43),
                "controllerCapabilitySha256":worker.capability_sha256(), "serialManifestSha256":"A".repeat(43),
                "sessionId":"AAAAAAAAAAAAAAAAAAAAAA", "hostNonce":"A".repeat(43), "deviceNonce":"A".repeat(43)}
        })).expect("proof request");
        proof.push(b'\n');
        let response = worker.prepare_frame(&proof, 0).expect("proof response");
        worker
            .confirm_sent(response)
            .expect("authenticated possession");
    }
    let response = worker.prepare_frame(b"{\"protocolVersion\":\"bwg-worker-controller/0.4\",\"requestId\":\"serial_status\",\"command\":\"status\"}\n", 0).expect("read-only status");
    serde_json::from_slice(response.frame()).expect("status JSON")
}

#[test]
fn preservation_is_available_after_possession_before_any_mining() {
    // Arrange / Act
    let reply = status(7, b"rotation=0", b"{}");
    // Assert
    assert_eq!(reply["result"]["state"], "baseline");
    assert!(reply["result"].get("qualification").is_none());
    let preservation = &reply["result"]["preservation"];
    assert_eq!(preservation["schema"], "worker-preservation-v1");
    assert_eq!(preservation["mine_on_boot"], false);
    for field in [
        "settings_sha256",
        "authorization_high_water_sha256",
        "device_identity_sha256",
    ] {
        let digest = preservation[field].as_str().expect("digest string");
        assert_eq!(digest.len(), 64);
        assert!(digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }
    assert!(!reply.to_string().contains("rotation=0"));
}

#[test]
fn identity_settings_and_authorization_drift_have_independent_fingerprints() {
    // Arrange
    let baseline = status(7, b"rotation=0", b"{}");
    // Act / Assert
    assert_eq!(baseline, status(7, b"rotation=0", b"{}"));
    for (changed, field) in [
        (status(8, b"rotation=0", b"{}"), "device_identity_sha256"),
        (status(7, b"rotation=180", b"{}"), "settings_sha256"),
        (
            status(7, b"rotation=0", b"{\"public-authority\":1}"),
            "authorization_high_water_sha256",
        ),
    ] {
        assert_ne!(
            baseline["result"]["preservation"][field],
            changed["result"]["preservation"][field]
        );
    }
}

#[test]
fn prepossession_status_does_not_expose_stable_fingerprints() {
    // Arrange / Act
    let reply = status_with_admission(7, b"rotation=0", b"{}", false);
    // Assert
    assert!(reply["result"].get("preservation").is_none());
}
