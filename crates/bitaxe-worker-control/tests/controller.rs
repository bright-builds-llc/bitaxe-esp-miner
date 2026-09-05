#[path = "controller/liveness.rs"]
mod liveness;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bitaxe_worker_control::{
    DeviceIdentity, LeaseAuthorizationError, LeaseAuthorizationVerifier, LeaseDeadlines,
    RestorationReason, WorkerControl, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
};
use serde_json::json;

#[derive(Default)]
struct FixtureVerifier;

impl LeaseAuthorizationVerifier for FixtureVerifier {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

    fn verify_start(
        &mut self,
        grant: &WorkerLeaseGrant,
        _context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        if grant.authorization() == "fixture-authentication-not-a-production-secret"
            && grant.challenge_id() == "challenge_00000000000000000000000000000001"
        {
            Ok(())
        } else {
            Err(LeaseAuthorizationError::InvalidAuthorization)
        }
    }

    fn verify_renewal(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        challenge_id: &str,
        _context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        if renewal.authorization() == "fixture-renewal-authentication"
            && challenge_id == "challenge_00000000000000000000000000000001"
        {
            Ok(())
        } else {
            Err(LeaseAuthorizationError::InvalidAuthorization)
        }
    }
}

#[derive(Default)]
struct FakeSession {
    events: Vec<&'static str>,
    fail_start: bool,
    remaining_safe_stop_failures: usize,
}

impl WorkerSession for FakeSession {
    fn start(
        &mut self,
        _grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("start");
        if self.fail_start {
            Err(WorkerSessionError::Rejected)
        } else {
            Ok(())
        }
    }

    fn renew(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("renew");
        Ok(())
    }

    fn safe_stop(&mut self, reason: RestorationReason) -> Result<(), WorkerSessionError> {
        self.events.push(reason.category());
        if self.remaining_safe_stop_failures > 0 {
            self.remaining_safe_stop_failures -= 1;
            Err(WorkerSessionError::SafeStopFailed)
        } else {
            Ok(())
        }
    }
}

#[test]
fn possession_must_be_sent_before_a_work_lease_can_start() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh serial session");

    // Act
    let rejected = worker.prepare_frame(start_frame().as_bytes(), 1_000);

    // Assert
    assert_eq!(
        rejected.expect_err("unadmitted Start must fail").category(),
        "admission_required"
    );
    assert!(worker.session().events.is_empty());

    // Act
    let proof = worker
        .prepare_frame(
            possession_frame(worker.capability_sha256()).as_bytes(),
            1_000,
        )
        .expect("possession response should prepare");
    let still_rejected = worker.prepare_frame(start_frame().as_bytes(), 1_000);

    // Assert
    assert_eq!(
        still_rejected
            .expect_err("unsent proof must not admit")
            .category(),
        "admission_required"
    );

    // Act
    worker
        .confirm_sent(proof)
        .expect("sent proof should admit this logical session");
    let started = worker
        .prepare_frame(start_frame().as_bytes(), 1_000)
        .expect("admitted authenticated Start should succeed");

    // Assert
    assert!(String::from_utf8_lossy(started.frame()).contains("\"state\":\"mining\""));
    assert_eq!(worker.session().events, ["start"]);
}

#[test]
fn disconnect_safe_stops_before_it_clears_logical_session_admission() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1_000)
        .expect("admitted Start should succeed");

    // Act
    worker
        .disconnect(1_001)
        .expect("disconnect should complete safe stop");

    // Assert
    assert_eq!(worker.session().events, ["start", "connectivity_lost"]);
    assert_eq!(
        worker
            .prepare_frame(start_frame().as_bytes(), 1_002)
            .expect_err("disconnect must clear admission")
            .category(),
        "admission_required"
    );
    assert!(!format!("{worker:?}").contains("fixture-session-password"));
}

#[test]
fn every_local_restoration_reason_safe_stops_and_clears_admission() {
    // Arrange
    let reasons = [
        RestorationReason::Paused,
        RestorationReason::Cancelled,
        RestorationReason::LeaseExpired,
        RestorationReason::LostContinuity,
        RestorationReason::MonotonicReset,
        RestorationReason::Reboot,
        RestorationReason::ChallengeSatisfied,
        RestorationReason::ChallengeExpired,
        RestorationReason::TabClosed,
        RestorationReason::ConnectivityLost,
        RestorationReason::ControlFailed,
    ];

    // Act / Assert
    for reason in reasons {
        let mut worker = admitted_worker();
        worker
            .prepare_frame(start_frame().as_bytes(), 1_000)
            .expect("admitted Start should succeed");
        worker
            .prepare_frame(restore_frame(reason).as_bytes(), 1_001)
            .expect("restoration should complete");
        assert_eq!(worker.session().events, ["start", reason.category()]);
        assert_eq!(
            worker
                .prepare_frame(start_frame().as_bytes(), 1_002)
                .expect_err("restoration must clear admission")
                .category(),
            "admission_required"
        );
    }
}

