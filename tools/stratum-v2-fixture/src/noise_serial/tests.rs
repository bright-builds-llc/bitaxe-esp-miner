use super::*;
use bitaxe_stratum::v2::{
    frame::Frame,
    noise::{NoiseInitiator, ACT_TWO_LEN},
};
use std::io::Write;
use std::net::TcpStream;

fn with_fixture(client: impl FnOnce(TcpStream, [u8; 32])) -> Terminal {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback fixture");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let (private, public) = generate_serial_authority().expect("authority");
    let worker = std::thread::spawn(move || {
        let mut receipt = Terminal::new(URL_SAFE_NO_PAD.encode([1; 16]));
        let limits = Limits {
            accept: Duration::from_secs(3),
            read: Duration::from_millis(900),
            lifetime: Duration::from_secs(5),
            eof: Duration::from_millis(100),
        };
        let result = exchange(
            listener,
            address.ip(),
            &private,
            &public,
            Instant::now(),
            limits,
            &mut receipt,
        );
        receipt.failure = result.err();
        receipt
    });
    let stream = TcpStream::connect(address).expect("connect fixture");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("read timeout");
    client(stream, public);
    worker.join().expect("fixture worker")
}

fn send_proof(stream: &mut TcpStream, public: [u8; 32]) {
    let mut rng = OsRng;
    let mut initiator = NoiseInitiator::new(Some(public), &mut rng).expect("initiator");
    stream
        .write_all(&initiator.act_one().expect("act one"))
        .expect("act one write");
    let mut act_two = [0; ACT_TWO_LEN];
    stream.read_exact(&mut act_two).expect("act two read");
    let mut transport = initiator
        .complete(&act_two, 100)
        .expect("authority verified");
    let proof = Frame::new(0xffff, 0xff, Vec::new()).expect("diagnostic frame");
    stream
        .write_all(&transport.encrypt_frame(&proof).expect("proof encrypt"))
        .expect("proof write");
}

#[test]
fn real_noise_exchange_requires_exact_encrypted_proof_and_peer_eof() {
    // Arrange / Act: both ends run production Noise implementations over TCP.
    let receipt = with_fixture(|mut stream, public| {
        send_proof(&mut stream, public);
        stream.shutdown(Shutdown::Write).expect("peer half-close");
    });
    // Assert
    assert!(receipt.failure.is_none(), "{:?}", receipt.failure);
    assert_eq!(receipt.expected_peer_connection_count, 1);
    assert_eq!(receipt.candidates[0].act_one_bytes, 64);
    assert_eq!(receipt.act_two_bytes_written, ACT_TWO_LEN);
    assert_eq!(receipt.proof_bytes_received, 22);
    assert!(receipt.encrypted_proof_exact && receipt.peer_closed);
}

#[test]
fn extra_byte_after_valid_proof_is_not_an_accepted_exchange() {
    let receipt = with_fixture(|mut stream, public| {
        send_proof(&mut stream, public);
        stream.write_all(&[1]).expect("extra byte");
    });
    assert_eq!(receipt.failure.expect("failure").detail, "extra");
    assert_eq!(receipt.extra_bytes_received, 1);
}

#[test]
fn valid_proof_without_peer_eof_expires() {
    let receipt = with_fixture(|mut stream, public| {
        send_proof(&mut stream, public);
        std::thread::sleep(Duration::from_millis(250));
    });
    assert_eq!(receipt.failure.expect("failure").detail, "timeout");
    assert!(receipt.encrypted_proof_exact);
    assert!(!receipt.peer_closed);
}

#[test]
fn fragmented_partial_act_one_cannot_extend_absolute_read_deadline() {
    let receipt = with_fixture(|mut stream, _| {
        stream.write_all(&[1; 16]).expect("partial act one");
        std::thread::sleep(Duration::from_millis(600));
        stream.write_all(&[2; 16]).expect("second partial act one");
        std::thread::sleep(Duration::from_millis(600));
    });
    assert_eq!(receipt.failure.expect("failure").detail, "timeout");
    assert_eq!(receipt.candidates[0].act_one_bytes, 32);
}

