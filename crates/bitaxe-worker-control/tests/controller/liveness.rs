use super::*;
use bitaxe_worker_control::serial::{SerialEnvelope, SerialKind, SerialLinkLiveness};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::Duration;

struct CoolingSession {
    revoked: Arc<AtomicBool>,
    began: mpsc::SyncSender<()>,
    completion: mpsc::Receiver<()>,
}

impl WorkerSession for CoolingSession {
    fn start(
        &mut self,
        _grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }
    fn renew(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }
    fn safe_stop(&mut self, _reason: RestorationReason) -> Result<(), WorkerSessionError> {
        self.revoked.store(true, Ordering::Release);
        self.began
            .try_send(())
            .map_err(|_| WorkerSessionError::SafeStopFailed)?;
        self.completion
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| WorkerSessionError::SafeStopFailed)
    }
}

#[test]
fn authenticated_channel_delivers_restore_after_120_seconds_of_revoked_work() {
    // Arrange
    let revoked = Arc::new(AtomicBool::new(false));
    let (began, began_rx) = mpsc::sync_channel(1);
    let (completion, completion_rx) = mpsc::sync_channel(1);
    let session = CoolingSession {
        revoked: Arc::clone(&revoked),
        began,
        completion: completion_rx,
    };
    let capability = json!({"protocolVersion":"bwg-worker-controller/0.4"});
    let firmware = bitaxe_worker_control::FirmwareIdentity::new(
        bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
            .expect("fixture source"),
        &"b".repeat(64),
    )
    .expect("fixture firmware");
    let mut worker = WorkerControl::new(
        DeviceIdentity::from_seed([7; 32]),
        FixtureVerifier,
        session,
        None,
        firmware,
        capability,
        "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA",
    )
    .expect("controller");
    worker
        .begin_serial_session(fixture_binding())
        .expect("logical session");
    let proof = worker
        .prepare_frame(possession_frame(worker.capability_sha256()).as_bytes(), 0)
        .expect("proof");
    worker.confirm_sent(proof).expect("proof delivered");
    worker
        .prepare_frame(start_frame().as_bytes(), 1)
        .expect("Start admitted");
    let mut liveness = SerialLinkLiveness::new(0);
    liveness.authenticate();
    let owner = std::thread::spawn(move || {
        let frame = b"{\"protocolVersion\":\"bwg-worker-controller/0.4\",\"requestId\":\"serial_restore\",\"command\":\"restore\",\"payload\":{\"reason\":\"cancelled\"}}\n";
        let response = worker
            .prepare_frame(frame, 2)
            .expect("cooling then Restore acknowledgement");
        assert!(
            !worker.is_admitted(),
            "work authority remains closed after Restore"
        );
        response
    });
    began_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("safe stop begins before cooling completion");

    // Act: the independent link clock advances while the actual control owner is blocked.
    for second in 1..=120u64 {
        assert!(revoked.load(Ordering::Acquire));
        assert!(liveness.heartbeat(second * 1000));
    }
    completion
        .try_send(())
        .expect("qualified cooling completed");
    let response = owner.join().expect("control owner joins");
    let raw: Box<serde_json::value::RawValue> =
        serde_json::from_slice(response.frame()).expect("Restore response");
    let wire = SerialEnvelope::encode(
        SerialKind::Control,
        Some("AAAAAAAAAAAAAAAAAAAAAA"),
        121,
        &raw,
    )
    .expect("same-session reply");

    // Assert
    assert!(liveness.poll(120_001));
    assert!(liveness.is_authenticated());
    let envelope = SerialEnvelope::parse(&wire).expect("browser receives framed acknowledgement");
    let reply: serde_json::Value =
        serde_json::from_str(envelope.payload.get()).expect("reply JSON");
    assert_eq!(reply["result"]["restoration"]["status"], "confirmed");
}

#[test]
fn lost_heartbeats_close_and_deauthenticate_even_while_work_is_already_revoked() {
    // Arrange
    let mut liveness = SerialLinkLiveness::new(0);
    liveness.authenticate();
    assert!(liveness.heartbeat(1000));

    // Act
    let channel_live = liveness.poll(3800);

    // Assert
    assert!(!channel_live);
    assert!(!liveness.is_authenticated());
    liveness.authenticate();
    assert!(
        !liveness.heartbeat(4000),
        "late heartbeat and proof cannot revive the closed channel"
    );
}
