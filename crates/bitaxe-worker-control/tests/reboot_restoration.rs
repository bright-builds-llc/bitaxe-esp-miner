use std::cell::RefCell;
use std::rc::Rc;

use bitaxe_worker_control::{
    DeviceIdentity, LeaseAuthorizationError, LeaseAuthorizationVerifier, LeaseDeadlines,
    RestorationReason, WorkerControl, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
};
use serde_json::json;

#[derive(Default)]
struct LifecycleState {
    pending: bool,
    remaining_mark_failures: usize,
    remaining_clear_failures: usize,
}

struct LifecycleVerifier {
    state: Rc<RefCell<LifecycleState>>,
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl LeaseAuthorizationVerifier for LifecycleVerifier {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.events.borrow_mut().push("mark_pending");
        let mut state = self.state.borrow_mut();
        if state.remaining_mark_failures > 0 {
            state.remaining_mark_failures -= 1;
            return Err(LeaseAuthorizationError::Persistence);
        }
        state.pending = true;
        Ok(())
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.events.borrow_mut().push("clear_pending");
        let mut state = self.state.borrow_mut();
        if state.remaining_clear_failures > 0 {
            state.remaining_clear_failures -= 1;
            return Err(LeaseAuthorizationError::Persistence);
        }
        state.pending = false;
        Ok(())
    }

    fn verify_start(
        &mut self,
        grant: &WorkerLeaseGrant,
        _context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        self.events.borrow_mut().push("verify_start");
        if grant.authorization() == "fixture-authentication-not-a-production-secret" {
            Ok(())
        } else {
            Err(LeaseAuthorizationError::InvalidAuthorization)
        }
    }

    fn verify_renewal(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _challenge_id: &str,
        _context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }
}

struct LifecycleSession {
    events: Rc<RefCell<Vec<&'static str>>>,
}

