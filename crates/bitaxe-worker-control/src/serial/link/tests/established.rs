//! Supervision, framing, sequence, heartbeat, control and Close on an admitted link.
use super::*;

const TIMEOUT: u64 = crate::serial::HEARTBEAT_TIMEOUT_MILLISECONDS;

fn tamper(mut bytes: Vec<u8>, from: &[u8], to: &[u8]) -> Vec<u8> {
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("pattern present");
    bytes.splice(start..start + from.len(), to.iter().copied());
    bytes
}

#[test]
fn supervise_without_link_has_no_effects() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.supervise(10_000, &effects);
    // Assert
    assert!(effects.log.borrow().is_empty());
}

#[test]
fn revoked_epoch_closes_with_session_revoked() {
    // Arrange
    let (mut receiver, effects) = admitted();
    effects.current.set(0);
    // Act
    receiver.supervise(10, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::SessionRevoked));
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn heartbeat_deadline_closes_with_heartbeat_timeout() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.supervise(TIMEOUT, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::HeartbeatTimeout));
}

#[test]
fn link_survives_just_before_the_deadline() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.supervise(TIMEOUT - 1, &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
}

#[test]
fn clock_regression_closes_the_link() {
    // Arrange
    let (mut receiver, effects) = admitted();
    receiver.supervise(100, &effects);
    // Act
    receiver.supervise(50, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::HeartbeatTimeout));
}

#[test]
fn supervision_discards_a_partial_record() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let frame = control(1);
    receiver.receive(&frame[..20], &effects);
    effects.current.set(0);
    receiver.supervise(10, &effects);
    receiver.receive(&hello_record(), &effects);
    // Act
    receiver.receive(&frame[20..], &effects);
    // Assert
    assert!(effects.lines("enqueue_control").is_empty());
}

#[test]
fn read_failure_with_link_records_read() {
    // Arrange
    let (mut receiver, effects) = admitted();
    receiver.receive(b"partial", &effects);
    // Act
    receiver.read_failed(&effects);
    // Assert
    assert_eq!(effects.failure.get(), Some((SerialLinkFailure::Read, 7)));
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn read_failure_without_link_records_nothing() {
    // Arrange
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    // Act
    receiver.read_failed(&effects);
    // Assert
    assert_eq!(effects.failure(), None);
}

#[test]
fn each_non_lf_byte_publishes_cumulative_credit() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(b"abc", &effects);
    // Assert
    assert_eq!(effects.credit.maybe_received_bytes(1), Some(3));
}

#[test]
fn record_lf_is_credited_after_validation() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let frame = control(1);
    // Act
    receiver.receive(&frame, &effects);
    // Assert
    assert_eq!(
        effects.credit.maybe_received_bytes(1),
        Some(frame.len() as u32)
    );
}

#[test]
fn receive_counter_exhaustion_closes_the_link() {
    // Arrange
    let (mut receiver, effects) = admitted();
    receiver.force_received(u32::MAX);
    // Act
    receiver.receive(b"a", &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::ReceiveCounter));
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn observed_bytes_saturate_at_the_wire_bound() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let mut line = vec![b'a'; MAXIMUM_WIRE_FRAME_BYTES + 100];
    line.push(b'\n');
    // Act
    receiver.receive(&line, &effects);
    // Assert
    assert_eq!(
        effects.failure.get(),
        Some((SerialLinkFailure::Framing, MAXIMUM_WIRE_FRAME_BYTES))
    );
    assert_eq!(
        effects.lines("trace ValidationRejected"),
        [format!(
            "trace ValidationRejected 1 0 {MAXIMUM_WIRE_FRAME_BYTES}"
        )]
    );
}

#[test]
fn malformed_envelope_records_envelope() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(b"{\"not\":\"an envelope\"}\n", &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::Envelope));
}

#[test]
fn digest_mismatch_records_integrity() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(&tamper(control(1), b"status", b"statuz"), &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::Integrity));
}

#[test]
fn foreign_session_records_sequence() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let foreign = record(SerialKind::Control, Some(&session(3)), 1, "{}");
    // Act
    receiver.receive(&foreign, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::Sequence));
}

#[test]
fn repeated_sequence_records_sequence() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(&[control(2), control(2)].concat(), &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::Sequence));
    assert_eq!(effects.controls.borrow().len(), 1);
}

#[test]
fn sequence_gaps_are_accepted() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(&[control(2), control(9)].concat(), &effects);
    // Assert
    assert_eq!(effects.controls.borrow().len(), 2);
}

#[test]
fn maximum_sequence_records_sequence() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(&control(u32::MAX), &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::Sequence));
    assert!(effects.controls.borrow().is_empty());
}

