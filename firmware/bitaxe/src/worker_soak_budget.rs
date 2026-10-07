//! One-shot soak allowance accounting (ADR-0033), kept apart from the legacy campaign and the
//! qualification ledger. Every soak reserves its whole budget before preparation and is never refunded.
use crate::bwg_worker_nvs::BwgWorkerNvs;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_worker_control::{SoakAllowance, SoakLedger};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Mutex,
};
struct Pending {
    previous: SoakLedger,
    reserved: SoakLedger,
    generation: u32,
}
static PENDING: Mutex<Option<Pending>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU32 = AtomicU32::new(0);
static ORDINAL: AtomicU32 = AtomicU32::new(0);
static COMPLETE: AtomicBool = AtomicBool::new(false);
/// A soak ledger that failed boot recovery blocks soak admission only, never other Work Leases.
static RECOVERY_FAILED: AtomicBool = AtomicBool::new(false);
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}
fn acquire() -> anyhow::Result<Guard> {
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| anyhow::anyhow!("soak_budget=busy"))?;
    Ok(Guard)
}

/// Reserve and persist the whole soak before activation; storage failure never activates work.
pub(crate) fn admit(generation: WorkerGeneration, allowance: &SoakAllowance) -> anyhow::Result<()> {
    let _guard = acquire()?;
    if RECOVERY_FAILED.load(Ordering::SeqCst) {
        anyhow::bail!("soak_budget=recovery_failed");
    }
    if !revocation::begin_reservation(generation) {
        anyhow::bail!("soak_budget=revoked");
    }
    let mut store = BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("soak_budget=storage"))?;
    if !store
        .maybe_acceptance_budget()?
        .is_some_and(|legacy| legacy.complete() && legacy.charged_milliseconds() == 240000)
    {
        anyhow::bail!("soak_budget=original_incomplete");
    }
    if store.qualification_ledger()?.pending() {
        anyhow::bail!("soak_budget=qualification_pending");
    }
    let previous = store.soak_ledger()?;
    let reserved = previous.reserve(allowance)?;
    *PENDING
        .lock()
        .map_err(|_| anyhow::anyhow!("soak_budget=poisoned"))? = Some(Pending {
        previous,
        reserved: reserved.clone(),
        generation: generation.raw(),
    });
    GENERATION.store(0, Ordering::SeqCst);
    ORDINAL.store(allowance.ordinal(), Ordering::SeqCst);
    COMPLETE.store(false, Ordering::SeqCst);
    GENERATION.store(generation.raw(), Ordering::SeqCst);
    store.store_soak_ledger(&reserved)?;
    if !revocation::admit_budget(generation, allowance.maximum_active_milliseconds()) {
        anyhow::bail!("soak_budget=revoked");
    }
    Ok(())
}

/// Idempotent completion; an interrupted or failed reservation write still charges the full soak.
pub(crate) fn finish(generation: WorkerGeneration) -> anyhow::Result<()> {
    let _guard = acquire()?;
    let mut pending = PENDING
        .lock()
        .map_err(|_| anyhow::anyhow!("soak_budget=poisoned"))?;
    let Some(expected) = pending
        .as_ref()
        .filter(|value| value.generation == generation.raw())
    else {
        return Ok(());
    };
    let mut store = BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("soak_budget=storage"))?;
    let observed = store.soak_ledger()?;
    let stopped = expected.reserved.finish()?;
    if observed != expected.previous && observed != expected.reserved && observed != stopped {
        anyhow::bail!("soak_budget=unexpected_storage");
    }
    store.store_soak_ledger(&stopped)?;
    COMPLETE.store(true, Ordering::SeqCst);
    *pending = None;
    Ok(())
}

/// Completes a pending soak without refund. A failure is retained and blocks only soak admission, so
/// a corrupt or incompatible soak ledger never disables conservative or qualification Work Leases.
pub(crate) fn recover_after_boot(proof: &crate::startup::BootMiningBaselineConfirmed) {
    let failed = recover_ledger(proof).is_err();
    RECOVERY_FAILED.store(failed, Ordering::SeqCst);
    if failed {
        log::warn!("soak_budget=recovery_failed");
    }
}

fn recover_ledger(_proof: &crate::startup::BootMiningBaselineConfirmed) -> anyhow::Result<()> {
    let _guard = acquire()?;
    let mut store = BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("soak_budget=storage"))?;
    let ledger = store.soak_ledger()?;
    let finished = ledger.finish()?;
    if ledger != finished {
        store.store_soak_ledger(&finished)?;
    }
    Ok(())
}

pub(crate) fn review() -> anyhow::Result<serde_json::Value> {
    let _guard = acquire()?;
    let store = BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("soak_budget=storage"))?;
    let ledger = store.soak_ledger()?;
    Ok(
        serde_json::json!({"schema":"worker-soak-ledger-v1","next_ordinal":ledger.next_ordinal(),"total_charged_ms":ledger.total_charged_ms(),"pending":ledger.pending(),"last_completed_ordinal":ledger.last_completed_ordinal()}),
    )
}

/// Cached observations do not block the link or perform storage reads.
pub(crate) fn observation(generation: u32, active_ms: u64) -> Option<serde_json::Value> {
    if generation == 0 || GENERATION.load(Ordering::SeqCst) != generation {
        return None;
    }
    let maximum = bitaxe_worker_control::SOAK_MAXIMUM_ACTIVE_MS;
    let value = serde_json::json!({"schema":"worker-soak-observation-v1","ordinal":ORDINAL.load(Ordering::SeqCst),"maximum_active_ms":maximum,"reserved_ms":maximum,"complete":COMPLETE.load(Ordering::SeqCst),"active_ms":active_ms});
    (GENERATION.load(Ordering::SeqCst) == generation).then_some(value)
}