#[test]
fn unused_possession_context_expires_at_sixty_monotonic_seconds() {
    // Arrange
    let mut accepted = admitted_worker();
    let mut expired = admitted_worker();

    // Act
    let last_millisecond = accepted.prepare_frame(start_frame().as_bytes(), 60_999);
    let boundary = expired.prepare_frame(start_frame().as_bytes(), 61_000);

    // Assert
    assert!(last_millisecond.is_ok());
    assert_eq!(
        boundary
            .expect_err("sixty-second context must expire")
            .category(),
        "admission_required"
    );
    assert!(expired.session().events.is_empty());
}

#[test]
fn partial_start_retains_cleanup_responsibility_until_safe_stop_confirms() {
    // Arrange
    let mut worker = worker_with_session(FakeSession {
        fail_start: true,
        remaining_safe_stop_failures: 1,
        ..FakeSession::default()
    });
    admit(&mut worker, 1_000);

    // Act
    let start = worker.prepare_frame(start_frame().as_bytes(), 1_000);
    let pending_after_failure = worker.has_active_lease();
    let retry = worker.tick(1_001);

    // Assert
    assert_eq!(
        start
            .expect_err("partial Start must fail closed")
            .category(),
        "session_failed"
    );
    assert!(pending_after_failure);
    assert!(retry.is_ok());
    assert!(!worker.has_active_lease());
    assert_eq!(
        worker.session().events,
        ["start", "control_failed", "control_failed"]
    );
}

#[test]
fn possession_nonce_is_not_evicted_after_seventeen_proofs() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh serial session");
    let first_nonce = URL_SAFE_NO_PAD.encode([0_u8; 32]);
    for index in 0_u16..17 {
        let mut nonce_bytes = [0_u8; 32];
        nonce_bytes[..2].copy_from_slice(&index.to_be_bytes());
        let frame = possession_frame_with_nonce(
            worker.capability_sha256(),
            &format!("pos_nonce_{index}"),
            &URL_SAFE_NO_PAD.encode(nonce_bytes),
        );
        let proof = worker
            .prepare_frame(frame.as_bytes(), u64::from(index))
            .expect("unique possession proof should prepare");
        worker
            .confirm_sent(proof)
            .expect("unique possession proof should admit");
    }

    // Act
    let replay = worker.prepare_frame(
        possession_frame_with_nonce(worker.capability_sha256(), "pos_replay", &first_nonce)
            .as_bytes(),
        17,
    );

    // Assert
    assert_eq!(
        replay
            .expect_err("old nonce must remain consumed")
            .category(),
        "invalid_proof"
    );
}

#[test]
fn possession_nonce_capacity_fails_closed_without_eviction() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh serial session");
    for index in 0_u16..256 {
        let mut nonce_bytes = [0_u8; 32];
        nonce_bytes[..2].copy_from_slice(&index.to_be_bytes());
        let frame = possession_frame_with_nonce(
            worker.capability_sha256(),
            &format!("pos_capacity_{index}"),
            &URL_SAFE_NO_PAD.encode(nonce_bytes),
        );
        let proof = worker
            .prepare_frame(frame.as_bytes(), u64::from(index))
            .expect("bounded unique possession proof should prepare");
        worker
            .confirm_sent(proof)
            .expect("bounded unique possession proof should admit");
    }
    let overflow_nonce = URL_SAFE_NO_PAD.encode([0xff_u8; 32]);

    // Act
    let overflow = worker.prepare_frame(
        possession_frame_with_nonce(
            worker.capability_sha256(),
            "pos_capacity_overflow",
            &overflow_nonce,
        )
        .as_bytes(),
        256,
    );

    // Assert
    assert_eq!(
        overflow
            .expect_err("nonce cache exhaustion must fail closed")
            .category(),
        "invalid_proof"
    );
}

fn worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    worker_with_session(FakeSession::default())
}

fn worker_with_session(session: FakeSession) -> WorkerControl<FixtureVerifier, FakeSession> {
    WorkerControl::new(
        DeviceIdentity::from_seed([7_u8; 32]),
        FixtureVerifier,
        session,
        None,
        bitaxe_worker_control::FirmwareIdentity::new(
            bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
                .expect("fixture source commit should parse"),
            &"b".repeat(64),
        )
        .expect("fixture firmware identity"),
        json!({
            "protocolVersion": "bwg-worker-controller/0.4",
            "transportProfile": "bwg-worker-serial/0.1"
        }),
        "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA",
    )
    .expect("fixture Worker configuration should be valid")
}

fn admitted_worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = worker();
    admit(&mut worker, 1_000);
    worker
}

fn admit(worker: &mut WorkerControl<FixtureVerifier, FakeSession>, now: u64) {
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh serial session");
    let proof = worker
        .prepare_frame(possession_frame(worker.capability_sha256()).as_bytes(), now)
        .expect("possession response should prepare");
    worker
        .confirm_sent(proof)
        .expect("sent proof should admit this logical session");
}

