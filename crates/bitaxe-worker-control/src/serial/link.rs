//! Inbound serial link decisions shared by the firmware RX thread and host simulation.
//!
//! The receiver owns framing, Hello admission, sequence, heartbeat liveness and
//! receive-credit accounting. Every effect it needs (clock, entropy, revocation,
//! the Worker owner queue, the writer) goes through [`SerialLinkEffects`], which
//! callers implement with static dispatch so the firmware loop keeps its stack
//! shape and allocation profile.

mod failure;

pub use failure::SerialLinkFailure;

use super::trace::{SerialTraceCorrelation, SerialTraceStage};
use super::{
    canonical_nonce, serial_manifest, SerialEnvelope, SerialError, SerialFrameAccumulator,
    SerialKind, SerialLinkLiveness, SerialReceiveProgress, SerialSessionBinding,
    HOST_RECEIVE_WINDOW_BYTES, MAXIMUM_WIRE_FRAME_BYTES,
};
use serde::Deserialize;
use serde_json::{json, Value};
use zeroize::Zeroizing;

/// The owner-queue copy of an admitted control payload.
pub const CONTROL_COPY_SITE: bitaxe_core::fallible::Site =
    bitaxe_core::fallible::Site("serial.control_copy");

/// Effects the link receiver performs; each call mirrors one firmware collaborator.
pub trait SerialLinkEffects {
    /// Opaque revocation generation bound to one admitted link.
    type Generation: Copy;

    fn now_ms(&self) -> u64;
    fn current_epoch(&self) -> u32;
    fn publish_epoch(&self, epoch: u32);
    fn authenticated_epoch(&self) -> u32;
    fn revoke_epoch(&self, epoch: u32);
    fn begin_generation(&self, now_ms: u64) -> Option<Self::Generation>;
    /// Returns whether work stays live; cooling may revoke work while the link stays open.
    fn heartbeat_generation(&self, generation: Self::Generation, now_ms: u64) -> bool;
    fn close_generation(&self, generation: Self::Generation, now_ms: u64);
    fn begin_credit(&self, epoch: u32);
    fn publish_credit(&self, received_bytes: u32);
    fn close_credit_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce());
    fn clear_failure(&self);
    fn record_failure(&self, failure: SerialLinkFailure, observed_bytes: usize);
    fn trace(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    );
    fn fill_entropy(&self, bytes: &mut [u8]) -> bool;
    fn enqueue_session(
        &self,
        epoch: u32,
        binding: SerialSessionBinding,
        generation: Self::Generation,
    ) -> bool;
    fn enqueue_control(
        &self,
        correlation: SerialTraceCorrelation,
        wire_bytes: usize,
        frame: Zeroizing<Vec<u8>>,
    ) -> bool;
    fn firmware_source_commit(&self) -> &str;
    fn app_elf_sha256(&self) -> String;
    fn send_hello_ack(&self, epoch: u32, session_id: &str, payload: Value) -> bool;
}

