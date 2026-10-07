//! Decreasing-clock branch plus the bounded BWG-007 clock-reset stimulus.
use super::*;
use bitaxe_worker_control::WorkerControlError;

const NONCE: &str = "AAECAwQFBgcICQoLDA0ODw";

fn command_frame(command: &str, maybe_payload: Option<serde_json::Value>) -> Vec<u8> {
    let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4",
        "requestId":"serial_clock_seam","command":command});
    if let Some(payload) = maybe_payload {
        request["payload"] = payload;
    }
    let mut frame = serde_json::to_vec(&request).expect("frame JSON");
    frame.push(b'\n');
    frame
}

fn stimulus_frame() -> Vec<u8> {
    command_frame(
        "clock_discontinuity_stimulus",
        Some(json!({"requestNonce": NONCE})),
    )
}

fn review_frame() -> Vec<u8> {
    command_frame("clock_discontinuity_stimulus_review", Some(json!({})))
}

fn result(prepared: &bitaxe_worker_control::PreparedResponse) -> serde_json::Value {
    let response: serde_json::Value =
        serde_json::from_slice(prepared.frame()).expect("response JSON");
    response["result"].clone()
}

/// Admitted at 1 s with a 60 s lease, last observed clock 10 s.
fn leased_worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = admitted_worker();
    worker
        .prepare_frame(start_frame().as_bytes(), 1_000)
        .expect("admitted Start");
    worker.tick(10_000).expect("increasing tick");
    worker
}

fn armed_worker() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = leased_worker();
    let ack = worker
        .prepare_frame(&stimulus_frame(), 10_000)
        .expect("stimulus ACK");
    worker
        .confirm_sent_at(ack, 10_000)
        .expect("delivered ACK arms");
    worker
}

fn reviewed(
    worker: &mut WorkerControl<FixtureVerifier, FakeSession>,
    now: u64,
) -> serde_json::Value {
    admit(worker, now);
    result(
        &worker
            .prepare_frame(&review_frame(), now)
            .expect("idle review"),
    )
}

fn category(
    result: Result<bitaxe_worker_control::PreparedResponse, WorkerControlError>,
) -> &'static str {
    result.expect_err("refused").category()
}

#[test]
fn decreasing_tick_during_a_lease_safe_stops_as_monotonic_reset() {
    // Arrange
    let mut worker = leased_worker();

    // Act
    let reset = worker.tick(9_999);

    // Assert
    assert_eq!(
        reset.expect_err("clock went backwards").category(),
        "monotonic_reset"
    );
    assert_eq!(worker.session().events, ["start", "monotonic_reset"]);
    assert!(!worker.is_admitted());
    assert!(!worker.has_active_lease());
}

#[test]
fn decreasing_tick_counts_one_discontinuity() {
    // Arrange
    let mut worker = leased_worker();

    // Act
    let _ = worker.tick(9_999);

    // Assert
    assert_eq!(worker.clock_discontinuities_detected(), 1);
}

#[test]
fn restoring_with_a_monotonic_reset_reason_is_not_a_detected_discontinuity() {
    // Arrange
    let mut worker = leased_worker();

    // Act
    worker
        .prepare_frame(
            restore_frame(RestorationReason::MonotonicReset).as_bytes(),
            10_001,
        )
        .expect("relabelled stop");

    // Assert
    assert_eq!(worker.session().events, ["start", "monotonic_reset"]);
    assert_eq!(worker.clock_discontinuities_detected(), 0);
}

#[test]
fn stimulus_acknowledgement_has_the_exact_wire_shape() {
    // Arrange
    let mut worker = leased_worker();

    // Act
    let ack = worker
        .prepare_frame(&stimulus_frame(), 10_000)
        .expect("stimulus ACK");

    // Assert
    assert_eq!(
        result(&ack),
        json!({"schema":"worker-clock-discontinuity-stimulus-v1","requestNonce":NONCE,
            "offsetMilliseconds":1000,"armedForMilliseconds":2000})
    );
}

#[test]
fn an_unconfirmed_acknowledgement_never_arms_the_stimulus() {
    // Arrange
    let mut worker = leased_worker();
    let _unsent = worker
        .prepare_frame(&stimulus_frame(), 10_000)
        .expect("stimulus ACK");

    // Act
    let tick = worker.tick(10_100);

    // Assert
    assert!(tick.is_ok());
    assert!(worker.has_active_lease());
}

