use bitaxe_worker_control::serial::{
    SerialEnvelope, SerialError, SerialFrameAccumulator, SerialKind, SerialSessionBinding,
    MAXIMUM_CONTROL_PAYLOAD_BYTES, MAXIMUM_WIRE_FRAME_BYTES,
};
use serde_json::value::RawValue;

fn record(payload: &str) -> Vec<u8> {
    let raw = RawValue::from_string(payload.to_owned()).expect("fixture JSON");
    SerialEnvelope::encode(SerialKind::Control, Some("AAAAAAAAAAAAAAAAAAAAAA"), 1, &raw)
        .expect("fixture envelope")
}

#[test]
fn fragmented_and_coalesced_records_preserve_all_frames() {
    // Arrange
    let expected = record("{}");
    let stream = [expected.clone(), expected.clone()].concat();
    let mut accumulator = SerialFrameAccumulator::default();

    // Act
    let frames: Vec<_> = stream
        .chunks(7)
        .flat_map(|chunk| {
            chunk
                .iter()
                .filter_map(|byte| accumulator.push_byte(*byte))
                .collect::<Vec<_>>()
        })
        .collect();

    // Assert
    assert_eq!(frames, vec![Ok(expected.clone()), Ok(expected)]);
}

#[test]
fn oversized_line_resynchronizes_only_after_newline() {
    // Arrange
    let mut accumulator = SerialFrameAccumulator::default();
    let mut stream = vec![b'x'; MAXIMUM_WIRE_FRAME_BYTES * 2];
    stream.push(b'\n');
    stream.extend(record("{}"));

    // Act
    let results: Vec<_> = stream
        .into_iter()
        .filter_map(|byte| accumulator.push_byte(byte))
        .collect();

    // Assert
    assert_eq!(results, vec![Err(SerialError::Oversized), Ok(record("{}"))]);
}

#[test]
fn boot_text_does_not_become_control() {
    // Arrange
    let mut accumulator = SerialFrameAccumulator::default();
    let stream = [b"boot: starting\n".to_vec(), record("{}")].concat();

    // Act
    let parsed: Vec<_> = stream
        .into_iter()
        .filter_map(|byte| accumulator.push_byte(byte))
        .map(|line| SerialEnvelope::parse(&line.expect("bounded line")).is_ok())
        .collect();

    // Assert
    assert_eq!(parsed, [false, true]);
}

#[test]
fn maximum_control_payload_fits_wire_bound() {
    // Arrange
    let payload = format!(
        "{{\"x\":\"{}\"}}",
        "x".repeat(MAXIMUM_CONTROL_PAYLOAD_BYTES - 8)
    );

    // Act
    let wire = record(&payload);

    // Assert
    assert!(wire.len() <= MAXIMUM_WIRE_FRAME_BYTES);
    assert_eq!(
        SerialEnvelope::parse(&wire)
            .expect("maximum payload")
            .payload
            .get()
            .len(),
        MAXIMUM_CONTROL_PAYLOAD_BYTES
    );
}

#[test]
fn control_payload_over_limit_is_rejected() {
    // Arrange
    let payload = RawValue::from_string(format!(
        "{{\"x\":\"{}\"}}",
        "x".repeat(MAXIMUM_CONTROL_PAYLOAD_BYTES - 7)
    ))
    .expect("valid JSON");

    // Act
    let result = SerialEnvelope::encode(
        SerialKind::Control,
        Some("AAAAAAAAAAAAAAAAAAAAAA"),
        1,
        &payload,
    );

    // Assert
    assert_eq!(result, Err(SerialError::Oversized));
}

#[test]
fn noncanonical_session_nonce_is_rejected() {
    // Arrange / Act
    let result = SerialSessionBinding::parse(
        "AAAAAAAAAAAAAAAAAAAAAB",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );

    // Assert
    assert_eq!(result, Err(SerialError::Invalid));
}

#[test]
fn unknown_envelope_fields_are_rejected() {
    // Arrange
    let mut value: serde_json::Value = serde_json::from_slice(&record("{}")).expect("fixture JSON");
    value["unexpected"] = true.into();
    let mut bytes = serde_json::to_vec(&value).expect("fixture encoding");
    bytes.push(b'\n');

    // Act / Assert
    assert!(matches!(
        SerialEnvelope::parse(&bytes),
        Err(SerialError::Invalid)
    ));
}

