//! Network-only authority never enters the mining shutdown owner.
use super::*;

impl GenerationGate {
    /// Network-only reservation; it never enters the ASIC shutdown mailbox.
    pub fn claim_diagnostic(&self, generation: WorkerGeneration, deadline_ms: u64) -> bool {
        let claimed = self
            .state
            .compare_exchange(
                generation.0 | LIVE,
                generation.0 | DIAGNOSTIC,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
        if claimed {
            // Reuse the independent generation clock, without any signed lease,
            // reservation, mining activation or accounting mutation.
            self.lease_deadline_ms
                .store(deadline_ms as u32, Ordering::Release);
            self.lease_limited.store(true, Ordering::Release);
        }
        claimed
    }

    pub fn diagnostic_live(&self, generation: WorkerGeneration) -> bool {
        self.state.load(Ordering::Acquire) == generation.0 | DIAGNOSTIC
    }

    /// Called only after the separate network owner has actually joined.
    pub fn release_diagnostic(&self, generation: WorkerGeneration) -> bool {
        for (from, to) in [(DIAGNOSTIC, generation.0 | LIVE), (DIAGNOSTIC_REVOKED, 0)] {
            if self
                .state
                .compare_exchange(generation.0 | from, to, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.lease_limited.store(false, Ordering::Release);
                return true;
            }
        }
        false
    }

    pub fn diagnostic_reason(&self, generation: WorkerGeneration) -> RevocationReason {
        if self.state.load(Ordering::Acquire) == generation.0 | DIAGNOSTIC_REVOKED {
            RevocationReason::from_code(self.diagnostic_reason.load(Ordering::Acquire))
        } else {
            RevocationReason::NotRevoked
        }
    }
}
