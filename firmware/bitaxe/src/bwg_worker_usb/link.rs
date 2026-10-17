//! Serial RX and heartbeat supervision never wait for Worker effects or TX completion.
//!
//! Link decisions live in the shared `SerialLinkReceiver`; this adapter owns the
//! read loop, the revocation deadline and the firmware effects.

use super::rx_diagnostics::FAILURE;
use super::*;
use bitaxe_worker_control::serial::link::{
    SerialLinkEffects, SerialLinkFailure, SerialLinkReceiver,
};
use zeroize::Zeroizing;

pub(super) fn run() {
    let mut receiver = SerialLinkReceiver::<WorkerGeneration>::new();
    let mut bytes = [0u8; 512];
    loop {
        let now = crate::runtime_uptime::millis();
        revocation::check_deadline(now);
        receiver.supervise(now, &Native);
        let count = match crate::usb_runtime::read(&mut bytes) {
            Ok(count) => count,
            Err(_) => {
                receiver.read_failed(&Native);
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
        };
        receiver.receive(&bytes[..count], &Native);
        bytes.zeroize();
    }
}

struct Native;

impl SerialLinkEffects for Native {
    type Generation = WorkerGeneration;

    fn now_ms(&self) -> u64 {
        crate::runtime_uptime::millis()
    }
    fn current_epoch(&self) -> u32 {
        CURRENT_SESSION.load(Ordering::Acquire)
    }
    fn publish_epoch(&self, epoch: u32) {
        CURRENT_SESSION.store(epoch, Ordering::Release);
    }
    fn authenticated_epoch(&self) -> u32 {
        AUTHENTICATED_SESSION.load(Ordering::Acquire)
    }
    fn revoke_epoch(&self, epoch: u32) {
        revoke_epoch(epoch);
    }
    fn begin_generation(&self, now_ms: u64) -> Option<WorkerGeneration> {
        let maybe_generation = revocation::begin_link(now_ms);
        // A refused Hello found local mining holding the gate: local mining stops
        // and waits for the Gate's next Connect (ADR-0038 step 4).
        crate::production_mining_session::local_owner::note_gate_hello(
            now_ms,
            maybe_generation.is_some(),
        );
        maybe_generation
    }
    fn heartbeat_generation(&self, generation: WorkerGeneration, now_ms: u64) -> bool {
        crate::production_mining_session::local_owner::note_gate_link_activity(now_ms);
        revocation::heartbeat(generation, now_ms)
    }
    fn close_generation(&self, generation: WorkerGeneration, now_ms: u64) {
        revocation::revoke_reason_at(generation, now_ms, revocation::RevocationReason::LinkClosed);
    }
    fn begin_credit(&self, epoch: u32) {
        RECEIVE_CREDIT.begin(epoch);
    }
    fn publish_credit(&self, received_bytes: u32) {
        RECEIVE_CREDIT.publish(received_bytes);
    }
    fn close_credit_record(&self, epoch: u32, received: u32, now_ms: u64, revoke: impl FnOnce()) {
        RECEIVE_CREDIT.close_record(epoch, received, now_ms, revoke);
    }
    fn clear_failure(&self) {
        FAILURE.clear();
    }
    fn record_failure(&self, failure: SerialLinkFailure, observed_bytes: usize) {
        FAILURE.record(failure, observed_bytes);
    }
    fn trace(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    ) {
        trace::event(correlation, stage, wire_bytes);
    }
    fn fill_entropy(&self, bytes: &mut [u8]) -> bool {
        crate::crypto_entropy::fill(bytes).is_ok()
    }
    fn enqueue_session(
        &self,
        epoch: u32,
        binding: SerialSessionBinding,
        generation: WorkerGeneration,
    ) -> bool {
        enqueue(ControlEvent::Session {
            epoch,
            binding,
            generation,
        })
    }
    fn enqueue_control(
        &self,
        correlation: SerialTraceCorrelation,
        wire_bytes: usize,
        mut frame: Zeroizing<Vec<u8>>,
    ) -> bool {
        enqueue(ControlEvent::Frame {
            correlation,
            wire_bytes,
            bytes: SecretBytes(core::mem::take(&mut *frame)),
        })
    }
    fn firmware_source_commit(&self) -> &str {
        crate::firmware_commit()
    }
    fn app_elf_sha256(&self) -> String {
        crate::app_elf_sha256()
    }
    fn send_hello_ack(&self, epoch: u32, session_id: &str, payload: serde_json::Value) -> bool {
        writer::hello(epoch, session_id, payload).is_ok()
    }
}

fn enqueue(event: ControlEvent) -> bool {
    EVENTS
        .get()
        .is_some_and(|events| events.try_send(event).is_ok())
}
