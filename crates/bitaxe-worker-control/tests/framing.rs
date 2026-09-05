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
    let bytes = b"{\"profile\":\"bwg-worker-serial/0.1\",\"kind\":\"session\",\"sequence\":0,\"payload\":{}}\n";

    // Act / Assert
    assert!(matches!(
        SerialEnvelope::parse(bytes),
        Err(SerialError::Invalid)
    ));
}
