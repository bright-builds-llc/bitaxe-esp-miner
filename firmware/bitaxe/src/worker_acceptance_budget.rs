//! Persistent admission of the fixed acceptance budget; no lease renewal path mutates it.
use crate::bwg_worker_nvs::BwgWorkerNvs;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_api::acceptance_budget::AcceptanceBudget;
use bitaxe_worker_control::WorkerLeaseGrant;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static BUSY: AtomicBool = AtomicBool::new(false);
static ACCEPTANCE_GENERATION: AtomicU32 = AtomicU32::new(0);
static RESERVED_MS: AtomicU32 = AtomicU32::new(0);
static COMPLETE: AtomicBool = AtomicBool::new(false);
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}
fn acquire() -> anyhow::Result<Guard> {
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| anyhow::anyhow!("acceptance_budget=busy"))?;
    Ok(Guard)
}

/// Called after full Start authorization, before activation or hardware preparation.
pub(crate) fn admit(generation: WorkerGeneration, grant: &WorkerLeaseGrant) -> anyhow::Result<()> {
    let _guard = acquire()?;
    if !revocation::is_live(generation) {
        anyhow::bail!("acceptance_budget=revoked_generation");
    }
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    let maybe_budget = store.maybe_acceptance_budget()?;
    let active_limit_ms = if let Some(campaign) = grant.maybe_acceptance_campaign() {
        let ledger = match maybe_budget {
            Some(value) => value,
            None => AcceptanceBudget::new(campaign.id())?,
        };
        let reserved = ledger.reserve(
            campaign.id(),
            campaign.window(),
            campaign.maximum_active_milliseconds(),
        )?;
        store.store_acceptance_budget(&reserved)?;
        RESERVED_MS.store(reserved.charged_milliseconds(), Ordering::Release);
        COMPLETE.store(false, Ordering::Release);
        ACCEPTANCE_GENERATION.store(generation.raw(), Ordering::Release);
        // Revocation arms this window at first dispatch and reserves the validated shutdown tail.
        campaign.maximum_active_milliseconds()
    } else {
        if maybe_budget.is_some_and(|budget| !budget.complete()) {
            anyhow::bail!("acceptance_budget=unfinished_campaign");
        }
        u64::MAX
    };
    if !revocation::admit_budget(generation, active_limit_ms) {
        anyhow::bail!("acceptance_budget=revoked_generation");
    }
    Ok(())
}

/// Qualified stop completes a reserved window; repeated completion is idempotent.
pub(crate) fn finish(generation: WorkerGeneration) -> anyhow::Result<()> {
    if ACCEPTANCE_GENERATION.load(Ordering::Acquire) != generation.raw() {
        return Ok(());
    }
    let _guard = acquire()?;
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    if let Some(ledger) = store.maybe_acceptance_budget()? {
        let stopped = ledger.finish()?;
        store.store_acceptance_budget(&stopped)?;
        COMPLETE.store(stopped.complete(), Ordering::Release);
    }
    ACCEPTANCE_GENERATION.store(0, Ordering::Release);
    Ok(())
}

/// Only called with the existing boot-safe proof; interruption never refunds its reservation.
pub(crate) fn recover_after_boot(
    _proof: &crate::startup::BootMiningBaselineConfirmed,
) -> anyhow::Result<()> {
    let _guard = acquire()?;
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    if let Some(ledger) = store.maybe_acceptance_budget()? {
        let stopped = ledger.finish()?;
        if stopped != ledger {
            store.store_acceptance_budget(&stopped)?;
        }
        RESERVED_MS.store(stopped.charged_milliseconds(), Ordering::Release);
        COMPLETE.store(stopped.complete(), Ordering::Release);
    }
    Ok(())
}

/// Cached closed facts; diagnostic polling never opens NVS or waits for the budget owner.
pub(crate) fn diagnostic_snapshot() -> (u32, bool) {
    (
        RESERVED_MS.load(Ordering::Acquire),
        COMPLETE.load(Ordering::Acquire),
    )
}
