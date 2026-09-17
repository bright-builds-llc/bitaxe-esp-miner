use super::super::*;
use bitaxe_stratum::v2::{
    messages::ChannelKind,
    noise::{
        diagnostic::{self, Event, Failure, Observer, Phase},
        NoiseTransport,
    },
    session::SessionConfig,
    standard::StandardEvent,
    standard_io::{self, ChannelObserver},
};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use zeroize::Zeroizing;
struct Driver {
    began: Instant,
    config: SessionConfig,
    maybe_failure: Option<Failure>,
    maybe_commitment: Option<String>,
    closed: bool,
}
impl Observer for Driver {
    fn permitted(&mut self) -> bool {
        self.began.elapsed() < Duration::from_secs(15)
    }
    fn now_us(&self) -> Option<u64> {
        self.began.elapsed().as_micros().try_into().ok()
    }
    fn event(&mut self, event: Event) {
        if matches!(
            event,
            Event::Complete {
                phase: Phase::Close,
                ..
            }
        ) {
            self.closed = true;
        }
    }
    fn failed(&mut self, _: Phase, failure: Failure) {
        self.maybe_failure.get_or_insert(failure);
    }
    fn authenticated(
        &mut self,
        stream: &mut TcpStream,
        noise: &mut NoiseTransport,
    ) -> std::result::Result<(), Failure> {
        standard_io::run_channel(stream, noise, self.config.clone(), self)
    }
}
impl ChannelObserver for Driver {
    fn protocol_event(&mut self, event: &StandardEvent) -> std::result::Result<(), Failure> {
        if let StandardEvent::Work { commitment, .. } = event {
            self.maybe_commitment = Some(commitment.clone());
        }
        Ok(())
    }
}
fn config(address: SocketAddr) -> SessionConfig {
    SessionConfig {
        endpoint_host: address.ip().to_string(),
        endpoint_port: address.port(),
        vendor: "synthetic".into(),
        hardware_version: "205".into(),
        firmware: "synthetic".into(),
        device_id: "synthetic".into(),
        user_identity: "synthetic-only".into(),
        nominal_hashrate: 1.0,
        channel_kind: ChannelKind::Standard,
        minimum_extranonce_size: 0,
    }
}
fn input() -> Input {
    Input {
        schema: "str005-v2-fixture-input-v1".into(),
        scope: input::Scope::Channel,
        attempt_id: "AQEBAQEBAQEBAQEBAQEBAQ".into(),
        listen_ipv4: Zeroizing::new("127.0.0.1".into()),
        expected_peer_ipv4: Zeroizing::new("127.0.0.1".into()),
        user_identity: Zeroizing::new("synthetic-only".into()),
    }
}
fn production_exchange(
    wrong_authority: bool,
) -> (std::path::PathBuf, model::Result<()>, Evidence, Driver) {
    // Arrange: only loopback software; no qualification or device authority exists here.
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let address = listener.local_addr().expect("address");
    let (secret, public) = generate_serial_authority().expect("fixture authority");
    let root = std::env::temp_dir().join(format!(
        "v2-serial-protocol-{}-{}",
        std::process::id(),
        nonce()
    ));
    DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .expect("private synthetic root");
    let server_root = root.clone();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test client");
        stream.set_nonblocking(true).expect("nonblocking");
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut act_one = [0; 64];
        io::exact(&mut stream, &mut act_one, deadline, "setup_received").expect("actual act one");
        let mut codec = session::authenticate(&mut stream, act_one, &secret, &public, deadline)
            .expect("real responder");
        let mut evidence = Evidence::new(nonce(), nonce(), Instant::now());
        let result = session::run(
            &mut stream,
            &mut codec,
            &input(),
            &server_root,
            deadline,
            &mut evidence,
        );
        (result, evidence)
    });
    let mut driver = Driver {
        began: Instant::now(),
        config: config(address),
        maybe_failure: None,
        maybe_commitment: None,
        closed: false,
    };
    // Act
    let configured = if wrong_authority {
        generate_serial_authority().expect("different authority").1
    } else {
        public
    };
    diagnostic::run(address, configured, &mut OsRng, &mut driver);
    let (result, evidence) = server.join().expect("fixture thread");
    (root, result, evidence, driver)
}
#[test]
fn actual_shared_production_driver_receives_fixture_job_and_cleanly_closes() {
    let (root, result, evidence, driver) = production_exchange(false);
    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(driver.maybe_failure.is_none(), "{:?}", driver.maybe_failure);
    assert!(driver.closed);
    assert!(evidence.terminal.peer_closed);
    assert_eq!(evidence.terminal.received_shares, 0);
    let job: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("job.json")).expect("job facts"))
            .expect("facts");
    assert_eq!(
        driver.maybe_commitment.as_deref(),
        job["jobCommitment"].as_str()
    );
    assert_eq!(
        evidence
            .events
            .events
            .iter()
            .map(|e| e.kind)
            .collect::<Vec<_>>(),
        [
            "setup_received",
            "channel_open_received",
            "channel_success_sent",
            "job_sent",
            "target_sent",
            "prev_hash_sent",
            "peer_eof"
        ]
    );
    std::fs::remove_dir_all(root).expect("remove synthetic root");
}
#[test]
fn already_expired_read_and_write_never_accept_available_data() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let mut peer = TcpStream::connect(listener.local_addr().expect("address")).expect("connect");
    let (mut stream, _) = listener.accept().expect("accept");
    stream.set_nonblocking(true).expect("nonblocking");
    peer.write_all(b"x").expect("queued byte");
    let expired = Instant::now() - Duration::from_millis(1);
    let mut byte = [0];
    assert_eq!(
        io::exact(&mut stream, &mut byte, expired, "setup_received")
            .expect_err("expired")
            .category,
        "timeout"
    );
    assert_eq!(
        io::write(&mut stream, b"x", expired, "setup_received")
            .expect_err("expired")
            .category,
        "timeout"
    );
    peer.set_read_timeout(Some(Duration::from_millis(10)))
        .expect("timeout");
    assert!(peer.read(&mut byte).is_err());
}