struct Link<G> {
    epoch: u32,
    binding: SerialSessionBinding,
    generation: G,
    sequence: u32,
    liveness: SerialLinkLiveness,
    received: SerialReceiveProgress,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Hello {
    op: String,
    host_nonce: String,
}

/// One logical serial connection at a time; serial input never waits for Worker effects.
pub struct SerialLinkReceiver<G> {
    accumulator: SerialFrameAccumulator,
    maybe_link: Option<Link<G>>,
    next_epoch: u32,
    observed_bytes: usize,
}

impl<G: Copy> Default for SerialLinkReceiver<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Copy> SerialLinkReceiver<G> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            accumulator: SerialFrameAccumulator::default(),
            maybe_link: None,
            next_epoch: 0,
            observed_bytes: 0,
        }
    }

    /// The admitted link's epoch, if one is open.
    #[must_use]
    pub fn maybe_epoch(&self) -> Option<u32> {
        self.maybe_link.as_ref().map(|link| link.epoch)
    }

    /// Closes a link whose epoch was revoked or whose heartbeat deadline passed.
    ///
    /// The caller runs its revocation deadline check with the same `now_ms` first.
    pub fn supervise<E: SerialLinkEffects<Generation = G>>(&mut self, now_ms: u64, effects: &E) {
        let Some(link) = self.maybe_link.as_mut() else {
            return;
        };
        let failure = if effects.current_epoch() != link.epoch {
            SerialLinkFailure::SessionRevoked
        } else if !link.liveness.poll(now_ms) {
            SerialLinkFailure::HeartbeatTimeout
        } else {
            return;
        };
        effects.record_failure(failure, self.observed_bytes);
        self.close(effects);
        self.accumulator.clear();
        self.observed_bytes = 0;
    }

    /// A transport read failed: the link and any partial record are discarded.
    pub fn read_failed<E: SerialLinkEffects<Generation = G>>(&mut self, effects: &E) {
        if self.maybe_link.is_some() {
            effects.record_failure(SerialLinkFailure::Read, self.observed_bytes);
        }
        self.close(effects);
        self.accumulator.clear();
        self.observed_bytes = 0;
    }

    /// Consumes one transport read in order.
    pub fn receive<E: SerialLinkEffects<Generation = G>>(&mut self, bytes: &[u8], effects: &E) {
        for byte in bytes {
            self.receive_byte(*byte, effects);
        }
    }

    fn receive_byte<E: SerialLinkEffects<Generation = G>>(&mut self, byte: u8, effects: &E) {
        // Account per byte so a Hello LF inside this read batch establishes
        // the exact next epoch's counter origin. Partial frames earn credit.
        if let Some(link) = self.maybe_link.as_mut() {
            let Some(received) = link.received.advance(1) else {
                effects.record_failure(SerialLinkFailure::ReceiveCounter, self.observed_bytes);
                self.close(effects);
                self.accumulator.clear();
                self.observed_bytes = 0;
                return;
            };
            // The LF is credited after classification so a validated Close
            // has one strictly newer terminal receipt, never a duplicate.
            if byte != b'\n' {
                effects.publish_credit(received);
            }
        }
        self.observed_bytes = self
            .observed_bytes
            .saturating_add(1)
            .min(MAXIMUM_WIRE_FRAME_BYTES);
        let Some(result) = self.accumulator.push_byte(byte) else {
            return;
        };
        let frame_bytes = self.observed_bytes;
        self.observed_bytes = 0;
        let Ok(frame) = result else {
            if let Some(link) = self.maybe_link.as_ref() {
                rejected(effects, link.epoch, frame_bytes);
                effects.record_failure(SerialLinkFailure::Framing, frame_bytes);
            }
            self.close(effects);
            return;
        };
        let frame = Zeroizing::new(frame);
        let envelope = match SerialEnvelope::parse(&frame) {
            Ok(envelope) => envelope,
            Err(error) => {
                // Boot text is permissible before hello; corrupt established input revokes.
                if let Some(link) = self.maybe_link.as_ref() {
                    rejected(effects, link.epoch, frame_bytes);
                    let failure = if error == SerialError::Integrity {
                        SerialLinkFailure::Integrity
                    } else {
                        SerialLinkFailure::Envelope
                    };
                    effects.record_failure(failure, frame_bytes);
                    self.close(effects);
                }
                return;
            }
        };
        self.receive_record(envelope, frame_bytes, effects);
    }

    fn receive_record<E: SerialLinkEffects<Generation = G>>(
        &mut self,
        envelope: SerialEnvelope,
        frame_bytes: usize,
        effects: &E,
    ) {
        if envelope.kind == SerialKind::Session
            && envelope.session_id.is_none()
            && envelope.sequence == 0
        {
            self.close(effects);
            if frame_bytes <= HOST_RECEIVE_WINDOW_BYTES as usize {
                self.maybe_link = hello(envelope, &mut self.next_epoch, frame_bytes, effects);
            }
            return;
        }
        let Some(link) = self.maybe_link.as_mut() else {
            return;
        };
        if envelope.session_id.as_deref() != Some(link.binding.session_id.as_str())
            || envelope.sequence <= link.sequence
        {
            rejected(effects, link.epoch, frame_bytes);
            effects.record_failure(SerialLinkFailure::Sequence, frame_bytes);
            self.close(effects);
            return;
        }
        link.sequence = envelope.sequence;
        if envelope.sequence == u32::MAX {
            rejected(effects, link.epoch, frame_bytes);
            effects.record_failure(SerialLinkFailure::Sequence, frame_bytes);
            self.close(effects);
            return;
        }
        let clean_close = envelope.kind == SerialKind::Session && envelope.is_close();
        let correlation = SerialTraceCorrelation {
            epoch: link.epoch,
            request_sequence: envelope.sequence,
        };
        if envelope.kind == SerialKind::Control {
            effects.trace(correlation, SerialTraceStage::Validated, frame_bytes);
        }
        if !clean_close {
            effects.publish_credit(link.received.received_bytes());
        }
        match envelope.kind {
            SerialKind::Heartbeat => self.heartbeat(&envelope, frame_bytes, effects),
            SerialKind::Control => self.control(&envelope, correlation, frame_bytes, effects),
            SerialKind::Session if clean_close => {
                let epoch = link.epoch;
                let received = link.received.received_bytes();
                let maybe_link = &mut self.maybe_link;
                effects.close_credit_record(epoch, received, effects.now_ms(), || {
                    close_link(maybe_link, effects);
                });
            }
            SerialKind::Session | SerialKind::Diagnostic => {
                effects.record_failure(SerialLinkFailure::UnexpectedKind, frame_bytes);
                self.close(effects);
            }
        }
    }

    fn heartbeat<E: SerialLinkEffects<Generation = G>>(
        &mut self,
        envelope: &SerialEnvelope,
        frame_bytes: usize,
        effects: &E,
    ) {
        if !envelope
            .payload
            .get()
            .bytes()
            .filter(|byte| !byte.is_ascii_whitespace())
            .eq(b"{}".iter().copied())
        {
            effects.record_failure(SerialLinkFailure::HeartbeatPayload, frame_bytes);
            self.close(effects);
            return;
        }
        let Some(link) = self.maybe_link.as_mut() else {
            return;
        };
        // A host can receive the proof just before the owner confirms TX.
        // Pre-admission heartbeats never refresh the deadline.
        if effects.authenticated_epoch() != link.epoch {
            return;
        }
        link.liveness.authenticate();
        let now = effects.now_ms();
        if !link.liveness.heartbeat(now) {
            effects.record_failure(SerialLinkFailure::HeartbeatTimeout, frame_bytes);
            self.close(effects);
            return;
        }
        // Cooling revokes work while the authenticated reply channel stays live.
        let _work_still_live = effects.heartbeat_generation(link.generation, now);
    }

    fn control<E: SerialLinkEffects<Generation = G>>(
        &mut self,
        envelope: &SerialEnvelope,
        correlation: SerialTraceCorrelation,
        frame_bytes: usize,
        effects: &E,
    ) {
        effects.trace(correlation, SerialTraceStage::EnqueueStarted, frame_bytes);
        let payload = envelope.payload.get().as_bytes();
        let mut frame = Vec::new();
        if bitaxe_core::fallible::reserve_exact(&mut frame, payload.len() + 1, CONTROL_COPY_SITE)
            .is_err()
        {
            effects.trace(correlation, SerialTraceStage::EnqueueRejected, frame_bytes);
            effects.record_failure(SerialLinkFailure::ControlAllocation, frame_bytes);
            self.close(effects);
            return;
        }
        frame.extend_from_slice(payload);
        frame.push(b'\n');
        if !effects.enqueue_control(correlation, frame_bytes, Zeroizing::new(frame)) {
            effects.trace(correlation, SerialTraceStage::EnqueueRejected, frame_bytes);
            effects.record_failure(SerialLinkFailure::ControlQueue, frame_bytes);
            self.close(effects);
        } else {
            effects.trace(correlation, SerialTraceStage::Enqueued, frame_bytes);
        }
    }

    fn close<E: SerialLinkEffects<Generation = G>>(&mut self, effects: &E) {
        close_link(&mut self.maybe_link, effects);
    }

    #[cfg(test)]
    pub(crate) fn with_next_epoch(mut self, next_epoch: u32) -> Self {
        self.next_epoch = next_epoch;
        self
    }

    #[cfg(test)]
    pub(crate) fn force_received(&mut self, received: u32) {
        if let Some(link) = self.maybe_link.as_mut() {
            link.received = SerialReceiveProgress::at(received);
        }
    }
}

