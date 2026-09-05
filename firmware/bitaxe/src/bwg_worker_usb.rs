//! Thin TinyUSB owner around the pure possession-bound Worker-control core.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::Duration;

use bitaxe_core::usb_maintenance::{
    MaintenanceTraceEffect, MaintenanceTraceOutcome, TracedUsbMaintenanceState,
};
use bitaxe_core::usb_worker_diagnostics::{
    CdcEvidenceWriter, WorkerDiagnosticReplay, DIAGNOSTIC_LINE_BYTES,
};
use bitaxe_worker_control::{
    load_or_generate_device_identity, WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier,
    WorkerControl, WorkerControlError, WorkerControlFrameAccumulator,
};
use zeroize::Zeroize;

use crate::bwg_worker_nvs::{BwgWorkerNvs, EspDeviceIdentitySeedGenerator};
use crate::bwg_worker_session::ProductionWorkerSession;
use crate::startup::BootMiningBaselineConfirmed;
use crate::usb_runtime::{MaintenanceAction, MaintenanceEvent, UsbRuntimeFailure};

const OWNER_STACK_BYTES: usize = 16 * 1024;
const EVENT_CAPACITY: usize = 8;
const MAXIMUM_FRAME_BYTES: usize = 65_536;
const DESCRIPTOR_SHA256: &str = "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA";
const DEPLOYMENT_TRUST: &str = include_str!("../bwg/deployment-trust.json");
const ULTRA205_CAPABILITY: &str = include_str!("../bwg/ultra205-capability.json");

static EVENTS: OnceLock<SyncSender<UsbEvent>> = OnceLock::new();
static INGRESS_LOST: AtomicBool = AtomicBool::new(false);

enum UsbEvent {
    Attached,
    Detached,
    Bytes(SecretUsbBytes),
    LineCoding(u32),
    LineState { dtr: bool, rts: bool },
}

struct SecretUsbBytes(Vec<u8>);

pub(crate) struct BwgWorkerRecovery {
    nvs: BwgWorkerNvs,
    reboot_report_required: bool,
}

pub(crate) struct PreparedWorkerRuntime(());

impl Drop for SecretUsbBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub(crate) fn recover_interrupted_effect(
    proof: BootMiningBaselineConfirmed,
) -> anyhow::Result<BwgWorkerRecovery> {
    let mut nvs = BwgWorkerNvs::open()
        .map_err(|error| anyhow::anyhow!("BWG NVS unavailable: {}", error.category()))?;
    let reboot_report_required = nvs.confirm_reboot_baseline(proof).map_err(|error| {
        anyhow::anyhow!("BWG reboot restoration unavailable: {}", error.category())
    })?;
    Ok(BwgWorkerRecovery {
        nvs,
        reboot_report_required,
    })
}

pub(crate) fn prepare(recovery: BwgWorkerRecovery) -> anyhow::Result<PreparedWorkerRuntime> {
    let BwgWorkerRecovery {
        mut nvs,
        reboot_report_required,
    } = recovery;
    let identity = load_or_generate_device_identity(&mut nvs, &mut EspDeviceIdentitySeedGenerator)
        .map_err(|error| {
            anyhow::anyhow!("BWG Device Identity unavailable: {}", error.category())
        })?;
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(DEPLOYMENT_TRUST)
        .map_err(|error| anyhow::anyhow!("BWG deployment trust invalid: {}", error.category()))?;
    let verifier = WorkLeaseAuthorizationVerifier::new(trust, nvs);
    let capability: serde_json::Value = serde_json::from_str(ULTRA205_CAPABILITY)
        .map_err(|_| anyhow::anyhow!("BWG Ultra 205 capability is invalid"))?;
    if capability
        .pointer("/board/model")
        .and_then(|value| value.as_str())
        != Some("bitaxe-ultra")
        || capability
            .pointer("/board/revision")
            .and_then(|value| value.as_str())
            != Some("205")
        || capability
            .pointer("/firmware/version")
            .and_then(|value| value.as_str())
            != Some(crate::semantic_version())
        || capability
            .pointer("/attestation/claims/applicationDescriptorSha256")
            .and_then(|value| value.as_str())
            != Some(DESCRIPTOR_SHA256)
    {
        return Err(anyhow::anyhow!(
            "BWG Ultra 205 capability does not match firmware"
        ));
    }
    let firmware_source_commit =
        bitaxe_worker_control::FirmwareSourceCommit::parse(crate::firmware_commit())
            .map_err(|_| anyhow::anyhow!("BWG firmware source commitment is invalid"))?;
    let mut worker = WorkerControl::new(
        identity,
        verifier,
        ProductionWorkerSession,
        reboot_report_required.then_some(bitaxe_worker_control::RestorationReason::Reboot),
        firmware_source_commit,
        capability,
        DESCRIPTOR_SHA256,
    )
    .map_err(|error| anyhow::anyhow!("BWG Worker control unavailable: {}", error.category()))?;
    let (sender, receiver) = mpsc::sync_channel(EVENT_CAPACITY);
    EVENTS
        .set(sender)
        .map_err(|_| anyhow::anyhow!("BWG USB owner already started"))?;
    std::thread::Builder::new()
        .name("bwg-worker-control".to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run_owner(receiver, &mut worker))
        .map_err(|error| anyhow::anyhow!("owner_spawn: {error}"))?;
    Ok(PreparedWorkerRuntime(()))
}