#[test]
fn a_delivered_acknowledgement_makes_the_next_tick_observe_a_reset() {
    // Arrange
    let mut worker = armed_worker();

    // Act
    let tick = worker.tick(10_100);

    // Assert
    assert_eq!(
        tick.expect_err("stimulated reset").category(),
        "monotonic_reset"
    );
    assert_eq!(worker.session().events, ["start", "monotonic_reset"]);
    assert_eq!(worker.clock_discontinuities_detected(), 1);
}

#[test]
fn the_send_only_confirmation_cannot_arm_the_stimulus() {
    // Arrange
    let mut worker = leased_worker();
    let ack = worker
        .prepare_frame(&stimulus_frame(), 10_000)
        .expect("stimulus ACK");

    // Act
    let confirmed = worker.confirm_sent(ack);

    // Assert
    assert_eq!(
        confirmed.expect_err("needs a fresh clock").category(),
        "stale_response"
    );
    assert!(worker.tick(10_100).is_ok());
}

#[test]
fn frames_never_consume_the_armed_stimulus() {
    // Arrange
    let mut worker = armed_worker();
    worker
        .prepare_frame(&command_frame("status", None), 10_050)
        .expect("status while armed");

    // Act
    let tick = worker.tick(10_100);

    // Assert
    assert_eq!(
        tick.expect_err("tick consumes").category(),
        "monotonic_reset"
    );
}

#[test]
fn the_armed_window_expires_two_seconds_after_confirmation() {
    // Arrange
    let mut worker = armed_worker();

    // Act
    let tick = worker.tick(12_000);

    // Assert
    assert!(tick.is_ok());
    assert!(worker.has_active_lease());
    assert_eq!(worker.clock_discontinuities_detected(), 0);
}

#[test]
fn an_expired_stimulus_is_still_spent_for_the_boot() {
    // Arrange
    let mut worker = armed_worker();
    worker.tick(12_000).expect("window expired");
    worker
        .prepare_frame(command_frame("pause", None).as_slice(), 12_001)
        .expect("pause");

    // Act
    let review = reviewed(&mut worker, 12_002);

    // Assert
    assert_eq!(review["state"], "expired");
}

#[test]
fn the_stimulus_arms_at_most_once_per_boot() {
    // Arrange
    let mut worker = armed_worker();
    let _ = worker.tick(10_100);
    admit(&mut worker, 10_200);
    worker
        .prepare_frame(start_frame().as_bytes(), 10_200)
        .expect("second lease");
    worker.tick(11_000).expect("increasing tick");

    // Act
    let second = worker.prepare_frame(&stimulus_frame(), 11_000);

    // Assert
    assert_eq!(category(second), "invalid_transition");
}

#[test]
fn the_stimulus_requires_an_active_lease() {
    // Arrange
    let mut worker = admitted_worker();
    worker.tick(10_000).expect("increasing tick");

    // Act
    let refused = worker.prepare_frame(&stimulus_frame(), 10_000);

    // Assert
    assert_eq!(category(refused), "invalid_transition");
}

#[test]
fn the_stimulus_requires_current_admission() {
    // Arrange
    let mut worker = worker();
    worker
        .begin_serial_session(fixture_binding())
        .expect("fresh session");

    // Act
    let refused = worker.prepare_frame(&stimulus_frame(), 10_000);

    // Assert
    assert_eq!(category(refused), "admission_required");
}

#[test]
fn the_stimulus_requires_five_seconds_of_lease_headroom() {
    // Arrange
    let mut boundary = leased_worker();
    let mut late = leased_worker();

    // Act
    let accepted = boundary.prepare_frame(&stimulus_frame(), 56_000);
    let refused = late.prepare_frame(&stimulus_frame(), 56_001);

    // Assert
    assert!(accepted.is_ok());
    assert_eq!(category(refused), "invalid_transition");
}

#[test]
fn confirmation_rechecks_the_lease_headroom() {
    // Arrange
    let mut worker = leased_worker();
    let ack = worker
        .prepare_frame(&stimulus_frame(), 55_000)
        .expect("stimulus ACK");

    // Act
    let confirmed = worker.confirm_sent_at(ack, 56_001);

    // Assert
    assert_eq!(
        confirmed.expect_err("too late to arm").category(),
        "invalid_transition"
    );
    assert!(worker.tick(56_002).is_ok());
}