fn hello<G: Copy, E: SerialLinkEffects<Generation = G>>(
    envelope: SerialEnvelope,
    next_epoch: &mut u32,
    wire_bytes: usize,
    effects: &E,
) -> Option<Link<G>> {
    let hello: Hello = serde_json::from_str(envelope.payload.get()).ok()?;
    if hello.op != "hello" || !canonical_nonce(&hello.host_nonce, 32) {
        return None;
    }
    *next_epoch = next_epoch.checked_add(1)?;
    let binding = SerialSessionBinding::parse(
        &device_nonce::<16, _>(effects)?,
        &hello.host_nonce,
        &device_nonce::<32, _>(effects)?,
    )
    .ok()?;
    let generation = effects.begin_generation(effects.now_ms())?;
    let epoch = *next_epoch;
    effects.clear_failure();
    effects.begin_credit(epoch);
    effects.publish_epoch(epoch);
    effects.trace(
        SerialTraceCorrelation {
            epoch,
            request_sequence: 0,
        },
        SerialTraceStage::Hello,
        wire_bytes,
    );
    if !effects.enqueue_session(epoch, binding.clone(), generation) {
        effects.close_generation(generation, effects.now_ms());
        effects.revoke_epoch(epoch);
        return None;
    }
    let payload = hello_ack_payload(
        &binding,
        effects.firmware_source_commit(),
        effects.app_elf_sha256(),
    );
    if !effects.send_hello_ack(epoch, &binding.session_id, payload) {
        effects.close_generation(generation, effects.now_ms());
        effects.revoke_epoch(epoch);
        return None;
    }
    Some(Link {
        epoch,
        binding,
        generation,
        sequence: 0,
        liveness: SerialLinkLiveness::new(effects.now_ms()),
        received: SerialReceiveProgress::default(),
    })
}