pub(crate) fn install(_prepared: PreparedWorkerRuntime) -> anyhow::Result<()> {
    crate::usb_runtime::install_worker_runtime()
        .map_err(|error| anyhow::anyhow!("usb_install: {error}"))?;
    Ok(())
}

fn run_owner<V, S>(receiver: Receiver<UsbEvent>, worker: &mut WorkerControl<V, S>)
where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    let mut evidence = CdcEvidenceWriter::new();
    let mut accumulator = WorkerControlFrameAccumulator::new();
    let mut maintenance = TracedUsbMaintenanceState::default();
    let mut diagnostics = WorkerDiagnosticReplay::default();
    let mut maintenance_ingress_open = true;
    loop {
        let now = crate::runtime_uptime::millis();
        let event = match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                note_control_result(&mut evidence, worker.tick(now));
                maintenance.expire(now);
                emit_due_diagnostics(&mut evidence, &mut diagnostics, &maintenance, now);
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match event {
            UsbEvent::Attached => {
                evidence = CdcEvidenceWriter::new();
                diagnostics = WorkerDiagnosticReplay::default();
                accumulator.clear();
                maintenance_ingress_open = true;
                worker.begin_enumeration();
                write_evidence(&mut evidence, b"bwg_worker={\"event\":\"attached\"}\n");
                emit_mount_boot_evidence(&mut evidence);
            }
            UsbEvent::Detached => {
                diagnostics = WorkerDiagnosticReplay::default();
                accumulator.clear();
                maintenance_ingress_open = false;
                let _ = maintenance.observe(MaintenanceEvent::Detached, now);
                note_control_result(&mut evidence, worker.disconnect(now));
                write_evidence(
                    &mut evidence,
                    b"bwg_worker={\"event\":\"restoration_pending\"}\n",
                );
            }
            UsbEvent::Bytes(chunk) => {
                if !maintenance_ingress_open {
                    continue;
                }
                let Ok(maybe_frame) = accumulator.push(&chunk.0) else {
                    note_control_result(&mut evidence, worker.control_failed(now));
                    continue;
                };
                if let Some(mut frame) = maybe_frame {
                    process_frame(&mut evidence, worker, &frame, now);
                    frame.zeroize();
                }
            }
            UsbEvent::LineCoding(bit_rate) => {
                diagnostics.line_coding(bit_rate, now);
                handle_maintenance(
                    &mut evidence,
                    maintenance.observe(MaintenanceEvent::LineCoding { bit_rate }, now),
                    &mut maintenance,
                    &mut maintenance_ingress_open,
                    worker,
                    now,
                );
            }
            UsbEvent::LineState { dtr, rts } => {
                diagnostics.line_state(dtr, now);
                handle_maintenance(
                    &mut evidence,
                    maintenance.observe(MaintenanceEvent::LineState { dtr, rts }, now),
                    &mut maintenance,
                    &mut maintenance_ingress_open,
                    worker,
                    now,
                );
            }
        }
        emit_due_diagnostics(&mut evidence, &mut diagnostics, &maintenance, now);
        if INGRESS_LOST.swap(false, Ordering::AcqRel) {
            maintenance.record_effect(
                MaintenanceTraceEffect::QueueLoss,
                MaintenanceTraceOutcome::None,
                now,
            );
            accumulator.clear();
            note_control_result(&mut evidence, worker.control_failed(now));
        }
    }
    note_control_result(
        &mut evidence,
        worker.control_failed(crate::runtime_uptime::millis()),
    );
}

