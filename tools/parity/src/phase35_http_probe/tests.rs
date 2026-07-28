use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use super::*;

fn read_request(stream: &mut impl Read) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 256];
    while !request.ends_with(b"\r\n\r\n") {
        let read = stream
            .read(&mut buffer)
            .expect("loopback request read failed");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
    }
    request
}

#[test]
fn silent_peer_proves_send_completion_before_response_timeout() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind failed");
    let address = listener.local_addr().expect("loopback address unavailable");
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback accept failed");
        let request = read_request(&mut stream);
        thread::sleep(Duration::from_millis(250));
        request
    });

    // Act
    let result = probe_phase35_http_with_timeouts(
        &format!("http://{address}"),
        Duration::from_secs(1),
        Duration::from_millis(100),
    )
    .expect("loopback probe failed");
    let request = peer.join().expect("loopback peer panicked");

    // Assert
    assert!(request.starts_with(b"GET /api/system/info HTTP/1.1\r\n"));
    assert!(request.ends_with(b"\r\n\r\n"));
    assert_eq!(
        result.metrics.transport_outcome,
        TransportOutcome::ResponseTimeout
    );
    assert_eq!(result.metrics.scheme_category, SchemeCategory::Http);
    assert!(result.metrics.tcp_connect_millis > 0);
    assert_eq!(result.metrics.tls_handshake_millis, 0);
    assert!(result.metrics.request_send_complete_millis > 0);
    assert_eq!(result.metrics.request_bytes, request.len() as u64);
    assert_eq!(result.metrics.response_status, 0);
    assert_eq!(result.metrics.response_header_count, 0);
    assert_eq!(result.metrics.response_header_bytes, 0);
    assert_eq!(result.metrics.response_body_bytes, 0);
    assert_eq!(result.metrics.first_byte_millis, 0);
    assert_eq!(
        result.metrics.tls_verification,
        TlsVerification::NotApplicable
    );
    assert!(result.headers.is_empty());
    assert!(result.body.is_empty());
}

#[test]
fn valid_response_is_ready_for_classification() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind failed");
    let address = listener.local_addr().expect("loopback address unavailable");
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback accept failed");
        let request = read_request(&mut stream);
        let body = br#"{"hostname":"loopback-fixture"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("loopback response header write failed");
        stream
            .write_all(body)
            .expect("loopback response body write failed");
        request
    });

    // Act
    let result = probe_phase35_http_with_timeouts(
        &format!("http://{address}"),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("loopback probe failed");
    let request = peer.join().expect("loopback peer panicked");
    let metrics = serde_json::to_vec(&result.metrics).expect("metrics encoding failed");
    let classified = crate::phase35_http::classify_phase35_http(&metrics, &result.body)
        .expect("classification failed");

    // Assert
    assert!(request.ends_with(b"\r\n\r\n"));
    assert_eq!(result.metrics.transport_outcome, TransportOutcome::Complete);
    assert_eq!(result.metrics.scheme_category, SchemeCategory::Http);
    assert!(result.metrics.tcp_connect_millis > 0);
    assert_eq!(result.metrics.tls_handshake_millis, 0);
    assert!(result.metrics.request_send_complete_millis > 0);
    assert_eq!(result.metrics.request_bytes, request.len() as u64);
    assert_eq!(result.metrics.response_status, 200);
    assert_eq!(result.metrics.response_header_count, 3);
    assert_eq!(
        result.metrics.response_header_bytes,
        u64::try_from(result.headers.len()).expect("header size fits u64")
    );
    assert_eq!(
        result.metrics.response_body_bytes,
        u64::try_from(result.body.len()).expect("body size fits u64")
    );
    assert!(result.metrics.first_byte_millis > 0);
    assert_eq!(
        result.metrics.tls_verification,
        TlsVerification::NotApplicable
    );
    assert_eq!(
        classified.terminal_category,
        crate::phase35_http::HttpTerminalCategory::Ready
    );
}

#[test]
fn tls_handshake_failure_never_claims_request_completion() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind failed");
    let address = listener.local_addr().expect("loopback address unavailable");
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback accept failed");
        stream
            .write_all(b"not-a-tls-record")
            .expect("loopback write failed");
    });

    // Act
    let result = probe_phase35_http_with_timeouts(
        &format!("https://{address}"),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("TLS failure should remain a typed probe result");
    peer.join().expect("loopback peer panicked");

    // Assert
    assert_eq!(
        result.metrics.transport_outcome,
        TransportOutcome::TlsHandshakeFailure
    );
    assert_eq!(result.metrics.request_send_complete_millis, 0);
    assert_eq!(result.metrics.request_bytes, 0);
    assert!(result.metrics.tcp_connect_millis > 0);
    assert_eq!(result.metrics.tls_handshake_millis, 0);
    assert_eq!(result.metrics.response_status, 0);
    assert_eq!(result.metrics.first_byte_millis, 0);
    assert_eq!(result.metrics.tls_verification, TlsVerification::Failed);
}

#[test]
fn timeout_after_headers_preserves_the_parseable_partial_response() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind failed");
    let address = listener.local_addr().expect("loopback address unavailable");
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback accept failed");
        let request = read_request(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\npart")
            .expect("partial response write failed");
        thread::sleep(Duration::from_millis(250));
        request
    });

    // Act
    let result = probe_phase35_http_with_timeouts(
        &format!("http://{address}"),
        Duration::from_secs(1),
        Duration::from_millis(100),
    )
    .expect("loopback probe failed");
    let request = peer.join().expect("loopback peer panicked");

    // Assert
    assert_eq!(
        result.metrics.transport_outcome,
        TransportOutcome::ResponseTimeout
    );
    assert_eq!(result.metrics.request_bytes, request.len() as u64);
    assert!(result.metrics.request_send_complete_millis > 0);
    assert_eq!(result.metrics.response_status, 200);
    assert_eq!(result.metrics.response_header_count, 1);
    assert_eq!(result.body, b"part");
    assert!(result.metrics.first_byte_millis > 0);
}

#[test]
fn malformed_response_projects_absent_http_fields() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind failed");
    let address = listener.local_addr().expect("loopback address unavailable");
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback accept failed");
        let request = read_request(&mut stream);
        stream
            .write_all(b"not-http")
            .expect("malformed response write failed");
        request
    });

    // Act
    let result = probe_phase35_http_with_timeouts(
        &format!("http://{address}"),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("loopback probe failed");
    let request = peer.join().expect("loopback peer panicked");

    // Assert
    assert_eq!(result.metrics.transport_outcome, TransportOutcome::Complete);
    assert_eq!(result.metrics.request_bytes, request.len() as u64);
    assert!(result.metrics.request_send_complete_millis > 0);
    assert_eq!(result.metrics.response_status, 0);
    assert_eq!(result.metrics.response_header_count, 0);
    assert_eq!(result.metrics.response_header_bytes, 0);
    assert_eq!(result.metrics.response_body_bytes, 0);
    assert!(result.metrics.first_byte_millis > 0);
    assert!(result.headers.is_empty());
    assert!(result.body.is_empty());
}
