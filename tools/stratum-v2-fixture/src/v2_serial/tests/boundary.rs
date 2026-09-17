use super::super::*;
use bitaxe_stratum::v2::{
    frame::Frame,
    messages::{SetupConnection, SubmitSharesStandard},
};
use zeroize::Zeroizing;
fn secret(frame: Frame) -> io::SecretFrame {
    io::SecretFrame {
        header: frame.header,
        payload: Zeroizing::new(frame.payload().to_vec()),
    }
}
fn options() -> Options {
    Options {
        root: "/synthetic/not-used".into(),
        attempt: "AQEBAQEBAQEBAQEBAQEBAQ".into(),
        scope: input::Scope::Channel,
    }
}
fn value() -> serde_json::Value {
    serde_json::json!({"schema":"str005-v2-fixture-input-v1","scope":"channel","attemptId":"AQEBAQEBAQEBAQEBAQEBAQ","listenIpv4":"192.168.1.20","expectedPeerIpv4":"192.168.1.10","userIdentity":"synthetic-only"})
}
fn encoded(value: &serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("test JSON");
    bytes.push(b'\n');
    bytes
}
#[test]
fn input_rejects_unknown_duplicate_trailing_and_noncanonical_fields() {
    assert!(input::parse_input(&encoded(&value()), &options()).is_ok());
    let mut unknown = value();
    unknown["password"] = serde_json::json!("synthetic-never-write");
    assert!(input::parse_input(&encoded(&unknown), &options()).is_err());
    let duplicate = String::from_utf8(encoded(&value()))
        .expect("utf8")
        .replacen('{', "{\"scope\":\"channel\",", 1);
    assert!(input::parse_input(duplicate.as_bytes(), &options()).is_err());
    let mut trailing = encoded(&value());
    trailing.extend_from_slice(b"{}\n");
    assert!(input::parse_input(&trailing, &options()).is_err());
    for address in [
        "127.0.0.1",
        "192.168.01.20",
        "169.254.1.2",
        "8.8.8.8",
        "::ffff:192.168.1.20",
        "10.1",
    ] {
        let mut bad = value();
        bad["listenIpv4"] = serde_json::json!(address);
        assert!(input::parse_input(&encoded(&bad), &options()).is_err());
    }
    assert!(input::parse_input(&vec![b'x'; 4097], &options()).is_err());
}
#[test]
fn new_mode_rejects_effect_flags_duplicates_before_root_access() {
    let args = [
        "--mode",
        "v2-serial",
        "--scope",
        "channel",
        "--private-root",
        "/absent/root",
        "--attempt-id",
        "AQEBAQEBAQEBAQEBAQEBAQ",
    ]
    .map(str::to_string);
    for key in [
        "--listen-address",
        "--pool-credentials",
        "--scope",
        "--lifetime-seconds",
    ] {
        let mut bad = args.to_vec();
        bad.extend([key.into(), "synthetic".into()]);
        assert!(input::parse_options(&bad).is_err());
    }
}
#[test]
fn setup_requires_rolling_flags_and_correct_extension_without_hashing_secrets() {
    let address = "127.0.0.1:1234".parse().expect("test address");
    let mut setup = SetupConnection {
        endpoint_host: "127.0.0.1".into(),
        endpoint_port: 1234,
        vendor: "synthetic".into(),
        hardware_version: "205".into(),
        firmware: "synthetic".into(),
        device_id: "synthetic".into(),
        flags: 5,
    };
    assert!(wire::setup(&secret(setup.encode().expect("encode")), address).is_ok());
    setup.flags = 1;
    assert!(wire::setup(&secret(setup.encode().expect("encode")), address).is_err());
    setup.flags = 5;
    let mut frame = secret(setup.encode().expect("encode"));
    frame.header.extension_type = 0x8000;
    assert!(wire::setup(&frame, address).is_err());
}
#[test]
fn share_decoder_requires_flag_exact_payload_and_preserves_stage() {
    let mut frame = secret(
        SubmitSharesStandard {
            channel_id: 1,
            sequence_number: 1,
            job_id: 1,
            nonce: 1,
            ntime: 1,
            version: oracle::VERSION,
        }
        .encode()
        .expect("encode"),
    );
    assert!(wire::share(&frame).is_ok());
    frame.payload.push(0);
    assert_eq!(
        wire::share(&frame).expect_err("extra").stage,
        "share_received"
    );
    frame.payload.pop();
    frame.header.extension_type = 0;
    assert!(wire::share(&frame).is_err());
}