fn emit_mount_boot_evidence(evidence: &mut CdcEvidenceWriter) {
    let mut marker = crate::boot_evidence::worker_usb_boot_marker();
    marker.push('\n');
    write_evidence(evidence, marker.as_bytes());
    if let Some(mut marker) = crate::boot_evidence::worker_rust_panic_marker() {
        marker.push('\n');
        write_evidence(evidence, marker.as_bytes());
    }
    if let Some(mut marker) = crate::boot_evidence::worker_allocation_failure_marker() {
        marker.push('\n');
        write_evidence(evidence, marker.as_bytes());
    }
}

fn emit_due_diagnostics(
    evidence: &mut CdcEvidenceWriter,
    replay: &mut WorkerDiagnosticReplay,
    maintenance: &TracedUsbMaintenanceState,
    now: u64,
) {
    let (bit_rate, dtr) = crate::usb_runtime::worker_observer_state();
    replay.line_coding(bit_rate, now);
    replay.line_state(dtr, now);
    let Some(slot) = replay.maybe_due_slot(now, maintenance.diagnostics_allowed()) else {
        return;
    };
    let maybe_line = if slot < 12 {
        crate::boot_evidence::maybe_worker_diagnostic_line(slot)
    } else {
        maintenance.maybe_trace_marker(slot - 12)
    };
    let Some(mut line) = maybe_line else {
        replay.advance(now);
        return;
    };
    line.push('\n');
    if line.len() > DIAGNOSTIC_LINE_BYTES {
        log::warn!("worker_diagnostics=unavailable reason=line_bound");
        replay.advance(now);
        return;
    }
    if crate::usb_runtime::emit_diagnostic(evidence, line.as_bytes()).is_ok() {
        replay.advance(now);
    } else {
        replay.retry_later(now);
    }
}

fn handle_maintenance<V, S>(
    evidence: &mut CdcEvidenceWriter,
    action: MaintenanceAction,
    state: &mut TracedUsbMaintenanceState,
    maintenance_ingress_open: &mut bool,
    worker: &mut WorkerControl<V, S>,
    now: u64,
) where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    match action {
        MaintenanceAction::None => {}
        MaintenanceAction::RequestSafeStop => {
            *maintenance_ingress_open = false;
            let active_effect = worker.has_active_lease();
            let safe_stop_complete = worker.control_failed(now).is_ok();
            let event = if safe_stop_complete && !active_effect {
                MaintenanceEvent::SafeStopComplete
            } else {
                MaintenanceEvent::SafeStopFailed
            };
            handle_maintenance(
                evidence,
                state.observe(event, now),
                state,
                maintenance_ingress_open,
                worker,
                now,
            );
        }
        MaintenanceAction::EmitReady => {
            let result = crate::usb_runtime::emit_evidence(
                evidence,
                b"usb_maintenance={\"status\":\"ready\"}\n",
            );
            state.record_effect(
                MaintenanceTraceEffect::ReadyEnqueue,
                trace_outcome(&result),
                crate::runtime_uptime::millis(),
            );
        }
        MaintenanceAction::CommitRestart => {
            let result = crate::usb_runtime::emit_evidence(
                evidence,
                b"usb_maintenance={\"status\":\"committed\"}\n",
            );
            state.record_effect(
                MaintenanceTraceEffect::CommitEnqueue,
                trace_outcome(&result),
                crate::runtime_uptime::millis(),
            );
            if result.is_err() {
                log::warn!("usb_maintenance=failed category=commit_receipt");
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
            state.record_effect(
                MaintenanceTraceEffect::PhyInvoked,
                MaintenanceTraceOutcome::None,
                crate::runtime_uptime::millis(),
            );
            let result = crate::usb_runtime::restart_into_rom_downloader();
            state.record_effect(
                MaintenanceTraceEffect::PhyReturned,
                trace_outcome(&result),
                crate::runtime_uptime::millis(),
            );
            if let Err(error) = result {
                log::warn!("usb_maintenance=failed category=rom_handoff error={error:#}");
            }
        }
    }
}

