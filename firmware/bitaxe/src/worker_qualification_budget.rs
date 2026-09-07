//! Independent no-refund accounting; legacy campaign bytes are never written here.
use crate::bwg_worker_nvs::BwgWorkerNvs;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_worker_control::{QualificationAttempt, QualificationLedger, QualificationPurpose};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Mutex,
};
struct Pending {
    previous: QualificationLedger,
    reserved: QualificationLedger,
    generation: u32,
}
static PENDING: Mutex<Option<Pending>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU32 = AtomicU32::new(0);
static ORDINAL: AtomicU32 = AtomicU32::new(0);
static PURPOSE: AtomicU32 = AtomicU32::new(0);
static MAXIMUM: AtomicU32 = AtomicU32::new(0);
static COMPLETE: AtomicBool = AtomicBool::new(false);
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}
fn acquire() -> anyhow::Result<Guard> {
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| anyhow::anyhow!("qualification_budget=busy"))?;
    Ok(Guard)
}

pub(crate) fn admit(
    generation: WorkerGeneration,
    allowance: &QualificationAttempt,
) -> anyhow::Result<()> {
    let _guard = acquire()?;
    if !revocation::begin_reservation(generation) {
        anyhow::bail!("qualification_budget=revoked");
    }
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("qualification_budget=storage"))?;
    if !store
        .maybe_acceptance_budget()?
        .is_some_and(|legacy| legacy.complete() && legacy.charged_milliseconds() == 240000)
    {
        anyhow::bail!("qualification_budget=original_incomplete");
    }
    let previous = store.qualification_ledger()?;
    let reserved = previous.reserve(allowance)?;
    *PENDING
        .lock()
        .map_err(|_| anyhow::anyhow!("qualification_budget=poisoned"))? = Some(Pending {
        previous,
        reserved: reserved.clone(),
        generation: generation.raw(),
    });
    GENERATION.store(0, Ordering::SeqCst);
    ORDINAL.store(allowance.ordinal(), Ordering::SeqCst);
    PURPOSE.store(
        match allowance.purpose() {
            QualificationPurpose::Diagnostic => 1,
            QualificationPurpose::Normal => 2,
            QualificationPurpose::ForegroundLoss => 3,
            QualificationPurpose::HeartbeatLoss => 4,
        },
        Ordering::SeqCst,
    );
    MAXIMUM.store(
        allowance.maximum_active_milliseconds() as u32,
        Ordering::SeqCst,
    );
    COMPLETE.store(false, Ordering::SeqCst);
    GENERATION.store(generation.raw(), Ordering::SeqCst);
    store.store_qualification_ledger(&reserved)?;
    if !revocation::admit_budget(generation, allowance.maximum_active_milliseconds()) {
        anyhow::bail!("qualification_budget=revoked");
    }
    Ok(())
}
pub(crate) fn finish(generation: WorkerGeneration) -> anyhow::Result<()> {
    let _guard = acquire()?;
    let mut pending = PENDING
        .lock()
        .map_err(|_| anyhow::anyhow!("qualification_budget=poisoned"))?;
    let Some(expected) = pending
        .as_ref()
        .filter(|value| value.generation == generation.raw())
    else {
        return Ok(());
    };
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("qualification_budget=storage"))?;
    let observed = store.qualification_ledger()?;
    let stopped = expected.reserved.finish()?;
    if observed != expected.previous && observed != expected.reserved && observed != stopped {
        anyhow::bail!("qualification_budget=unexpected_storage");
    }
    store.store_qualification_ledger(&stopped)?;
    COMPLETE.store(true, Ordering::SeqCst);
    *pending = None;
    Ok(())
}
pub(crate) fn recover_after_boot(
    _proof: &crate::startup::BootMiningBaselineConfirmed,
) -> anyhow::Result<()> {
    let _guard = acquire()?;
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("qualification_budget=storage"))?;
    let ledger = store.qualification_ledger()?;
    let finished = ledger.finish()?;
    if ledger != finished {
        store.store_qualification_ledger(&finished)?;
    }
    Ok(())
}
pub(crate) fn review() -> anyhow::Result<serde_json::Value> {
    let _guard = acquire()?;
    let store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("qualification_budget=storage"))?;
    let ledger = store.qualification_ledger()?;
    Ok(
        serde_json::json!({"schema":"worker-qualification-ledger-v1","next_ordinal":ledger.next_ordinal(),"total_charged_ms":ledger.total_charged_ms(),"pending":ledger.pending(),"last_completed_ordinal":ledger.last_completed_ordinal()}),
    )
}
/// Cached observations do not block the link or perform storage reads.
pub(crate) fn observation(generation: u32, active_ms: u64) -> Option<serde_json::Value> {
    if GENERATION.load(Ordering::SeqCst) != generation {
        return None;
    }
    let purpose = match PURPOSE.load(Ordering::SeqCst) {
        1 => "diagnostic",
        2 => "normal",
        3 => "foreground_loss",
        4 => "heartbeat_loss",
        _ => return None,
    };
    let maximum = MAXIMUM.load(Ordering::SeqCst);
    let value = serde_json::json!({"schema":"worker-qualification-observation-v1","ordinal":ORDINAL.load(Ordering::SeqCst),"purpose":purpose,"maximum_active_ms":maximum,"reserved_ms":maximum,"complete":COMPLETE.load(Ordering::SeqCst),"active_ms":active_ms});
    (GENERATION.load(Ordering::SeqCst) == generation).then_some(value)
}
