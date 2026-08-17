//! Shared observation store for producer-owned ESP task-watchdog facts.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

use bitaxe_core::runtime_health::{
    TaskWatchdogObservation, TaskWatchdogOwnerPhase, TaskWatchdogWaitObservation,
};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TaskWatchdogObservationHistory {
    pub(crate) maybe_previous: Option<TaskWatchdogObservation>,
    pub(crate) maybe_latest: Option<TaskWatchdogObservation>,
}

static OBSERVATIONS: OnceLock<Mutex<TaskWatchdogObservationHistory>> = OnceLock::new();
static OWNER_PHASE: AtomicU8 = AtomicU8::new(TaskWatchdogOwnerPhase::Unavailable as u8);
static OWNER_WAIT_DEADLINE_MILLIS: AtomicU32 = AtomicU32::new(0);
static OWNER_WAIT_DEADLINE_VALID: AtomicBool = AtomicBool::new(false);

pub(crate) fn record_owner_phase(phase: TaskWatchdogOwnerPhase) {
    OWNER_PHASE.store(phase as u8, Ordering::Release);
}

pub(crate) fn record_owner_wait(maybe_deadline_millis: Option<u64>) {
    OWNER_WAIT_DEADLINE_MILLIS.store(maybe_deadline_millis.unwrap_or(0) as u32, Ordering::Relaxed);
    OWNER_WAIT_DEADLINE_VALID.store(maybe_deadline_millis.is_some(), Ordering::Relaxed);
    OWNER_PHASE.store(
        TaskWatchdogOwnerPhase::WaitingInbox as u8,
        Ordering::Release,
    );
}

pub(crate) fn owner_observation() -> (TaskWatchdogOwnerPhase, TaskWatchdogWaitObservation) {
    let phase = TaskWatchdogOwnerPhase::from_u8(OWNER_PHASE.load(Ordering::Acquire));
    let wait = if phase == TaskWatchdogOwnerPhase::WaitingInbox {
        let deadline_millis_low = OWNER_WAIT_DEADLINE_MILLIS.load(Ordering::Relaxed);
        let deadline_valid = OWNER_WAIT_DEADLINE_VALID.load(Ordering::Relaxed);
        TaskWatchdogWaitObservation::waiting_until(
            deadline_valid.then_some(u64::from(deadline_millis_low)),
        )
    } else {
        TaskWatchdogWaitObservation::NotWaiting
    };
    (phase, wait)
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
