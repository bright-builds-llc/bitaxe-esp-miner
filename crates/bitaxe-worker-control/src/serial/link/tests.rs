use super::*;
use crate::serial::{ReceiveCreditMailbox, SerialEnvelope};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::value::RawValue;
use std::cell::{Cell, RefCell};

const HOST_NONCE: [u8; 32] = [7; 32];

/// Records every effect in order; knobs script collaborator results.
#[derive(Default)]
struct Recording {
    now: Cell<u64>,
    current: Cell<u32>,
    authenticated: Cell<u32>,
    entropy: Cell<u8>,
    next_generation: Cell<u32>,
    entropy_fails: Cell<bool>,
    begin_fails: Cell<bool>,
    session_rejected: Cell<bool>,
    control_rejected: Cell<bool>,
    ack_fails: Cell<bool>,
    cooling: Cell<bool>,
    failure: Cell<Option<(SerialLinkFailure, usize)>>,
    credit: ReceiveCreditMailbox,
    controls: RefCell<Vec<Vec<u8>>>,
    log: RefCell<Vec<String>>,
}

impl Recording {
    fn note(&self, line: String) {
        self.log.borrow_mut().push(line);
    }
    fn lines(&self, prefix: &str) -> Vec<String> {
        self.log
            .borrow()
            .iter()
            .filter(|line| line.starts_with(prefix))
            .cloned()
            .collect()
    }
    fn failure(&self) -> Option<SerialLinkFailure> {
        self.failure.get().map(|(failure, _)| failure)
    }
}

impl SerialLinkEffects for Recording {
    type Generation = u32;
    fn now_ms(&self) -> u64 {
        self.now.get()
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
        self.note(format!("revoke_epoch {epoch}"));
        if self.current.get() == epoch {
            self.current.set(0);
            self.credit.close(epoch);
        }
    }
    fn begin_generation(&self, now_ms: u64) -> Option<u32> {
        self.note(format!("begin_generation {now_ms}"));
        if self.begin_fails.get() {
            return None;
        }
        let generation = self.next_generation.get() + 1;
        self.next_generation.set(generation);
        Some(generation)
    }
    fn heartbeat_generation(&self, generation: u32, now_ms: u64) -> bool {
        self.note(format!("heartbeat_generation {generation} {now_ms}"));
        !self.cooling.get()
    }
    fn close_generation(&self, generation: u32, now_ms: u64) {
        self.note(format!("close_generation {generation} {now_ms}"));
    }
    fn begin_credit(&self, epoch: u32) {
        self.note(format!("begin_credit {epoch}"));
        self.credit.begin(epoch);
    }
    fn publish_credit(&self, received_bytes: u32) {
        self.credit.publish(received_bytes);
    }
    fn close_credit_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce()) {
        self.note(format!("close_credit_record {epoch} {received}"));
        self.credit.close_record(epoch, received, now_ms, revoke);
    }
    fn clear_failure(&self) {
        self.failure.set(None);
    }
    fn record_failure(&self, failure: SerialLinkFailure, observed_bytes: usize) {
        if self.failure.get().is_none() {
            self.failure.set(Some((failure, observed_bytes)));
        }
    }
    fn trace(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    ) {
        self.note(format!(
            "trace {stage:?} {} {} {wire_bytes}",
            correlation.epoch, correlation.request_sequence
        ));
    }
    fn fill_entropy(&self, bytes: &mut [u8]) -> bool {
        if self.entropy_fails.get() {
            return false;
        }
        for byte in bytes {
            *byte = self.entropy.get();
            self.entropy.set(self.entropy.get().wrapping_add(1));
        }
        true
    }
    fn enqueue_session(&self, epoch: u32, binding: SerialSessionBinding, generation: u32) -> bool {
        self.note(format!(
            "enqueue_session {epoch} {} {generation}",
            binding.session_id
        ));
        !self.session_rejected.get()
    }
    fn enqueue_control(
        &self,
        correlation: SerialTraceCorrelation,
        wire_bytes: usize,
        frame: Zeroizing<Vec<u8>>,
    ) -> bool {
        self.note(format!(
            "enqueue_control {} {} {wire_bytes} capacity={}",
            correlation.epoch,
            correlation.request_sequence,
            frame.capacity()
        ));
        if self.control_rejected.get() {
            return false;
        }
        self.controls.borrow_mut().push(frame.to_vec());
        true
    }
    fn firmware_source_commit(&self) -> &str {
        "c0ffee"
    }
    fn app_elf_sha256(&self) -> String {
        "e".repeat(64)
    }
    fn send_hello_ack(&self, epoch: u32, session_id: &str, payload: Value) -> bool {
        self.note(format!("hello_ack {epoch} {session_id} {payload}"));
        !self.ack_fails.get()
    }
}

type Receiver = SerialLinkReceiver<u32>;

/// Session id of the nth admitted Hello under counting entropy (48 bytes per Hello).
fn session(hello: u8) -> String {
    let start = 48 * hello;
    URL_SAFE_NO_PAD.encode((start..start + 16).collect::<Vec<u8>>())
}

fn record(kind: SerialKind, maybe_session: Option<&str>, sequence: u32, payload: &str) -> Vec<u8> {
    let payload = RawValue::from_string(payload.to_owned()).expect("raw payload");
    SerialEnvelope::encode(kind, maybe_session, sequence, &payload).expect("encodable")
}

fn hello_payload() -> String {
    format!(
        r#"{{"op":"hello","hostNonce":"{}"}}"#,
        URL_SAFE_NO_PAD.encode(HOST_NONCE)
    )
}

fn hello_record() -> Vec<u8> {
    record(SerialKind::Session, None, 0, &hello_payload())
}

fn heartbeat(sequence: u32) -> Vec<u8> {
    record(SerialKind::Heartbeat, Some(&session(0)), sequence, "{}")
}

fn control(sequence: u32) -> Vec<u8> {
    record(
        SerialKind::Control,
        Some(&session(0)),
        sequence,
        r#"{"command":"status"}"#,
    )
}

fn admitted() -> (Receiver, Recording) {
    let effects = Recording::default();
    let mut receiver = Receiver::new();
    receiver.receive(&hello_record(), &effects);
    assert_eq!(
        receiver.maybe_epoch(),
        Some(1),
        "fixture Hello admits epoch 1"
    );
    (receiver, effects)
}

fn authenticated() -> (Receiver, Recording) {
    let (receiver, effects) = admitted();
    effects.authenticated.set(1);
    (receiver, effects)
}

mod admission;
mod established;
