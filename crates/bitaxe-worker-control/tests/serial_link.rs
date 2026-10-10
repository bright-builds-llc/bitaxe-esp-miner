//! Drives the shared inbound link core with published Gate vectors and failed allocations.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;

use bitaxe_worker_control::serial::link::{
    SerialLinkEffects, SerialLinkFailure, SerialLinkReceiver,
};
use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
use bitaxe_worker_control::serial::{
    ReceiveCreditMailbox, SerialEnvelope, SerialKind, SerialSessionBinding,
};
use serde_json::{value::RawValue, Value};
use zeroize::Zeroizing;

/// Zero entropy reproduces the fixtures' all-zero session id.
#[derive(Default)]
struct GateEffects {
    current: Cell<u32>,
    authenticated: Cell<u32>,
    failure: Cell<Option<SerialLinkFailure>>,
    heartbeats: Cell<u32>,
    credit: ReceiveCreditMailbox,
    sessions: RefCell<Vec<String>>,
    controls: RefCell<Vec<Vec<u8>>>,
}

impl SerialLinkEffects for GateEffects {
    type Generation = ();
    fn now_ms(&self) -> u64 {
        0
    }
    fn current_epoch(&self) -> u32 {
        self.current.get()
    }
    fn publish_epoch(&self, epoch: u32) {
        self.current.set(epoch);
    }
    fn authenticated_epoch(&self) -> u32 {
        self.authenticated.get()
    }
    fn revoke_epoch(&self, epoch: u32) {
        if self.current.get() == epoch {
            self.current.set(0);
        }
    }
    fn begin_generation(&self, _now_ms: u64) -> Option<()> {
        Some(())
    }
    fn heartbeat_generation(&self, _generation: (), _now_ms: u64) -> bool {
        self.heartbeats.set(self.heartbeats.get() + 1);
        true
    }
    fn close_generation(&self, _generation: (), _now_ms: u64) {}
    fn begin_credit(&self, epoch: u32) {
        self.credit.begin(epoch);
    }
    fn publish_credit(&self, received_bytes: u32) {
        self.credit.publish(received_bytes);
    }
    fn close_credit_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce()) {
        self.credit.close_record(epoch, received, now_ms, revoke);
    }
    fn clear_failure(&self) {
        self.failure.set(None);
    }
    fn record_failure(&self, failure: SerialLinkFailure, _observed_bytes: usize) {
        if self.failure.get().is_none() {
            self.failure.set(Some(failure));
        }
    }
    fn trace(&self, _: SerialTraceCorrelation, _: SerialTraceStage, _: usize) {}
    fn fill_entropy(&self, bytes: &mut [u8]) -> bool {
        bytes.fill(0);
        true
    }
    fn enqueue_session(&self, _epoch: u32, binding: SerialSessionBinding, _: ()) -> bool {
        self.sessions.borrow_mut().push(binding.session_id);
        true
    }
    fn enqueue_control(
        &self,
        _correlation: SerialTraceCorrelation,
        _wire_bytes: usize,
        frame: Zeroizing<Vec<u8>>,
    ) -> bool {
        self.controls.borrow_mut().push(frame.to_vec());
        true
    }
    fn firmware_source_commit(&self) -> &str {
        "0000000000000000000000000000000000000000"
    }
    fn app_elf_sha256(&self) -> String {
        "0".repeat(64)
    }
    fn send_hello_ack(&self, _epoch: u32, _session_id: &str, _payload: Value) -> bool {
        true
    }
}

fn maybe_fixture(variable: &str) -> Option<Value> {
    let Ok(path) = env::var(variable) else {
        assert_ne!(
            env::var("BWG_REQUIRE_PINNED_FIXTURES").ok().as_deref(),
            Some("1"),
            "{variable} is required"
        );
        return None;
    };
    let text = fs::read_to_string(path).expect("fixture readable");
    Some(serde_json::from_str(&text).expect("fixture JSON"))
}