fn trace_outcome(result: &Result<(), UsbRuntimeFailure>) -> MaintenanceTraceOutcome {
    match result {
        Ok(()) => MaintenanceTraceOutcome::Ok,
        Err(UsbRuntimeFailure::UnavailableTransport) => {
            MaintenanceTraceOutcome::UnavailableTransport
        }
        Err(UsbRuntimeFailure::Disconnected) => MaintenanceTraceOutcome::Disconnected,
        Err(UsbRuntimeFailure::PartialWrite) => MaintenanceTraceOutcome::PartialWrite,
        Err(UsbRuntimeFailure::Timeout) => MaintenanceTraceOutcome::Timeout,
        Err(UsbRuntimeFailure::Install(_)) => MaintenanceTraceOutcome::Install,
        Err(UsbRuntimeFailure::Handoff(code)) if *code == esp_idf_sys::ESP_ERR_TIMEOUT => {
            MaintenanceTraceOutcome::Timeout
        }
        Err(UsbRuntimeFailure::Handoff(_)) => MaintenanceTraceOutcome::Handoff,
    }
}

fn process_frame<V, S>(
    evidence: &mut CdcEvidenceWriter,
    worker: &mut WorkerControl<V, S>,
    frame: &[u8],
    now: u64,
) where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    let Ok(response) = worker.prepare_frame(frame, now) else {
        if worker.has_active_lease() {
            note_control_result(evidence, worker.control_failed(now));
        }
        write_evidence(evidence, b"bwg_worker={\"event\":\"request_rejected\"}\n");
        return;
    };
    if crate::usb_runtime::send_worker_frame(response.frame()).is_err()
        || worker.confirm_sent(response).is_err()
    {
        note_control_result(evidence, worker.control_failed(now));
        write_evidence(evidence, b"bwg_worker={\"event\":\"response_unknown\"}\n");
    }
}

fn note_control_result(evidence: &mut CdcEvidenceWriter, result: Result<(), WorkerControlError>) {
    if result.is_err() {
        write_evidence(
            evidence,
            b"bwg_worker={\"event\":\"restoration_pending\"}\n",
        );
    }
}

fn write_evidence(evidence: &mut CdcEvidenceWriter, bytes: &[u8]) {
    let _result = crate::usb_runtime::emit_evidence(evidence, bytes);
}

fn send_event(event: UsbEvent) {
    let Some(sender) = EVENTS.get() else {
        return;
    };
    match sender.try_send(event) {
        Ok(()) => {}
        Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
            INGRESS_LOST.store(true, Ordering::Release);
        }
    }
}

pub(crate) fn enqueue_attached() {
    send_event(UsbEvent::Attached);
}

pub(crate) fn enqueue_detached() {
    send_event(UsbEvent::Detached);
}

pub(crate) fn enqueue_vendor_bytes(bytes: &[u8]) {
    if bytes.is_empty() || bytes.len() > MAXIMUM_FRAME_BYTES {
        INGRESS_LOST.store(true, Ordering::Release);
        return;
    }
    send_event(UsbEvent::Bytes(SecretUsbBytes(bytes.to_vec())));
}

pub(crate) fn enqueue_line_coding(bit_rate: u32) {
    send_event(UsbEvent::LineCoding(bit_rate));
}

pub(crate) fn enqueue_line_state(dtr: bool, rts: bool) {
    send_event(UsbEvent::LineState { dtr, rts });
}

pub(crate) fn note_ingress_lost() {
    INGRESS_LOST.store(true, Ordering::Release);
}
