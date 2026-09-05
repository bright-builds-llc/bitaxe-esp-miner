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
    let mut last_heartbeat = 0;
    let mut maybe_write_failure = None;
    loop {
        let now = crate::runtime_uptime::millis();
        if epoch != 0
            && CURRENT_SESSION.load(Ordering::Acquire) == epoch
            && now.saturating_sub(last_heartbeat) >= 1000
        {
            last_heartbeat = now;
            if let Err(error) =
                next_record(SerialKind::Heartbeat, &session_id, &mut sequence, b"{}")
            {
                retain_write_failure(&mut maybe_write_failure, &error);
                revoke_epoch(epoch);
            }
        }
        match output.recv_timeout(Duration::from_millis(10)) {
            Ok(Output::Hello {
                epoch: next,
                session_id: id,
                payload,
            }) => {
                if CURRENT_SESSION.load(Ordering::Acquire) != next {
                    continue;
                }
                maybe_write_failure = None;
                epoch = next;
                session_id = id;
                sequence = 0;
                last_heartbeat = crate::runtime_uptime::millis();
                replay_slot = 0;
                let result = serde_json::value::to_raw_value(&payload)
                    .map_err(anyhow::Error::from)
                    .and_then(|raw| emit(SerialKind::Session, &session_id, 0, &raw));
                if let Err(error) = result {
                    retain_write_failure(&mut maybe_write_failure, &error);
                    revoke_epoch(epoch);
                }
            }
            Ok(Output::Control {
                epoch: wanted,
                bytes,
                receipt,
            }) => {
                let current = wanted == epoch && CURRENT_SESSION.load(Ordering::Acquire) == epoch;
                let mut result = Ok(());
                if current && bytes.0.len() > 4096 {
                    // An indivisible long record gets a fresh peer deadline before transmission.
                    last_heartbeat = crate::runtime_uptime::millis();
                    result = next_record(SerialKind::Heartbeat, &session_id, &mut sequence, b"{}");
                }
                if current && result.is_ok() {
                    result = next_record(SerialKind::Control, &session_id, &mut sequence, &bytes.0);
                }
                if let Err(error) = &result {
                    retain_write_failure(&mut maybe_write_failure, error);
                }
                let sent = current && result.is_ok();
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
                    let line = if replay_slot == 18 {
                        rx_diagnostics::FAILURE.marker()
                    } else if replay_slot == 15 {
                        maybe_write_failure.map(crate::usb_runtime::WriteFailure::marker)
                    } else if replay_slot == 14 {
                        crate::wifi_adapter::maybe_startup_failure_marker()
                    } else {
                        crate::boot_evidence::maybe_worker_diagnostic_line(replay_slot)
                    };
                    replay_slot = (replay_slot + 1) % 19;
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
                if let Err(error) =
                    next_record(SerialKind::Diagnostic, &session_id, &mut sequence, &payload)
                {
                    retain_write_failure(&mut maybe_write_failure, &error);
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

fn retain_write_failure(
    slot: &mut Option<crate::usb_runtime::WriteFailure>,
    error: &anyhow::Error,
) {
    if slot.is_none() {
        *slot = error
            .downcast_ref::<crate::usb_runtime::WriteFailure>()
            .copied();
    }
}
