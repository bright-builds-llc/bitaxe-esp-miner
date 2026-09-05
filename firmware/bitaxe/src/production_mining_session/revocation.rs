//! Lock-free link revocation and generation-stamped effect admission.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const HEARTBEAT_CUTOFF_MS: u32 = 2_800;
const LIVE: u32 = 1;
const ACTIVE: u32 = 2;
const REVOKED: u32 = 3;
const FLAGS: u32 = 3;

#[path = "revocation/reason.rs"]
mod reason;
#[path = "revocation/timing.rs"]
mod timing;
pub(crate) use reason::RevocationReason;

#[derive(Clone, Copy)]
pub(crate) struct RevocationTiming {
    pub generation: u32,
    pub revocation_reason: RevocationReason,
    pub last_valid_heartbeat_ms: u32,
    pub maybe_gate_closed_ms: Option<u32>,
    pub maybe_shutdown_started_ms: Option<u32>,
    pub active_ms: u32,
    pub generation_elapsed_ms: u32,
    pub active_limit_ms: Option<u32>,
    pub shutdown_budget_ms: u32,
    pub work_gate_remaining_ms: Option<u32>,
    pub shutdown_stage: u32,
    pub shutdown_complete: bool,
    pub submitted: u32,
    pub accepted: u32,
    pub rejected: u32,
    pub nonce_work_correlations: u32,
    pub work_dispatched: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerGeneration(u32);

impl WorkerGeneration {
    pub const fn raw(self) -> u32 {
        self.0 >> 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorkPermit {
    maybe_generation: Option<WorkerGeneration>,
    epoch: u32,
}

impl WorkPermit {
    pub(crate) const fn maybe_generation(self) -> Option<WorkerGeneration> {
        self.maybe_generation
    }
}

pub(crate) struct GenerationGate {
    state: AtomicU32,
    next_generation: AtomicU32,
    heartbeat_ms: AtomicU32,
    work_epoch: AtomicU32,
    budget_generation: AtomicU32,
    budget_deadline_ms: AtomicU32,
    budget_limit_ms: AtomicU32,
    budget_armed_generation: AtomicU32,
    timing_budget_limit_ms: AtomicU32,
    budget_limited: AtomicBool,
    lease_deadline_ms: AtomicU32,
    lease_limited: AtomicBool,
    timing_generation: AtomicU32,
    activated_ms: AtomicU32,
    closed_heartbeat_ms: AtomicU32,
    closed_ms: AtomicU32,
    closed_reason: AtomicU32,
    shutdown_started_ms: AtomicU32,
    shutdown_stage: AtomicU32,
    closed_generation: AtomicU32,
    shutdown_generation: AtomicU32,
    complete_generation: AtomicU32,
    first_dispatch_generation: AtomicU32,
    first_dispatch_ms: AtomicU32,
    halted_generation: AtomicU32,
    halted_ms: AtomicU32,
    fan_proof_generation: AtomicU32,
    last_safety_ms: AtomicU32,
    submitted: AtomicU32,
    accepted: AtomicU32,
    rejected: AtomicU32,
    correlated: AtomicU32,
    dispatched: AtomicU32,
}

impl GenerationGate {
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            next_generation: AtomicU32::new(1),
            heartbeat_ms: AtomicU32::new(0),
            work_epoch: AtomicU32::new(1),
            budget_generation: AtomicU32::new(0),
            budget_deadline_ms: AtomicU32::new(0),
            budget_limit_ms: AtomicU32::new(0),
            budget_armed_generation: AtomicU32::new(0),
            timing_budget_limit_ms: AtomicU32::new(0),
            budget_limited: AtomicBool::new(false),
            lease_deadline_ms: AtomicU32::new(0),
            lease_limited: AtomicBool::new(false),
            timing_generation: AtomicU32::new(0),
            activated_ms: AtomicU32::new(0),
            closed_heartbeat_ms: AtomicU32::new(0),
            closed_ms: AtomicU32::new(0),
            closed_reason: AtomicU32::new(0),
            shutdown_started_ms: AtomicU32::new(0),
            shutdown_stage: AtomicU32::new(0),
            closed_generation: AtomicU32::new(0),
            shutdown_generation: AtomicU32::new(0),
            complete_generation: AtomicU32::new(0),
            first_dispatch_generation: AtomicU32::new(0),
            first_dispatch_ms: AtomicU32::new(0),
            halted_generation: AtomicU32::new(0),
            halted_ms: AtomicU32::new(0),
            fan_proof_generation: AtomicU32::new(0),
            last_safety_ms: AtomicU32::new(0),
            submitted: AtomicU32::new(0),
            accepted: AtomicU32::new(0),
            rejected: AtomicU32::new(0),
            correlated: AtomicU32::new(0),
            dispatched: AtomicU32::new(0),
        }
    }

