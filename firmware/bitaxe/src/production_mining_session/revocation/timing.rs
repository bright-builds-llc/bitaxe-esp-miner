//! Bounded atomic projection of the latest admitted mining generation.
use super::*;

impl GenerationGate {
    pub fn timing(&self, now_ms: u64) -> Option<RevocationTiming> {
        let generation = self.timing_generation.load(Ordering::Acquire);
        if generation == 0 {
            return None;
        }
        let closed_for_generation = self.closed_generation.load(Ordering::Acquire) == generation;
        let shutdown_for_generation =
            self.shutdown_generation.load(Ordering::Acquire) == generation;
        let closed = self.closed_ms.load(Ordering::Acquire);
        let halted = self.halted_generation.load(Ordering::Acquire) == generation;
        let started = self.first_dispatch_generation.load(Ordering::Acquire) == generation;
        let end_ms = if halted {
            self.halted_ms.load(Ordering::Acquire)
        } else if closed_for_generation && !started {
            closed
        } else {
            now_ms as u32
        };
        let active_limit = self.timing_budget_limit_ms.load(Ordering::Acquire);
        let timing = RevocationTiming {
            generation: generation >> 2,
            revocation_reason: if closed_for_generation {
                RevocationReason::from_code(self.closed_reason.load(Ordering::Acquire))
            } else {
                RevocationReason::NotRevoked
            },
            last_valid_heartbeat_ms: if closed_for_generation {
                self.closed_heartbeat_ms.load(Ordering::Acquire)
            } else {
                self.heartbeat_ms.load(Ordering::Acquire)
            },
            maybe_gate_closed_ms: closed_for_generation.then_some(closed),
            maybe_shutdown_started_ms: shutdown_for_generation
                .then(|| self.shutdown_started_ms.load(Ordering::Acquire)),
            active_ms: if started {
                end_ms.wrapping_sub(self.first_dispatch_ms.load(Ordering::Acquire))
            } else {
                0
            },
            generation_elapsed_ms: end_ms.wrapping_sub(self.activated_ms.load(Ordering::Acquire)),
            active_limit_ms: (active_limit != 0).then_some(active_limit),
            shutdown_budget_ms: super::super::shutdown_budget::PRE_RESET_BOUND_MS,
            work_gate_remaining_ms: if active_limit == 0 || !started {
                None
            } else if closed_for_generation {
                Some(0)
            } else {
                let deadline = self.budget_deadline_ms.load(Ordering::Acquire);
                let remaining = deadline.wrapping_sub(now_ms as u32);
                Some(if remaining as i32 <= 0 {
                    0
                } else {
                    remaining.min(active_limit)
                })
            },
            shutdown_stage: self.shutdown_stage.load(Ordering::Acquire),
            shutdown_complete: self.complete_generation.load(Ordering::Acquire) == generation,
            submitted: self.submitted.load(Ordering::Acquire),
            accepted: self.accepted.load(Ordering::Acquire),
            rejected: self.rejected.load(Ordering::Acquire),
            nonce_work_correlations: self.correlated.load(Ordering::Acquire),
            work_dispatched: self.dispatched.load(Ordering::Acquire),
        };
        (self.timing_generation.load(Ordering::Acquire) == generation).then_some(timing)
    }
}
