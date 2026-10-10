//! One script per link behavior; each transcript is pinned byte-for-byte.
use super::{transcript, Step, BEGIN_LINK_FAILS, ENTROPY_FAILS, HELLO_ACK_FAILS, WORK_COOLING};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bitaxe_worker_control::serial::{SerialEnvelope, SerialKind};
use serde_json::value::RawValue;

const HOST_NONCE: [u8; 32] = [7; 32];

/// Session id of the nth admitted Hello: the fake entropy counts up, 16 then 32 bytes per Hello.
fn session(hello: u8) -> String {
    let start = 48 * hello;
    URL_SAFE_NO_PAD.encode((start..start + 16).collect::<Vec<u8>>())
}

fn record(kind: SerialKind, maybe_session: Option<&str>, sequence: u32, payload: &str) -> Vec<u8> {
    let payload = RawValue::from_string(payload.to_owned()).expect("raw payload");
    SerialEnvelope::encode(kind, maybe_session, sequence, &payload).expect("encodable record")
}

fn hello_payload(padding: usize) -> String {
    format!(
        r#"{{"op":"hello","hostNonce":"{}"{}}}"#,
        URL_SAFE_NO_PAD.encode(HOST_NONCE),
        " ".repeat(padding)
    )
}

fn hello() -> Vec<u8> {
    record(SerialKind::Session, None, 0, &hello_payload(0))
}

/// A Hello record of exactly `wire_bytes`, padded with insignificant payload whitespace.
fn hello_of(wire_bytes: usize) -> Vec<u8> {
    let base = hello().len();
    let padded = record(
        SerialKind::Session,
        None,
        0,
        &hello_payload(wire_bytes - base),
    );
    // The digest field has a fixed width and payloadBytes gains at most one digit.
    let padded = if padded.len() == wire_bytes {
        padded
    } else {
        record(
            SerialKind::Session,
            None,
            0,
            &hello_payload(wire_bytes - base - (padded.len() - wire_bytes)),
        )
    };
    assert_eq!(padded.len(), wire_bytes);
    padded
}

fn heartbeat(hello: u8, sequence: u32, payload: &str) -> Vec<u8> {
    record(
        SerialKind::Heartbeat,
        Some(&session(hello)),
        sequence,
        payload,
    )
}

fn control(hello: u8, sequence: u32) -> Vec<u8> {
    record(
        SerialKind::Control,
        Some(&session(hello)),
        sequence,
        r#"{"command":"status"}"#,
    )
}

fn close(hello: u8, sequence: u32) -> Vec<u8> {
    record(
        SerialKind::Session,
        Some(&session(hello)),
        sequence,
        r#"{"op":"close","reason":"tab_closed"}"#,
    )
}

fn at(at_ms: u64, parts: &[&[u8]]) -> Step {
    Step::Chunk {
        at_ms,
        bytes: parts.concat(),
    }
}

fn idle(at_ms: u64) -> Step {
    Step::Chunk {
        at_ms,
        bytes: Vec::new(),
    }
}

fn replace(mut bytes: Vec<u8>, from: &[u8], to: &[u8]) -> Vec<u8> {
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("pattern present");
    bytes.splice(start..start + from.len(), to.iter().copied());
    bytes
}

macro_rules! pinned {
    ($name:ident, $steps:expr) => {
        #[test]
        fn $name() {
            // Arrange
            let steps = $steps;
            // Act
            let actual = transcript(steps);
            // Assert
            if let Ok(directory) = std::env::var("LINK_TRANSCRIPT_BLESS_DIR") {
                let path = format!("{directory}/{}.txt", stringify!($name));
                std::fs::write(path, &actual).expect("bless transcript");
                return;
            }
            let expected = include_str!(concat!("transcripts/", stringify!($name), ".txt"));
            assert_eq!(actual, expected);
        }
    };
}

pinned!(
    boot_text_then_full_lifecycle,
    vec![
        at(10, &[b"boot text before the link\n", b"{\"x\":1}\n"]),
        at(20, &[&hello(), &heartbeat(0, 1, "{}")]),
        Step::Authenticate,
        at(1000, &[&heartbeat(0, 2, "{ \t }")]),
        at(1100, &[&control(0, 3)]),
        Step::Drain,
        at(1200, &[&close(0, 4)]),
        idle(1300),
    ]
);

pinned!(lifecycle_in_seven_byte_chunks, {
    let stream = [hello(), heartbeat(0, 1, "{}"), control(0, 2), close(0, 3)].concat();
    stream
        .chunks(7)
        .enumerate()
        .map(|(index, chunk)| at(10 + index as u64, &[chunk]))
        .collect::<Vec<Step>>()
});

pinned!(
    heartbeat_timeout_closes_link,
    vec![
        at(0, &[&hello()]),
        Step::Authenticate,
        at(1000, &[&heartbeat(0, 1, "{}")]),
        idle(3799),
        idle(3800),
    ]
);

pinned!(
    late_heartbeat_closes_link,
    vec![
        at(0, &[&hello()]),
        Step::Authenticate,
        at(3000, &[&heartbeat(0, 1, "{}")])
    ]
);

pinned!(
    pre_admission_heartbeat_does_not_refresh,
    vec![
        at(0, &[&hello()]),
        at(2000, &[&heartbeat(0, 1, "{}")]),
        idle(2800),
        idle(2801)
    ]
);

