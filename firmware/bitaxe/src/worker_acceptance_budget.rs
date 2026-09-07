//! Persistent admission of the fixed acceptance budget; no lease renewal path mutates it.
use crate::bwg_worker_nvs::BwgWorkerNvs;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_api::acceptance_budget::AcceptanceBudget;
use bitaxe_worker_control::WorkerLeaseGrant;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

static BUSY: AtomicBool = AtomicBool::new(false);
static ACCEPTANCE_GENERATION: AtomicU32 = AtomicU32::new(0);
static RESERVED_MS: AtomicU32 = AtomicU32::new(0);
static COMPLETE: AtomicBool = AtomicBool::new(false);
struct PendingReservation {
    maybe_previous: Option<AcceptanceBudget>,
    reserved: AcceptanceBudget,
}
static PENDING: Mutex<Option<PendingReservation>> = Mutex::new(None);
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
    if let Some(allowance) = grant.maybe_qualification_attempt() {
        return crate::worker_qualification_budget::admit(generation, allowance);
    }
    let _guard = acquire()?;
    if !revocation::begin_reservation(generation) {
        anyhow::bail!("acceptance_budget=revoked_generation");
    }
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    let maybe_budget = store.maybe_acceptance_budget()?;
    let active_limit_ms = if let Some(campaign) = grant.maybe_acceptance_campaign() {
        let ledger = match maybe_budget.as_ref() {
            Some(value) => value.clone(),
            None => AcceptanceBudget::new(campaign.id())?,
        };
        let reserved = ledger.reserve(
            campaign.id(),
            campaign.window(),
            campaign.maximum_active_milliseconds(),
        )?;
        *PENDING
            .lock()
            .map_err(|_| anyhow::anyhow!("acceptance_budget=poisoned"))? =
            Some(PendingReservation {
                maybe_previous: maybe_budget,
                reserved: reserved.clone(),
            });
        ACCEPTANCE_GENERATION.store(generation.raw(), Ordering::Release);
        // Track the intended full charge before a write whose readback may fail.
        RESERVED_MS.store(reserved.charged_milliseconds(), Ordering::Release);
        COMPLETE.store(false, Ordering::Release);
        store.store_acceptance_budget(&reserved)?;
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
    crate::worker_qualification_budget::finish(generation)?;
    let _guard = acquire()?;
    if ACCEPTANCE_GENERATION.load(Ordering::Acquire) != generation.raw() {
        return Ok(());
    }
    let mut maybe_pending = PENDING
        .lock()
        .map_err(|_| anyhow::anyhow!("acceptance_budget=poisoned"))?;
    let expected = maybe_pending
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("acceptance_budget=missing_pending"))?;
    let mut store =
        BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    let observed = store.maybe_acceptance_budget()?;
    let stopped = expected.reserved.finish()?;
    if observed.as_ref() != Some(&expected.reserved)
        && observed.as_ref() != Some(&stopped)
        && observed != expected.maybe_previous
    {
        anyhow::bail!("acceptance_budget=unexpected_storage");
    }
    // The original write may have failed before commit or during readback. Both
    // cases conservatively charge the full intended window; cleanup never refunds.
    store.store_acceptance_budget(&stopped)?;
    COMPLETE.store(stopped.complete(), Ordering::Release);
    *maybe_pending = None;
    ACCEPTANCE_GENERATION.store(0, Ordering::Release);
    Ok(())
}

/// Only called with the existing boot-safe proof; interruption never refunds its reservation.
pub(crate) fn recover_after_boot(
    proof: &crate::startup::BootMiningBaselineConfirmed,
) -> anyhow::Result<()> {
    crate::worker_qualification_budget::recover_after_boot(proof)?;
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

/// Fresh read-only NVS review. This neither admits work nor repairs the ledger.
pub(crate) fn review(expected_campaign: &str) -> anyhow::Result<serde_json::Value> {
    let _guard = acquire()?;
    let store = BwgWorkerNvs::open().map_err(|_| anyhow::anyhow!("acceptance_budget=storage"))?;
    let maybe_ledger = store.maybe_acceptance_budget()?;
    if let Some(ledger) = maybe_ledger.as_ref() {
        ledger.validate()?;
    }
    Ok(serde_json::json!({
        "schema": "worker-budget-review-v1",
        "campaign_match": maybe_ledger.as_ref().is_some_and(|ledger| ledger.matches_campaign(expected_campaign)),
        "reserved_mask": maybe_ledger.as_ref().map_or(0, AcceptanceBudget::reserved_mask),
        "completed_mask": maybe_ledger.as_ref().map_or(0, AcceptanceBudget::completed_mask),
        "charged_ms": maybe_ledger.as_ref().map_or(0, AcceptanceBudget::charged_milliseconds),
        "pending": maybe_ledger.as_ref().is_some_and(AcceptanceBudget::pending),
    }))
}
