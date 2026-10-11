//! Hello, heartbeat and receive credit.
use super::*;

#[test]
fn hello_ack_is_a_session_record_at_sequence_zero() {
    // Arrange / Act
    let (_, _, sink) = open();
    // Assert
    let envelope = &sink.envelopes()[0];
    assert_eq!((envelope.kind, envelope.sequence), (SerialKind::Session, 0));
    assert_eq!(
        sink.records[0].1,
        SerialOutputPurpose::Protocol(ProtocolRecord::Hello)
    );
}

#[test]
fn stale_hello_changes_nothing() {
    // Arrange
    let view = View::default();
    view.begin(2);
    let mut sink = Sink::default();
    let mut writer = SerialRecordWriter::new();
    // Act
    let result = writer.hello(1, SESSION.into(), &json!({}), &view, &mut sink);
    // Assert
    assert!(result.is_none());
    assert_eq!(writer.epoch(), 0);
    assert!(sink.records.is_empty());
}

#[test]
fn hello_resynchronizes_before_its_ack() {
    // Arrange / Act
    let (_, _, sink) = open();
    // Assert
    assert_eq!(sink.resynchronized, 1);
}

#[test]
fn failed_hello_ack_revokes_the_new_epoch() {
    // Arrange
    let view = View::default();
    view.begin(1);
    let mut sink = Sink {
        maybe_fail_write: Some(1),
        ..Sink::default()
    };
    // Act
    let result = SerialRecordWriter::new().hello(1, SESSION.into(), &json!({}), &view, &mut sink);
    // Assert
    assert!(matches!(
        result,
        Some(Err(SerialWriteError::Sink(SinkError::Transport)))
    ));
    assert_eq!(view.current.get(), 0);
}

#[test]
fn first_record_after_the_ack_is_sequence_one() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .heartbeat_if_due(1000, &view, &mut sink)
        .expect("heartbeat");
    // Assert
    assert_eq!(sink.sequences(), [0, 1]);
}

#[test]
fn heartbeat_is_due_at_exactly_one_second() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .heartbeat_if_due(999, &view, &mut sink)
        .expect("not due");
    writer
        .heartbeat_if_due(1000, &view, &mut sink)
        .expect("due");
    // Assert
    assert_eq!(sink.records.len(), 2);
    assert_eq!(writer.maybe_next_heartbeat_ms(), Some(2000));
}

#[test]
fn no_heartbeat_for_a_revoked_epoch() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.current.set(0);
    // Act
    writer
        .heartbeat_if_due(5000, &view, &mut sink)
        .expect("skipped");
    // Assert
    assert_eq!(sink.records.len(), 1);
}

#[test]
fn heartbeat_failure_revokes_and_does_not_retry_immediately() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    sink.maybe_fail_write = Some(2);
    // Act
    let result = writer.heartbeat_if_due(1000, &view, &mut sink);
    // Assert
    assert!(result.is_err());
    assert_eq!(view.lines("revoke"), ["revoke 1"]);
    assert_eq!(writer.maybe_next_heartbeat_ms(), Some(2000));
}

#[test]
fn credit_is_sent_once_per_advance() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(5);
    // Act
    writer.receive_credit(&view, &mut sink).expect("credit");
    writer.receive_credit(&view, &mut sink).expect("unchanged");
    // Assert
    assert_eq!(sink.records.len(), 2);
    assert_eq!(
        sink.records[1].1,
        SerialOutputPurpose::Protocol(ProtocolRecord::ReceiveCredit)
    );
}

#[test]
fn credit_payload_is_the_exact_receive_credit_object() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(1024);
    // Act
    writer.receive_credit(&view, &mut sink).expect("credit");
    // Assert
    assert_eq!(
        sink.envelopes()[1].payload.get(),
        r#"{"op":"receive_credit","receivedBytes":1024}"#
    );
}

#[test]
fn failed_credit_is_retried_after_it_fails() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(5);
    sink.maybe_fail_write = Some(2);
    // Act
    let failed = writer.receive_credit(&view, &mut sink);
    view.begin(1);
    view.credit.publish(5);
    writer.receive_credit(&view, &mut sink).expect("credit");
    // Assert
    assert!(failed.is_err());
    assert_eq!(sink.records.len(), 2);
}

#[test]
fn terminal_credit_follows_a_validated_close_once() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(20);
    writer.receive_credit(&view, &mut sink).expect("credit");
    view.credit.close_record(1, 21, 0, || view.revoke_epoch(1));
    // Act
    writer.receive_credit(&view, &mut sink).expect("terminal");
    writer.receive_credit(&view, &mut sink).expect("finished");
    // Assert
    assert_eq!(
        sink.envelopes()[2].payload.get(),
        r#"{"op":"receive_credit","receivedBytes":21}"#
    );
    assert_eq!(sink.records.len(), 3);
}

#[test]
fn partial_output_blocks_the_terminal_credit_but_finishes_it() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(20);
    writer.receive_credit(&view, &mut sink).expect("credit");
    view.credit.close_record(1, 21, 0, || view.revoke_epoch(1));
    view.partial.set(true);
    // Act
    writer.receive_credit(&view, &mut sink).expect("blocked");
    // Assert
    assert_eq!(sink.records.len(), 2);
    assert_eq!(view.lines("finish_terminal"), ["finish_terminal 1"]);
}

#[test]
fn failed_terminal_credit_still_finishes_the_receipt() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(20);
    writer.receive_credit(&view, &mut sink).expect("credit");
    view.credit.close_record(1, 21, 0, || view.revoke_epoch(1));
    sink.maybe_fail_write = Some(3);
    // Act
    let result = writer.receive_credit(&view, &mut sink);
    // Assert
    assert!(result.is_err());
    assert_eq!(view.lines("finish_terminal"), ["finish_terminal 1"]);
}
