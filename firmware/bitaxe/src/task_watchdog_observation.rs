//! Shared observation store for producer-owned ESP task-watchdog facts.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

use bitaxe_core::runtime_health::{TaskWatchdogObservation, TaskWatchdogOwnerPhase};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TaskWatchdogObservationHistory {
    pub(crate) maybe_previous: Option<TaskWatchdogObservation>,
    pub(crate) maybe_latest: Option<TaskWatchdogObservation>,
}

static OBSERVATIONS: OnceLock<Mutex<TaskWatchdogObservationHistory>> = OnceLock::new();
static OWNER_PHASE: AtomicU8 = AtomicU8::new(TaskWatchdogOwnerPhase::Unavailable as u8);

pub(crate) fn record_owner_phase(phase: TaskWatchdogOwnerPhase) {
    OWNER_PHASE.store(phase as u8, Ordering::Release);
}

pub(crate) fn owner_phase() -> TaskWatchdogOwnerPhase {
    TaskWatchdogOwnerPhase::from_u8(OWNER_PHASE.load(Ordering::Acquire))
}

pub(crate) fn observation_history() -> TaskWatchdogObservationHistory {
    observations().lock().map_or_else(
        |_| TaskWatchdogObservationHistory::default(),
        |value| *value,
    )
}

pub(crate) fn record(observation: TaskWatchdogObservation) {
    let Ok(mut history) = observations().lock() else {
        log::error!("task_watchdog_observation=unavailable reason=mutex_poisoned");
        return;
    };
    if history.maybe_latest == Some(observation) {
        return;
    }
    history.maybe_previous = history.maybe_latest;
    history.maybe_latest = Some(observation);
}

fn observations() -> &'static Mutex<TaskWatchdogObservationHistory> {
    OBSERVATIONS.get_or_init(|| Mutex::new(TaskWatchdogObservationHistory::default()))
}
