//! Whether the Gamma 601's BM1370 init has completed for a run (ADR-0041 item 11). Every board
//! compiles it, since the executor records and clears it on every board; only the 601 reads it.

use std::sync::atomic::{AtomicU32, Ordering};

/// The generation whose BM1370 init last completed, plus one; zero for none. An atomic, so the
/// 601's sensor sweep reads it without waiting on the executor's lock, which the init ramp
/// holds. Cleared wherever `production_ready` is cleared.
static BM1370_INIT_GENERATION: AtomicU32 = AtomicU32::new(0);

/// Records a completed BM1370 init for `maybe_generation`; an init without a generation (the
/// hash check's) records none.
pub fn record_bm1370_initialized(
    maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
) {
    let value = maybe_generation.map_or(0, |generation| generation.raw().saturating_add(1));
    BM1370_INIT_GENERATION.store(value, Ordering::Release);
}

pub fn clear_bm1370_initialized() {
    BM1370_INIT_GENERATION.store(0, Ordering::Release);
}

/// Whether the BM1370's last completed init belongs to generation `raw`.
#[cfg_attr(not(feature = "board-gamma-601"), allow(dead_code))]
pub fn bm1370_initialized_for(raw: u32) -> bool {
    let value = BM1370_INIT_GENERATION.load(Ordering::Acquire);
    value != 0 && value == raw.saturating_add(1)
}
