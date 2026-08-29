use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::mpsc,
};

use tempfile::tempdir;

use super::*;

fn credentials(port: u16) -> PoolCredentials {
    PoolCredentials {
        host: "127.0.0.1".to_owned(),
        port,
        username: "private-owner.worker".to_owned(),
        password: "private-password".to_owned(),
    }
}

fn loopback_addresses(port: u16) -> [SocketAddr; 1] {
    [SocketAddr::from(([127, 0, 0, 1], port))]
}

fn spawn_pool(
    authorize: bool,
    malformed: bool,
) -> (u16, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback pool should bind");
    let port = listener
        .local_addr()
        .expect("loopback address should resolve")
        .port();
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback pool should accept");
        let reader_stream = stream.try_clone().expect("loopback stream should clone");
        let mut reader = BufReader::new(reader_stream);
        let mut methods = Vec::new();
        for _ in 0..3 {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("loopback request should read");
            let value: serde_json::Value =
                serde_json::from_str(&line).expect("request should be JSON");
            let method = value["method"]
                .as_str()
                .expect("request method should be text")
                .to_owned();
            methods.push(method.clone());
            let response = match method.as_str() {
                "mining.configure" => {
                    if malformed {
                        "not-json\n".to_owned()
                    } else {
                        "{\"id\":1,\"result\":{\"version-rolling\":true,\"version-rolling.mask\":\"1fffe000\"},\"error\":null}\n".to_owned()
                    }
                }
                "mining.subscribe" => {
                    "{\"id\":2,\"result\":[[],\"01020304\",4],\"error\":null}\n".to_owned()
                }
                "mining.authorize" => {
                    format!("{{\"id\":3,\"result\":{authorize},\"error\":null}}\n")
                }
                _ => String::new(),
            };
            stream
                .write_all(response.as_bytes())
                .expect("loopback response should write");
            if malformed {
                break;
            }
        }
        sender.send(methods).expect("methods should return");
    });
    (port, receiver, handle)
}

fn spawn_nonresponsive_pool() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback pool should bind");
    let port = listener
        .local_addr()
        .expect("loopback address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("loopback pool should accept");
        thread::sleep(Duration::from_millis(300));
    });
    (port, handle)
}

fn spawn_oversized_pool() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback pool should bind");
    let port = listener
        .local_addr()
        .expect("loopback address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("loopback pool should accept");
        let reader_stream = stream.try_clone().expect("loopback stream should clone");
        let mut reader = BufReader::new(reader_stream);
        for _ in 0..2 {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("loopback request should read");
        }
        stream
            .write_all(&vec![b'x'; MAX_SERVER_BYTES + 1])
            .expect("oversized response should write");
    });
    (port, handle)
}

#[test]
fn real_loopback_session_configures_subscribes_and_authorizes_without_submit() {
    // Arrange
    let (port, receiver, handle) = spawn_pool(true, false);

    // Act
    let progress = probe_session(
        &credentials(port),
        Duration::from_secs(2),
        &loopback_addresses(port),
    )
    .expect("loopback readiness should pass");
    let methods = receiver.recv().expect("methods should be available");
    handle.join().expect("loopback pool should finish");

    // Assert
    assert!(progress.configure && progress.subscribe && progress.authorize);
    assert_eq!(
        methods,
        ["mining.configure", "mining.subscribe", "mining.authorize"]
    );
    assert!(!methods.iter().any(|method| method == "mining.submit"));
}

#[test]
fn authorize_rejection_is_closed_and_secret_free() {
    // Arrange
    let (port, receiver, handle) = spawn_pool(false, false);

    // Act
    let error = probe_session(
        &credentials(port),
        Duration::from_secs(2),
        &loopback_addresses(port),
    )
    .expect_err("authorize rejection should fail");
    let _ = receiver.recv().expect("methods should be available");
    handle.join().expect("loopback pool should finish");
    let rendered = format!("{error:?} {error}");

    // Assert
    assert_eq!(error.category(), ReadinessCategory::AuthorizeRejected);
    assert!(!rendered.contains("private-owner"));
    assert!(!rendered.contains("private-password"));
}

