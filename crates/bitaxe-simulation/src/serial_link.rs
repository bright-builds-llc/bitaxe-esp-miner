//! Host side of the production serial link: real `bwg-worker-serial/0.2` envelopes
//! cross the shared firmware link receiver before any frame reaches the Controller.
//!
//! The simulated owner drains the bounded queue synchronously after each delivery.
//! Heartbeats the actuation model sends while virtual time advances are queued with
//! their timestamps and replayed through the receiver, in order and at those times,
//! before any later link event; envelope parsing therefore stays out of the deep
//! preparation stack that the target margin audit measures.

use crate::actuation::RuntimeBoardAdapter;
use crate::{CheckStatus, ProfileCheck, SimulationError};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_runtime::revocation::{RevocationReason, WorkerGeneration};
use bitaxe_worker_control::serial::link::{
    SerialLinkEffects, SerialLinkFailure, SerialLinkReceiver,
};
use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
use bitaxe_worker_control::serial::{
    ReceiveCreditMailbox, SerialEnvelope, SerialKind, SerialSessionBinding,
};
use serde_json::{value::RawValue, Value};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use zeroize::Zeroizing;

/// Matches the firmware owner queue (`EVENT_CAPACITY`).
const OWNER_CAPACITY: usize = 4;
const HOST_ENTROPY: u64 = 0x5e71_a1b0_57a7_e5ed;
const DEVICE_ENTROPY: u64 = 0xde71_ce0e_7a1b_1e55;

enum OwnerEvent {
    Session(SerialSessionBinding),
    Frame(Vec<u8>),
}

#[derive(Default)]
struct LinkState {
    current: Cell<u32>,
    authenticated: Cell<u32>,
    failure: Cell<Option<SerialLinkFailure>>,
    credit: ReceiveCreditMailbox,
    device_entropy: Cell<u64>,
    owner: RefCell<VecDeque<OwnerEvent>>,
    maybe_replay_ms: Cell<Option<u64>>,
}

/// Host transport plus the device-side link receiver it feeds.
pub(crate) struct SerialHost {
    receiver: SerialLinkReceiver<WorkerGeneration>,
    state: LinkState,
    host_entropy: u64,
    host_sequence: u32,
    maybe_session_id: Option<String>,
}

impl SerialHost {
    pub(crate) fn new(seed: u64) -> Self {
        let state = LinkState::default();
        state.device_entropy.set(seed ^ DEVICE_ENTROPY);
        Self {
            receiver: SerialLinkReceiver::new(),
            state,
            host_entropy: seed ^ HOST_ENTROPY,
            host_sequence: 0,
            maybe_session_id: None,
        }
    }

    /// Opens a logical session and returns the binding the owner received.
    pub(crate) fn hello(
        &mut self,
        actuation: &mut RuntimeBoardAdapter,
    ) -> Result<SerialSessionBinding, SimulationError> {
        let mut nonce = [0u8; 32];
        fill(&mut self.host_entropy, &mut nonce);
        let payload = format!(
            r#"{{"op":"hello","hostNonce":"{}"}}"#,
            URL_SAFE_NO_PAD.encode(nonce)
        );
        self.replay_heartbeats(actuation);
        self.send(actuation, SerialKind::Session, None, 0, &payload)?;
        let Some(OwnerEvent::Session(binding)) = self.state.owner.borrow_mut().pop_front() else {
            return Err(SimulationError::Boundary("serial_hello"));
        };
        self.host_sequence = 0;
        self.maybe_session_id = Some(binding.session_id.clone());
        Ok(binding)
    }

    /// Sends one Control record and returns the exact owner frame it produced.
    pub(crate) fn deliver(
        &mut self,
        actuation: &mut RuntimeBoardAdapter,
        payload: &[u8],
    ) -> Result<Vec<u8>, SimulationError> {
        let payload = std::str::from_utf8(payload)
            .map_err(|_| SimulationError::Boundary("serial_payload"))?;
        self.send_in_session(actuation, SerialKind::Control, payload)?;
        match self.state.owner.borrow_mut().pop_front() {
            Some(OwnerEvent::Frame(frame)) => Ok(frame),
            _ => Err(SimulationError::Boundary(self.closed_category())),
        }
    }

