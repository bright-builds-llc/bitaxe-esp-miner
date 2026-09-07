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
    let mut next_admission_marker = 0;
    let mut last_heartbeat = 0;
    let mut credited_bytes = 0;
    let mut maybe_write_failure = None;
    loop {
        let now = crate::runtime_uptime::millis();
        if epoch != 0
            && CURRENT_SESSION.load(Ordering::Acquire) == epoch
            && now.saturating_sub(last_heartbeat) >= 1000
        {
            last_heartbeat = now;
            if let Err(error) = next_record(
                OutputAdmission::active(epoch),
                SerialKind::Heartbeat,
                &session_id,
                &mut sequence,
                b"{}",
            ) {
                retain_write_failure(&mut maybe_write_failure, &error);
                revoke_epoch(epoch);
            }
        }
        // Credit is sampled independently of Worker commands and never waits for
        // a complete input record. Heartbeat authority has priority over progress.
        if let Err(error) =
            send_receive_credit(epoch, &session_id, &mut sequence, &mut credited_bytes)
        {
            retain_write_failure(&mut maybe_write_failure, &error);
            revoke_epoch(epoch);
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
                credited_bytes = 0;
                last_heartbeat = crate::runtime_uptime::millis();
                replay_slot = 0;
                let result = serde_json::value::to_raw_value(&payload)
                    .map_err(anyhow::Error::from)
                    .and_then(|raw| {
                        let admission = OutputAdmission::active(epoch);
                        crate::usb_runtime::resynchronize_if(|| admission.permits())?;
                        emit(admission, SerialKind::Session, &session_id, 0, &raw)
                    });
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
                    result = next_record(
                        OutputAdmission::active(epoch),
                        SerialKind::Heartbeat,
                        &session_id,
                        &mut sequence,
                        b"{}",
                    );
                }
                if current && result.is_ok() {
                    result =
                        send_receive_credit(epoch, &session_id, &mut sequence, &mut credited_bytes);
                }
                if current && result.is_ok() {
                    result = next_record(
                        OutputAdmission::active(epoch),
                        SerialKind::Control,
                        &session_id,
                        &mut sequence,
                        &bytes.0,
                    );
                }
                if let Err(error) = &result {
                    retain_write_failure(&mut maybe_write_failure, error);
                }
                let sent = current && result.is_ok() && OutputAdmission::active(epoch).permits();
                if receipt.try_send(sent).is_err() || !sent {
                    revoke_epoch(wanted);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(error) =
                    send_receive_credit(epoch, &session_id, &mut sequence, &mut credited_bytes)
                {
                    retain_write_failure(&mut maybe_write_failure, &error);
                    revoke_epoch(epoch);
                    continue;
                }
                let now = crate::runtime_uptime::millis();
                let maybe_line = if now >= next_startup_marker {
                    next_startup_marker = now.saturating_add(500);
                    Some(progress.marker(now))
                } else if now >= next_admission_marker {
                    next_admission_marker = now.saturating_add(1000);
                    Some(crate::production_mining_session::admission_diagnostics::marker())
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
                    if crate::usb_runtime::write_if(line.as_bytes(), || {
                        CURRENT_SESSION.load(Ordering::Acquire) == 0
                    })
                    .is_err()
                    {
                        DROPPED_DIAGNOSTICS.fetch_add(1, Ordering::Relaxed);
                    }
                    continue;
                }
                let Ok(payload) = serde_json::to_vec(&serde_json::json!({"line":line})) else {
                    continue;
                };
                if let Err(error) = next_record(
                    OutputAdmission::active(epoch),
                    SerialKind::Diagnostic,
                    &session_id,
                    &mut sequence,
                    &payload,
                ) {
                    retain_write_failure(&mut maybe_write_failure, &error);
                    revoke_epoch(epoch);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn send_receive_credit(
    epoch: u32,
    session_id: &str,
    sequence: &mut u32,
    credited_bytes: &mut u32,
) -> anyhow::Result<()> {
    let admission = OutputAdmission {
        epoch,
        terminal: CURRENT_SESSION.load(Ordering::Acquire) != epoch,
    };
    let maybe_received = if admission.terminal {
        RECEIVE_CREDIT.maybe_terminal_bytes(epoch, crate::runtime_uptime::millis())
    } else {
        RECEIVE_CREDIT.maybe_received_bytes(epoch)
    };
    let Some(received) = maybe_received else {
        return Ok(());
    };
    if received <= *credited_bytes || !admission.permits() {
        if admission.terminal {
            RECEIVE_CREDIT.finish_terminal(epoch);
        }
        return Ok(());
    }
    let payload = serde_json::to_vec(&serde_json::json!({
        "op": "receive_credit", "receivedBytes": received,
    }))?;
    let result = next_record(
        admission,
        SerialKind::Session,
        session_id,
        sequence,
        &payload,
    );
    if admission.terminal {
        RECEIVE_CREDIT.finish_terminal(epoch);
    }
    result?;
    *credited_bytes = received;
    Ok(())
}

#[derive(Clone, Copy)]
struct OutputAdmission {
    epoch: u32,
    terminal: bool,
}
impl OutputAdmission {
    fn active(epoch: u32) -> Self {
        Self {
            epoch,
            terminal: false,
        }
    }
    fn permits(self) -> bool {
        let current = CURRENT_SESSION.load(Ordering::Acquire);
        if self.terminal {
            return current == 0
                && !crate::usb_runtime::has_partial_output()
                && RECEIVE_CREDIT
                    .maybe_terminal_bytes(self.epoch, crate::runtime_uptime::millis())
                    .is_some();
        }
        self.epoch != 0 && current == self.epoch
    }
}

fn next_record(
    admission: OutputAdmission,
    kind: SerialKind,
    session_id: &str,
    sequence: &mut u32,
    bytes: &[u8],
) -> anyhow::Result<()> {
    anyhow::ensure!(admission.permits(), "serial_output_revoked");
    *sequence = sequence
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("serial_sequence_exhausted"))?;
    let raw: Box<RawValue> = serde_json::from_slice(bytes)?;
    emit(admission, kind, session_id, *sequence, &raw)
}

fn emit(
    admission: OutputAdmission,
    kind: SerialKind,
    session_id: &str,
    sequence: u32,
    payload: &RawValue,
) -> anyhow::Result<()> {
    anyhow::ensure!(admission.permits(), "serial_output_revoked");
    let bytes = zeroize::Zeroizing::new(SerialEnvelope::encode(
        kind,
        Some(session_id),
        sequence,
        payload,
    )?);
    crate::usb_runtime::write_if(&bytes, || admission.permits())
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