    pub fn begin_link(&self, now_ms: u64) -> Option<WorkerGeneration> {
        let id = self.next_generation.fetch_add(1, Ordering::AcqRel);
        if id == 0 || id > (u32::MAX >> 2) {
            return None;
        }
        let generation = WorkerGeneration(id << 2);
        self.state
            .compare_exchange(0, generation.0, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        self.heartbeat_ms.store(now_ms as u32, Ordering::Release);
        self.lease_limited.store(false, Ordering::Release);
        self.state.store(generation.0 | LIVE, Ordering::Release);
        Some(generation)
    }

    pub fn heartbeat(&self, generation: WorkerGeneration, now_ms: u64) -> bool {
        if !self.is_live(generation) {
            return false;
        }
        self.heartbeat_ms.store(now_ms as u32, Ordering::Release);
        self.is_live(generation)
    }

    pub fn is_live(&self, generation: WorkerGeneration) -> bool {
        let state = self.state.load(Ordering::Acquire);
        state == generation.0 | LIVE || state == generation.0 | ACTIVE
    }

    pub fn activate(&self, generation: WorkerGeneration) -> bool {
        self.activate_at(
            generation,
            u64::from(self.heartbeat_ms.load(Ordering::Acquire)),
        )
    }

    pub fn activate_at(&self, generation: WorkerGeneration, now_ms: u64) -> bool {
        if self.budget_generation.load(Ordering::Acquire) != generation.0 {
            return false;
        }
        let activated = self
            .state
            .compare_exchange(
                generation.0 | LIVE,
                generation.0 | ACTIVE,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
        if activated {
            self.block_work();
            self.timing_generation.store(0, Ordering::Release);
            self.activated_ms.store(now_ms as u32, Ordering::Release);
            self.timing_budget_limit_ms.store(
                self.budget_limit_ms.load(Ordering::Acquire),
                Ordering::Release,
            );
            self.shutdown_stage.store(0, Ordering::Release);
            for counter in [
                &self.submitted,
                &self.accepted,
                &self.rejected,
                &self.correlated,
                &self.dispatched,
            ] {
                counter.store(0, Ordering::Release);
            }
            self.timing_generation
                .store(generation.0, Ordering::Release);
        }
        activated
    }

    pub fn set_lease_deadline(&self, generation: WorkerGeneration, deadline_ms: u64) -> bool {
        if !self.is_live(generation) {
            return false;
        }
        self.lease_deadline_ms
            .store(deadline_ms as u32, Ordering::Release);
        self.lease_limited.store(true, Ordering::Release);
        self.is_live(generation)
    }

    /// Called once only after the persistent campaign reservation is committed.
    pub fn admit_budget(&self, generation: WorkerGeneration, active_limit_ms: u64) -> bool {
        if active_limit_ms != u64::MAX
            && (active_limit_ms <= u64::from(super::shutdown_budget::PRE_RESET_BOUND_MS)
                || active_limit_ms > 240_000)
        {
            return false;
        }
        if self.state.load(Ordering::Acquire) != generation.0 | LIVE {
            return false;
        }
        if self
            .budget_generation
            .compare_exchange(0, generation.0, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        self.budget_limit_ms.store(
            if active_limit_ms == u64::MAX {
                0
            } else {
                active_limit_ms as u32
            },
            Ordering::Release,
        );
        self.budget_limited
            .store(active_limit_ms != u64::MAX, Ordering::Release);
        self.is_live(generation)
    }

    pub fn permits(&self, maybe_generation: Option<WorkerGeneration>) -> bool {
        let state = self.state.load(Ordering::Acquire);
        match maybe_generation {
            Some(generation) => state == generation.0 | ACTIVE,
            None => state == 0 || state & FLAGS == LIVE,
        }
    }

    #[cfg(test)]
    pub fn revoke(&self, generation: WorkerGeneration) -> bool {
        self.revoke_reason_at(
            generation,
            u64::from(self.heartbeat_ms.load(Ordering::Acquire)),
            RevocationReason::ControlFailed,
        )
    }

    pub fn revoke_at(&self, generation: WorkerGeneration, now_ms: u64) -> bool {
        self.revoke_reason_at(generation, now_ms, RevocationReason::RestorationRequested)
    }

    pub fn revoke_reason_at(
        &self,
        generation: WorkerGeneration,
        now_ms: u64,
        reason: RevocationReason,
    ) -> bool {
        // Only the winning CAS publishes a reason. A later cleanup cannot replace it.
        for flag in [ACTIVE, LIVE, ACTIVE] {
            let revoked_state = if flag == LIVE {
                0
            } else {
                generation.0 | REVOKED
            };
            if self
                .state
                .compare_exchange(
                    generation.0 | flag,
                    revoked_state,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
            {
                continue;
            }
            if flag == ACTIVE {
                self.block_work();
                self.closed_heartbeat_ms
                    .store(self.heartbeat_ms.load(Ordering::Acquire), Ordering::Release);
                self.closed_ms.store(now_ms as u32, Ordering::Release);
                self.closed_reason.store(reason as u32, Ordering::Release);
                self.closed_generation
                    .store(generation.0, Ordering::Release);
            } else {
                let _result = self.budget_generation.compare_exchange(
                    generation.0,
                    0,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            }
            return true;
        }
        false
    }

    pub fn check_deadline(&self, now_ms: u64) {
        let state = self.state.load(Ordering::Acquire);
        if !matches!(state & FLAGS, LIVE | ACTIVE) {
            return;
        }
        let now = now_ms as u32;
        let reason = if now.wrapping_sub(self.heartbeat_ms.load(Ordering::Acquire))
            >= HEARTBEAT_CUTOFF_MS
        {
            RevocationReason::HeartbeatTimeout
        } else if (self.lease_limited.load(Ordering::Acquire)
            && now.wrapping_sub(self.lease_deadline_ms.load(Ordering::Acquire)) as i32 >= 0)
            || (state & FLAGS == ACTIVE
                && self.budget_limited.load(Ordering::Acquire)
                && self.budget_armed_generation.load(Ordering::Acquire) == state & !FLAGS
                && now.wrapping_sub(self.budget_deadline_ms.load(Ordering::Acquire)) as i32 >= 0)
        {
            RevocationReason::LeaseOrBudgetExpired
        } else if state & FLAGS == ACTIVE
            && self.fan_proof_generation.load(Ordering::Acquire) == state & !FLAGS
            && now.wrapping_sub(self.last_safety_ms.load(Ordering::Acquire)) > 1_000
        {
            RevocationReason::UnsafeObservation
        } else {
            return;
        };
        self.revoke_reason_at(WorkerGeneration(state & !FLAGS), now_ms, reason);
    }

    pub fn note_fan_proof(&self, generation: WorkerGeneration, now_ms: u64) {
        if !self.permits(Some(generation)) {
            return;
        }
        self.last_safety_ms.store(now_ms as u32, Ordering::Release);
        self.fan_proof_generation
            .store(generation.0, Ordering::Release);
    }

    pub fn check_safety(&self, safe: bool, nonzero_fan: bool, now_ms: u64) {
        let state = self.state.load(Ordering::Acquire);
        if state & FLAGS != ACTIVE {
            return;
        }
        let generation = WorkerGeneration(state & !FLAGS);
        if !safe
            || (self.fan_proof_generation.load(Ordering::Acquire) == generation.0 && !nonzero_fan)
        {
            self.revoke_reason_at(generation, now_ms, RevocationReason::UnsafeObservation);
            return;
        }
        self.last_safety_ms.store(now_ms as u32, Ordering::Release);
    }

    pub fn maybe_revoked(&self) -> Option<WorkerGeneration> {
        let state = self.state.load(Ordering::Acquire);
        (state & FLAGS == REVOKED).then_some(WorkerGeneration(state & !FLAGS))
    }

    pub fn finish_shutdown(&self, generation: WorkerGeneration) {
        if self.state.load(Ordering::Acquire) != generation.0 | REVOKED {
            return;
        }
        self.budget_generation.store(0, Ordering::Release);
        if self.timing_generation.load(Ordering::Acquire) == generation.0 {
            self.complete_generation
                .store(generation.0, Ordering::Release);
        }
        let _result = self.state.compare_exchange(
            generation.0 | REVOKED,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub fn note_shutdown(&self, generation: WorkerGeneration, stage: u32, now_ms: u64) {
        if self.timing_generation.load(Ordering::Acquire) != generation.0 {
            return;
        }
        if self.shutdown_generation.load(Ordering::Acquire) != generation.0 {
            self.shutdown_started_ms
                .store(now_ms as u32, Ordering::Release);
            self.shutdown_generation
                .store(generation.0, Ordering::Release);
        }
        self.shutdown_stage.store(stage, Ordering::Release);
    }

    pub fn publish_counts(
        &self,
        generation: WorkerGeneration,
        accepted: u64,
        rejected: u64,
        correlated: u64,
    ) {
        if self.timing_generation.load(Ordering::Acquire) != generation.0 {
            return;
        }
        self.accepted.store(
            u32::try_from(accepted).unwrap_or(u32::MAX),
            Ordering::Release,
        );
        self.rejected.store(
            u32::try_from(rejected).unwrap_or(u32::MAX),
            Ordering::Release,
        );
        self.correlated.store(
            u32::try_from(correlated).unwrap_or(u32::MAX),
            Ordering::Release,
        );
    }

    pub fn note_io(&self, maybe_generation: Option<WorkerGeneration>, submission: bool) {
        let Some(generation) = maybe_generation else {
            return;
        };
        if self.timing_generation.load(Ordering::Acquire) != generation.0 {
            return;
        }
        let counter = if submission {
            &self.submitted
        } else {
            &self.dispatched
        };
        counter.fetch_add(1, Ordering::AcqRel);
    }

    pub fn begin_dispatch(&self, permit: WorkPermit, now_ms: u64) -> bool {
        self.check_deadline(now_ms);
        if !self.permits_work(permit) {
            return false;
        }
        let Some(generation) = permit.maybe_generation else {
            return true;
        };
        if self.first_dispatch_generation.load(Ordering::Acquire) == generation.0 {
            return true;
        }
        if self.timing_generation.load(Ordering::Acquire) != generation.0 {
            return false;
        }
        if self.budget_limited.load(Ordering::Acquire) {
            let limit = self.budget_limit_ms.load(Ordering::Acquire);
            let Some(window) = limit.checked_sub(super::shutdown_budget::PRE_RESET_BOUND_MS) else {
                return false;
            };
            self.budget_deadline_ms
                .store((now_ms as u32).wrapping_add(window), Ordering::Release);
            self.budget_armed_generation
                .store(generation.0, Ordering::Release);
        }
        self.first_dispatch_ms
            .store(now_ms as u32, Ordering::Release);
        self.first_dispatch_generation
            .store(generation.0, Ordering::Release);
        self.permits_work(permit)
    }

    #[cfg(test)]
    pub fn note_first_dispatch(&self, maybe_generation: Option<WorkerGeneration>, now_ms: u64) {
        let _ = self.begin_dispatch(self.stamp(maybe_generation), now_ms);
    }

    /// Records a successful hardware halt before any logging or cooling work.
    pub fn note_asic_halted(&self, now_ms: u64) {
        let generation = self.timing_generation.load(Ordering::Acquire);
        if generation == 0
            || self.first_dispatch_generation.load(Ordering::Acquire) != generation
            || self.halted_generation.load(Ordering::Acquire) == generation
        {
            return;
        }
        self.halted_ms.store(now_ms as u32, Ordering::Release);
        self.halted_generation.store(generation, Ordering::Release);
        if self.state.load(Ordering::Acquire) == generation | ACTIVE {
            self.revoke_reason_at(
                WorkerGeneration(generation),
                now_ms,
                RevocationReason::ControlFailed,
            );
        }
    }

    pub fn block_work(&self) {
        self.work_epoch.fetch_add(1, Ordering::AcqRel);
    }

    pub fn stamp(&self, maybe_generation: Option<WorkerGeneration>) -> WorkPermit {
        WorkPermit {
            maybe_generation,
            epoch: self.work_epoch.load(Ordering::Acquire),
        }
    }

    pub fn permits_work(&self, permit: WorkPermit) -> bool {
        permit.epoch == self.work_epoch.load(Ordering::Acquire)
            && self.permits(permit.maybe_generation)
    }
}

static GATE: GenerationGate = GenerationGate::new();

pub fn begin_link(now_ms: u64) -> Option<WorkerGeneration> {
    GATE.begin_link(now_ms)
}
pub fn heartbeat(generation: WorkerGeneration, now_ms: u64) -> bool {
    GATE.heartbeat(generation, now_ms)
}
pub(crate) fn revoke_reason_at(
    generation: WorkerGeneration,
    now_ms: u64,
    reason: RevocationReason,
) -> bool {
    GATE.revoke_reason_at(generation, now_ms, reason)
}
pub fn revoke_at(generation: WorkerGeneration, now_ms: u64) -> bool {
    GATE.revoke_at(generation, now_ms)
}
pub(crate) fn set_lease_deadline(generation: WorkerGeneration, deadline_ms: u64) -> bool {
    GATE.set_lease_deadline(generation, deadline_ms)
}
pub(crate) fn note_shutdown(generation: WorkerGeneration, stage: u32, now_ms: u64) {
    GATE.note_shutdown(generation, stage, now_ms);
}
pub(crate) fn timing(now_ms: u64) -> Option<RevocationTiming> {
    GATE.timing(now_ms)
}
pub(crate) fn publish_counts(
    generation: WorkerGeneration,
    accepted: u64,
    rejected: u64,
    correlated: u64,
) {
    GATE.publish_counts(generation, accepted, rejected, correlated);
}
pub(crate) fn note_submission(maybe_generation: Option<WorkerGeneration>) {
    GATE.note_io(maybe_generation, true);
}
pub(crate) fn note_dispatch(maybe_generation: Option<WorkerGeneration>, _now_ms: u64) {
    GATE.note_io(maybe_generation, false);
}
pub fn check_deadline(now_ms: u64) {
    GATE.check_deadline(now_ms);
}
pub(crate) fn note_fan_proof(generation: WorkerGeneration, now_ms: u64) {
    GATE.note_fan_proof(generation, now_ms);
}
pub(crate) fn check_safety(safe: bool, nonzero_fan: bool, now_ms: u64) {
    GATE.check_safety(safe, nonzero_fan, now_ms);
}
pub(crate) fn admit_budget(generation: WorkerGeneration, active_limit_ms: u64) -> bool {
    super::shutdown_budget::conservative_plan_is_bounded()
        && GATE.admit_budget(generation, active_limit_ms)
}
pub(crate) fn activate(generation: WorkerGeneration, now_ms: u64) -> bool {
    GATE.activate_at(generation, now_ms)
}
pub(crate) fn is_live(generation: WorkerGeneration) -> bool {
    GATE.is_live(generation)
}
pub(crate) fn permits(maybe_generation: Option<WorkerGeneration>) -> bool {
    GATE.permits(maybe_generation)
}
pub(crate) fn maybe_revoked() -> Option<WorkerGeneration> {
    GATE.maybe_revoked()
}
pub(crate) fn finish_shutdown(generation: WorkerGeneration) {
    GATE.finish_shutdown(generation);
}
pub(crate) fn block_work() {
    GATE.block_work();
}
pub(crate) fn stamp(maybe_generation: Option<WorkerGeneration>) -> WorkPermit {
    GATE.stamp(maybe_generation)
}
pub(crate) fn permits_work(permit: WorkPermit) -> bool {
    GATE.permits_work(permit)
}

#[cfg(test)]
#[path = "revocation/tests.rs"]
mod tests;

pub(crate) fn begin_dispatch(permit: WorkPermit, now_ms: u64) -> bool {
    GATE.begin_dispatch(permit, now_ms)
}
pub(crate) fn note_asic_halted(now_ms: u64) {
    GATE.note_asic_halted(now_ms);
}
