//! Serial RX and heartbeat supervision never wait for Worker effects or TX completion.

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
    let mut bytes = [0u8; 512];
    loop {
        let now = crate::runtime_uptime::millis();
        revocation::check_deadline(now);
        if maybe_link.as_mut().is_some_and(|link| {
            CURRENT_SESSION.load(Ordering::Acquire) != link.epoch || !link.liveness.poll(now)
        }) {
            close(&mut maybe_link);
        }
        let count = match crate::usb_runtime::read(&mut bytes) {
            Ok(count) => count,
            Err(_) => {
                close(&mut maybe_link);
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
        };
        for byte in &bytes[..count] {
            let Some(result) = accumulator.push_byte(*byte) else {
                continue;
            };
            let Ok(frame) = result else {
                close(&mut maybe_link);
                continue;
            };
            let frame = Zeroizing::new(frame);
            let Ok(envelope) = SerialEnvelope::parse(&frame) else {
                // Boot text is permissible before hello; malformed established input revokes.
                if maybe_link.is_some() {
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
                close(&mut maybe_link);
                continue;
            }
            link.sequence = envelope.sequence;
            if envelope.sequence == u32::MAX {
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
                        close(&mut maybe_link);
                    }
                }
                SerialKind::Session | SerialKind::Diagnostic => close(&mut maybe_link),
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
    let generation = revocation::begin_link(crate::runtime_uptime::millis())?;
    let binding = SerialSessionBinding::parse(
        &random_nonce::<16>(),
        &hello.host_nonce,
        &random_nonce::<32>(),
    )
    .ok()?;
    let epoch = *next_epoch;
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

fn random_nonce<const N: usize>() -> String {
    // ESP-IDF hardware entropy is available after Wi-Fi initialization.
    let mut bytes = [0u8; N];
    unsafe { esp_idf_sys::esp_fill_random(bytes.as_mut_ptr().cast(), bytes.len()) };
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    for byte in bytes {
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
    result
}
