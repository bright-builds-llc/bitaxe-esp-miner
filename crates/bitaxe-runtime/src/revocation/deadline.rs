//! Deadline-driven revocation: the heartbeat cutoff, the lease and budget deadlines,
//! and safety silence after the fan proof. A local generation (ADR-0038) skips the
//! first two and keeps the rest.
use super::*;

impl GenerationGate {
    pub fn check_deadline(&self, now_ms: u64) {
        let state = self.state.load(Ordering::Acquire);
        if !matches!(
            state & FLAGS,
            LIVE | RESERVED | BUDGETED | ACTIVE | DIAGNOSTIC
        ) {
            return;
        }
        let now = now_ms as u32;
        // A local generation has no Gate link: no heartbeat and no lease deadline.
        let linked = !self.is_local(WorkerGeneration(state & !FLAGS));
        let reason = if linked
            && elapsed_ms(now, self.heartbeat_ms.load(Ordering::Acquire)) >= HEARTBEAT_CUTOFF_MS
        {
            RevocationReason::HeartbeatTimeout
        } else if (linked
            && self.lease_limited.load(Ordering::Acquire)
            && now.wrapping_sub(self.lease_deadline_ms.load(Ordering::Acquire)) as i32 >= 0)
            || (state & FLAGS == ACTIVE
                && self.budget_limited.load(Ordering::Acquire)
                && self.budget_armed_generation.load(Ordering::Acquire) == state & !FLAGS
                && now.wrapping_sub(self.budget_deadline_ms.load(Ordering::Acquire)) as i32 >= 0)
        {
            RevocationReason::LeaseOrBudgetExpired
        } else if state & FLAGS == ACTIVE
            && self.fan_proof_generation.load(Ordering::Acquire) == state & !FLAGS
            && elapsed_ms(now, self.last_safety_ms.load(Ordering::Acquire)) > 1_000
        {
            self.revoke_unsafe_at(
                WorkerGeneration(state & !FLAGS),
                now_ms,
                UnsafeTrigger::NoSafeSample,
                None,
            );
            return;
        } else {
            return;
        };
        self.revoke_reason_at(WorkerGeneration(state & !FLAGS), now_ms, reason);
    }
}
