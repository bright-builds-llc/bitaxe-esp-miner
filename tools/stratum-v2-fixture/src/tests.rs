use super::*;
use bitaxe_stratum::v2::messages::{ChannelKind, ServerMessage};
use bitaxe_stratum::v2::noise::{NoiseInitiator, NoiseTransport, ACT_TWO_LEN};
use bitaxe_stratum::v2::session::{SessionConfig, SessionEvent, V2Session};

#[test]
fn real_tcp_fixture_completes_noise_channel_job_and_accepted_share() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let (authority_private, authority_public) = generate_authority_keypair().expect("keypair");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let started = Instant::now();
        let mut progress = FixtureProgress {
            listener_ready: true,
            connection_accepted: true,
            ..FixtureProgress::default()
        };
        let mut codec = respond_noise(
            &mut stream,
            authority_private,
            authority_public,
            &mut progress,
        )
        .expect("Noise");
        let result = run_pool_session(
            &mut stream,
            &mut codec,
            started,
            Duration::from_secs(10),
            &mut progress,
        )
        .expect("pool session");
        (result, progress)
    });
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    let mut rng = OsRng;
    let mut initiator = NoiseInitiator::new(Some(authority_public), &mut rng).expect("initiator");
    stream
        .write_all(&initiator.act_one().expect("act one"))
        .expect("write act one");
    let mut act_two = [0; ACT_TWO_LEN];
    stream.read_exact(&mut act_two).expect("read act two");
    let mut noise = initiator
        .complete(&act_two, u32::MAX)
        .expect("complete Noise");
    let mut session = V2Session::new(test_config()).expect("session");

    // Act
    send_session_event(
        &mut stream,
        &mut noise,
        session.start().expect("setup event"),
    );
    let open_events = handle_one_server_frame(&mut stream, &mut noise, &mut session);
    let open = open_events
        .into_iter()
        .find(|event| matches!(event, SessionEvent::Outbound(_)))
        .expect("open event");
    send_session_event(&mut stream, &mut noise, open);
    handle_one_server_frame(&mut stream, &mut noise, &mut session);
    handle_one_server_frame(&mut stream, &mut noise, &mut session);
    let work_events = handle_one_server_frame(&mut stream, &mut noise, &mut session);
    let work = work_events
        .into_iter()
        .find_map(|event| match event {
            SessionEvent::Work(work) => Some(work),
            _ => None,
        })
        .expect("work event");
    let submit = session
        .observe_nonce(Bm1366NonceResult {
            job_id: work.asic_job_id,
            nonce: 1,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            version_bits: 0,
        })
        .expect("nonce")
        .expect("submit event");
    send_session_event(&mut stream, &mut noise, submit);
    let accepted = handle_one_server_frame(&mut stream, &mut noise, &mut session);
    let (server_result, progress) = server.join().expect("server join");

    // Assert
    assert!(accepted
        .iter()
        .any(|event| matches!(event, SessionEvent::ShareAccepted { accepted_count: 1 })));
    assert_eq!(server_result.status, "accepted");
    assert!(server_result.share_target_valid);
    assert_eq!(
        fixture_terminal_category(&progress, FixtureMode::Pool),
        "accepted"
    );
    assert_eq!(FIXTURE_CERTIFICATE_VALIDITY.as_secs(), u64::from(u32::MAX));
}

#[test]
fn real_tcp_handshake_only_mode_proves_client_authentication() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let (authority_private, authority_public) = generate_authority_keypair().expect("keypair");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut progress = FixtureProgress {
            listener_ready: true,
            connection_accepted: true,
            ..FixtureProgress::default()
        };
        let mut codec = respond_noise(
            &mut stream,
            authority_private,
            authority_public,
            &mut progress,
        )
        .expect("Noise response");
        read_client_proof(&mut stream, &mut codec).expect("client proof");
        progress.client_authenticated = true;
        progress.noise_authenticated = true;
        progress
    });
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    let mut rng = OsRng;
    let mut initiator = NoiseInitiator::new(Some(authority_public), &mut rng).expect("initiator");
    stream
        .write_all(&initiator.act_one().expect("act one"))
        .expect("write act one");
    let mut act_two = [0; ACT_TWO_LEN];
    stream.read_exact(&mut act_two).expect("read act two");
    let mut noise = initiator
        .complete(&act_two, u32::MAX)
        .expect("complete Noise");
    let proof = Frame::new(0, 0, Vec::new()).expect("proof frame");

    // Act
    stream
        .write_all(&noise.encrypt_frame(&proof).expect("encrypt proof"))
        .expect("write proof");
    let progress = server.join().expect("server join");

    // Assert
    assert!(progress.act_one_received);
    assert!(progress.responder_created);
    assert!(progress.act_two_created);
    assert!(progress.act_two_sent);
    assert!(progress.client_authenticated);
    assert_eq!(
        fixture_terminal_category(&progress, FixtureMode::HandshakeOnly),
        "accepted"
    );
}

#[test]
fn fixture_peer_admission_rejects_an_unexpected_address() {
    // Arrange
    let expected: IpAddr = "192.0.2.1".parse().expect("expected peer");
    let observed: SocketAddr = "192.0.2.2:1234".parse().expect("observed peer");

    // Act
    let allowed = Some(expected).is_none_or(|candidate| observed.ip() == candidate);

    // Assert
    assert!(!allowed);
}

#[test]
fn act_one_reader_distinguishes_partial_eof_from_timeout() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).expect("connect");
        stream.write_all(&[0x11; 7]).expect("partial act one");
    });
    let (mut stream, _) = listener.accept().expect("accept");
    let mut progress = FixtureProgress::default();

    // Act
    let result = read_act_one(&mut stream, &mut progress);
    client.join().expect("client join");

    // Assert
    assert!(result.is_err());
    assert_eq!(progress.act_one_bytes_received, 7);
    assert_eq!(progress.act_one_read_category, "eof");
}

fn test_config() -> SessionConfig {
    SessionConfig {
        endpoint_host: "127.0.0.1".to_owned(),
        endpoint_port: 1,
        vendor: "test".to_owned(),
        hardware_version: "BM1366".to_owned(),
        firmware: String::new(),
        device_id: String::new(),
        user_identity: "worker".to_owned(),
        nominal_hashrate: 1.0e12,
        channel_kind: ChannelKind::Standard,
        minimum_extranonce_size: 6,
    }
}

fn handle_one_server_frame(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    session: &mut V2Session,
) -> Vec<SessionEvent> {
    let frame = read_noise_frame(stream, noise);
    let message = ServerMessage::decode(&frame).expect("server message");
    session.handle(message).expect("handle server message")
}

fn send_session_event(stream: &mut TcpStream, noise: &mut NoiseTransport, event: SessionEvent) {
    let SessionEvent::Outbound(frame) = event else {
        panic!("expected outbound event");
    };
    let encrypted = noise.encrypt_frame(&frame).expect("encrypt frame");
    stream.write_all(&encrypted).expect("write frame");
}

fn read_noise_frame(stream: &mut TcpStream, noise: &mut NoiseTransport) -> Frame {
    let mut header = vec![0; ENCRYPTED_HEADER_LEN];
    stream.read_exact(&mut header).expect("read header");
    let pending = noise.decrypt_header(&header).expect("decrypt header");
    let mut payload = vec![0; pending.encrypted_payload_len()];
    stream.read_exact(&mut payload).expect("read payload");
    noise
        .decrypt_payload(pending, &payload)
        .expect("decrypt payload")
}
