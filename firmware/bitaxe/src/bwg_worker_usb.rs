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

const OWNER_STACK_BYTES: usize = 16 * 1024;
const EVENT_CAPACITY: usize = 8;
const MAXIMUM_FRAME_BYTES: usize = 65_536;
const DESCRIPTOR_SHA256: &str = "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA";
const DEPLOYMENT_TRUST: &str = include_str!("../bwg/deployment-trust.json");
const ULTRA205_CAPABILITY: &str = include_str!("../bwg/ultra205-capability.json");

static EVENTS: OnceLock<SyncSender<UsbEvent>> = OnceLock::new();
static INGRESS_LOST: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    fn bwg_usb_install() -> i32;
    fn bwg_usb_vendor_write(bytes: *const u8, length: u32) -> u32;
    fn bwg_usb_evidence_write(bytes: *const u8, length: u32) -> u32;
}

enum UsbEvent {
    Attached,
    Detached,
    Bytes(SecretUsbBytes),
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
    let mut worker = WorkerControl::new(
        identity,
        verifier,
        ProductionWorkerSession,
        reboot_report_required.then_some(bitaxe_worker_control::RestorationReason::Reboot),
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
    let result = unsafe { bwg_usb_install() };
    if result != esp_idf_sys::ESP_OK {
        return Err(anyhow::anyhow!("BWG TinyUSB install failed: {result}"));
    }
    Ok(())
}

fn run_owner<V, S>(receiver: Receiver<UsbEvent>, worker: &mut WorkerControl<V, S>)
where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
    S: bitaxe_worker_control::WorkerSession,
{
    let mut accumulator = WorkerControlFrameAccumulator::new();
    loop {
        let now = crate::runtime_uptime::millis();
        let event = match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                note_control_result(worker.tick(now));
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match event {
            UsbEvent::Attached => {
                accumulator.clear();
                worker.begin_enumeration();
                write_evidence(b"bwg_worker={\"event\":\"attached\"}\n");
            }
            UsbEvent::Detached => {
                accumulator.clear();
                note_control_result(worker.disconnect(now));
                write_evidence(b"bwg_worker={\"event\":\"restoration_pending\"}\n");
            }
            UsbEvent::Bytes(chunk) => {
                let Ok(maybe_frame) = accumulator.push(&chunk.0) else {
                    note_control_result(worker.control_failed(now));
                    continue;
                };
                if let Some(mut frame) = maybe_frame {
                    process_frame(worker, &frame, now);
                    frame.zeroize();
                }
            }
        }
        if INGRESS_LOST.swap(false, Ordering::AcqRel) {
            accumulator.clear();
            note_control_result(worker.control_failed(now));
        }
    }
    note_control_result(worker.control_failed(crate::runtime_uptime::millis()));
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
    let length = u32::try_from(response.frame().len()).unwrap_or(0);
    let written = unsafe { bwg_usb_vendor_write(response.frame().as_ptr(), length) };
    if length == 0 || written != length || worker.confirm_sent(response).is_err() {
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
    let Ok(length) = u32::try_from(bytes.len()) else {
        return;
    };
    let _ = unsafe { bwg_usb_evidence_write(bytes.as_ptr(), length) };
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

#[no_mangle]
extern "C" fn bwg_worker_usb_attached() {
    send_event(UsbEvent::Attached);
}

#[no_mangle]
extern "C" fn bwg_worker_usb_detached() {
    send_event(UsbEvent::Detached);
}

#[no_mangle]
unsafe extern "C" fn bwg_worker_usb_vendor_received(bytes: *const u8, length: u32) {
    if bytes.is_null() || length == 0 {
        INGRESS_LOST.store(true, Ordering::Release);
        return;
    }
    let Ok(length) = usize::try_from(length) else {
        INGRESS_LOST.store(true, Ordering::Release);
        return;
    };
    if length > MAXIMUM_FRAME_BYTES {
        INGRESS_LOST.store(true, Ordering::Release);
        return;
    }
    let value = unsafe { std::slice::from_raw_parts(bytes, length) }.to_vec();
    send_event(UsbEvent::Bytes(SecretUsbBytes(value)));
}