struct LifecycleHarness {
    worker: WorkerControl<LifecycleVerifier, LifecycleSession>,
    state: Rc<RefCell<LifecycleState>>,
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl WorkerSession for LifecycleSession {
    fn start(
        &mut self,
        _grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.borrow_mut().push("start");
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
        self.events.borrow_mut().push("safe_stop");
        Ok(())
    }
}

#[test]
fn reboot_confirmation_stays_durable_until_a_followup_request() {
    // Arrange
    let LifecycleHarness {
        mut worker,
        state,
        events,
    } = lifecycle_worker(true, 0, 0);

    // Act
    worker
        .disconnect(1)
        .expect("pre-report disconnect should remain baseline-safe");
    let first_status = worker
        .prepare_frame(status_frame().as_bytes(), 2)
        .expect("boot-restored status should render");
    let reported_status: serde_json::Value =
        serde_json::from_slice(first_status.frame()).expect("reported status should be JSON");
    worker
        .confirm_sent(first_status)
        .expect("boot-restored status should be sent");
    let pending_after_status = state.borrow().pending;
    let invalid_followup = worker.prepare_frame(b"{invalid}\n", 3);
    let pending_after_invalid = state.borrow().pending;
    let status = worker
        .prepare_frame(status_frame().as_bytes(), 4)
        .expect("follow-up status should acknowledge restoration");
    let status: serde_json::Value =
        serde_json::from_slice(status.frame()).expect("status should be JSON");

    // Assert
    assert_eq!(events.borrow().as_slice(), ["clear_pending"]);
    assert!(pending_after_status);
    assert_eq!(
        invalid_followup
            .expect_err("invalid traffic must not acknowledge restoration")
            .category(),
        "invalid_frame"
    );
    assert!(pending_after_invalid);
    assert!(!state.borrow().pending);
    assert_eq!(status["result"]["state"], "baseline");
    assert_eq!(status["result"]["restoration"]["status"], "confirmed");
    assert_eq!(reported_status["result"]["restoration"]["reason"], "reboot");
    assert_eq!(
        status["result"]["restoration"]["reason"],
        "connectivity_lost"
    );
}

#[test]
fn a_new_enumeration_cannot_acknowledge_an_old_status_response() {
    // Arrange
    let LifecycleHarness {
        mut worker,
        state,
        events,
    } = lifecycle_worker(true, 0, 0);
    worker.begin_enumeration();
    let first_status = worker
        .prepare_frame(status_frame().as_bytes(), 1)
        .expect("first reboot status should render");
    worker
        .confirm_sent(first_status)
        .expect("first reboot status should be sent");
    worker
        .disconnect(2)
        .expect("disconnect should preserve the confirmed baseline");

    // Act
    worker
        .prepare_frame(discover_frame().as_bytes(), 3)
        .expect("new enumeration discover should remain available");
    let pending_after_new_enumeration = state.borrow().pending;
    let second_status = worker
        .prepare_frame(status_frame().as_bytes(), 4)
        .expect("new enumeration should report reboot again");
    let reported: serde_json::Value = serde_json::from_slice(second_status.frame())
        .expect("repeated reboot status should be JSON");

    // Assert
    assert!(pending_after_new_enumeration);
    assert!(events.borrow().is_empty());
    assert_eq!(reported["result"]["restoration"]["reason"], "reboot");
}

#[test]
fn effect_marker_precedes_start_and_failed_clear_remains_retryable() {
    // Arrange
    let LifecycleHarness {
        mut worker,
        state,
        events,
    } = lifecycle_worker(false, 0, 1);
    admit(&mut worker, 1_000);

    // Act
    worker
        .prepare_frame(start_frame().as_bytes(), 1_000)
        .expect("marked Start should succeed");
    let after_start = events.borrow().clone();
    let first_stop = worker.prepare_frame(pause_frame().as_bytes(), 1_001);
    let pending_after_failure = state.borrow().pending;
    let active_after_failure = worker.has_active_lease();
    let retry = worker.tick(1_002);

    // Assert
    assert_eq!(after_start, ["verify_start", "mark_pending", "start"]);
    assert_eq!(
        first_stop
            .expect_err("failed marker clear must stay pending")
            .category(),
        "persistence_failed"
    );
    assert!(pending_after_failure);
    assert!(active_after_failure);
    assert!(retry.is_ok());
    assert!(!state.borrow().pending);
    assert!(!worker.has_active_lease());
    assert_eq!(
        events.borrow().as_slice(),
        [
            "verify_start",
            "mark_pending",
            "start",
            "safe_stop",
            "clear_pending",
            "safe_stop",
            "clear_pending",
        ]
    );
}

#[test]
fn effect_marker_failure_stops_before_the_session_effect() {
    // Arrange
    let LifecycleHarness {
        mut worker,
        state,
        events,
    } = lifecycle_worker(false, 1, 0);
    admit(&mut worker, 1_000);

    // Act
    let result = worker.prepare_frame(start_frame().as_bytes(), 1_000);

    // Assert
    assert_eq!(
        result
            .expect_err("marker failure must reject Start")
            .category(),
        "persistence_failed"
    );
    assert!(!state.borrow().pending);
    assert!(!worker.has_active_lease());
    assert_eq!(events.borrow().as_slice(), ["verify_start", "mark_pending"]);
}

fn lifecycle_worker(
    pending: bool,
    remaining_mark_failures: usize,
    remaining_clear_failures: usize,
) -> LifecycleHarness {
    let state = Rc::new(RefCell::new(LifecycleState {
        pending,
        remaining_mark_failures,
        remaining_clear_failures,
    }));
    let events = Rc::new(RefCell::new(Vec::new()));
    let worker = WorkerControl::new(
        DeviceIdentity::from_seed([7_u8; 32]),
        LifecycleVerifier {
            state: Rc::clone(&state),
            events: Rc::clone(&events),
        },
        LifecycleSession {
            events: Rc::clone(&events),
        },
        pending.then_some(RestorationReason::Reboot),
        bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
            .expect("fixture source commit should parse"),
        json!({
            "protocolVersion": "bwg-worker-controller/0.3",
            "transportProfile": "bwg-worker-usb/0.2"
        }),
        "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA",
    )
    .expect("lifecycle Worker should configure");
    LifecycleHarness {
        worker,
        state,
        events,
    }
}

fn admit(worker: &mut WorkerControl<LifecycleVerifier, LifecycleSession>, now: u64) {
    worker.begin_enumeration();
    let proof = worker
        .prepare_frame(possession_frame(worker.capability_sha256()).as_bytes(), now)
        .expect("possession response should prepare");
    worker
        .confirm_sent(proof)
        .expect("possession response should admit");
}

fn possession_frame(capability_sha256: &str) -> String {
    format!(
        concat!(
            "{{\"profile\":\"bwg-worker-possession/0.1\",",
            "\"requestId\":\"pos_initial_01\",",
            "\"command\":\"prove_possession\",",
            "\"payload\":{{",
            "\"purpose\":\"initial_admission\",",
            "\"possessionNonce\":\"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB\",",
            "\"challengeBindingSha256\":\"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC\",",
            "\"controllerCapabilitySha256\":\"{}\",",
            "\"applicationDescriptorSha256\":\"rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA\"",
            "}}}}\n"
        ),
        capability_sha256
    )
}

fn start_frame() -> String {
    concat!(
        "{\"protocolVersion\":\"bwg-worker-controller/0.3\",",
        "\"requestId\":\"usb_v03_start\",",
        "\"command\":\"start_lease\",",
        "\"payload\":{",
        "\"protocolVersion\":\"bwg-worker-controller/0.3\",",
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

fn pause_frame() -> String {
    concat!(
        "{\"protocolVersion\":\"bwg-worker-controller/0.3\",",
        "\"requestId\":\"usb_v03_pause\",",
        "\"command\":\"pause\"}\n"
    )
    .to_owned()
}

fn status_frame() -> String {
    concat!(
        "{\"protocolVersion\":\"bwg-worker-controller/0.3\",",
        "\"requestId\":\"usb_v03_status\",",
        "\"command\":\"status\"}\n"
    )
    .to_owned()
}

fn discover_frame() -> String {
    concat!(
        "{\"protocolVersion\":\"bwg-worker-controller/0.3\",",
        "\"requestId\":\"usb_v03_discover\",",
        "\"command\":\"discover\"}\n"
    )
    .to_owned()
}
