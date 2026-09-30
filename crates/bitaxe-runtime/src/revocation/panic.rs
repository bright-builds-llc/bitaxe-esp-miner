//! Native panic revocation after IDF has stalled all other execution.
use super::{GenerationGate, Ordering, FLAGS, REVOKED};

impl GenerationGate {
    /// Native panic only: the other CPU is stalled and no task can resume.
    /// One load/store revokes every permit without a lock or retry loop.
    #[inline(always)]
    pub fn panic_revoke_all(&self) -> u32 {
        let prior = self.state.load(Ordering::Relaxed);
        self.state
            .store((prior & !FLAGS) | REVOKED, Ordering::Relaxed);
        self.state.load(Ordering::Relaxed)
    }
}
