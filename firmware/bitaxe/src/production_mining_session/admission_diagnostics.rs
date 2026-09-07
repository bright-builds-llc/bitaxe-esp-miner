//! Boot-local diagnostic observations, never session, campaign, or work authority.
//! Relaxed atomic reads may span transitions; consumers must not authorize from them.

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum Stage {
    Idle,
    Admission,
    Readiness,
    Preparation,
    PoolActivation,
    Active,
    Cleanup,
    Complete,
}
#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum Failure {
    Admission = 1,
    Readiness,
    Preparation,
    PoolActivation,
    Cleanup,
}

static STAGE: AtomicU32 = AtomicU32::new(Stage::Idle as u32);
static FIRST_FAILURE: AtomicU32 = AtomicU32::new(0);
static READINESS: AtomicU32 = AtomicU32::new(0);

/// Starts a diagnostic attempt only; this creates no admission or lease.
pub(crate) fn begin() {
    FIRST_FAILURE.store(0, Ordering::Relaxed);
    READINESS.store(0, Ordering::Relaxed);
    stage(Stage::Admission);
}
pub(crate) fn stage(value: Stage) {
    STAGE.store(value as u32, Ordering::Relaxed);
}
pub(crate) fn fail(value: Failure) {
    retain_first_failure(&FIRST_FAILURE, value);
}
fn retain_first_failure(latch: &AtomicU32, value: Failure) {
    let _ = latch.compare_exchange(0, value as u32, Ordering::Relaxed, Ordering::Relaxed);
}
/// Records the current incomplete phase before cleanup can replace it.
pub(crate) fn fail_current() {
    let value = match STAGE.load(Ordering::Relaxed) {
        1 => Failure::Admission,
        2 => Failure::Readiness,
        3 => Failure::Preparation,
        4 => Failure::PoolActivation,
        _ => return,
    };
    fail(value);
}
/// Bits 0..5: run intent, network, protocol, fresh safety, lease, actuation owner.
pub(super) fn readiness(value: bitaxe_stratum::v1::production_session::ProductionReadiness) {
    let flags = [
        value.operator_intent == bitaxe_stratum::v1::state::MiningOperatorIntent::Run,
        value.network_ready,
        value.stratum_v1_supported,
        value.safety_prerequisites_fresh,
        value.maybe_campaign_lease.is_some(),
        value.actuation_qualified,
    ];
    READINESS.store(
        flags
            .iter()
            .enumerate()
            .fold(0, |mask, (bit, set)| mask | (u32::from(*set) << bit)),
        Ordering::Relaxed,
    );
}

pub(crate) fn marker() -> String {
    let (reserved, complete) = crate::worker_acceptance_budget::diagnostic_snapshot();
    render(
        STAGE.load(Ordering::Relaxed),
        FIRST_FAILURE.load(Ordering::Relaxed),
        READINESS.load(Ordering::Relaxed),
        reserved,
        complete,
    )
}
fn render(stage: u32, failure: u32, readiness: u32, reserved: u32, complete: bool) -> String {
    let stage = match stage {
        1 => "admission",
        2 => "readiness",
        3 => "preparation",
        4 => "pool_activation",
        5 => "active",
        6 => "cleanup",
        7 => "complete",
        _ => "idle",
    };
    let failure = match failure {
        1 => "admission",
        2 => "readiness",
        3 => "preparation",
        4 => "pool_activation",
        5 => "cleanup",
        _ => "none",
    };
    format!("worker_admission schema=v1 stage={stage} first_failure={failure} readiness={readiness} budget_reserved_ms={reserved} budget_complete={complete} redacted=true")
}

#[cfg(test)]
mod tests {
    #[test]
    fn cleanup_cannot_replace_the_original_admission_failure() {
        // Arrange
        let latch = super::AtomicU32::new(0);
        // Act
        super::retain_first_failure(&latch, super::Failure::Admission);
        super::retain_first_failure(&latch, super::Failure::Cleanup);
        // Assert
        assert_eq!(
            latch.load(super::Ordering::Relaxed),
            super::Failure::Admission as u32
        );
    }

    #[test]
    fn budget_observation_does_not_require_generation_timing() {
        assert_eq!(super::render(0, 0, 0, 180_000, false), "worker_admission schema=v1 stage=idle first_failure=none readiness=0 budget_reserved_ms=180000 budget_complete=false redacted=true");
    }
}
