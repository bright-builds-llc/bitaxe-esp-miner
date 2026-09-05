//! Serial RX and heartbeat supervision never wait for Worker effects or TX completion.

use super::rx_diagnostics::{Stage, FAILURE};
use super::*;
use bitaxe_worker_control::serial::{
    canonical_nonce, serial_manifest, SerialEnvelope, SerialFrameAccumulator, SerialLinkLiveness,
};
use serde::Deserialize;
use serde_json::json;
use zeroize::Zeroizing;

struct Link {
    epoch: u32,
    binding: SerialSessionBinding,
    generation: WorkerGeneration,
    sequence: u32,
    liveness: SerialLinkLiveness,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Hello {
    op: String,
    host_nonce: String,
}

pub(super) fn run() {
    let mut accumulator = SerialFrameAccumulator::default();
    let mut maybe_link: Option<Link> = None;
    let mut next_epoch = 0u32;
    let mut observed_bytes = 0usize;
    let mut bytes = [0u8; 512];
    loop {
        let now = crate::runtime_uptime::millis();
        revocation::check_deadline(now);
        if let Some(link) = maybe_link.as_mut() {
            let stage = if CURRENT_SESSION.load(Ordering::Acquire) != link.epoch {
                Some(Stage::SessionRevoked)
            } else if !link.liveness.poll(now) {
                Some(Stage::HeartbeatTimeout)
            } else {
                None
            };
            if let Some(stage) = stage {
                FAILURE.record(stage, observed_bytes);
                close(&mut maybe_link);
                accumulator.clear();
                observed_bytes = 0;
            }
        }
        let count = match crate::usb_runtime::read(&mut bytes) {
            Ok(count) => count,
            Err(_) => {
                if maybe_link.is_some() {
                    FAILURE.record(Stage::Read, observed_bytes);
                }
                close(&mut maybe_link);
                accumulator.clear();
                observed_bytes = 0;
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
        };
        for byte in &bytes[..count] {
            observed_bytes = observed_bytes.saturating_add(1).min(66560);
            let Some(result) = accumulator.push_byte(*byte) else {
                continue;
            };
            let frame_bytes = observed_bytes;
            observed_bytes = 0;
            let Ok(frame) = result else {
                if maybe_link.is_some() {
                    FAILURE.record(Stage::Framing, frame_bytes);
                }
                close(&mut maybe_link);
                continue;
            };
            let frame = Zeroizing::new(frame);
            let Ok(envelope) = SerialEnvelope::parse(&frame) else {
                // Boot text is permissible before hello; malformed established input revokes.
                if maybe_link.is_some() {
                    FAILURE.record(Stage::Envelope, frame_bytes);
                    close(&mut maybe_link);
                }
                continue;
            };
            if envelope.kind == SerialKind::Session
                && envelope.session_id.is_none()
                && envelope.sequence == 0
            {
                close(&mut maybe_link);
                maybe_link = hello(envelope, &mut next_epoch);
                continue;
            }
            let Some(link) = maybe_link.as_mut() else {
                continue;
            };
            if envelope.session_id.as_deref() != Some(link.binding.session_id.as_str())
                || envelope.sequence <= link.sequence
            {
                FAILURE.record(Stage::Sequence, frame_bytes);
                close(&mut maybe_link);
                continue;
            }
            link.sequence = envelope.sequence;
            if envelope.sequence == u32::MAX {
                FAILURE.record(Stage::Sequence, frame_bytes);
                close(&mut maybe_link);
                continue;
            }
            match envelope.kind {
                SerialKind::Heartbeat => {
                    if !envelope
                        .payload
                        .get()
                        .bytes()
                        .filter(|byte| !byte.is_ascii_whitespace())
                        .eq(b"{}".iter().copied())
                    {
                        FAILURE.record(Stage::HeartbeatPayload, frame_bytes);
                        close(&mut maybe_link);
                        continue;
                    }
                    // A host can receive the proof just before the owner confirms TX.
                    // Pre-admission heartbeats never refresh the deadline.
                    if AUTHENTICATED_SESSION.load(Ordering::Acquire) != link.epoch {
                        continue;
                    }
                    link.liveness.authenticate();
                    let now = crate::runtime_uptime::millis();
                    if !link.liveness.heartbeat(now) {
                        FAILURE.record(Stage::HeartbeatTimeout, frame_bytes);
                        close(&mut maybe_link);
                        continue;
                    }
                    // Cooling revokes work while the authenticated reply channel stays live.
                    let _work_still_live = revocation::heartbeat(link.generation, now);
                }
                SerialKind::Control => {
                    let payload = envelope.payload.get().as_bytes();
                    let mut frame = Vec::new();
                    if frame.try_reserve_exact(payload.len() + 1).is_err() {
                        FAILURE.record(Stage::ControlAllocation, frame_bytes);
                        close(&mut maybe_link);
                        continue;
                    }
                    frame.extend_from_slice(payload);
                    frame.push(b'\n');
                    let event = ControlEvent::Frame {
                        epoch: link.epoch,
                        bytes: SecretBytes(frame),
                    };
                    if !enqueue(event) {
                        FAILURE.record(Stage::ControlQueue, frame_bytes);
                        close(&mut maybe_link);
                    }
                }
                SerialKind::Session => {
                    if !envelope.is_close() {
                        FAILURE.record(Stage::UnexpectedKind, frame_bytes);
                    }
                    close(&mut maybe_link);
                }
                SerialKind::Diagnostic => {
                    FAILURE.record(Stage::UnexpectedKind, frame_bytes);
                    close(&mut maybe_link);
                }
            }
        }
        bytes.zeroize();
    }
}

fn hello(envelope: SerialEnvelope, next_epoch: &mut u32) -> Option<Link> {
    let hello: Hello = serde_json::from_str(envelope.payload.get()).ok()?;
    if hello.op != "hello" || !canonical_nonce(&hello.host_nonce, 32) {
        return None;
    }
    *next_epoch = next_epoch.checked_add(1)?;
    let binding = SerialSessionBinding::parse(
        &random_nonce::<16>()?,
        &hello.host_nonce,
        &random_nonce::<32>()?,
    )
    .ok()?;
    let generation = revocation::begin_link(crate::runtime_uptime::millis())?;
    let epoch = *next_epoch;
    FAILURE.clear();
    CURRENT_SESSION.store(epoch, Ordering::Release);
    if !enqueue(ControlEvent::Session {
        epoch,
        binding: binding.clone(),
        generation,
    }) {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            revocation::RevocationReason::LinkClosed,
        );
        revoke_epoch(epoch);
        return None;
    }
    let payload = json!({
        "op": "hello_ack", "hostNonce": binding.host_nonce, "deviceNonce": binding.device_nonce,
        "serialManifest": serial_manifest(), "firmwareSourceCommit": crate::firmware_commit(),
        "appElfSha256": crate::app_elf_sha256(),
    });
    if writer::hello(epoch, &binding.session_id, payload).is_err() {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            revocation::RevocationReason::LinkClosed,
        );
        revoke_epoch(epoch);
        return None;
    }
    Some(Link {
        epoch,
        binding,
        generation,
        sequence: 0,
        liveness: SerialLinkLiveness::new(crate::runtime_uptime::millis()),
    })
}

fn close(maybe_link: &mut Option<Link>) {
    if let Some(link) = maybe_link.take() {
        revocation::revoke_reason_at(
            link.generation,
            crate::runtime_uptime::millis(),
            revocation::RevocationReason::LinkClosed,
        );
        revoke_epoch(link.epoch);
    }
}

fn enqueue(event: ControlEvent) -> bool {
    EVENTS
        .get()
        .is_some_and(|events| events.try_send(event).is_ok())
}

fn random_nonce<const N: usize>() -> Option<String> {
    let mut bytes = Zeroizing::new([0u8; N]);
    crate::crypto_entropy::fill(bytes.as_mut()).ok()?;
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