    /// The owner confirmed possession: admitted heartbeats now refresh liveness.
    pub(crate) fn authenticate(&self) {
        self.state.authenticated.set(self.state.current.get());
    }

    /// Replays queued heartbeats, then applies the link deadline at the current time.
    pub(crate) fn supervise(&mut self, actuation: &mut RuntimeBoardAdapter) {
        self.replay_heartbeats(actuation);
        let effects = SimEffects {
            state: &self.state,
            actuation,
        };
        self.receiver.supervise(effects.now_ms(), &effects);
    }

    /// Supervises, sends a clean Close and reports whether the link held and closed.
    pub(crate) fn close_check(&mut self, actuation: &mut RuntimeBoardAdapter) -> ProfileCheck {
        self.supervise(actuation);
        let maybe_epoch = self.receiver.maybe_epoch();
        let close = r#"{"op":"close","reason":"tab_closed"}"#;
        let sent = maybe_epoch.is_some()
            && self
                .send_in_session(actuation, SerialKind::Session, close)
                .is_ok();
        let terminal = maybe_epoch.and_then(|epoch| {
            self.state
                .credit
                .maybe_terminal_bytes(epoch, actuation.now_ms())
        });
        let held = sent && terminal.is_some() && self.receiver.maybe_epoch().is_none();
        ProfileCheck {
            id: "serial_link".into(),
            status: if held && self.state.failure.get().is_none() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: match self.state.failure.get() {
                Some(failure) => failure.label().into(),
                None if held => "framed_link_closed_with_terminal_receipt".into(),
                None => "link_not_open_at_close".into(),
            },
        }
    }

    pub(crate) fn maybe_failure(&self) -> Option<SerialLinkFailure> {
        self.state.failure.get()
    }

    fn closed_category(&self) -> &'static str {
        self.state
            .failure
            .get()
            .map_or("serial_link_closed", SerialLinkFailure::label)
    }

    fn send_in_session(
        &mut self,
        actuation: &mut RuntimeBoardAdapter,
        kind: SerialKind,
        payload: &str,
    ) -> Result<(), SimulationError> {
        // Earlier heartbeats precede this record on the wire and take earlier sequences.
        self.replay_heartbeats(actuation);
        let session = self
            .maybe_session_id
            .clone()
            .ok_or(SimulationError::Boundary("serial_session_missing"))?;
        self.host_sequence = self
            .host_sequence
            .checked_add(1)
            .ok_or(SimulationError::Boundary("serial_sequence"))?;
        let sequence = self.host_sequence;
        self.send(actuation, kind, Some(&session), sequence, payload)
    }

    fn send(
        &mut self,
        actuation: &mut RuntimeBoardAdapter,
        kind: SerialKind,
        maybe_session: Option<&str>,
        sequence: u32,
        payload: &str,
    ) -> Result<(), SimulationError> {
        let raw = RawValue::from_string(payload.to_owned())?;
        let record = SerialEnvelope::encode(kind, maybe_session, sequence, &raw)
            .map_err(|_| SimulationError::Boundary("serial_encode"))?;
        let effects = SimEffects {
            state: &self.state,
            actuation,
        };
        self.receiver.receive(&record, &effects);
        Ok(())
    }

    /// Feeds each queued heartbeat at the instant the host sent it.
    fn replay_heartbeats(&mut self, actuation: &mut RuntimeBoardAdapter) {
        let instants = std::mem::take(&mut actuation.link_heartbeats);
        let Some(session) = self.maybe_session_id.clone() else {
            return;
        };
        for at_ms in instants {
            let Some(sequence) = self.host_sequence.checked_add(1) else {
                return;
            };
            self.host_sequence = sequence;
            let Ok(record) = SerialEnvelope::encode(
                SerialKind::Heartbeat,
                Some(&session),
                sequence,
                &RawValue::from_string("{}".to_owned()).expect("constant heartbeat payload"),
            ) else {
                return;
            };
            self.state.maybe_replay_ms.set(Some(at_ms));
            let effects = SimEffects {
                state: &self.state,
                actuation,
            };
            self.receiver.supervise(at_ms, &effects);
            self.receiver.receive(&record, &effects);
            self.state.maybe_replay_ms.set(None);
        }
    }
}