pinned!(
    cooling_heartbeat_keeps_link,
    vec![
        Step::Set(&WORK_COOLING, true),
        at(0, &[&hello()]),
        Step::Authenticate,
        at(1000, &[&heartbeat(0, 1, "{}")]),
        idle(1100),
    ]
);

pinned!(
    clock_regression_closes_link,
    vec![at(1000, &[&hello()]), idle(500), idle(600)]
);

pinned!(
    owner_revocation_closes_link,
    vec![at(0, &[&hello()]), Step::Revoke, idle(100), idle(200)]
);

pinned!(
    read_errors_clear_partial_input,
    vec![
        Step::ReadError { at_ms: 5 },
        at(10, &[&hello()]),
        at(20, &[&control(0, 1)[..30]]),
        Step::ReadError { at_ms: 30 },
        at(40, &[&control(0, 1)[30..]]),
    ]
);

pinned!(oversized_line_records_framing, {
    let mut line = vec![b'a'; 70_000];
    line.push(b'\n');
    vec![
        at(0, &[&hello()]),
        at(10, &[&line]),
        at(20, &[&control(0, 1)]),
    ]
});

pinned!(
    malformed_envelope_records_envelope,
    vec![
        at(0, &[&hello()]),
        at(10, &[b"{\"not\":\"an envelope\"}\n"])
    ]
);

pinned!(
    payload_tamper_records_integrity,
    vec![
        at(0, &[&hello()]),
        at(10, &[&replace(control(0, 1), b"status", b"statuz")])
    ]
);

pinned!(
    carriage_return_records_envelope,
    vec![
        at(0, &[&hello()]),
        at(
            10,
            &[&replace(control(0, 1), b"{\"command\"", b"{\r\"command\"")]
        )
    ]
);

pinned!(
    sequence_rules,
    vec![
        at(0, &[&hello()]),
        at(
            10,
            &[&record(
                SerialKind::Control,
                Some(&session(4)),
                1,
                r#"{"command":"status"}"#
            )]
        ),
        Step::Drain,
        at(20, &[&hello()]),
        at(30, &[&control(1, 5), &control(1, 5)]),
        Step::Drain,
        at(40, &[&hello()]),
        at(50, &[&control(2, 3), &control(2, u32::MAX)]),
        Step::Drain,
    ]
);

pinned!(
    heartbeat_payload_must_be_empty_object,
    vec![
        at(0, &[&hello()]),
        Step::Authenticate,
        at(10, &[&heartbeat(0, 1, r#"{"a":1}"#)])
    ]
);

pinned!(
    control_queue_full_records_control_queue,
    vec![
        at(0, &[&hello()]),
        at(
            10,
            &[
                &control(0, 1),
                &control(0, 2),
                &control(0, 3),
                &control(0, 4)
            ]
        )
    ]
);

pinned!(
    unexpected_kinds_close_link,
    vec![
        at(0, &[&hello()]),
        at(
            10,
            &[&record(
                SerialKind::Session,
                Some(&session(0)),
                1,
                r#"{"op":"receive_credit","receivedBytes":1}"#
            )]
        ),
        at(20, &[&hello()]),
        at(
            30,
            &[&record(SerialKind::Diagnostic, Some(&session(1)), 1, "{}")]
        ),
    ]
);

pinned!(
    hello_admission_rejections,
    vec![
        at(
            0,
            &[&record(
                SerialKind::Session,
                None,
                0,
                &hello_payload(0).replace("\"hello\"", "\"hullo\"")
            )]
        ),
        at(
            10,
            &[&record(
                SerialKind::Session,
                None,
                0,
                r#"{"op":"hello","hostNonce":"short"}"#
            )]
        ),
        at(
            20,
            &[&record(
                SerialKind::Session,
                None,
                0,
                &hello_payload(0).replace("}", r#","extra":1}"#)
            )]
        ),
        at(
            30,
            &[&record(
                SerialKind::Session,
                Some(&session(0)),
                0,
                &hello_payload(0)
            )]
        ),
        at(
            40,
            &[&record(SerialKind::Session, None, 1, &hello_payload(0))]
        ),
        idle(50),
    ]
);

pinned!(
    hello_receive_window_bound,
    vec![
        at(0, &[&hello_of(2048)]),
        at(10, &[&hello_of(2049)]),
        idle(20)
    ]
);

pinned!(
    hello_effect_failures,
    vec![
        Step::Set(&ENTROPY_FAILS, true),
        at(0, &[&hello()]),
        Step::Set(&ENTROPY_FAILS, false),
        Step::Set(&BEGIN_LINK_FAILS, true),
        at(10, &[&hello()]),
        Step::Set(&BEGIN_LINK_FAILS, false),
        Step::Set(&HELLO_ACK_FAILS, true),
        at(20, &[&hello()]),
        Step::Set(&HELLO_ACK_FAILS, false),
        Step::Drain,
        at(30, &[&hello()]),
        Step::Drain,
    ]
);

pinned!(
    session_enqueue_failure_revokes,
    vec![
        at(0, &[&hello()]),
        at(10, &[&control(0, 1), &control(0, 2), &control(0, 3)]),
        at(20, &[&hello()]),
        Step::Drain,
    ]
);
