//! serve-serial through a real process boundary: stdin in, stdout out, exit and outcome file.
use bitaxe_worker_control::serial::{SerialEnvelope, SerialKind};
use serde_json::{value::RawValue, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use tempfile::TempDir;

const HOST_NONCE: &str = "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk";

struct Session {
    directory: TempDir,
    child: std::process::Child,
    stdout: BufReader<std::process::ChildStdout>,
}

fn start() -> Session {
    let directory = TempDir::new().expect("temp");
    let capability = directory.path().join("capability.json");
    std::fs::write(&capability, br#"{"fixture":"capability"}"#).expect("capability");
    let mut child = Command::new(env!("CARGO_BIN_EXE_bitaxe-virtual-board-tool"))
        .arg("serve-serial")
        .arg("--seed=3")
        .arg(format!("--capability={}", capability.display()))
        .arg(format!(
            "--outcome={}",
            directory.path().join("outcome.json").display()
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("device process");
    let stdout = BufReader::new(child.stdout.take().expect("stdout"));
    Session {
        directory,
        child,
        stdout,
    }
}

fn hello() -> Vec<u8> {
    let payload = format!(r#"{{"op":"hello","hostNonce":"{HOST_NONCE}"}}"#);
    let raw = RawValue::from_string(payload).expect("payload");
    SerialEnvelope::encode(SerialKind::Session, None, 0, &raw).expect("record")
}

fn first_record(session: &mut Session) -> SerialEnvelope {
    let mut line = String::new();
    loop {
        line.clear();
        session.stdout.read_line(&mut line).expect("device output");
        if line.starts_with('{') {
            return SerialEnvelope::parse(line.as_bytes()).expect("device record");
        }
    }
}

#[test]
fn hello_gets_an_acknowledgement_on_stdout() {
    // Arrange
    let mut session = start();
    // Act
    session
        .child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(&hello())
        .expect("hello");
    let ack = first_record(&mut session);
    // Assert
    let payload: Value = serde_json::from_str(ack.payload.get()).expect("payload");
    assert_eq!((ack.kind, ack.sequence), (SerialKind::Session, 0));
    assert_eq!(payload["op"], "hello_ack");
    assert_eq!(payload["hostNonce"], HOST_NONCE);
}

#[test]
fn closing_stdin_without_close_ends_unclean_with_a_private_outcome() {
    // Arrange
    let mut session = start();
    session
        .child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(&hello())
        .expect("hello");
    first_record(&mut session);
    // Act
    drop(session.child.stdin.take());
    let status = session.child.wait().expect("exit");
    // Assert
    assert_eq!(status.code(), Some(2));
    let path = session.directory.path().join("outcome.json");
    let outcome: Value =
        serde_json::from_slice(&std::fs::read(&path).expect("outcome")).expect("JSON");
    assert_eq!(outcome["clean"], false);
    assert_eq!(outcome["outcome"]["hello_acknowledged"], true);
    assert_eq!(
        std::fs::metadata(&path).expect("mode").permissions().mode() & 0o777,
        0o600
    );
}