#[test]
fn runtime_ready_pipe_closes_and_cannot_be_redirected_to_a_file() {
    use std::io::Read;
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    // Arrange
    let (writer, mut reader) = UnixStream::pair().expect("private pipe");
    let ready = serde_json::json!({"schema":"str005-v2-fixture-ready-runtime-v1","scope":"channel","attemptId":"AQEBAQEBAQEBAQEBAQEBAQ","instanceId":"AQEBAQEBAQEBAQEBAQEBAQ","listenIpv4":"192.168.1.20","listenPort":1234,"authorityPublicKey":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"});
    // Act
    input::write_pipe(File::from(OwnedFd::from(writer)), &ready).expect("private runtime receipt");
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).expect("actual pipe EOF");
    // Assert
    assert!(bytes.ends_with(b"\n"));
    assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 1);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).expect("runtime record"),
        ready
    );
    let path = std::env::temp_dir().join(format!(
        "v2-no-ready-file-{}-{}",
        std::process::id(),
        nonce()
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("synthetic file");
    assert!(input::write_pipe(file, &ready).is_err());
    assert_eq!(std::fs::metadata(&path).expect("empty file").len(), 0);
    std::fs::remove_file(path).expect("remove file");
}
#[test]
fn private_input_requires_one_complete_pipe_record() {
    use std::io::Write;
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    let (reader, mut writer) = UnixStream::pair().expect("private pipe");
    writer
        .write_all(&encoded(&value()))
        .expect("synthetic runtime input");
    drop(writer);
    let received =
        input::read_input(File::from(OwnedFd::from(reader)), &options()).expect("closed input");
    assert_eq!(received.scope, input::Scope::Channel);
}
#[test]
fn invalid_job_fields_cannot_reach_independent_target_success() {
    let job = job::Job::new(1, nonce()).expect("synthetic job");
    let share = model::Submission {
        channel_id: job.channel,
        job_id: job.id,
        sequence_number: 0,
        nonce: 0,
        ntime: job.ntime,
        version: oracle::VERSION,
    };
    assert!(job.validate(&share).is_ok());
    for changed in [
        model::Submission {
            channel_id: job.channel.wrapping_add(1),
            ..share
        },
        model::Submission {
            job_id: job.id.wrapping_add(1),
            ..share
        },
        model::Submission {
            ntime: job.ntime.wrapping_add(1),
            ..share
        },
        model::Submission {
            version: oracle::VERSION | 1,
            ..share
        },
    ] {
        assert!(job.validate(&changed).is_err());
    }
}

#[test]
fn absent_pipe_is_rejected_without_invalid_owned_descriptor() {
    assert!(input::inherited_pipe(-1).is_err());
}

#[test]
fn incremental_share_requires_completed_ack_and_preserves_exclusive_row() {
    // Arrange: synthetic facts exercise publication, not proof of actual mined work.
    let root = std::env::temp_dir().join(format!(
        "v2-share-publication-{}-{}",
        std::process::id(),
        nonce()
    ));
    DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .expect("synthetic root");
    let mut evidence = Evidence::new(nonce(), nonce(), Instant::now());
    evidence.terminal.accepted_shares = 1;
    evidence.shares.shares.push(model::Share {
        submission: model::Submission {
            channel_id: 1,
            sequence_number: 0,
            job_id: 2,
            nonce: 3,
            ntime: 4,
            version: oracle::VERSION,
        },
        received_at_fixture_us: 10,
        header_sha256d: "a".repeat(64),
        target_valid: true,
        write_started_at_fixture_us: Some(11),
        write_completed_at_fixture_us: None,
        accepted_count: None,
        shares_sum: None,
    });
    // Act / Assert: partial or failed ACK has no success file.
    assert!(session::persist_completed_share(&root, &evidence).is_err());
    assert!(!root.join("share-0001.json").exists());
    let row = evidence.shares.shares.last_mut().expect("synthetic share");
    row.write_completed_at_fixture_us = Some(12);
    row.accepted_count = Some(1);
    row.shares_sum = Some(1024);
    session::persist_completed_share(&root, &evidence).expect("completed ACK publication");
    let bytes = std::fs::read(root.join("share-0001.json")).expect("published row");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("safe facts");
    assert_eq!(value["connectionId"], evidence.shares.connection_id);
    assert_eq!(
        value["shares"][0],
        serde_json::to_value(&evidence.shares.shares[0]).expect("same actual row")
    );
    assert!(session::persist_completed_share(&root, &evidence).is_err());
    assert_eq!(
        std::fs::read(root.join("share-0001.json")).expect("unchanged"),
        bytes
    );
    std::fs::remove_dir_all(root).expect("remove synthetic root");
}