#[test]
fn malformed_pool_response_fails_protocol_admission() {
    // Arrange
    let (port, receiver, handle) = spawn_pool(true, true);

    // Act
    let error = probe_session(
        &credentials(port),
        Duration::from_secs(2),
        &loopback_addresses(port),
    )
    .expect_err("malformed response should fail");
    let _ = receiver.recv().expect("methods should be available");
    handle.join().expect("loopback pool should finish");

    // Assert
    assert_eq!(error.category(), ReadinessCategory::ProtocolInvalid);
}

#[test]
fn nonresponsive_pool_respects_the_sample_timeout() {
    // Arrange
    let (port, handle) = spawn_nonresponsive_pool();

    // Act
    let error = probe_session(
        &credentials(port),
        Duration::from_millis(100),
        &loopback_addresses(port),
    )
    .expect_err("nonresponsive pool should time out");
    handle.join().expect("loopback pool should finish");

    // Assert
    assert_eq!(error.category(), ReadinessCategory::Timeout);
}

#[test]
fn oversized_server_line_fails_before_unbounded_allocation() {
    // Arrange
    let (port, handle) = spawn_oversized_pool();

    // Act
    let error = probe_session(
        &credentials(port),
        Duration::from_secs(2),
        &loopback_addresses(port),
    )
    .expect_err("oversized pool response should fail");
    handle.join().expect("loopback pool should finish");

    // Assert
    assert_eq!(error.category(), ReadinessCategory::InputLimitExceeded);
}

#[test]
fn private_report_is_mode_0600_and_contains_only_closed_fields() {
    // Arrange
    let root = tempdir().expect("temporary root should create");
    let private = root.path().join("private");
    create_private_root(&private).expect("private root should create");
    let report = PoolReadinessReport {
        schema_version: REPORT_SCHEMA.to_owned(),
        attempt_ordinal: 5,
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        pool_config: "local-owner-supplied".to_owned(),
        pool_credentials_sha256: "c".repeat(64),
        private_lan_only: true,
        resolved_endpoints_sha256: "d".repeat(64),
        protocol: "stratum_v1_configure_subscribe_authorize".to_owned(),
        samples_required: 3,
        samples_completed: 3,
        ready_samples: 3,
        consecutive_ready: true,
        configure_succeeded: true,
        subscribe_succeeded: true,
        authorize_succeeded: true,
        shares_submitted: false,
        sample_timeout_seconds: 15,
        sample_delay_seconds: 2,
        max_server_bytes: MAX_SERVER_BYTES,
        max_server_messages: MAX_SERVER_MESSAGES,
        endpoint_redacted: true,
        credentials_redacted: true,
        bounded: true,
        terminal_category: ReadinessCategory::Ready,
    };

    // Act
    write_private_report(&private, &report).expect("private report should write");
    let path = private.join(REPORT_NAME);
    let document = fs::read_to_string(&path).expect("private report should read");
    let mode = fs::metadata(path)
        .expect("private report metadata should read")
        .permissions()
        .mode()
        & 0o777;
    let root_mode = fs::metadata(&private)
        .expect("private root metadata should read")
        .permissions()
        .mode()
        & 0o777;

    // Assert
    assert_eq!(root_mode, 0o700);
    assert_eq!(mode, 0o600);
    assert!(!document.contains("private-owner"));
    assert!(!document.contains("private-password"));
    assert!(!document.contains("poolURL"));
}

#[test]
fn exact_attempt_contract_rejects_any_changed_bound() {
    // Arrange
    let mut options = ReadinessOptions {
        private_root: PathBuf::from("scratch/private"),
        pool_credentials: PathBuf::from("pool-credentials.json"),
        attempt_ordinal: 5,
        samples: 3,
        sample_timeout: Duration::from_secs(15),
        sample_delay: Duration::from_secs(2),
    };

    // Act / Assert
    assert_eq!(validate_contract(&options), Ok(()));
    options.samples = 2;
    assert_eq!(
        validate_contract(&options),
        Err(ReadinessError::new(ReadinessCategory::ContractInvalid))
    );
}
