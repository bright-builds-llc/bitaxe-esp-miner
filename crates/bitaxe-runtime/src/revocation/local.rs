//! Local generations (ADR-0038). Local mining runs under the same gate as a lease,
//! so every device-local stop applies to it: an unsafe reading, a zero fan reading,
//! and no safe sample within 1 s after the fan proof. It has no Gate link, so it has
//! no heartbeat cutoff and no lease deadline (`deadline.rs`); its budget may be
//! unlimited (`admit_budget` with `u64::MAX`).
use super::*;

impl GenerationGate {
    /// Begins a local generation. Refused while any other generation exists,
    /// including an idle Gate link: handing over between them is ADR-0038 step 4.
    pub fn begin_local(&self) -> Option<WorkerGeneration> {
        let id = self.next_generation.fetch_add(1, Ordering::AcqRel);
        if id == 0 || id > (u32::MAX >> GENERATION_SHIFT) {
            return None;
        }
        let generation = WorkerGeneration(id << GENERATION_SHIFT);
        self.state
            .compare_exchange(0, generation.0, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        // Published before the generation becomes live, so a deadline check never
        // sees a live local generation as linked.
        self.local_generation.store(generation.0, Ordering::Release);
        self.lease_limited.store(false, Ordering::Release);
        self.state.store(generation.0 | LIVE, Ordering::Release);
        Some(generation)
    }

    /// Whether a Gate link (or its diagnostic, reservation or lease) holds the gate.
    /// Local mining stays off then; a finished generation holds nothing.
    pub fn holds_link(&self) -> bool {
        let state = self.state.load(Ordering::Acquire);
        let generation = state & !FLAGS;
        generation != 0 && generation != self.local_generation.load(Ordering::Acquire)
    }

    /// Whether `generation` was begun by [`Self::begin_local`].
    pub fn is_local(&self, generation: WorkerGeneration) -> bool {
        generation.0 != 0 && self.local_generation.load(Ordering::Acquire) == generation.0
    }
}
