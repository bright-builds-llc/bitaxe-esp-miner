//! Fixed Serial/JTAG Worker owner with independent link supervision.

mod link;
mod writer;

use crate::bwg_worker_nvs::{BwgWorkerNvs, EspDeviceIdentitySeedGenerator};
use crate::bwg_worker_session::ProductionWorkerSession;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use crate::startup::BootMiningBaselineConfirmed;
use bitaxe_worker_control::serial::{SerialKind, SerialSessionBinding};
use bitaxe_worker_control::{
    load_or_generate_device_identity, WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier,
    WorkerControl,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::OnceLock;
use std::time::Duration;
use zeroize::Zeroize;

const OWNER_STACK_BYTES: usize = 16 * 1024;
const EVENT_CAPACITY: usize = 4;
const DEPLOYMENT_TRUST: &str = include_str!("../bwg/deployment-trust.json");
const ULTRA205_CAPABILITY: &str = include_str!("../bwg/ultra205-capability.json");
static EVENTS: OnceLock<SyncSender<ControlEvent>> = OnceLock::new();
static OUTPUT: OnceLock<SyncSender<writer::Output>> = OnceLock::new();
static CURRENT_SESSION: AtomicU32 = AtomicU32::new(0);
static AUTHENTICATED_SESSION: AtomicU32 = AtomicU32::new(0);

enum ControlEvent {
    Session {
        epoch: u32,
        binding: SerialSessionBinding,
        generation: WorkerGeneration,
    },
    Frame {
        epoch: u32,
        bytes: SecretBytes,
    },
}
struct SecretBytes(Vec<u8>);
impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
pub(crate) struct BwgWorkerRecovery {
    nvs: BwgWorkerNvs,
    reboot_report_required: bool,
}
pub(crate) struct PreparedWorkerRuntime(());
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
    let manifest_sha256 = bitaxe_worker_control::serial::serial_manifest_sha256()?;
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
            .pointer("/attestation/claims/serialManifestSha256")
            .and_then(|value| value.as_str())
            != Some(manifest_sha256.as_str())
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
        ProductionWorkerSession::default(),
        reboot_report_required.then_some(bitaxe_worker_control::RestorationReason::Reboot),
        bitaxe_worker_control::FirmwareIdentity::new(
            firmware_source_commit,
            &crate::app_elf_sha256(),
        )?,
        capability,
        &manifest_sha256,
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
    install_writer()?;
    std::thread::Builder::new()
        .name("bwg-serial-link".into())
        .stack_size(8192)
        .spawn(link::run)?;
    Ok(())
}

pub(crate) fn install_diagnostics() -> anyhow::Result<()> {
    install_writer()
}

fn install_writer() -> anyhow::Result<()> {
    let (sender, output) = mpsc::sync_channel(EVENT_CAPACITY);
    OUTPUT
        .set(sender)
        .map_err(|_| anyhow::anyhow!("serial writer already started"))?;
    crate::usb_runtime::install()?;
    std::thread::Builder::new()
        .name("bwg-serial-writer".into())
        .stack_size(8192)
        .spawn(move || writer::run(output))?;
    Ok(())
}

fn run_owner<V>(
    receiver: Receiver<ControlEvent>,
    worker: &mut WorkerControl<V, ProductionWorkerSession>,
) where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
{
    let mut owner_epoch = 0;
    loop {
        let now = crate::runtime_uptime::millis();
        if owner_epoch != 0 && CURRENT_SESSION.load(Ordering::Acquire) != owner_epoch {
            let _ = AUTHENTICATED_SESSION.compare_exchange(
                owner_epoch,
                0,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            if worker.disconnect(now).is_err() {
                diagnostic("bwg_worker event=restoration_pending");
            }
            owner_epoch = 0;
        }
        let event = match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if worker.tick(now).is_err() {
                    diagnostic("bwg_worker event=restoration_pending");
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match event {
            ControlEvent::Session {
                epoch,
                binding,
                generation,
            } => {
                if CURRENT_SESSION.load(Ordering::Acquire) != epoch {
                    continue;
                }
                if worker.begin_serial_session(binding).is_err() {
                    revoke_epoch(epoch);
                    continue;
                }
                worker.session_mut().set_generation(generation);
                owner_epoch = epoch;
            }
            ControlEvent::Frame { epoch, bytes } => {
                if epoch != owner_epoch || CURRENT_SESSION.load(Ordering::Acquire) != epoch {
                    continue;
                }
                process_frame(worker, epoch, &bytes.0, now);
            }
        }
    }
    if worker.disconnect(crate::runtime_uptime::millis()).is_err() {
        diagnostic("bwg_worker event=restoration_pending");
    }
}

fn process_frame<V>(
    worker: &mut WorkerControl<V, ProductionWorkerSession>,
    epoch: u32,
    bytes: &[u8],
    now: u64,
) where
    V: bitaxe_worker_control::LeaseAuthorizationVerifier,
{
    let response = match worker.prepare_frame(bytes, now) {
        Ok(response) => response,
        Err(error) => {
            diagnostic(error.category());
            revoke_epoch(epoch);
            return;
        }
    };
    if CURRENT_SESSION.load(Ordering::Acquire) != epoch {
        return;
    }
    if writer::send_control(epoch, response.frame()).is_err()
        || worker.confirm_sent(response).is_err()
    {
        revoke_epoch(epoch);
        return;
    }
    if worker.is_admitted() && CURRENT_SESSION.load(Ordering::Acquire) == epoch {
        AUTHENTICATED_SESSION.store(epoch, Ordering::Release);
    }
}

fn revoke_epoch(epoch: u32) {
    let _ = CURRENT_SESSION.compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire);
    let _ = AUTHENTICATED_SESSION.compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire);
}

pub(crate) fn diagnostic(line: &str) {
    writer::diagnostic(line);
}