#[test]
fn omitted_session_field_is_not_a_hello() {
    // Arrange
    let mut value: serde_json::Value = serde_json::from_slice(&record("{}")).expect("fixture");
    value.as_object_mut().expect("envelope").remove("sessionId");
    let mut bytes = serde_json::to_vec(&value).expect("fixture encoding");
    bytes.push(b'\n');

    // Act / Assert
    assert!(matches!(
        SerialEnvelope::parse(&bytes),
        Err(SerialError::Invalid)
    ));
}

#[test]
fn changed_payload_bytes_of_the_same_length_fail_integrity() {
    // Arrange
    let wire = String::from_utf8(record(r#"{"padding":"xxxx"}"#)).expect("UTF-8 fixture");
    let changed = wire.replace("xxxx", "yyyy");
    // Act / Assert
    assert!(matches!(
        SerialEnvelope::parse(changed.as_bytes()),
        Err(SerialError::Integrity)
    ));
}

#[test]
fn lexical_payload_whitespace_key_order_and_escapes_are_preserved() {
    // Arrange
    for payload in [r#"{ "z": 1, "\u0061":"\u0078" }"#, r#"{"text":"雪🚀"}"#] {
        let wire = record(payload);
        // Act
        let envelope = SerialEnvelope::parse(&wire).expect("exact lexical integrity");
        // Assert
        assert_eq!(envelope.payload.get(), payload);
    }
}

#[test]
fn clean_close_requires_exact_session_payload_and_known_reason() {
    // Arrange
    for (payload, expected) in [
        (r#"{"op":"close","reason":"tab_closed"}"#, true),
        (r#"{"op":"close","reason":"unknown"}"#, false),
        (r#"{"op":"other","reason":"tab_closed"}"#, false),
        (
            r#"{"op":"close","reason":"tab_closed","extra":true}"#,
            false,
        ),
    ] {
        let raw = RawValue::from_string(payload.to_owned()).expect("fixture JSON");
        let wire =
            SerialEnvelope::encode(SerialKind::Session, Some("AAAAAAAAAAAAAAAAAAAAAA"), 3, &raw)
                .expect("session frame");
        // Act / Assert
        assert_eq!(
            SerialEnvelope::parse(&wire)
                .expect("valid frame")
                .is_close(),
            expected
        );
        assert!(!SerialEnvelope::parse(&record(payload))
            .expect("control frame")
            .is_close());
    }
}

#[test]
fn clearing_revoked_partial_input_allows_a_fresh_record() {
    // Arrange
    let mut accumulator = SerialFrameAccumulator::default();
    for byte in b"{\"profile\":\"truncated" {
        assert!(accumulator.push_byte(*byte).is_none());
    }
    // Act
    accumulator.clear();
    let frames: Vec<_> = record("{}")
        .into_iter()
        .filter_map(|byte| accumulator.push_byte(byte))
        .collect();
    // Assert
    assert_eq!(frames, vec![Ok(record("{}"))]);
}

#[test]
fn deleted_payload_bytes_never_reach_control_dispatch() {
    // Arrange: deleting repeated bytes leaves syntactically valid JSON.
    let wire = record(r#"{"padding":"xxxxxxxx"}"#);
    let text = String::from_utf8(wire).expect("UTF-8 fixture");
    let shortened = text.replace("xxxxxxxx", "xxxx");

    // Act
    let result = SerialEnvelope::parse(shortened.as_bytes());

    // Assert
    assert!(
        result.is_err(),
        "valid JSON is insufficient for payload integrity"
    );
}

#[test]
fn short_completed_records_do_not_own_a_maximum_sized_secret_allocation() {
    // Arrange
    let mut accumulator = SerialFrameAccumulator::default();
    let wire = record("{}");

    // Act
    let completed = wire
        .iter()
        .find_map(|byte| accumulator.push_byte(*byte))
        .expect("complete frame")
        .expect("bounded frame");

    // Assert
    assert_eq!(completed.capacity(), completed.len());
}

#[test]
fn a_short_record_after_maximum_input_has_no_retained_payload_tail() {
    // Arrange
    let mut accumulator = SerialFrameAccumulator::default();
    let large = record(&format!(
        "{{\"x\":\"{}\"}}",
        "x".repeat(MAXIMUM_CONTROL_PAYLOAD_BYTES - 8)
    ));
    let small = record("{}");
    for byte in large {
        let _completed = accumulator.push_byte(byte);
    }
    // Act
    let completed = small
        .iter()
        .find_map(|byte| accumulator.push_byte(*byte))
        .expect("short record")
        .expect("bounded frame");
    // Assert
    assert_eq!(completed, small);
    assert_eq!(completed.capacity(), completed.len());
}