/// The device's Hello acknowledgement for an admitted binding.
#[must_use]
pub fn hello_ack_payload(
    binding: &SerialSessionBinding,
    firmware_source_commit: &str,
    app_elf_sha256: String,
) -> Value {
    json!({
        "op": "hello_ack", "hostNonce": binding.host_nonce, "deviceNonce": binding.device_nonce,
        "serialManifest": serial_manifest(), "firmwareSourceCommit": firmware_source_commit,
        "appElfSha256": app_elf_sha256,
        "receiveWindowBytes": HOST_RECEIVE_WINDOW_BYTES, "receivedBytes": 0,
    })
}

fn rejected<E: SerialLinkEffects>(effects: &E, epoch: u32, wire_bytes: usize) {
    effects.trace(
        SerialTraceCorrelation {
            epoch,
            request_sequence: 0,
        },
        SerialTraceStage::ValidationRejected,
        wire_bytes,
    );
}

fn close_link<G: Copy, E: SerialLinkEffects<Generation = G>>(
    maybe_link: &mut Option<Link<G>>,
    effects: &E,
) {
    if let Some(link) = maybe_link.take() {
        effects.close_generation(link.generation, effects.now_ms());
        effects.revoke_epoch(link.epoch);
    }
}

/// Canonical unpadded base64url of `N` fresh entropy bytes.
fn device_nonce<const N: usize, E: SerialLinkEffects>(effects: &E) -> Option<String> {
    let mut bytes = Zeroizing::new([0u8; N]);
    if !effects.fill_entropy(bytes.as_mut()) {
        return None;
    }
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    for byte in bytes.iter().copied() {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            result.push(char::from(ALPHABET[((buffer >> bits) & 63) as usize]));
        }
    }
    if bits != 0 {
        result.push(char::from(ALPHABET[((buffer << (6 - bits)) & 63) as usize]));
    }
    Some(result)
}

#[cfg(test)]
mod tests;