fn wire<'a>(fixture: &'a Value, id: &str) -> &'a str {
    fixture["frames"]
        .as_array()
        .expect("frames")
        .iter()
        .find(|frame| frame["id"] == id)
        .and_then(|frame| frame["wire"].as_str())
        .expect("published frame")
}

fn admitted(fixture: &Value) -> (SerialLinkReceiver<()>, GateEffects) {
    let effects = GateEffects::default();
    let mut receiver = SerialLinkReceiver::new();
    receiver.receive(wire(fixture, "hello").as_bytes(), &effects);
    (receiver, effects)
}

#[test]
fn hello_vector_admits_the_fixture_session() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    // Act
    let (receiver, effects) = admitted(&fixture);
    // Assert
    assert_eq!(receiver.maybe_epoch(), Some(1));
    assert_eq!(
        effects.sessions.borrow().as_slice(),
        ["AAAAAAAAAAAAAAAAAAAAAA"]
    );
}

#[test]
fn published_stream_in_seven_byte_chunks_drives_the_link() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    let stream: String = ["hello", "heartbeat", "discover"]
        .iter()
        .map(|id| wire(&fixture, id))
        .collect();
    let effects = GateEffects::default();
    let mut receiver = SerialLinkReceiver::new();
    // Act
    for chunk in stream.as_bytes().chunks(7) {
        receiver.receive(chunk, &effects);
    }
    // Assert: the heartbeat precedes admission and refreshes nothing.
    assert_eq!(effects.heartbeats.get(), 0);
    let discover = wire(&fixture, "discover");
    let payload_start = discover.find("\"payload\":").expect("payload") + "\"payload\":".len();
    let expected = format!("{}\n", &discover[payload_start..discover.len() - 2]);
    assert_eq!(
        effects.controls.borrow().as_slice(),
        [expected.into_bytes()]
    );
    let after_hello = (stream.len() - wire(&fixture, "hello").len()) as u32;
    assert_eq!(effects.credit.maybe_received_bytes(1), Some(after_hello));
}

#[test]
fn heartbeat_vector_refreshes_after_admission() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    let (mut receiver, effects) = admitted(&fixture);
    effects.authenticated.set(1);
    // Act
    receiver.receive(wire(&fixture, "heartbeat").as_bytes(), &effects);
    // Assert
    assert_eq!(effects.heartbeats.get(), 1);
}

#[test]
fn inbound_receive_credit_is_an_unexpected_kind() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    let (mut receiver, effects) = admitted(&fixture);
    // Act
    receiver.receive(wire(&fixture, "receive_credit").as_bytes(), &effects);
    // Assert
    assert_eq!(
        effects.failure.get(),
        Some(SerialLinkFailure::UnexpectedKind)
    );
    assert_eq!(receiver.maybe_epoch(), None);
}

#[test]
fn carriage_return_in_a_published_frame_closes_with_envelope() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    let (mut receiver, effects) = admitted(&fixture);
    let mut crlf = wire(&fixture, "discover").as_bytes().to_vec();
    crlf.insert(crlf.len() - 1, b'\r');
    // Act
    receiver.receive(&crlf, &effects);
    // Assert
    assert_eq!(effects.failure.get(), Some(SerialLinkFailure::Envelope));
}