#[test]
fn non_empty_heartbeat_records_heartbeat_payload() {
    // Arrange
    let (mut receiver, effects) = authenticated();
    let bad = record(SerialKind::Heartbeat, Some(&session(0)), 1, r#"{"a":1}"#);
    // Act
    receiver.receive(&bad, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::HeartbeatPayload));
}

#[test]
fn whitespace_heartbeat_is_accepted() {
    // Arrange
    let (mut receiver, effects) = authenticated();
    effects.now.set(1000);
    // Act
    receiver.receive(
        &record(SerialKind::Heartbeat, Some(&session(0)), 1, "{ \t }"),
        &effects,
    );
    // Assert
    assert_eq!(
        effects.lines("heartbeat_generation"),
        ["heartbeat_generation 1 1000"]
    );
}

#[test]
fn pre_admission_heartbeat_does_not_refresh_the_deadline() {
    // Arrange
    let (mut receiver, effects) = admitted();
    effects.now.set(2000);
    receiver.receive(&heartbeat(1), &effects);
    // Act
    receiver.supervise(TIMEOUT, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::HeartbeatTimeout));
    assert!(effects.lines("heartbeat_generation").is_empty());
}

#[test]
fn admitted_heartbeat_refreshes_the_deadline() {
    // Arrange
    let (mut receiver, effects) = authenticated();
    effects.now.set(2000);
    receiver.receive(&heartbeat(1), &effects);
    // Act
    receiver.supervise(TIMEOUT + 100, &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
}

#[test]
fn late_heartbeat_records_heartbeat_timeout() {
    // Arrange
    let (mut receiver, effects) = authenticated();
    effects.now.set(TIMEOUT);
    // Act
    receiver.receive(&heartbeat(1), &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::HeartbeatTimeout));
}

#[test]
fn cooling_heartbeat_keeps_the_link_open() {
    // Arrange
    let (mut receiver, effects) = authenticated();
    effects.cooling.set(true);
    // Act
    receiver.receive(&heartbeat(1), &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
}

#[test]
fn control_payload_reaches_the_owner_lexically_with_one_lf() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(&control(1), &effects);
    // Assert
    assert_eq!(
        effects.controls.borrow().as_slice(),
        [b"{\"command\":\"status\"}\n".to_vec()]
    );
}

#[test]
fn control_copy_has_exact_capacity() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let frame = control(1);
    // Act
    receiver.receive(&frame, &effects);
    // Assert
    let expected = format!("enqueue_control 1 1 {} capacity=21", frame.len());
    assert_eq!(effects.lines("enqueue_control"), [expected]);
}

#[test]
fn control_traces_validate_then_enqueue() {
    // Arrange
    let (mut receiver, effects) = admitted();
    effects.log.borrow_mut().clear();
    // Act
    receiver.receive(&control(1), &effects);
    // Assert
    let stages: Vec<String> = effects
        .lines("trace")
        .iter()
        .map(|line| line.split(' ').nth(1).unwrap_or("").to_owned())
        .collect();
    assert_eq!(stages, ["Validated", "EnqueueStarted", "Enqueued"]);
}

#[test]
fn control_queue_rejection_records_control_queue() {
    // Arrange
    let (mut receiver, effects) = admitted();
    effects.control_rejected.set(true);
    // Act
    receiver.receive(&control(1), &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::ControlQueue));
    assert_eq!(effects.lines("trace EnqueueRejected").len(), 1);
}

#[test]
fn clean_close_leaves_a_terminal_receipt() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let close = record(
        SerialKind::Session,
        Some(&session(0)),
        1,
        r#"{"op":"close","reason":"tab_closed"}"#,
    );
    // Act
    receiver.receive(&close, &effects);
    // Assert
    assert_eq!(receiver.maybe_epoch(), None);
    assert_eq!(
        effects.credit.maybe_terminal_bytes(1, 0),
        Some(close.len() as u32)
    );
    assert_eq!(effects.failure(), None);
}

#[test]
fn session_record_other_than_close_records_unexpected_kind() {
    // Arrange
    let (mut receiver, effects) = admitted();
    let credit = record(
        SerialKind::Session,
        Some(&session(0)),
        1,
        r#"{"op":"receive_credit","receivedBytes":1}"#,
    );
    // Act
    receiver.receive(&credit, &effects);
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::UnexpectedKind));
}

#[test]
fn diagnostic_record_records_unexpected_kind() {
    // Arrange
    let (mut receiver, effects) = admitted();
    // Act
    receiver.receive(
        &record(SerialKind::Diagnostic, Some(&session(0)), 1, "{}"),
        &effects,
    );
    // Assert
    assert_eq!(effects.failure(), Some(SerialLinkFailure::UnexpectedKind));
}
