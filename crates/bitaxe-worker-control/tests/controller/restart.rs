use super::*;

fn ready() -> WorkerControl<FixtureVerifier, FakeSession> {
    let mut worker = admitted_worker();
    worker.session_mut().maybe_restart_context =
        Some(bitaxe_worker_control::QualificationRestartContext {
            boot_ordinal: 2,
            worker_generation: 7,
            transport_epoch: 1,
        });
    worker
}
fn frame(nonce: &str, ordinal: u64) -> String {
    format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_restart",
        "command":"qualification_restart","payload":{"requestNonce":nonce,"expectedBootOrdinal":ordinal}})
    )
}
fn request() -> String {
    frame(&URL_SAFE_NO_PAD.encode([3u8; 16]), 2)
}

#[test]
fn restart_ack_is_small_and_effect_occurs_only_after_fresh_confirmation() {
    // Arrange
    let mut worker = ready();
    // Act
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("admit");
    let value: serde_json::Value = serde_json::from_slice(response.frame()).expect("JSON");
    // Assert
    assert_eq!(
        value["result"],
        json!({"schema":"worker-qualification-restart-v1","requestNonce":URL_SAFE_NO_PAD.encode([3u8;16]),"bootOrdinal":2,"nextBootOrdinal":3})
    );
    assert!(worker.session().events.is_empty());
    assert!(!worker.has_active_lease());
    // Act
    worker.confirm_sent_at(response, 1002).expect("confirm");
    // Assert
    assert_eq!(worker.session().events, ["qualification_restart"]);
    assert!(worker.prepare_frame(request().as_bytes(), 1003).is_err());
    assert!(!worker.has_active_lease());
}

#[test]
fn unsupported_or_unpossessed_restart_never_reaches_session_effect() {
    // Arrange
    let mut unpossessed = worker();
    unpossessed
        .begin_serial_session(fixture_binding())
        .expect("session");
    let mut unsupported = admitted_worker();
    // Act / Assert
    assert!(unpossessed
        .prepare_frame(request().as_bytes(), 1001)
        .is_err());
    assert!(unsupported
        .prepare_frame(request().as_bytes(), 1001)
        .is_err());
    assert!(unpossessed.session().events.is_empty());
    assert!(unsupported.session().events.is_empty());
}

#[test]
fn invalid_nonce_and_boot_ordinal_do_not_consume_a_valid_restart() {
    // Arrange
    let mut worker = ready();
    // Act / Assert
    for (nonce, ordinal) in [
        ("bad".to_owned(), 2),
        (format!("{}=", URL_SAFE_NO_PAD.encode([3u8; 16])), 2),
        (URL_SAFE_NO_PAD.encode([3u8; 16]), 0),
        (URL_SAFE_NO_PAD.encode([3u8; 16]), 3),
        (URL_SAFE_NO_PAD.encode([3u8; 16]), 9_007_199_254_740_991),
    ] {
        assert!(worker
            .prepare_frame(frame(&nonce, ordinal).as_bytes(), 1001)
            .is_err());
    }
    assert!(worker.prepare_frame(request().as_bytes(), 1002).is_ok());
    assert!(worker.session().events.is_empty());
}

#[test]
fn active_work_rejects_restart() {
    // Arrange
    let mut worker = ready();
    worker
        .prepare_frame(start_frame().as_bytes(), 1001)
        .expect("start");
    // Act
    let result = worker.prepare_frame(request().as_bytes(), 1002);
    // Assert
    assert!(result.is_err());
    assert_eq!(worker.session().events, ["start"]);
}

#[test]
fn pending_fan_cleanup_rejects_restart() {
    // Arrange
    let mut worker = ready();
    let cooling = format!(
        "{}\n",
        json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_cooling","command":"qualification_cooling","payload":{"action":"prove_fan"}})
    );
    worker.prepare_frame(cooling.as_bytes(), 1001).expect("fan");
    // Act / Assert
    assert!(worker.prepare_frame(request().as_bytes(), 1002).is_err());
    assert_eq!(worker.session().events, ["cooling"]);
}

#[test]
fn abandoned_reply_consumes_once_latch_without_restarting() {
    // Arrange
    let mut worker = ready();
    // Act
    drop(
        worker
            .prepare_frame(request().as_bytes(), 1001)
            .expect("prepared"),
    );
    // Assert
    assert!(worker.prepare_frame(request().as_bytes(), 1002).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn old_confirmation_api_cannot_execute_restart_without_a_fresh_clock() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    // Act / Assert
    assert!(worker.confirm_sent(response).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn possession_expiring_during_reply_prevents_restart() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    // Act / Assert
    assert!(worker.confirm_sent_at(response, 61000).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn native_epoch_change_after_preparation_prevents_restart() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    worker
        .session_mut()
        .maybe_restart_context
        .as_mut()
        .expect("context")
        .transport_epoch = 2;
    // Act / Assert
    assert!(worker.confirm_sent_at(response, 1002).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn a_new_logical_session_cannot_reuse_the_consumed_restart() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    worker.disconnect(1002).expect("disconnect");
    admit(&mut worker, 1003);
    // Act / Assert
    assert!(worker.confirm_sent_at(response, 1004).is_err());
    assert!(worker.prepare_frame(request().as_bytes(), 1005).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn proof_replacement_during_reply_invalidates_the_restart_token() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    let proof = possession_frame_with_nonce(
        worker.capability_sha256(),
        "pos_replacement",
        &URL_SAFE_NO_PAD.encode([4u8; 32]),
    );
    // Act
    let refreshed = worker
        .prepare_frame(proof.as_bytes(), 1002)
        .expect("fresh proof");
    worker.confirm_sent(refreshed).expect("admit fresh proof");
    // Assert
    assert!(worker.confirm_sent_at(response, 1003).is_err());
    assert!(worker.session().events.is_empty());
}

#[test]
fn clock_reversal_during_reply_prevents_restart() {
    // Arrange
    let mut worker = ready();
    let response = worker
        .prepare_frame(request().as_bytes(), 1001)
        .expect("prepared");
    // Act / Assert
    assert!(worker.confirm_sent_at(response, 1000).is_err());
    assert!(!worker.session().events.contains(&"qualification_restart"));
}

#[test]
fn replacement_possession_pending_rejects_restart() {
    // Arrange
    let mut worker = ready();
    let proof = possession_frame_with_nonce(
        worker.capability_sha256(),
        "pos_pending_restart",
        &URL_SAFE_NO_PAD.encode([5u8; 32]),
    );
    let pending = worker
        .prepare_frame(proof.as_bytes(), 1001)
        .expect("prepare replacement proof");
    // Act
    let restart = worker.prepare_frame(request().as_bytes(), 1002);
    // Assert
    assert!(restart.is_err());
    assert!(worker.session().events.is_empty());
    // Act
    worker
        .confirm_sent(pending)
        .expect("confirm replacement proof");
    let restart = worker
        .prepare_frame(request().as_bytes(), 1003)
        .expect("restart after confirmed possession");
    worker
        .confirm_sent_at(restart, 1004)
        .expect("confirm restart");
    // Assert
    assert_eq!(worker.session().events, ["qualification_restart"]);
}
