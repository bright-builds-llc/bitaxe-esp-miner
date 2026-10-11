//! The shared outbound writer reproduces the published Gate device records byte for byte.
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;

use bitaxe_worker_control::serial::link::hello_ack_payload;
use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
use bitaxe_worker_control::serial::writer::{
    SerialOutputPurpose, SerialOutputSink, SerialRecordWriter, SerialWriterView,
};
use bitaxe_worker_control::serial::{ReceiveCreditMailbox, SerialSessionBinding};
use serde_json::{json, Value};

const SESSION: &str = "AAAAAAAAAAAAAAAAAAAAAA";

#[derive(Default)]
struct View {
    now: Cell<u64>,
    credit: ReceiveCreditMailbox,
}

impl SerialWriterView for View {
    fn now_ms(&self) -> u64 {
        self.now.get()
    }
    fn current_epoch(&self) -> u32 {
        1
    }
    fn has_partial_output(&self) -> bool {
        false
    }
    fn received_bytes(&self, epoch: u32) -> Option<u32> {
        self.credit.maybe_received_bytes(epoch)
    }
    fn terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        self.credit.maybe_terminal_bytes(epoch, now_ms)
    }
    fn finish_terminal(&self, epoch: u32) {
        self.credit.finish_terminal(epoch);
    }
    fn revoke_epoch(&self, _epoch: u32) {}
    fn trace(&self, _: SerialTraceCorrelation, _: SerialTraceStage, _: usize) {}
    fn diagnostic_dropped(&self) {}
}

#[derive(Default)]
struct Wire(RefCell<Vec<Vec<u8>>>);

impl SerialOutputSink for Wire {
    type Error = ();
    type DiagnosticKind = ();
    fn write<A: Fn() -> bool>(
        &mut self,
        record: &[u8],
        admitted: A,
        _: Option<SerialTraceCorrelation>,
        _: SerialOutputPurpose<()>,
    ) -> Result<(), ()> {
        if !admitted() {
            return Err(());
        }
        self.0.borrow_mut().push(record.to_vec());
        Ok(())
    }
    fn resynchronize<A: Fn() -> bool>(&mut self, _: A) -> Result<(), ()> {
        Ok(())
    }
    fn is_transport_failure(_: &()) -> bool {
        true
    }
}

fn maybe_fixture() -> Option<Value> {
    let Ok(path) = env::var("BWG_SERIAL_FIXTURES") else {
        assert_ne!(
            env::var("BWG_REQUIRE_PINNED_FIXTURES").ok().as_deref(),
            Some("1"),
            "BWG_SERIAL_FIXTURES is required"
        );
        return None;
    };
    Some(serde_json::from_str(&fs::read_to_string(path).expect("fixture")).expect("fixture JSON"))
}

fn wire<'a>(fixture: &'a Value, id: &str) -> &'a [u8] {
    fixture["frames"]
        .as_array()
        .expect("frames")
        .iter()
        .find(|frame| frame["id"] == id)
        .and_then(|frame| frame["wire"].as_str())
        .expect("published frame")
        .as_bytes()
}

/// Gate sequence: ack at 0, heartbeat at 1, a control reply at 2, credit at 3.
fn published_sequence() -> Vec<Vec<u8>> {
    let view = View::default();
    view.credit.begin(1);
    let mut sink = Wire::default();
    let mut writer = SerialRecordWriter::new();
    writer
        .hello(
            1,
            SESSION.into(),
            &json!({"op": "hello_ack"}),
            &view,
            &mut sink,
        )
        .expect("current Hello")
        .expect("ack");
    writer
        .heartbeat_if_due(1000, &view, &mut sink)
        .expect("heartbeat");
    writer
        .control(
            SerialTraceCorrelation {
                epoch: 1,
                request_sequence: 2,
            },
            b"{}",
            |_| true,
            &view,
            &mut sink,
        )
        .expect("reply");
    view.credit.publish(1024);
    writer.receive_credit(&view, &mut sink).expect("credit");
    sink.0.into_inner()
}

#[test]
fn device_heartbeat_matches_the_published_wire_bytes() {
    // Arrange
    let Some(fixture) = maybe_fixture() else {
        return;
    };
    // Act
    let records = published_sequence();
    // Assert
    assert_eq!(records[1], wire(&fixture, "heartbeat"));
}

#[test]
fn device_receive_credit_matches_the_published_wire_bytes() {
    // Arrange
    let Some(fixture) = maybe_fixture() else {
        return;
    };
    // Act
    let records = published_sequence();
    // Assert
    assert_eq!(records[3], wire(&fixture, "receive_credit"));
}

#[test]
fn hello_ack_payload_has_exactly_the_gate_fields() {
    // Arrange
    let binding =
        SerialSessionBinding::parse(SESSION, &"A".repeat(43), &"A".repeat(43)).expect("binding");
    // Act
    let payload = hello_ack_payload(&binding, &"a".repeat(40), "b".repeat(64));
    // Assert
    let mut keys: Vec<_> = payload
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "appElfSha256",
            "deviceNonce",
            "firmwareSourceCommit",
            "hostNonce",
            "op",
            "receiveWindowBytes",
            "receivedBytes",
            "serialManifest"
        ]
    );
    assert_eq!(
        (
            payload["receiveWindowBytes"].as_u64(),
            payload["receivedBytes"].as_u64()
        ),
        (Some(2048), Some(0))
    );
}
