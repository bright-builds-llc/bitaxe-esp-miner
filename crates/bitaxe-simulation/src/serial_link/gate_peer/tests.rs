use super::*;
use bitaxe_worker_control::serial::link::hello_ack_payload;
use bitaxe_worker_control::serial::SerialSessionBinding;
use serde_json::value::RawValue;

const SESSION: &str = "AAAAAAAAAAAAAAAAAAAAAA";
const NONCE: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn record(kind: SerialKind, maybe_session: Option<&str>, sequence: u32, payload: &str) -> Vec<u8> {
    let raw = RawValue::from_string(payload.to_owned()).expect("payload");
    SerialEnvelope::encode(kind, maybe_session, sequence, &raw).expect("record")
}

fn ack(host_nonce: &str) -> Vec<u8> {
    let binding = SerialSessionBinding::parse(SESSION, host_nonce, NONCE).expect("binding");
    let payload = hello_ack_payload(&binding, &"a".repeat(40), "b".repeat(64)).to_string();
    record(SerialKind::Session, Some(SESSION), 0, &payload)
}

fn credit(sequence: u32, received: u32) -> Vec<u8> {
    let payload = format!(r#"{{"op":"receive_credit","receivedBytes":{received}}}"#);
    record(SerialKind::Session, Some(SESSION), sequence, &payload)
}

fn heartbeat(sequence: u32) -> Vec<u8> {
    record(SerialKind::Heartbeat, Some(SESSION), sequence, "{}")
}

fn admitted() -> GatePeer {
    let mut peer = GatePeer::default();
    peer.hello_sent(NONCE, 0);
    peer.device_bytes(&ack(NONCE), 10);
    peer
}

fn detail(peer: &GatePeer) -> String {
    peer.check().detail
}

#[test]
fn a_valid_ack_admits_the_session() {
    // Arrange / Act
    let peer = admitted();
    // Assert
    assert_eq!(peer.check().status, CheckStatus::Passed);
}

#[test]
fn an_ack_for_another_hello_is_rejected() {
    // Arrange
    let mut peer = GatePeer::default();
    peer.hello_sent(NONCE, 0);
    // Act
    peer.device_bytes(&ack("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"), 10);
    // Assert
    assert_eq!(detail(&peer), "hello_ack");
}

#[test]
fn a_late_ack_is_rejected() {
    // Arrange
    let mut peer = GatePeer::default();
    peer.hello_sent(NONCE, 0);
    // Act
    peer.device_bytes(&ack(NONCE), HEARTBEAT_TIMEOUT_MILLISECONDS);
    // Assert
    assert_eq!(detail(&peer), "hello_ack");
}

#[test]
fn a_repeated_sequence_breaks_continuity() {
    // Arrange
    let mut peer = admitted();
    // Act
    peer.device_bytes(&[heartbeat(1), heartbeat(1)].concat(), 500);
    // Assert
    assert_eq!(detail(&peer), "continuity");
}

#[test]
fn device_silence_past_the_deadline_breaks_continuity() {
    // Arrange
    let mut peer = admitted();
    peer.device_bytes(&heartbeat(1), 1000);
    // Act
    peer.settle(1000 + HEARTBEAT_TIMEOUT_MILLISECONDS);
    // Assert
    assert_eq!(detail(&peer), "continuity");
}

#[test]
fn credit_beyond_the_bytes_sent_is_invalid() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, None, false, 20);
    // Act
    peer.device_bytes(&credit(1, 101), 30);
    // Assert
    assert_eq!(detail(&peer), "credit_invalid");
}

#[test]
fn credit_that_does_not_advance_is_invalid() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, None, false, 20);
    peer.device_bytes(&credit(1, 100), 30);
    // Act
    peer.device_bytes(&credit(2, 100), 40);
    // Assert
    assert_eq!(detail(&peer), "credit_invalid");
}

#[test]
fn sending_before_full_credit_stalls() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, None, false, 20);
    peer.device_bytes(&credit(1, 60), 30);
    // Act
    peer.host_record(50, None, false, 40);
    // Assert
    assert_eq!(detail(&peer), "credit_stalled");
}

#[test]
fn close_requires_a_terminal_credit_within_two_seconds() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, None, true, 20);
    // Act
    peer.settle(20 + TERMINAL_CREDIT_MILLISECONDS + 1);
    // Assert
    assert_eq!(detail(&peer), "terminal_credit");
}

#[test]
fn a_terminal_credit_covering_close_completes_the_session() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, None, true, 20);
    // Act
    peer.device_bytes(&credit(1, 100), 30);
    // Assert
    assert!(peer.terminal_credited());
    assert_eq!(peer.check().status, CheckStatus::Passed);
}

#[test]
fn a_reply_for_another_request_is_uncorrelated() {
    // Arrange
    let mut peer = admitted();
    peer.host_record(100, Some("serial_one".into()), false, 20);
    peer.device_bytes(&credit(1, 100), 30);
    // Act
    peer.device_bytes(
        &record(
            SerialKind::Control,
            Some(SESSION),
            2,
            r#"{"requestId":"serial_two"}"#,
        ),
        40,
    );
    // Assert
    assert_eq!(detail(&peer), "correlation");
}

#[test]
fn a_diagnostic_with_extra_fields_is_rejected() {
    // Arrange
    let mut peer = admitted();
    // Act
    peer.device_bytes(
        &record(
            SerialKind::Diagnostic,
            Some(SESSION),
            1,
            r#"{"line":"x","more":1}"#,
        ),
        40,
    );
    // Assert
    assert_eq!(detail(&peer), "fields");
}

#[test]
fn a_malformed_line_is_an_envelope_violation() {
    // Arrange
    let mut peer = admitted();
    // Act
    peer.device_bytes(b"{\"not\":\"an envelope\"}\n", 40);
    // Assert
    assert_eq!(detail(&peer), "envelope");
}