#[test]
fn full_act_one_followed_by_extra_data_is_rejected() {
    let receipt = with_fixture(|mut stream, _| {
        stream.write_all(&[1; 65]).expect("oversized act one");
        std::thread::sleep(Duration::from_millis(50));
    });
    assert_eq!(receipt.failure.expect("failure").detail, "extra");
    assert_eq!(receipt.candidates[0].act_one_bytes, 65);
}

#[test]
fn serial_options_are_closed_and_canonical_before_root_creation() {
    use clap::Parser;
    let args = [
        "fixture",
        "--mode",
        "noise-serial",
        "--private-root",
        "/missing/root",
        "--listen-address",
        "192.168.1.2:0",
        "--expected-peer-address",
        "192.168.1.3",
        "--attempt-id",
        "AQEBAQEBAQEBAQEBAQEBAQ",
        "--accept-timeout-seconds",
        "120",
        "--read-timeout-seconds",
        "10",
        "--lifetime-seconds",
        "150",
    ];
    assert!(Args::try_parse_from(args).is_ok());
    assert!(Args::try_parse_from(
        args.into_iter()
            .chain(["--attempt-id", "AQEBAQEBAQEBAQEBAQEBAQ"])
    )
    .is_err());
    assert!(
        Args::try_parse_from(args.into_iter().chain(["--pool-credentials", "forbidden"])).is_err()
    );
    let invalid = Args::try_parse_from(args.map(|v| {
        if v == "AQEBAQEBAQEBAQEBAQEBAQ" {
            "not-canonical"
        } else {
            v
        }
    }))
    .expect("parsed argv");
    assert!(validate(&invalid).is_err());
}

#[test]
fn terminal_schema_uses_only_closed_fields_and_no_authority_or_payload() {
    let receipt = Terminal::new(URL_SAFE_NO_PAD.encode([1; 16]));
    let object = serde_json::to_value(receipt).expect("serialized");
    let keys = object.as_object().expect("object");
    assert_eq!(keys.len(), 16);
    assert!(keys.contains_key("actTwoBytesWritten"));
    assert!(!keys.contains_key("authorityPublicKey"));
    assert!(!keys.contains_key("raw"));
}

#[test]
fn independent_watchdog_terminates_an_unresponsive_owned_process() {
    // Arrange / Act: a fresh test child deliberately does not reach completion.
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "noise_serial::tests::watchdog_child",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .expect("watchdog child");
    // Assert: no terminal-success receipt can be inferred from this process exit.
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("noise_serial_lifetime_expired"));
}

#[test]
#[ignore = "run only in a fresh child by independent_watchdog_terminates_an_unresponsive_owned_process"]
fn watchdog_child() {
    let _guard = lifetime_guard(Instant::now() + Duration::from_millis(30));
    std::thread::sleep(Duration::from_secs(1));
    panic!("watchdog did not terminate the child");
}

fn inventory_fixture(count: usize, expected: IpAddr) -> Result<Terminal, Cause> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let mut clients = Vec::new();
    for _ in 0..count {
        let mut stream = TcpStream::connect(address).expect("connect");
        stream.write_all(&[1; 64]).expect("complete act one");
        clients.push(stream);
    }
    let began = Instant::now();
    let mut receipt = Terminal::new(URL_SAFE_NO_PAD.encode([1; 16]));
    let result = inventory::select(
        &listener,
        expected,
        began + Duration::from_secs(1),
        began + Duration::from_secs(2),
        Duration::from_secs(1),
        &mut receipt,
    );
    receipt.failure = result.err();
    Ok(receipt)
}

#[test]
fn duplicate_expected_connections_fail_the_new_mode_inventory() {
    let receipt = inventory_fixture(2, "127.0.0.1".parse().expect("ip")).expect("inventory");
    assert_eq!(receipt.failure.expect("failure").detail, "peer_conflict");
    assert_eq!(receipt.expected_peer_connection_count, 2);
}

