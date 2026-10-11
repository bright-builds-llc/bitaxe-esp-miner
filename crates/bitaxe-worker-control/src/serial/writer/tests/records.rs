//! Control replies, sequencing and diagnostics.
use super::*;

fn payload(bytes: usize) -> Vec<u8> {
    format!(r#"{{"p":"{}"}}"#, "x".repeat(bytes - 8)).into_bytes()
}

#[test]
fn control_reply_follows_pending_credit() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.credit.publish(30);
    // Act
    writer
        .control(
            correlation(1, 5),
            br#"{"ok":true}"#,
            |_| true,
            &view,
            &mut sink,
        )
        .expect("reply");
    // Assert
    let purposes: Vec<_> = sink.records.iter().map(|(_, purpose)| *purpose).collect();
    assert_eq!(
        purposes,
        [
            SerialOutputPurpose::Protocol(ProtocolRecord::Hello),
            SerialOutputPurpose::Protocol(ProtocolRecord::ReceiveCredit),
            SerialOutputPurpose::Protocol(ProtocolRecord::ControlReply)
        ]
    );
}

#[test]
fn control_reply_payload_is_lexically_unchanged() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .control(
            correlation(1, 1),
            br#"{"b":1, "a":2}"#,
            |_| true,
            &view,
            &mut sink,
        )
        .expect("reply");
    // Assert
    assert_eq!(sink.envelopes()[1].payload.get(), r#"{"b":1, "a":2}"#);
}

#[test]
fn sent_reply_reports_true_to_its_receipt() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    let receipt = Cell::new(None);
    // Act
    writer
        .control(
            correlation(1, 1),
            b"{}",
            |sent| {
                receipt.set(Some(sent));
                true
            },
            &view,
            &mut sink,
        )
        .expect("reply");
    // Assert
    assert_eq!(receipt.get(), Some(true));
    assert!(view.lines("revoke").is_empty());
}

#[test]
fn stale_control_is_rejected_and_revokes_the_wanted_epoch() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    let result = writer.control(correlation(7, 1), b"{}", |_| true, &view, &mut sink);
    // Assert
    assert!(result.is_ok());
    assert_eq!(
        view.lines("trace WriterRejected"),
        ["trace WriterRejected 7 1"]
    );
    assert_eq!(view.lines("revoke"), ["revoke 7"]);
    assert_eq!(sink.records.len(), 1);
}

#[test]
fn a_4096_byte_reply_has_no_preceding_heartbeat() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .control(
            correlation(1, 1),
            &payload(4096),
            |_| true,
            &view,
            &mut sink,
        )
        .expect("reply");
    // Assert
    assert_eq!(
        sink.records[1].1,
        SerialOutputPurpose::Protocol(ProtocolRecord::ControlReply)
    );
}

#[test]
fn a_4097_byte_reply_is_preceded_by_a_heartbeat() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.now.set(400);
    // Act
    writer
        .control(
            correlation(1, 1),
            &payload(4097),
            |_| true,
            &view,
            &mut sink,
        )
        .expect("reply");
    // Assert
    assert_eq!(
        sink.records[1].1,
        SerialOutputPurpose::Protocol(ProtocolRecord::Heartbeat)
    );
    assert_eq!(writer.maybe_next_heartbeat_ms(), Some(1400));
}

#[test]
fn transport_failure_is_not_traced_as_rejected() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    sink.maybe_fail_write = Some(2);
    // Act
    let result = writer.control(correlation(1, 1), b"{}", |_| true, &view, &mut sink);
    // Assert
    assert!(result.is_err());
    assert!(view.lines("trace WriterRejected").is_empty());
    assert_eq!(view.lines("revoke"), ["revoke 1"]);
}