fn possession_frame(capability_sha256: &str) -> String {
    possession_frame_with_nonce(
        capability_sha256,
        "pos_initial_01",
        "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
    )
}

fn possession_frame_with_nonce(capability_sha256: &str, request_id: &str, nonce: &str) -> String {
    format!(
        concat!(
            "{{\"profile\":\"bwg-worker-possession/0.2\",",
            "\"requestId\":\"{}\",",
            "\"command\":\"prove_possession\",",
            "\"payload\":{{",
            "\"purpose\":\"initial_admission\",",
            "\"possessionNonce\":\"{}\",",
            "\"challengeBindingSha256\":\"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC\",",
            "\"controllerCapabilitySha256\":\"{}\",",
            "\"sessionId\":\"AAAAAAAAAAAAAAAAAAAAAA\",",
            "\"hostNonce\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",",
            "\"deviceNonce\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",",
            "\"serialManifestSha256\":\"rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA\"",
            "}}}}\n"
        ),
        request_id, nonce, capability_sha256
    )
}

fn start_frame() -> String {
    concat!(
        "{\"protocolVersion\":\"bwg-worker-controller/0.4\",",
        "\"requestId\":\"serial_v03_start\",",
        "\"command\":\"start_lease\",",
        "\"payload\":{",
        "\"protocolVersion\":\"bwg-worker-controller/0.4\",",
        "\"leaseId\":\"lease_fixture_03\",",
        "\"challengeId\":\"challenge_00000000000000000000000000000001\",",
        "\"authorization\":\"fixture-authentication-not-a-production-secret\",",
        "\"durationMilliseconds\":60000,",
        "\"renewAfterMilliseconds\":20000,",
        "\"stratum\":{",
        "\"endpoint\":\"stratum+tcp://127.0.0.1:3333/\",",
        "\"username\":\"fixture-session-user\",",
        "\"password\":\"fixture-session-password\"",
        "}}}\n"
    )
    .to_owned()
}

fn restore_frame(reason: RestorationReason) -> String {
    format!(
        concat!(
            "{{\"protocolVersion\":\"bwg-worker-controller/0.4\",",
            "\"requestId\":\"serial_v03_restore\",",
            "\"command\":\"restore\",",
            "\"payload\":{{\"reason\":\"{}\"}}}}\n"
        ),
        reason.category()
    )
}

fn fixture_binding() -> bitaxe_worker_control::serial::SerialSessionBinding {
    bitaxe_worker_control::serial::SerialSessionBinding::parse(
        "AAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    )
    .expect("fixture nonce encoding")
}

#[test]
fn possession_from_a_different_logical_connection_is_rejected() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh connection");
    let frame = possession_frame(worker.capability_sha256())
        .replace("AAAAAAAAAAAAAAAAAAAAAA\"", "AgICAgICAgICAgICAgICAg\"");

    // Act
    let result = worker.prepare_frame(frame.as_bytes(), 0);

    // Assert
    assert_eq!(
        result.expect_err("cross-session proof").category(),
        "invalid_proof"
    );
}

#[test]
fn a_new_logical_connection_cannot_replace_active_work() {
    // Arrange
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1_001)
        .expect("authorized Start");

    // Act
    let result = worker.begin_serial_session(fixture_binding());

    // Assert
    assert_eq!(
        result
            .expect_err("active session cannot be replaced")
            .category(),
        "invalid_transition"
    );
    assert!(worker.has_active_lease());
}

#[test]
fn admitted_transport_probe_round_trips_maximum_controller_payload() {
    // Arrange
    let mut worker = admitted_worker();
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4", "requestId":"serial_probe", "command":"transport_probe", "payload":{"padding":""}});
    let overhead = serde_json::to_vec(&request).expect("probe JSON").len();
    let padding =
        "x".repeat(bitaxe_worker_control::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES - overhead);
    request["payload"]["padding"] = padding.clone().into();
    let mut frame = serde_json::to_vec(&request).expect("bounded probe JSON");
    frame.push(b'\n');

    // Act
    let response = worker
        .prepare_frame(&frame, 1_001)
        .expect("maximum valid probe");
    let value: serde_json::Value =
        serde_json::from_slice(response.frame()).expect("probe response");

    // Assert
    assert_eq!(value["result"]["padding"], padding);
    assert_eq!(worker.session().events, Vec::<&str>::new());
}

#[test]
fn transport_probe_cannot_echo_arbitrary_data() {
    // Arrange
    let mut worker = admitted_worker();
    let mut frame = serde_json::to_vec(&json!({"protocolVersion":"bwg-worker-controller/0.4", "requestId":"serial_probe", "command":"transport_probe", "payload":{"padding":"arbitrary"}})).expect("probe JSON");
    frame.push(b'\n');

    // Act
    let result = worker.prepare_frame(&frame, 1_001);

    // Assert
    assert_eq!(
        result
            .expect_err("only fixed test pattern allowed")
            .category(),
        "invalid_request"
    );
}
