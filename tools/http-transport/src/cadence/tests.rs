use super::*;
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use tungstenite::{accept_hdr, Message};

fn handoff() -> Vec<u8> {
    br#"{"schema":"cpu0-cadence-observer-input-v1","ipv4":"192.168.1.2","port":80,"observedUnixMs":10000,"expiresUnixMs":15000,"bindingVerified":true}"#.to_vec()
}

#[test]
fn handoff_requires_fresh_verified_private_literal_endpoint() {
    // Arrange
    let valid = String::from_utf8(handoff()).expect("ASCII fixture");
    // Act / Assert
    assert!(origin(valid.as_bytes(), 11000).is_ok());
    for invalid in [
        valid.replace("192.168.1.2", "127.0.0.1"),
        valid.replace("192.168.1.2", "example.test"),
        valid.replace("true", "false"),
        valid.replace("15000", "16000"),
        valid.replace("80", "0"),
    ] {
        assert_eq!(
            origin(invalid.as_bytes(), 11000).err(),
            Some("invalid_input")
        );
    }
    assert!(origin(valid.as_bytes(), 9999).is_err());
    assert!(origin(valid.as_bytes(), 15000).is_err());
}

#[test]
fn envelope_rejects_unknown_duplicate_or_wrong_shape_without_retaining_values() {
    // Arrange
    let good =
        br#"{"event":"update","data":{"private":"sensitive-fixture","nested":[1,true,null]}}"#;
    let bad: &[&[u8]] = &[
        br#"{"event":"other","data":{}}"#,
        br#"{"event":"update","data":[]}"#,
        br#"{"event":"update","data":{},"secret":"private"}"#,
        br#"{"event":"update","event":"update","data":{}}"#,
        br#"{"event":"update","data":{"unfinished":}}"#,
    ];
    // Act / Assert
    assert!(validate_frame(good).is_ok());
    for bytes in bad {
        assert_eq!(validate_frame(bytes), Err("invalid_frame"));
    }
    assert_eq!(validate_frame(&vec![b'x'; 65537]), Err("invalid_frame"));
}

#[test]
fn real_socket_observer_emits_metadata_only_and_closes_on_stop() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let (sender, receiver) = mpsc::channel();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .expect("timeout");
        let mut ws = accept_hdr(
            stream,
            |request: &tungstenite::handshake::server::Request, response| {
                assert_eq!(request.uri().path(), "/api/ws/live");
                assert!(!request.headers().contains_key("origin"));
                Ok(response)
            },
        )
        .expect("handshake");
        ws.send(Message::Text(
            r#"{"event":"update","data":{"private":"sensitive-fixture"}}"#.into(),
        ))
        .expect("send");
        sender.send(()).expect("notify");
        assert!(matches!(ws.read(), Ok(Message::Close(_))));
    });
    let mut socket = PlainWebSocket::connect(
        &format!("http://{address}"),
        "/api/ws/live",
        Duration::from_secs(1),
        READ_TIMEOUT,
    )
    .expect("connect");
    receiver.recv().expect("message sent");
    let mut output = Vec::new();
    let mut receipts = Receipts::default();
    let mut polls = 0;
    // Act
    let outcome = observe(
        &mut socket,
        &mut output,
        Instant::now(),
        Duration::from_secs(2),
        || {
            polls += 1;
            Ok(polls > 1)
        },
        &mut receipts,
    );
    socket.close();
    server.join().expect("server completed");
    // Assert
    assert_eq!(outcome, Ok("requested"));
    assert_eq!(receipts.messages, 1);
    let text = String::from_utf8(output).expect("JSON output");
    assert!(text.contains("connected") && text.contains("arrival"));
    assert!(!text.contains("sensitive-fixture") && !text.contains("127.0.0.1"));
}

#[test]
fn stdin_deadline_and_eof_are_bounded_without_reader_threads() {
    // Arrange
    use std::os::fd::AsRawFd;
    let (reader, writer) = std::os::unix::net::UnixStream::pair().expect("pipe fixture");
    let mut input = input::PrivateInput::new(reader.as_raw_fd());
    // Act / Assert
    assert_eq!(
        input.initial_line(Instant::now()).err(),
        Some("input_timeout")
    );
    assert!(!input.stop_requested().expect("nonblocking poll"));
    drop(writer);
    assert!(input.stop_requested().expect("EOF stop"));
}

#[test]
fn stdin_rejects_extra_stop_fields_and_bounds_private_handoff() {
    // Arrange
    use std::os::fd::AsRawFd;
    let (reader, mut writer) = std::os::unix::net::UnixStream::pair().expect("pipe fixture");
    writer
        .write_all(b"{\"op\":\"stop\",\"other\":1}\n")
        .expect("write");
    let mut input = input::PrivateInput::new(reader.as_raw_fd());
    // Act / Assert
    assert_eq!(input.stop_requested(), Err("invalid_stop"));
    let (reader, mut writer) = std::os::unix::net::UnixStream::pair().expect("pipe fixture");
    writer.write_all(&vec![b'x'; 1025]).expect("write");
    let mut input = input::PrivateInput::new(reader.as_raw_fd());
    assert_eq!(
        input
            .initial_line(Instant::now() + Duration::from_secs(1))
            .err(),
        Some("input_bound")
    );
}

#[test]
fn real_socket_rejects_private_malformed_frame_with_closed_error() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let mut ws = tungstenite::accept(stream).expect("handshake");
        ws.send(Message::Text(
            r#"{"event":"private-fixture","data":{}}"#.into(),
        ))
        .expect("send");
    });
    let mut socket = PlainWebSocket::connect(
        &format!("http://{address}"),
        "/api/ws/live",
        Duration::from_secs(1),
        READ_TIMEOUT,
    )
    .expect("connect");
    let mut output = Vec::new();
    let mut receipts = Receipts::default();
    // Act
    let outcome = observe(
        &mut socket,
        &mut output,
        Instant::now(),
        Duration::from_secs(1),
        || Ok(false),
        &mut receipts,
    );
    socket.close();
    server.join().expect("server completed");
    // Assert
    assert_eq!(outcome, Err("invalid_frame"));
    assert_eq!(receipts.messages, 0);
    assert!(!String::from_utf8(output)
        .expect("output")
        .contains("private-fixture"));
}

#[test]
fn observation_deadline_fails_without_another_read() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout");
        let mut ws = tungstenite::accept(stream).expect("handshake");
        assert!(matches!(ws.read(), Ok(Message::Close(_))));
    });
    let mut socket = PlainWebSocket::connect(
        &format!("http://{address}"),
        "/api/ws/live",
        Duration::from_secs(1),
        READ_TIMEOUT,
    )
    .expect("connect");
    // Act
    let outcome = observe(
        &mut socket,
        &mut Vec::new(),
        Instant::now(),
        Duration::ZERO,
        || Ok(false),
        &mut Receipts::default(),
    );
    socket.close();
    server.join().expect("server completed");
    // Assert
    assert_eq!(outcome, Err("deadline"));
}
