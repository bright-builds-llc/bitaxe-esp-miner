//! Single application output owner; diagnostics never block control or liveness.

use super::*;
use crate::usb_runtime::measurement::{Category, RecordKind, Retained};
use bitaxe_worker_control::serial::writer::{
    SerialOutputPurpose, SerialOutputSink, SerialRecordWriter, SerialWriteError, SerialWriterView,
};
use serde_json::Value;
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
        request_sequence: u32,
        bytes: SecretBytes,
        receipt: bitaxe_runtime::reply::ReplySender<bool>,
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

pub(super) fn send_control(
    correlation: SerialTraceCorrelation,
    bytes: &[u8],
) -> anyhow::Result<()> {
    // A per-reply std channel realigns its stack frame without `movsp` on this
    // toolchain and can corrupt this owner's registers; see `bitaxe_runtime::reply`.
    let (receipt, completion) = bitaxe_runtime::reply::reply();
    let result = OUTPUT
        .get()
        .ok_or_else(|| anyhow::anyhow!("serial_writer_unavailable"))
        .and_then(|output| {
            output
                .try_send(Output::Control {
                    epoch: correlation.epoch,
                    request_sequence: correlation.request_sequence,
                    bytes: SecretBytes(bytes.to_vec()),
                    receipt,
                })
                .map_err(|_| anyhow::anyhow!("serial_writer_full"))
        });
    if result.is_err() {
        trace::event(correlation, SerialTraceStage::WriterRejected, 0);
    }
    result?;
    trace::event(correlation, SerialTraceStage::WriterAccepted, 0);
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
    let (sender, diagnostics) = queue::sync_channel(8);
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
    let view = &NativeView;
    let sink = &mut NativeSink(Retained::new());
    let mut writer = SerialRecordWriter::new();
    let mut replay_slot = 0;
    let mut last_replay = 0;
    let mut next_startup_marker = 0;
    let mut next_admission_marker = 0;
    let mut admission_turn = false;
    let mut next_heap_sample = 0;
    let mut maybe_write_failure = None;
    loop {
        let now = crate::runtime_uptime::millis();
        if let Err(error) = writer.heartbeat_if_due(now, view, sink) {
            retain_write_failure(&mut maybe_write_failure, &error);
        }
        // Credit is sampled independently of Worker commands and never waits for
        // a complete input record. Heartbeat authority has priority over progress.
        if let Err(error) = writer.receive_credit(view, sink) {
            retain_write_failure(&mut maybe_write_failure, &error);
        }
        match output.recv_timeout(Duration::from_millis(10)) {
            Ok(Output::Hello {
                epoch: next,
                session_id: id,
                payload,
            }) => {
                let Some(result) = writer.hello(next, id, &payload, view, sink) else {
                    continue;
                };
                maybe_write_failure = None;
                replay_slot = 0;
                if let Err(error) = result {
                    retain_write_failure(&mut maybe_write_failure, &error);
                }
            }
            Ok(Output::Control {
                epoch: wanted,
                request_sequence,
                bytes,
                receipt,
            }) => {
                let correlation = SerialTraceCorrelation {
                    epoch: wanted,
                    request_sequence,
                };
                let result = writer.control(
                    correlation,
                    &bytes.0,
                    |sent| receipt.send(sent).is_ok(),
                    view,
                    sink,
                );
                if let Err(error) = result {
                    retain_write_failure(&mut maybe_write_failure, &error);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(error) = writer.receive_credit(view, sink) {
                    retain_write_failure(&mut maybe_write_failure, &error);
                    continue;
                }
                let now = crate::runtime_uptime::millis();
                let maybe_line = if now >= next_startup_marker {
                    if let Err(error) = send_statistics_startup(&mut writer, view, sink) {
                        retain_write_failure(&mut maybe_write_failure, &error);
                        revoke_epoch(writer.epoch());
                        continue;
                    }
                    next_startup_marker = now.saturating_add(500);
                    Some((
                        progress.marker(crate::runtime_uptime::millis()),
                        RecordKind::StartupProgress,
                        false,
                    ))
                } else if now >= next_admission_marker {
                    next_admission_marker = now.saturating_add(1000);
                    // Once a revocation detail exists, alternate it with the
                    // admission marker so each still repeats every two seconds.
                    admission_turn = !admission_turn;
                    let maybe_detail = (!admission_turn)
                        .then(crate::production_mining_session::revocation::maybe_unsafe_detail_marker)
                        .flatten();
                    Some((
                        maybe_detail.unwrap_or_else(
                            crate::production_mining_session::admission_diagnostics::marker,
                        ),
                        RecordKind::Admission,
                        false,
                    ))
                } else if now >= next_heap_sample && CURRENT_SESSION.load(Ordering::Acquire) == 0 {
                    // Bootstrap-only, so Worker sessions and their judges see an unchanged stream.
                    next_heap_sample = now.saturating_add(
                        bitaxe_core::usb_diagnostics::INTERNAL_HEAP_SAMPLE_INTERVAL_MS,
                    );
                    Some((heap_sample::marker(now), RecordKind::RetainedDiagnostic, false))
                } else {
                    diagnostics
                        .try_recv()
                        .ok()
                        .map(|line| (line, RecordKind::RetainedDiagnostic, false))
                }
                .or_else(|| {
                    if now.saturating_sub(last_replay) < 250 {
                        return None;
                    }
                    last_replay = now;
                    if replay_slot == 23 {
                        replay_slot = 0;
                        return sink
                            .0
                            .marker()
                            .map(|line| (line, RecordKind::TxObservation, true));
                    }
                    let line = if replay_slot == 18 {
                        rx_diagnostics::FAILURE.marker()
                    } else if replay_slot == 15 {
                        maybe_write_failure.map(crate::usb_runtime::WriteFailure::marker)
                    } else if replay_slot == 14 {
                        crate::wifi_adapter::maybe_startup_failure_marker()
                    } else {
                        crate::boot_evidence::maybe_worker_diagnostic_line(replay_slot)
                    };
                    replay_slot = (replay_slot + 1) % 24;
                    line.map(|line| (line, RecordKind::RetainedDiagnostic, true))
                });
                let Some((line, kind, replay)) = maybe_line else {
                    continue;
                };
                if let Err(error) = writer.diagnostic_line(&line, kind, replay, view, sink) {
                    retain_write_failure(&mut maybe_write_failure, &error);
                    revoke_epoch(writer.epoch());
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn send_statistics_startup(
    writer: &mut SerialRecordWriter,
    view: &NativeView,
    sink: &mut NativeSink,
) -> Result<(), SerialWriteError<anyhow::Error>> {
    let Some(marker) = crate::statistics_runtime::diagnostics::STARTUP.maybe_marker() else {
        return Ok(());
    };
    writer.diagnostic_line(
        &crate::boot_evidence::worker_usb_boot_marker(),
        RecordKind::BootIdentity,
        false,
        view,
        sink,
    )?;
    writer.diagnostic_line(&marker, RecordKind::StatisticsStartup, false, view, sink)
}

/// Firmware device state for the shared writer core.
struct NativeView;

impl SerialWriterView for NativeView {
    fn now_ms(&self) -> u64 {
        crate::runtime_uptime::millis()
    }
    fn current_epoch(&self) -> u32 {
        CURRENT_SESSION.load(Ordering::Acquire)
    }
    fn has_partial_output(&self) -> bool {
        crate::usb_runtime::has_partial_output()
    }
    fn received_bytes(&self, epoch: u32) -> Option<u32> {
        RECEIVE_CREDIT.maybe_received_bytes(epoch)
    }
    fn terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        RECEIVE_CREDIT.maybe_terminal_bytes(epoch, now_ms)
    }
    fn finish_terminal(&self, epoch: u32) {
        RECEIVE_CREDIT.finish_terminal(epoch);
    }
    fn revoke_epoch(&self, epoch: u32) {
        revoke_epoch(epoch);
    }
    fn trace(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    ) {
        trace::event(correlation, stage, wire_bytes);
    }
    fn diagnostic_dropped(&self) {
        DROPPED_DIAGNOSTICS.fetch_add(1, Ordering::Relaxed);
    }
}

/// The fixed Serial/JTAG transport with retained TX measurements.
struct NativeSink(Retained);

impl SerialOutputSink for NativeSink {
    type Error = anyhow::Error;
    type DiagnosticKind = RecordKind;

    fn write<A: Fn() -> bool>(
        &mut self,
        record: &[u8],
        admitted: A,
        maybe_correlation: Option<SerialTraceCorrelation>,
        purpose: SerialOutputPurpose<RecordKind>,
    ) -> anyhow::Result<()> {
        let (category, kind) = measurement(purpose);
        match maybe_correlation {
            Some(correlation) => crate::usb_runtime::write_measured_if(
                &mut self.0,
                record,
                admitted,
                |observation| trace::observe(correlation, observation),
                category,
                kind,
            ),
            None => crate::usb_runtime::write_measured_if(
                &mut self.0,
                record,
                admitted,
                |_| {},
                category,
                kind,
            ),
        }
    }
    fn resynchronize<A: Fn() -> bool>(&mut self, admitted: A) -> anyhow::Result<()> {
        crate::usb_runtime::resynchronize_if(&mut self.0, admitted)
    }
    fn is_transport_failure(error: &anyhow::Error) -> bool {
        error
            .downcast_ref::<crate::usb_runtime::WriteFailure>()
            .is_some()
    }
}

// Indexed by `ProtocolRecord` and `DiagnosticOrigin`; table loads, not computed jumps,
// keep the writer thread's native decode complete.
const PROTOCOL_CATEGORIES: [Category; 4] = [
    Category::Hello,
    Category::Heartbeat,
    Category::ReceiveCredit,
    Category::ControlReply,
];
const DIAGNOSTIC_CATEGORIES: [Category; 3] = [
    Category::BootstrapDiagnostic,
    Category::Diagnostic,
    Category::DiagnosticReplay,
];

fn measurement(purpose: SerialOutputPurpose<RecordKind>) -> (Category, RecordKind) {
    match purpose {
        SerialOutputPurpose::Protocol(record) => {
            (PROTOCOL_CATEGORIES[record as usize], RecordKind::Protocol)
        }
        SerialOutputPurpose::Diagnostic(origin, kind) => {
            (DIAGNOSTIC_CATEGORIES[origin as usize], kind)
        }
    }
}

fn retain_write_failure(
    slot: &mut Option<crate::usb_runtime::WriteFailure>,
    error: &SerialWriteError<anyhow::Error>,
) {
    if slot.is_none() {
        *slot = error
            .maybe_sink()
            .and_then(|error| error.downcast_ref::<crate::usb_runtime::WriteFailure>())
            .copied();
    }
}
