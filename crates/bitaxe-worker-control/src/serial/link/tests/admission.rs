//! Hello admission, boot text and per-byte accounting before and at admission.
use super::*;

#[test]
fn boot_text_before_hello_has_no_effects() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.receive(b"boot text\n{\"not\":\"an envelope\"}\n", &effects);
    // Assert
    assert!(effects.log.borrow().is_empty());
    assert_eq!(effects.failure(), None);
}

#[test]
fn oversized_line_before_hello_records_nothing() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    let mut line = vec![b'a'; MAXIMUM_WIRE_FRAME_BYTES + 10];
    line.push(b'\n');
    // Act
    receiver.receive(&line, &effects);
    // Assert
    assert_eq!(effects.failure(), None);
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn hello_emits_admission_effects_in_order() {
    // Arrange / Act
    let (_, effects) = admitted();
    // Assert
    let log = effects.log.borrow();
    let order: Vec<&str> = log
        .iter()
        .map(|line| line.split(' ').next().unwrap_or(""))
        .collect();
    assert_eq!(
        order,
        [
            "begin_generation",
            "begin_credit",
            "trace",
            "enqueue_session",
            "hello_ack"
        ]
    );
}

#[test]
fn hello_publishes_its_epoch() {
    // Arrange / Act
    let (_, effects) = admitted();
    // Assert
    assert_eq!(effects.current.get(), 1);
}

#[test]
fn hello_ack_payload_carries_binding_and_identity() {
    // Arrange
    let binding = SerialSessionBinding::parse(
        &URL_SAFE_NO_PAD.encode([1; 16]),
        &URL_SAFE_NO_PAD.encode([2; 32]),
        &URL_SAFE_NO_PAD.encode([3; 32]),
    )
    .expect("binding");
    // Act
    let payload = hello_ack_payload(&binding, "c0ffee", "e".repeat(64));
    // Assert
    assert_eq!(payload["op"], "hello_ack");
    assert_eq!(payload["hostNonce"], binding.host_nonce);
    assert_eq!(payload["deviceNonce"], binding.device_nonce);
    assert_eq!(payload["firmwareSourceCommit"], "c0ffee");
    assert_eq!(payload["receiveWindowBytes"], HOST_RECEIVE_WINDOW_BYTES);
    assert_eq!(payload["receivedBytes"], 0);
    assert_eq!(payload["serialManifest"], serial_manifest());
}

#[test]
fn device_nonces_are_canonical_unpadded_base64url() {
    // Arrange
    let effects = Recording::default();
    // Act
    let short = device_nonce::<16, _>(&effects).expect("nonce");
    let long = device_nonce::<32, _>(&effects).expect("nonce");
    // Assert
    assert_eq!(short, URL_SAFE_NO_PAD.encode((0..16).collect::<Vec<u8>>()));
    assert_eq!(long, URL_SAFE_NO_PAD.encode((16..48).collect::<Vec<u8>>()));
}

#[test]
fn hello_lf_is_not_credited_and_starts_the_counter() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.receive(&[hello_record(), b"ab".to_vec()].concat(), &effects);
    // Assert
    assert_eq!(effects.credit.maybe_received_bytes(1), Some(2));
}

#[test]
fn session_record_with_session_id_is_not_hello() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.receive(
        &record(SerialKind::Session, Some(&session(0)), 0, &hello_payload()),
        &effects,
    );
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn session_record_with_nonzero_sequence_is_not_hello() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.receive(
        &record(SerialKind::Session, None, 1, &hello_payload()),
        &effects,
    );
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn wrong_op_or_noncanonical_nonce_or_unknown_field_is_rejected() {
    for payload in [
        hello_payload().replace("\"hello\"", "\"hullo\""),
        r#"{"op":"hello","hostNonce":"short"}"#.to_owned(),
        hello_payload().replace('}', r#","extra":1}"#),
    ] {
        // Arrange
        let effects = Recording::default();
        let mut receiver = Receiver::new();
        // Act
        receiver.receive(&record(SerialKind::Session, None, 0, &payload), &effects);
        // Assert
        assert_eq!(receiver.maybe_epoch(), None, "{payload}");
    }
}

#[test]
fn rejected_hello_does_not_consume_an_epoch() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    let wrong = record(
        SerialKind::Session,
        None,
        0,
        &hello_payload().replace("\"hello\"", "\"x\""),
    );
    // Act
    receiver.receive(&[wrong, hello_record()].concat(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
}

#[test]
fn hello_above_receive_window_closes_previous_link_without_admission() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let padded = format!(
        r#"{{"op":"hello","hostNonce":"{}"{}}}"#,
        URL_SAFE_NO_PAD.encode(HOST_NONCE),
        " ".repeat(HOST_RECEIVE_WINDOW_BYTES as usize)
    );
    // Act
    receiver.receive(&record(SerialKind::Session, None, 0, &padded), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert_eq!(effects.lines("revoke_epoch"), ["revoke_epoch 1"]);
}

#[test]
fn second_hello_closes_previous_link_before_beginning_a_generation() {
    // Arrange
    let (mut receiver, effects) = admitted();
    effects.log.borrow_mut().clear();
    // Act
    receiver.receive(&hello_record(), &effects);
    // Assert
    let log = effects.log.borrow();
    assert_eq!(log[0], "close_generation 1 0");
    assert_eq!(log[1], "revoke_epoch 1");
    assert_eq!(log[2], "begin_generation 0");
    assert_eq!(receiver.maybe_epoch(), Some(2));
}

#[test]
fn entropy_failure_consumes_an_epoch_and_admits_nothing() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    effects.entropy_fails.set(true);
    // Act
    receiver.receive(&hello_record(), &effects);
    effects.entropy_fails.set(false);
    receiver.receive(&hello_record(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(2));
}

#[test]
fn unavailable_generation_publishes_no_epoch() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    effects.begin_fails.set(true);
    // Act
    receiver.receive(&hello_record(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert_eq!(effects.current.get(), 0);
    assert!(effects.lines("begin_credit").is_empty());
}

#[test]
fn session_enqueue_rejection_revokes_without_acknowledging() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    effects.session_rejected.set(true);
    // Act
    receiver.receive(&hello_record(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert_eq!(effects.lines("close_generation"), ["close_generation 1 0"]);
    assert_eq!(effects.lines("revoke_epoch"), ["revoke_epoch 1"]);
    assert!(effects.lines("hello_ack").is_empty());
}

#[test]
fn hello_ack_failure_revokes_the_generation_and_epoch() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    effects.ack_fails.set(true);
    // Act
    receiver.receive(&hello_record(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert_eq!(effects.lines("close_generation"), ["close_generation 1 0"]);
    assert_eq!(effects.current.get(), 0);
}

#[test]
fn exhausted_epoch_counter_rejects_hello() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new().with_next_epoch(u32::MAX);
    // Act
    receiver.receive(&hello_record(), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert!(effects.lines("begin_generation").is_empty());
}

#[test]
fn liveness_starts_at_admission_time() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    effects.now.set(500);
    receiver.receive(&hello_record(), &effects);
    // Act
    receiver.supervise(500 + HEARTBEAT_TIMEOUT - 1, &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
}

const HEARTBEAT_TIMEOUT: u64 = crate::serial::HEARTBEAT_TIMEOUT_MILLISECONDS;