#[test]
fn fourth_expected_connection_records_overflow_without_unbounded_inventory() {
    let receipt = inventory_fixture(4, "127.0.0.1".parse().expect("ip")).expect("inventory");
    assert_eq!(receipt.failure.expect("failure").detail, "overflow");
    assert_eq!(receipt.expected_peer_connection_count, 4);
    assert_eq!(receipt.candidates.len(), 3);
    assert!(receipt.candidate_overflow);
}

#[test]
fn an_unexpected_peer_is_not_silently_discarded() {
    let receipt = inventory_fixture(1, "127.0.0.2".parse().expect("ip")).expect("inventory");
    assert_eq!(receipt.failure.expect("failure").detail, "peer_conflict");
    assert_eq!(receipt.unexpected_peer_count, 1);
}

#[test]
fn timely_selection_keeps_the_full_tail_beyond_the_read_deadline() {
    // Arrange: read completes before 400ms; the mandatory tail ends after it.
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let mut client = TcpStream::connect(address).expect("connect");
    let writer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        client.write_all(&[1; 64]).expect("act one");
        std::thread::sleep(Duration::from_millis(700));
    });
    let began = Instant::now();
    let mut receipt = Terminal::new(URL_SAFE_NO_PAD.encode([1; 16]));
    // Act
    let selected = inventory::select(
        &listener,
        address.ip(),
        began + Duration::from_secs(1),
        began + Duration::from_secs(2),
        Duration::from_millis(400),
        &mut receipt,
    );
    // Assert
    assert!(selected.is_ok(), "{:?}", selected.err());
    assert!(began.elapsed() >= Duration::from_millis(650));
    writer.join().expect("writer");
}

#[test]
fn a_queued_connection_after_accept_expiry_cannot_start_a_fresh_read_budget() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let mut client = TcpStream::connect(address).expect("connect");
    client.write_all(&[1; 64]).expect("act one");
    let now = Instant::now();
    let mut receipt = Terminal::new(URL_SAFE_NO_PAD.encode([1; 16]));
    let result = inventory::select(
        &listener,
        address.ip(),
        now - Duration::from_millis(1),
        now + Duration::from_secs(2),
        Duration::from_secs(1),
        &mut receipt,
    );
    assert_eq!(
        result.expect_err("late acceptance rejected").detail,
        "timeout"
    );
    assert!(receipt.selected_index.is_none());
}

#[test]
fn late_queued_bytes_cannot_satisfy_expired_io() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let mut client = TcpStream::connect(listener.local_addr().expect("address")).expect("connect");
    let (mut server, _) = listener.accept().expect("accept");
    server.set_nonblocking(true).expect("nonblocking");
    client.write_all(&[1; 22]).expect("queued data");
    let expired = Instant::now() - Duration::from_millis(1);
    let mut received = 0;
    let result = io::read_exact(
        &mut server,
        &mut [0; 22],
        expired,
        &mut received,
        "proof_received",
    );
    assert_eq!(result.expect_err("expired read").detail, "timeout");
    assert_eq!(received, 0);
    let mut written = 0;
    assert!(io::write_all(&mut server, &[1; 22], expired, &mut written).is_err());
    assert_eq!(written, 0);
}

#[test]
fn late_watchdog_thread_does_not_restart_its_absolute_budget() {
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "noise_serial::tests::late_watchdog_child",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .expect("late watchdog child");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("noise_serial_lifetime_expired"));
}

#[test]
#[ignore = "run only in a fresh child by late_watchdog_thread_does_not_restart_its_absolute_budget"]
fn late_watchdog_child() {
    // Scheduling cannot turn an already expired deadline into a fresh interval.
    let _guard = lifetime_guard(Instant::now() - Duration::from_secs(1));
    std::thread::sleep(Duration::from_millis(250));
    panic!("expired watchdog did not terminate the child");
}