#[test]
fn a_pending_acknowledgement_refuses_another_stimulus() {
    // Arrange
    let mut worker = leased_worker();
    let _pending = worker
        .prepare_frame(&stimulus_frame(), 10_000)
        .expect("first ACK");

    // Act
    let second = worker.prepare_frame(&stimulus_frame(), 10_001);

    // Assert
    assert_eq!(category(second), "invalid_transition");
}

#[test]
fn a_stratum_v2_lease_refuses_the_stimulus() {
    // Arrange
    let mut worker = admitted_worker();
    let observation = worker
        .prepare_frame(
            &command_frame(
                "stratum_v2_status",
                Some(
                    json!({"schema":"worker-stratum-v2-query-v1","scope":"share","attemptId":null}),
                ),
            ),
            1_001,
        )
        .expect("idle share");
    worker.confirm_sent(observation).expect("delivered");
    let mut grant: serde_json::Value = serde_json::from_str(&start_frame()).expect("grant");
    grant["payload"]["stratum"] = json!({"profile":"bwg-worker-stratum-v2-standard/0.1",
        "endpoint":"stratum+tcp://192.168.1.3:12345/",
        "authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"});
    grant["payload"]["qualificationAttempt"] = json!({"schema":"worker-qualification-attempt-v1",
        "id":URL_SAFE_NO_PAD.encode([1;16]),"ordinal":18,"purpose":"normal",
        "maximumActiveMilliseconds":180000});
    let mut start = serde_json::to_vec(&grant).expect("grant frame");
    start.push(b'\n');
    worker.prepare_frame(&start, 1_002).expect("V2 Start");
    worker.tick(10_000).expect("increasing tick");

    // Act
    let refused = worker.prepare_frame(&stimulus_frame(), 10_000);

    // Assert
    assert_eq!(category(refused), "invalid_transition");
}

#[test]
fn a_noncanonical_nonce_is_an_invalid_request() {
    // Arrange
    let mut worker = leased_worker();
    let frame = command_frame(
        "clock_discontinuity_stimulus",
        Some(json!({"requestNonce": "AAECAwQFBgcICQoLDA0ODx"})),
    );

    // Act
    let refused = worker.prepare_frame(&frame, 10_000);

    // Assert
    assert_eq!(category(refused), "invalid_request");
}

#[test]
fn the_review_is_refused_during_a_lease() {
    // Arrange
    let mut worker = leased_worker();

    // Act
    let refused = worker.prepare_frame(&review_frame(), 10_000);

    // Assert
    assert_eq!(category(refused), "invalid_transition");
}

#[test]
fn the_review_requires_fresh_possession() {
    // Arrange
    let mut worker = admitted_worker();

    // Act
    let refused = worker.prepare_frame(&review_frame(), 61_000);

    // Assert
    assert_eq!(category(refused), "admission_required");
}

#[test]
fn the_review_reports_a_consumed_stimulus_and_its_detection() {
    // Arrange
    let mut worker = armed_worker();
    let _ = worker.tick(10_100);

    // Act
    let review = reviewed(&mut worker, 10_200);

    // Assert
    assert_eq!(
        review,
        json!({"schema":"worker-clock-discontinuity-stimulus-review-v1","state":"consumed",
            "offsetMilliseconds":1000,"discontinuitiesDetected":1})
    );
}

#[test]
fn the_review_is_read_only() {
    // Arrange
    let mut worker = admitted_worker();
    let first = result(
        &worker
            .prepare_frame(&review_frame(), 1_001)
            .expect("first review"),
    );

    // Act
    let second = result(
        &worker
            .prepare_frame(&review_frame(), 1_002)
            .expect("second review"),
    );

    // Assert
    assert_eq!(first, second);
    assert_eq!(first["state"], "idle");
    assert!(worker.session().events.is_empty());
    assert!(worker.is_admitted());
}

#[test]
fn the_review_requires_an_exactly_empty_payload() {
    // Arrange
    let mut worker = admitted_worker();
    let frame = command_frame("clock_discontinuity_stimulus_review", Some(json!({"x": 1})));

    // Act
    let refused = worker.prepare_frame(&frame, 1_001);

    // Assert
    assert_eq!(category(refused), "invalid_request");
}
