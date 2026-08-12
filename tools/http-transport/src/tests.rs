use super::*;

#[test]
fn fixed_command_helpers_issue_each_post_route_once() {
    use std::net::TcpListener;

    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let server = std::thread::spawn(move || {
        let mut request_lines = Vec::new();
        for _ in 0..4 {
            let (mut socket, _) = listener.accept().expect("accept command request");
            let mut bytes = Vec::new();
            let mut chunk = [0_u8; 512];
            loop {
                let count = socket.read(&mut chunk).expect("read command request");
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if maybe_header_end(&bytes).is_some() {
                    break;
                }
            }
            request_lines.push(
                String::from_utf8(bytes)
                    .expect("request is UTF-8")
                    .lines()
                    .next()
                    .expect("request line")
                    .to_owned(),
            );
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write response");
        }
        request_lines
    });
    let client = StrictHttpClient::new(&format!("http://{address}")).expect("client");

    // Act
    let deadline = || Instant::now() + Duration::from_secs(2);
    client.post_pause_once(deadline()).expect("pause request");
    client.post_resume_once(deadline()).expect("resume request");
    client
        .post_identify_once(deadline())
        .expect("identify request");
    client
        .post_block_found_dismiss_once(deadline())
        .expect("dismiss request");

    // Assert
    assert_eq!(
        server.join().expect("server thread"),
        [
            "POST /api/system/pause HTTP/1.1",
            "POST /api/system/resume HTTP/1.1",
            "POST /api/system/identify HTTP/1.1",
            "POST /api/system/blockFound/dismiss HTTP/1.1",
        ]
    );
}

#[test]
fn incomplete_write_is_never_complete() {
    struct FailingWriter {
        remaining: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"));
            }
            let count = self.remaining.min(buffer.len());
            self.remaining -= count;
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Arrange
    let mut writer = FailingWriter { remaining: 7 };

    // Act
    let observation = send_request(&mut writer, b"complete request");

    // Assert
    assert_eq!(
        observation,
        RequestSendOutcome::Incomplete { bytes_written: 7 }
    );
}

#[test]
fn binary_request_appends_the_exact_body_after_bounded_headers() {
    // Arrange
    let body = [0x00, 0xff, 0x3a, 0x0a];

    // Act
    let request = request_bytes(
        "POST",
        "/api/system/OTA",
        "device.invalid",
        "application/octet-stream",
        &body,
    );
    let header_end = maybe_header_end(&request).expect("request headers must terminate");

    // Assert
    assert!(request[..header_end]
        .windows(b"Content-Length: 4".len())
        .any(|window| window == b"Content-Length: 4"));
    assert_eq!(&request[header_end..], body);
}

#[test]
fn invalid_origins_fail_closed() {
    for origin in [
        "ftp://example.invalid",
        "http://",
        "http://user@example.invalid",
        "http://example.invalid/path",
        "http://::1",
    ] {
        assert!(StrictHttpClient::new(origin).is_err());
    }
}

#[test]
fn chunked_body_requires_terminal_chunk() {
    assert_eq!(
        maybe_decode_chunked(b"4\r\ntest\r\n0\r\n\r\n"),
        Some(b"test".to_vec())
    );
    assert_eq!(maybe_decode_chunked(b"4\r\ntest\r\n"), None);
}

#[test]
fn pre_request_failures_cannot_report_request_or_response_progress() {
    // Arrange
    let tcp_failure = ExchangeObservation::tcp_connection_failed(Scheme::Https, 3);
    let tls_failure = ExchangeObservation::tls_handshake_failed(nonzero_u64(2), 5);

    // Act
    let observations = [&tcp_failure, &tls_failure];

    // Assert
    for observation in observations {
        assert_eq!(observation.request_progress(), RequestProgress::NotStarted);
        assert!(observation.maybe_response_read().is_none());
        assert!(observation.maybe_http_response().is_none());
    }
    assert_eq!(
        tcp_failure.transport_outcome(),
        TransportOutcome::TcpConnectionFailure
    );
    assert_eq!(tcp_failure.tls_verification(), TlsVerification::Failed);
    assert_eq!(
        tls_failure.transport_outcome(),
        TransportOutcome::TlsHandshakeFailure
    );
    assert_eq!(tls_failure.maybe_tcp_connect_millis(), Some(nonzero_u64(2)));
}

#[test]
fn completed_request_carries_nonzero_progress() {
    // Arrange
    let transport = EstablishedTransport::plain(nonzero_u64(2));
    let request = CompletedRequest::new(nonzero_u64(17), nonzero_u64(4));
    let response = ResponseRead::new(ResponseReadOutcome::Complete, None, None);

    // Act
    let observation = ExchangeObservation::response_read(transport, request, response, 7);

    // Assert
    assert_eq!(
        observation.request_progress(),
        RequestProgress::Complete {
            bytes_written: nonzero_u64(17),
            completed_millis: nonzero_u64(4),
        }
    );
    assert_eq!(
        observation.tls_verification(),
        TlsVerification::NotApplicable
    );
    assert_eq!(observation.total_millis(), 7);
}

#[test]
fn response_failure_can_retain_a_validated_http_response() {
    // Arrange
    let transport = EstablishedTransport::tls(nonzero_u64(2), nonzero_u64(3));
    let request = CompletedRequest::new(nonzero_u64(17), nonzero_u64(6));
    let http_response = HttpResponse::maybe_new(
        503,
        1,
        b"HTTP/1.1 503 Service Unavailable\r\n\r\n".to_vec(),
        b"partial".to_vec(),
    )
    .expect("status is valid");
    let response = ResponseRead::new(
        ResponseReadOutcome::Timeout,
        Some(nonzero_u64(7)),
        Some(http_response),
    );

    // Act
    let observation = ExchangeObservation::response_read(transport, request, response, 9);
    let parsed = observation
        .maybe_http_response()
        .expect("partial response remains available");

    // Assert
    assert_eq!(
        observation.transport_outcome(),
        TransportOutcome::ResponseTimeout
    );
    assert_eq!(observation.tls_verification(), TlsVerification::Verified);
    assert_eq!(parsed.status(), 503);
    assert_eq!(parsed.body(), b"partial");
}

#[test]
fn invalid_http_status_cannot_construct_a_response() {
    // Arrange
    let invalid_statuses = [0, 99, 600, u16::MAX];

    // Act
    let responses =
        invalid_statuses.map(|status| HttpResponse::maybe_new(status, 0, vec![], vec![]));

    // Assert
    assert!(responses
        .into_iter()
        .all(|maybe_response| maybe_response.is_none()));
}