/// Device effects over the simulated board; one value per receiver call.
struct SimEffects<'a> {
    state: &'a LinkState,
    actuation: &'a RuntimeBoardAdapter,
}

impl SerialLinkEffects for SimEffects<'_> {
    type Generation = WorkerGeneration;

    fn now_ms(&self) -> u64 {
        self.state
            .maybe_replay_ms
            .get()
            .unwrap_or_else(|| self.actuation.now_ms())
    }
    fn current_epoch(&self) -> u32 {
        self.state.current.get()
    }
    fn publish_epoch(&self, epoch: u32) {
        self.state.current.set(epoch);
    }
    fn authenticated_epoch(&self) -> u32 {
        self.state.authenticated.get()
    }
    fn revoke_epoch(&self, epoch: u32) {
        if self.state.current.get() == epoch {
            self.state.current.set(0);
            self.state.credit.close(epoch);
        }
    }
    /// The simulated board begins its generation at boot; Hello binds that generation.
    fn begin_generation(&self, _now_ms: u64) -> Option<WorkerGeneration> {
        Some(self.actuation.generation)
    }
    /// The actuation model already applied this heartbeat to the gate at the same instant.
    fn heartbeat_generation(&self, _generation: WorkerGeneration, _now_ms: u64) -> bool {
        true
    }
    fn close_generation(&self, generation: WorkerGeneration, now_ms: u64) {
        self.actuation
            .gate
            .revoke_reason_at(generation, now_ms, RevocationReason::LinkClosed);
    }
    fn begin_credit(&self, epoch: u32) {
        self.state.credit.begin(epoch);
    }
    fn publish_credit(&self, received_bytes: u32) {
        self.state.credit.publish(received_bytes);
    }
    fn close_credit_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce()) {
        self.state
            .credit
            .close_record(epoch, received, now_ms, revoke);
    }
    fn clear_failure(&self) {
        self.state.failure.set(None);
    }
    fn record_failure(&self, failure: SerialLinkFailure, _observed_bytes: usize) {
        if self.state.failure.get().is_none() {
            self.state.failure.set(Some(failure));
        }
    }
    fn trace(&self, _: SerialTraceCorrelation, _: SerialTraceStage, _: usize) {}
    fn fill_entropy(&self, bytes: &mut [u8]) -> bool {
        let mut state = self.state.device_entropy.get();
        fill(&mut state, bytes);
        self.state.device_entropy.set(state);
        true
    }
    fn enqueue_session(
        &self,
        _epoch: u32,
        binding: SerialSessionBinding,
        _: WorkerGeneration,
    ) -> bool {
        self.enqueue(OwnerEvent::Session(binding))
    }
    fn enqueue_control(
        &self,
        _correlation: SerialTraceCorrelation,
        _wire_bytes: usize,
        mut frame: Zeroizing<Vec<u8>>,
    ) -> bool {
        self.enqueue(OwnerEvent::Frame(std::mem::take(&mut *frame)))
    }
    fn firmware_source_commit(&self) -> &str {
        crate::controller::FIRMWARE_SOURCE_COMMIT
    }
    fn app_elf_sha256(&self) -> String {
        crate::controller::APP_ELF_SHA256.to_owned()
    }
    fn send_hello_ack(&self, _epoch: u32, _session_id: &str, _payload: Value) -> bool {
        true
    }
}

impl SimEffects<'_> {
    fn enqueue(&self, event: OwnerEvent) -> bool {
        let mut owner = self.state.owner.borrow_mut();
        if owner.len() >= OWNER_CAPACITY {
            return false;
        }
        owner.push_back(event);
        true
    }
}

/// SplitMix64: deterministic per-seed nonces that never touch the board scheduler.
fn fill(state: &mut u64, bytes: &mut [u8]) {
    for chunk in bytes.chunks_mut(8) {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = *state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^= value >> 31;
        chunk.copy_from_slice(&value.to_le_bytes()[..chunk.len()]);
    }
}

#[cfg(test)]
mod tests;
