//! Thin TinyUSB owner around the pure possession-bound Worker-control core.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::Duration;

use bitaxe_worker_control::{
    load_or_generate_device_identity, WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier,
    WorkerControl, WorkerControlError, WorkerControlFrameAccumulator,
};
use zeroize::Zeroize;

use crate::bwg_worker_nvs::{BwgWorkerNvs, EspDeviceIdentitySeedGenerator};
use crate::bwg_worker_session::ProductionWorkerSession;
use crate::startup::BootMiningBaselineConfirmed;
use crate::usb_runtime::{MaintenanceAction, MaintenanceEvent, UsbMaintenanceState};

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

pub(crate) fn start(recovery: BwgWorkerRecovery) -> anyhow::Result<()> {
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
        .spawn(move || run_owner(receiver, &mut worker))?;
    Ok(crate::usb_runtime::install_worker_runtime()?)
}

fn run_owner<V, S>(receiver: Receiver<UsbEvent>, worker: &mut WorkerControl<V, S>)
where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    let mut accumulator = WorkerControlFrameAccumulator::new();
    let mut maintenance = UsbMaintenanceState::default();
    let mut maintenance_ingress_open = true;
    loop {
        let now = crate::runtime_uptime::millis();
        let event = match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                note_control_result(worker.tick(now));
                maintenance.expire(now);
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match event {
            UsbEvent::Attached => {
                accumulator.clear();
                maintenance_ingress_open = true;
                worker.begin_enumeration();
                write_evidence(b"bwg_worker={\"event\":\"attached\"}\n");
            }
            UsbEvent::Detached => {
                accumulator.clear();
                maintenance_ingress_open = false;
                let _ = maintenance.observe(MaintenanceEvent::Detached, now);
                note_control_result(worker.disconnect(now));
                write_evidence(b"bwg_worker={\"event\":\"restoration_pending\"}\n");
            }
            UsbEvent::Bytes(chunk) => {
                if !maintenance_ingress_open {
                    continue;
                }
                let Ok(maybe_frame) = accumulator.push(&chunk.0) else {
                    note_control_result(worker.control_failed(now));
                    continue;
                };
                if let Some(mut frame) = maybe_frame {
                    process_frame(worker, &frame, now);
                    frame.zeroize();
                }
            }
            UsbEvent::LineCoding(bit_rate) => handle_maintenance(
                maintenance.observe(MaintenanceEvent::LineCoding { bit_rate }, now),
                &mut maintenance,
                &mut maintenance_ingress_open,
                worker,
                now,
            ),
            UsbEvent::LineState { dtr, rts } => handle_maintenance(
                maintenance.observe(MaintenanceEvent::LineState { dtr, rts }, now),
                &mut maintenance,
                &mut maintenance_ingress_open,
                worker,
                now,
            ),
        }
        if INGRESS_LOST.swap(false, Ordering::AcqRel) {
            accumulator.clear();
            note_control_result(worker.control_failed(now));
        }
    }
    note_control_result(worker.control_failed(crate::runtime_uptime::millis()));
}

fn handle_maintenance<V, S>(
    action: MaintenanceAction,
    state: &mut UsbMaintenanceState,
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
                state.observe(event, now),
                state,
                maintenance_ingress_open,
                worker,
                now,
            );
        }
        MaintenanceAction::EmitReady => {
            write_evidence(b"usb_maintenance={\"status\":\"ready\"}\n");
        }
        MaintenanceAction::CommitRestart => {
            if crate::usb_runtime::emit_evidence(b"usb_maintenance={\"status\":\"committed\"}\n")
                .is_err()
            {
                log::warn!("usb_maintenance=failed category=commit_receipt");
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
            if let Err(error) = crate::usb_runtime::restart_into_rom_downloader() {
                log::warn!("usb_maintenance=failed category=rom_handoff error={error:#}");
            }
        }
    }
}

fn process_frame<V, S>(worker: &mut WorkerControl<V, S>, frame: &[u8], now: u64)
where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    let Ok(response) = worker.prepare_frame(frame, now) else {
        if worker.has_active_lease() {
            note_control_result(worker.control_failed(now));
        }
        write_evidence(b"bwg_worker={\"event\":\"request_rejected\"}\n");
        return;
    };
    if crate::usb_runtime::send_worker_frame(response.frame()).is_err()
        || worker.confirm_sent(response).is_err()
    {
        note_control_result(worker.control_failed(now));
        write_evidence(b"bwg_worker={\"event\":\"response_unknown\"}\n");
    }
}

fn note_control_result(result: Result<(), WorkerControlError>) {
    if result.is_err() {
        write_evidence(b"bwg_worker={\"event\":\"restoration_pending\"}\n");
    }
}

fn write_evidence(bytes: &[u8]) {
    let _result = crate::usb_runtime::emit_evidence(bytes);
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
