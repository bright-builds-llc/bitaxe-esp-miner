//! Single application output owner; diagnostics never block control or liveness.

use super::*;
use bitaxe_worker_control::serial::SerialEnvelope;
use serde_json::{value::RawValue, Value};
use std::sync::atomic::AtomicUsize;

const DIAGNOSTIC_BYTES: usize = 1024;
static DIAGNOSTICS: OnceLock<SyncSender<String>> = OnceLock::new();
static DROPPED_DIAGNOSTICS: AtomicUsize = AtomicUsize::new(0);

pub(super) enum Output {
    Hello {
        epoch: u32,
        session_id: String,
        payload: Value,
    },
    Control {
        epoch: u32,
        bytes: SecretBytes,
        receipt: SyncSender<bool>,
    },
}

pub(super) fn hello(epoch: u32, session_id: &str, payload: Value) -> anyhow::Result<()> {
    OUTPUT
        .get()
        .ok_or_else(|| anyhow::anyhow!("serial_writer_unavailable"))?
        .try_send(Output::Hello {
            epoch,
            session_id: session_id.to_owned(),
            payload,
        })
        .map_err(|_| anyhow::anyhow!("serial_writer_full"))
}

pub(super) fn send_control(epoch: u32, bytes: &[u8]) -> anyhow::Result<()> {
    let (receipt, completion) = mpsc::sync_channel(1);
    OUTPUT
        .get()
        .ok_or_else(|| anyhow::anyhow!("serial_writer_unavailable"))?
        .try_send(Output::Control {
            epoch,
            bytes: SecretBytes(bytes.to_vec()),
            receipt,
        })
        .map_err(|_| anyhow::anyhow!("serial_writer_full"))?;
    anyhow::ensure!(
        completion.recv_timeout(Duration::from_millis(2200)) == Ok(true),
        "serial_response_unconfirmed"
    );
    Ok(())
}

pub(super) fn diagnostic(line: &str) {
    if line.len() > DIAGNOSTIC_BYTES {
        return;
    }
    let maybe_line = if bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(line)
        || matches!(
            line,
            "bwg_worker event=restoration_pending"
                | "invalid_frame"
                | "invalid_request"
                | "admission_required"
                | "invalid_proof"
                | "authentication_failed"
                | "invalid_transition"
                | "persistence_failed"
                | "monotonic_reset"
                | "session_failed"
                | "restoration_pending"
                | "stale_response"
                | "encoding_failed"
        ) {
        Some(line.to_owned())
    } else {
        bitaxe_api::UsbBootProfileMarker::parse(line)
            .ok()
            .map(|marker| marker.render())
    };
    let Some(line) = maybe_line else {
        return;
    };
    if let Some(sender) = DIAGNOSTICS.get() {
        if sender.try_send(line).is_err() {
            DROPPED_DIAGNOSTICS.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub(super) fn prepare_diagnostics() -> anyhow::Result<Receiver<String>> {
    let (sender, diagnostics) = mpsc::sync_channel(8);
    DIAGNOSTICS
        .set(sender)
        .map_err(|_| anyhow::anyhow!("diagnostic_writer_already_registered"))?;
    Ok(diagnostics)
}

pub(super) fn run(
    output: Receiver<Output>,
    diagnostics: Receiver<String>,
    progress: &startup_diagnostics::StartupProgress,
) {
    let mut epoch = 0;
    let mut session_id = String::new();
    let mut sequence = 0u32;
    let mut replay_slot = 0;
    let mut last_replay = 0;
    let mut next_startup_marker = 0;
    loop {
        match output.recv_timeout(Duration::from_millis(10)) {
            Ok(Output::Hello {
                epoch: next,
                session_id: id,
                payload,
            }) => {
                if CURRENT_SESSION.load(Ordering::Acquire) != next {
                    continue;
                }
                epoch = next;
                session_id = id;
                sequence = 0;
                replay_slot = 0;
                let result = serde_json::value::to_raw_value(&payload)
                    .map_err(anyhow::Error::from)
                    .and_then(|raw| emit(SerialKind::Session, &session_id, 0, &raw));
                if result.is_err() {
                    revoke_epoch(epoch);
                }
            }
            Ok(Output::Control {
                epoch: wanted,
                bytes,
                receipt,
            }) => {
                let sent = wanted == epoch
                    && CURRENT_SESSION.load(Ordering::Acquire) == epoch
                    && next_record(SerialKind::Control, &session_id, &mut sequence, &bytes.0)
                        .is_ok();
                if receipt.try_send(sent).is_err() || !sent {
                    revoke_epoch(wanted);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let now = crate::runtime_uptime::millis();
                let maybe_line = if now >= next_startup_marker {
                    next_startup_marker = now.saturating_add(500);
                    Some(progress.marker(now))
                } else {
                    diagnostics.try_recv().ok()
                }
                .or_else(|| {
                    if now.saturating_sub(last_replay) < 250 {
                        return None;
                    }
                    last_replay = now;
                    let line = crate::boot_evidence::maybe_worker_diagnostic_line(replay_slot);
                    replay_slot = (replay_slot + 1) % 12;
                    line
                });
                let Some(line) = maybe_line else {
                    continue;
                };
                if CURRENT_SESSION.load(Ordering::Acquire) != epoch || epoch == 0 {
                    let line = format!("{line}\n");
                    if crate::usb_runtime::write(line.as_bytes()).is_err() {
                        DROPPED_DIAGNOSTICS.fetch_add(1, Ordering::Relaxed);
                    }
                    continue;
                }
                let Ok(payload) = serde_json::to_vec(&serde_json::json!({"line":line})) else {
                    continue;
                };
                if next_record(SerialKind::Diagnostic, &session_id, &mut sequence, &payload)
                    .is_err()
                {
                    revoke_epoch(epoch);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn next_record(
    kind: SerialKind,
    session_id: &str,
    sequence: &mut u32,
    bytes: &[u8],
) -> anyhow::Result<()> {
    *sequence = sequence
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("serial_sequence_exhausted"))?;
    let raw: Box<RawValue> = serde_json::from_slice(bytes)?;
    emit(kind, session_id, *sequence, &raw)
}

fn emit(
    kind: SerialKind,
    session_id: &str,
    sequence: u32,
    payload: &RawValue,
) -> anyhow::Result<()> {
    let bytes = zeroize::Zeroizing::new(SerialEnvelope::encode(
        kind,
        Some(session_id),
        sequence,
        payload,
    )?);
    crate::usb_runtime::write(&bytes)
}
