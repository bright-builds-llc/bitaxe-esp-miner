//! Real production allocation sites under the capability-aware allocator: the receive
//! frame buffer (parser), the V2 record snapshot and the outbound envelope encode
//! (serializer). Each fails typed when its route is exhausted and falls back when only
//! PSRAM is.

use super::*;
use bitaxe_worker_control::serial::{
    SerialEnvelope, SerialError, SerialFrameAccumulator, SerialKind, ENCODE_SITE, FRAME_BUFFER_SITE,
};
use bitaxe_worker_control::v2::{CurrentObservation, Scope, V2Record, SNAPSHOT_SITE};
use serde_json::value::RawValue;

fn record() -> V2Record {
    let observation = CurrentObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        serial_transport_epoch: 4,
        maybe_observed_at_us: Some(1000),
        clock_valid: true,
        maybe_station_ipv4: None,
        wifi_connected: true,
        maybe_socket: None,
    };
    V2Record::admit(Scope::Share, "attempt".into(), &observation, None).expect("record")
}

fn payload() -> Box<RawValue> {
    RawValue::from_string(r#"{"command":"status"}"#.to_owned()).expect("payload")
}

#[test]
fn an_exhausted_route_fails_the_armed_site() {
    // Arrange
    let armed = arm(Site("test.site"), Exhausted::HEAP);
    let mut buffer: Vec<u8> = Vec::new();
    // Act
    let result = fallible::reserve_exact(&mut buffer, 64, Site("test.site"));
    // Assert
    assert!(result.is_err());
    assert_eq!(
        armed.attempts(),
        [Attempt {
            site: "test.site",
            bytes: 64,
            maybe_served_by: None
        }]
    );
}

#[test]
fn psram_exhaustion_alone_falls_back_to_internal() {
    // Arrange
    let armed = arm(Site("test.site"), Exhausted::PSRAM);
    let mut buffer: Vec<u8> = Vec::new();
    // Act
    let result = fallible::reserve_exact(&mut buffer, 64, Site("test.site"));
    // Assert
    assert!(result.is_ok());
    assert_eq!(
        armed.attempts()[0].maybe_served_by,
        Some(Capability::Internal)
    );
}

#[test]
fn allocations_outside_the_armed_site_pass_through() {
    // Arrange
    let armed = arm(Site("test.armed"), Exhausted::HEAP);
    let mut buffer: Vec<u8> = Vec::new();
    // Act
    let other = fallible::reserve_exact(&mut buffer, 64, Site("test.other"));
    let plain: Vec<u8> = Vec::with_capacity(64);
    // Assert
    assert!(other.is_ok());
    assert!(plain.capacity() >= 64);
    assert!(armed.attempts().is_empty());
}

#[test]
fn the_receive_frame_buffer_fails_typed() {
    // Arrange
    let armed = arm(FRAME_BUFFER_SITE, Exhausted::HEAP);
    let mut accumulator = SerialFrameAccumulator::default();
    // Act
    let result = accumulator.push_byte(b'{');
    // Assert
    assert_eq!(result, Some(Err(SerialError::Unavailable)));
    assert_eq!(armed.attempts().len(), 1);
}

#[test]
fn the_v2_snapshot_fails_typed() {
    // Arrange
    let retained = record();
    let armed = arm(SNAPSHOT_SITE, Exhausted::HEAP);
    // Act
    let result = retained.try_snapshot();
    // Assert
    assert!(result.is_err());
    assert_eq!(armed.attempts().len(), 1);
}

#[test]
fn the_v2_snapshot_survives_psram_exhaustion() {
    // Arrange
    let retained = record();
    let armed = arm(SNAPSHOT_SITE, Exhausted::PSRAM);
    // Act
    let result = retained.try_snapshot();
    // Assert
    drop(armed);
    let snapshot = result.expect("snapshot");
    assert_eq!(
        serde_json::to_value(&snapshot).expect("json"),
        serde_json::to_value(retained.snapshot()).expect("json")
    );
}

#[test]
fn the_envelope_encode_fails_typed() {
    // Arrange
    let raw = payload();
    let armed = arm(ENCODE_SITE, Exhausted::HEAP);
    // Act
    let result = SerialEnvelope::encode(SerialKind::Control, None, 1, &raw);
    // Assert
    assert_eq!(result.err(), Some(SerialError::Unavailable));
    assert_eq!(armed.attempts().len(), 1);
}
