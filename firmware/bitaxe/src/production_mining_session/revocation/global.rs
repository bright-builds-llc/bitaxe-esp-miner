//! Boot-lifetime generation gate shared by the existing hardware owners.
use super::{GenerationGate, RevocationReason, RevocationTiming, WorkPermit, WorkerGeneration};

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
pub(crate) fn release_unbudgeted_reservation(generation: WorkerGeneration) -> bool {
    GATE.release_unbudgeted_reservation(generation)
}
pub(crate) fn begin_reservation(generation: WorkerGeneration) -> bool {
    GATE.begin_reservation(generation)
}
pub(crate) fn admit_budget(generation: WorkerGeneration, active_limit_ms: u64) -> bool {
    super::super::shutdown_budget::conservative_plan_is_bounded()
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

pub(crate) fn begin_dispatch(permit: WorkPermit, now_ms: u64) -> bool {
    GATE.begin_dispatch(permit, now_ms)
}
pub(crate) fn note_asic_halted(now_ms: u64) {
    GATE.note_asic_halted(now_ms);
}