#[test]
fn actual_shared_production_driver_rejects_wrong_authority_without_setup() {
    let (root, result, evidence, driver) = production_exchange(true);
    assert!(result.is_err());
    assert!(driver.maybe_failure.is_some());
    assert!(driver.maybe_commitment.is_none());
    assert!(evidence.events.events.is_empty());
    assert!(!root.join("job.json").exists());
    std::fs::remove_dir_all(root).expect("remove synthetic root");
}

fn codec_pair() -> (NoiseTransport, noise_sv2::NoiseCodec) {
    use bitaxe_stratum::v2::noise::NoiseInitiator;
    let (secret, public) = generate_serial_authority().expect("synthetic authority");
    let mut rng = OsRng;
    let mut initiator = NoiseInitiator::new(Some(public), &mut rng).expect("initiator");
    let act_one = initiator.act_one().expect("act one");
    let mut responder = noise_sv2::Responder::from_authority_kp_with_rng(
        &public,
        &secret,
        Duration::from_secs(u32::MAX.into()),
        &mut rng,
    )
    .expect("responder");
    let (act_two, codec) = responder
        .step_1_with_now_rng(act_one, 0, &mut rng)
        .expect("act two");
    (
        initiator
            .complete(&act_two, 100)
            .expect("configured authority"),
        codec,
    )
}
fn tcp_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let client = TcpStream::connect(listener.local_addr().expect("address")).expect("client");
    let (server, _) = listener.accept().expect("server");
    server.set_nonblocking(true).expect("nonblocking");
    (client, server)
}
#[test]
fn coalesced_encrypted_frames_are_separate_records_not_surplus() {
    use bitaxe_stratum::v2::frame::Frame;
    // Arrange
    let (mut client, mut server) = tcp_pair();
    let (mut encrypt, mut decrypt) = codec_pair();
    let one = Frame::new(0x8000, 0x21, vec![1, 2, 3]).expect("synthetic frame");
    let two = Frame::new(0x8000, 0x20, vec![4, 5, 6]).expect("synthetic frame");
    let mut bytes = encrypt.encrypt_frame(&one).expect("encrypt");
    bytes.extend(encrypt.encrypt_frame(&two).expect("encrypt"));
    // Act
    client.write_all(&bytes).expect("coalesced TCP write");
    drop(client);
    let deadline = Instant::now() + Duration::from_secs(1);
    let first = io::read_frame(&mut server, &mut decrypt, deadline, false, "job_sent")
        .expect("read")
        .expect("first");
    let second = io::read_frame(&mut server, &mut decrypt, deadline, false, "job_sent")
        .expect("read")
        .expect("second");
    // Assert
    assert_eq!(first.payload.as_slice(), one.payload());
    assert_eq!(second.payload.as_slice(), two.payload());
    assert!(
        io::read_frame(&mut server, &mut decrypt, deadline, false, "peer_eof")
            .expect("EOF")
            .is_none()
    );
}
#[test]
fn partial_encrypted_header_cannot_extend_absolute_lifetime() {
    let (mut client, mut server) = tcp_pair();
    let (_, mut decrypt) = codec_pair();
    client.write_all(&[1]).expect("first byte");
    let began = Instant::now();
    let error = io::read_frame(
        &mut server,
        &mut decrypt,
        began + Duration::from_millis(30),
        false,
        "share_received",
    )
    .err()
    .expect("partial frame timeout");
    assert_eq!(error.category, "timeout");
    assert!(began.elapsed() < Duration::from_millis(500));
}