#[test]
fn invalid_reply_json_is_rejected_after_consuming_a_sequence() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    let result = writer.control(correlation(1, 1), b"not json", |_| true, &view, &mut sink);
    view.begin(1);
    writer
        .control(correlation(1, 2), b"{}", |_| true, &view, &mut sink)
        .expect("reply");
    // Assert
    assert!(matches!(result, Err(SerialWriteError::Payload(_))));
    assert_eq!(
        view.lines("trace WriterRejected"),
        ["trace WriterRejected 1 1"]
    );
    assert_eq!(sink.sequences(), [0, 2]);
}

#[test]
fn unheard_receipt_revokes_after_a_complete_write() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .control(correlation(1, 1), b"{}", |_| false, &view, &mut sink)
        .expect("written");
    // Assert
    assert_eq!(sink.records.len(), 2);
    assert_eq!(view.lines("revoke"), ["revoke 1"]);
}

#[test]
fn revoked_epoch_consumes_no_sequence() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.current.set(0);
    view.begin(1);
    // Act
    view.current.set(0);
    let failed = writer.heartbeat_if_due(1000, &view, &mut sink);
    view.current.set(1);
    writer
        .heartbeat_if_due(2000, &view, &mut sink)
        .expect("heartbeat");
    // Assert
    assert!(failed.is_ok());
    assert_eq!(sink.sequences(), [0, 1]);
}

#[test]
fn exhausted_sequence_writes_nothing() {
    // Arrange
    let view = View::default();
    view.begin(1);
    let mut sink = Sink::default();
    let mut writer = SerialRecordWriter::new();
    writer
        .hello(1, SESSION.into(), &json!({}), &view, &mut sink)
        .expect("current")
        .expect("ack");
    let mut writer = writer.with_sequence(u32::MAX);
    // Act
    let result = writer.heartbeat_if_due(1000, &view, &mut sink);
    // Assert
    assert!(matches!(result, Err(SerialWriteError::SequenceExhausted)));
    assert_eq!(sink.records.len(), 1);
}

#[test]
fn bootstrap_diagnostic_is_a_plain_line() {
    // Arrange
    let view = View::default();
    let mut sink = Sink::default();
    let mut writer = SerialRecordWriter::new();
    // Act
    writer
        .diagnostic_line("boot ready", 3, false, &view, &mut sink)
        .expect("line");
    // Assert
    assert_eq!(sink.records[0].0, b"boot ready\n");
    assert_eq!(
        sink.records[0].1,
        SerialOutputPurpose::Diagnostic(DiagnosticOrigin::Bootstrap, 3)
    );
}

#[test]
fn failed_plain_line_counts_a_dropped_diagnostic() {
    // Arrange
    let view = View::default();
    let mut sink = Sink {
        maybe_fail_write: Some(1),
        ..Sink::default()
    };
    // Act
    let result = SerialRecordWriter::new().diagnostic_line("x", 0, false, &view, &mut sink);
    // Assert
    assert!(result.is_err());
    assert_eq!(view.dropped.get(), 1);
}

#[test]
fn session_diagnostic_is_a_framed_line_record() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .diagnostic_line("invalid_frame", 1, false, &view, &mut sink)
        .expect("record");
    // Assert
    let envelope = &sink.envelopes()[1];
    assert_eq!(envelope.kind, SerialKind::Diagnostic);
    assert_eq!(envelope.payload.get(), r#"{"line":"invalid_frame"}"#);
}

#[test]
fn replayed_diagnostic_keeps_its_replay_origin() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    // Act
    writer
        .diagnostic_line("marker", 2, true, &view, &mut sink)
        .expect("record");
    // Assert
    assert_eq!(
        sink.records[1].1,
        SerialOutputPurpose::Diagnostic(DiagnosticOrigin::Replay, 2)
    );
}

#[test]
fn a_revoked_session_diagnostic_falls_back_to_plain_text() {
    // Arrange
    let (mut writer, view, mut sink) = open();
    view.current.set(0);
    // Act
    writer
        .diagnostic_line("late", 0, false, &view, &mut sink)
        .expect("line");
    // Assert
    assert_eq!(sink.records[1].0, b"late\n");
}