#[test]
fn integrity_vectors_reach_the_owner_lexically_unchanged() {
    // Arrange
    let (Some(serial), Some(integrity)) = (
        maybe_fixture("BWG_SERIAL_FIXTURES"),
        maybe_fixture("BWG_SERIAL_INTEGRITY_FIXTURES"),
    ) else {
        return;
    };
    let (mut receiver, effects) = admitted(&serial);
    let vectors = integrity["vectors"].as_array().expect("vectors");
    // Act
    for (index, vector) in vectors.iter().enumerate() {
        let payload =
            RawValue::from_string(vector["payloadJson"].as_str().expect("payload").to_owned())
                .expect("lexical JSON");
        let record = SerialEnvelope::encode(
            SerialKind::Control,
            Some("AAAAAAAAAAAAAAAAAAAAAA"),
            index as u32 + 1,
            &payload,
        )
        .expect("record");
        receiver.receive(&record, &effects);
    }
    // Assert
    let expected: Vec<Vec<u8>> = vectors
        .iter()
        .map(|vector| {
            format!("{}\n", vector["payloadJson"].as_str().expect("payload")).into_bytes()
        })
        .collect();
    assert_eq!(*effects.controls.borrow(), expected);
}

#[test]
fn corrupted_payload_length_closes_with_integrity() {
    // Arrange
    let Some(fixture) = maybe_fixture("BWG_SERIAL_FIXTURES") else {
        return;
    };
    let (mut receiver, effects) = admitted(&fixture);
    let corrupt =
        wire(&fixture, "discover").replacen("\"payloadBytes\":106", "\"payloadBytes\":0", 1);
    // Act
    receiver.receive(corrupt.as_bytes(), &effects);
    // Assert
    assert_eq!(effects.failure.get(), Some(SerialLinkFailure::Integrity));
}

/// Fails exactly one allocation of the armed size on the arming thread; zero disarms.
struct FailOnce;
thread_local! {
    static ARMED_SIZE: Cell<usize> = const { Cell::new(0) };
}

// SAFETY: delegates every successful request to the system allocator unchanged.
unsafe impl GlobalAlloc for FailOnce {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let fail = ARMED_SIZE
            .try_with(|armed| {
                let hit = size != 0 && armed.get() == size;
                if hit {
                    armed.set(0);
                }
                hit
            })
            .unwrap_or(false);
        if fail {
            return std::ptr::null_mut();
        }
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: FailOnce = FailOnce;

fn padded_control(padding: usize) -> Vec<u8> {
    let payload = RawValue::from_string(format!(r#"{{"padding":"{}"}}"#, "p".repeat(padding)))
        .expect("payload");
    SerialEnvelope::encode(
        SerialKind::Control,
        Some("AAAAAAAAAAAAAAAAAAAAAA"),
        1,
        &payload,
    )
    .expect("record")
}

fn established() -> (SerialLinkReceiver<()>, GateEffects) {
    let host_nonce = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let hello = RawValue::from_string(format!(r#"{{"op":"hello","hostNonce":"{host_nonce}"}}"#))
        .expect("hello");
    let effects = GateEffects::default();
    let mut receiver = SerialLinkReceiver::new();
    let record = SerialEnvelope::encode(SerialKind::Session, None, 0, &hello).expect("hello");
    receiver.receive(&record, &effects);
    (receiver, effects)
}

#[test]
fn control_copy_allocation_failure_records_control_allocation() {
    // Arrange
    let (mut receiver, effects) = established();
    let record = padded_control(3001);
    let payload_bytes = r#"{"padding":""}"#.len() + 3001;
    // Act
    ARMED_SIZE.with(|armed| armed.set(payload_bytes + 1));
    receiver.receive(&record, &effects);
    ARMED_SIZE.with(|armed| armed.set(0));
    // Assert
    assert_eq!(
        effects.failure.get(),
        Some(SerialLinkFailure::ControlAllocation)
    );
    assert!(effects.controls.borrow().is_empty());
}

#[test]
fn frame_copy_allocation_failure_records_framing() {
    // Arrange
    let (mut receiver, effects) = established();
    let record = padded_control(4003);
    // Act
    ARMED_SIZE.with(|armed| armed.set(record.len()));
    receiver.receive(&record, &effects);
    ARMED_SIZE.with(|armed| armed.set(0));
    // Assert
    assert_eq!(effects.failure.get(), Some(SerialLinkFailure::Framing));
    assert_eq!(receiver.maybe_epoch(), None);
}
