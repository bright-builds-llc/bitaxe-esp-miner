//! Obtain public deadlines through the real controller, then exercise actual owner Start.
use super::*;
use bitaxe_worker_control::{
    DeviceIdentity, FirmwareIdentity, FirmwareSourceCommit, LeaseAuthorizationError,
    LeaseAuthorizationVerifier, LeaseDeadlines, RestorationReason, WorkerControl,
    WorkerLeaseAuthorizationContext, WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession,
    WorkerSessionError,
};
use serde_json::json;

#[derive(Default)]
struct CaptureSession {
    maybe_deadlines: Option<LeaseDeadlines>,
}
impl WorkerSession for CaptureSession {
    fn start(
        &mut self,
        _: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.maybe_deadlines = Some(deadlines);
        Ok(())
    }
    fn renew(
        &mut self,
        _: &WorkerLeaseRenewal,
        _: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }
    fn safe_stop(&mut self, _: RestorationReason) -> Result<(), WorkerSessionError> {
        Ok(())
    }
}
struct SyntheticVerifier;
impl LeaseAuthorizationVerifier for SyntheticVerifier {
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
        Ok(())
    }
    fn verify_renewal(
        &mut self,
        _: &WorkerLeaseRenewal,
        _: &str,
        _: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }
}
fn grant() -> WorkerLeaseGrant {
    serde_json::from_value(grant_input()).expect("synthetic grant")
}
fn grant_input() -> serde_json::Value {
    json!({
        "protocolVersion":"bwg-worker-controller/0.4","leaseId":"synthetic_lease","challengeId":"synthetic_challenge",
        "authorization":"synthetic-not-production", "durationMilliseconds":60000,"renewAfterMilliseconds":20000,
        "stratum":{"endpoint":"stratum+tcp://example.invalid:3333/","username":"synthetic","password":"synthetic"}
    })
}

fn deadlines() -> LeaseDeadlines {
    let manifest = "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA";
    let mut controller = WorkerControl::new(
        DeviceIdentity::from_seed([7;32]), SyntheticVerifier, CaptureSession::default(), None,
        FirmwareIdentity::new(FirmwareSourceCommit::parse(&"a".repeat(40)).expect("source"), &"b".repeat(64)).expect("firmware"),
        json!({"protocolVersion":"bwg-worker-controller/0.4","transportProfile":"bwg-worker-serial/0.2"}),manifest
    ).expect("synthetic controller");
    controller
        .begin_serial_session(
            bitaxe_worker_control::serial::SerialSessionBinding::parse(
                "AAAAAAAAAAAAAAAAAAAAAA",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            )
            .expect("synthetic binding"),
        )
        .expect("fresh session");
    let proof = json!({"profile":"bwg-worker-possession/0.2","requestId":"pos_synthetic","command":"prove_possession","payload":{
        "purpose":"initial_admission","possessionNonce":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
        "challengeBindingSha256":"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC", "controllerCapabilitySha256":controller.capability_sha256(),
        "sessionId":"AAAAAAAAAAAAAAAAAAAAAA","hostNonce":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","deviceNonce":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","serialManifestSha256":manifest
    }}).to_string() + "\n";
    let prepared = controller
        .prepare_frame(proof.as_bytes(), 1_000)
        .expect("possession preparation");
    controller
        .confirm_sent(prepared)
        .expect("possession acknowledgement");
    controller.prepare_frame((json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_synthetic_start","command":"start_lease","payload":grant_input()}).to_string() + "\n").as_bytes(),1_000).expect("capture legitimate deadlines");
    controller
        .session()
        .maybe_deadlines
        .expect("controller produced deadlines")
}

#[test]
fn actual_start_rejects_full_owner_queue_without_activation() {
    // Arrange
    let mut scope = TestScope::new();
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let deadlines = deadlines();
    let (sender, receiver) = mpsc::sync_channel(1);
    assert!(
        NOTIFICATIONS.set(sender).is_ok(),
        "only this test installs owner queue"
    );
    let (reply, _reply_receiver) = mpsc::sync_channel(1);
    NOTIFICATIONS
        .get()
        .expect("owner queue")
        .try_send(OwnerInboxMessage::Bwg(bwg::OwnerCommand::SafeStop {
            reply,
        }))
        .expect("fill owner queue");
    // Act
    let result = bwg::start(&grant(), deadlines, generation);
    // Assert
    assert_eq!(result, Err(bwg::Error::Unavailable));
    assert!(!revocation::permits(Some(generation)));
    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_err(), "Start was never enqueued");
}

#[test]
fn actual_start_rejects_revoked_generation_before_owner_queue() {
    // Arrange
    let mut scope = TestScope::new();
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let deadlines = deadlines();
    revocation::revoke_at(generation, 1_000);
    // Act
    let result = bwg::start(&grant(), deadlines, generation);
    // Assert
    assert_eq!(result, Err(bwg::Error::Rejected));
}

#[test]
fn actual_start_rejects_unmappable_pool_before_owner_queue() {
    // Arrange
    let mut scope = TestScope::new();
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let deadlines = deadlines();
    let mut input = grant_input();
    input["stratum"]["endpoint"] = json!("stratum+tcp://example.invalid/");
    let invalid: WorkerLeaseGrant = serde_json::from_value(input).expect("typed boundary input");
    // Act
    let result = bwg::start(&invalid, deadlines, generation);
    // Assert
    assert_eq!(result, Err(bwg::Error::Rejected));
}
