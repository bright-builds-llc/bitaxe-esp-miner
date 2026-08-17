#![cfg_attr(test, allow(dead_code))]

//! Shared observation store for producer-owned ESP task-watchdog facts.

use std::sync::Mutex;

use bitaxe_core::runtime_health::{
    TaskWatchdogObservation, TaskWatchdogOwnerPhase, TaskWatchdogOwnerSubphase,
    TaskWatchdogReadOutcome, TaskWatchdogWaitObservation,
};

#[derive(Debug, Clone, Copy, Default)]
struct TaskWatchdogObservationHistory {
    maybe_previous: Option<TaskWatchdogObservation>,
    maybe_latest: Option<TaskWatchdogObservation>,
}

#[derive(Debug, Clone, Copy)]
struct TaskWatchdogObservationState {
    history: TaskWatchdogObservationHistory,
    owner_phase: TaskWatchdogOwnerPhase,
    owner_subphase: TaskWatchdogOwnerSubphase,
    owner_wait: TaskWatchdogWaitObservation,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskWatchdogObservationSnapshot {
    pub(crate) maybe_previous: Option<TaskWatchdogObservation>,
    pub(crate) maybe_latest: Option<TaskWatchdogObservation>,
    pub(crate) read_outcome: TaskWatchdogReadOutcome,
    pub(crate) owner_phase: TaskWatchdogOwnerPhase,
    pub(crate) owner_subphase: TaskWatchdogOwnerSubphase,
    pub(crate) owner_wait: TaskWatchdogWaitObservation,
}

impl Default for TaskWatchdogObservationSnapshot {
    fn default() -> Self {
        Self {
            maybe_previous: None,
            maybe_latest: None,
            read_outcome: TaskWatchdogReadOutcome::Uninitialized,
            owner_phase: TaskWatchdogOwnerPhase::Unavailable,
            owner_subphase: TaskWatchdogOwnerSubphase::Unavailable,
            owner_wait: TaskWatchdogWaitObservation::NotWaiting,
        }
    }
}

struct TaskWatchdogObservationStore {
    state: Mutex<TaskWatchdogObservationState>,
}

impl TaskWatchdogObservationStore {
    const fn new() -> Self {
        Self {
            state: Mutex::new(TaskWatchdogObservationState {
                history: TaskWatchdogObservationHistory {
                    maybe_previous: None,
                    maybe_latest: None,
                },
                owner_phase: TaskWatchdogOwnerPhase::Unavailable,
                owner_subphase: TaskWatchdogOwnerSubphase::Unavailable,
                owner_wait: TaskWatchdogWaitObservation::NotWaiting,
            }),
        }
    }

    fn record_owner_phase(&self, phase: TaskWatchdogOwnerPhase) {
        let Ok(mut state) = self.state.lock() else {
            log_state_poisoned();
            return;
        };
        state.owner_phase = phase;
        state.owner_subphase = TaskWatchdogOwnerSubphase::Unavailable;
    }

    fn record_owner_progress(
        &self,
        subphase: TaskWatchdogOwnerSubphase,
        maybe_observation: Option<TaskWatchdogObservation>,
    ) {
        let Ok(mut state) = self.state.lock() else {
            log_state_poisoned();
            return;
        };
        state.owner_subphase = subphase;
        if let Some(observation) = maybe_observation {
            Self::record_history(&mut state.history, observation);
        }
    }

    fn record_owner_wait(&self, maybe_deadline_millis: Option<u64>) {
        let Ok(mut state) = self.state.lock() else {
            log_state_poisoned();
            return;
        };
        state.owner_wait = TaskWatchdogWaitObservation::waiting_until(maybe_deadline_millis);
        state.owner_phase = TaskWatchdogOwnerPhase::WaitingInbox;
        state.owner_subphase = TaskWatchdogOwnerSubphase::Unavailable;
    }

    fn record(&self, observation: TaskWatchdogObservation) {
        let Ok(mut state) = self.state.lock() else {
            log_state_poisoned();
            return;
        };
        Self::record_history(&mut state.history, observation);
    }

    fn record_history(
        history: &mut TaskWatchdogObservationHistory,
        observation: TaskWatchdogObservation,
    ) {
        if history.maybe_latest == Some(observation) {
            return;
        }
        history.maybe_previous = history.maybe_latest;
        history.maybe_latest = Some(observation);
    }

    fn coherent_observation(&self) -> TaskWatchdogObservationSnapshot {
        let state = match self.state.lock() {
            Ok(state) => *state,
            Err(_) => {
                return TaskWatchdogObservationSnapshot::failed(
                    TaskWatchdogReadOutcome::HistoryPoisoned,
                )
            }
        };
        TaskWatchdogObservationSnapshot {
            maybe_previous: state.history.maybe_previous,
            maybe_latest: state.history.maybe_latest,
            read_outcome: if state.history.maybe_latest.is_some() {
                TaskWatchdogReadOutcome::Stable
            } else {
                TaskWatchdogReadOutcome::Uninitialized
            },
            owner_phase: state.owner_phase,
            owner_subphase: state.owner_subphase,
            owner_wait: if state.owner_phase == TaskWatchdogOwnerPhase::WaitingInbox {
                state.owner_wait
            } else {
                TaskWatchdogWaitObservation::NotWaiting
            },
        }
    }
}

fn log_state_poisoned() {
    log::error!("task_watchdog_observation=unavailable reason=mutex_poisoned");
}

impl TaskWatchdogObservationSnapshot {
    fn failed(read_outcome: TaskWatchdogReadOutcome) -> Self {
        Self {
            read_outcome,
            ..Self::default()
        }
    }
}

static STORE: TaskWatchdogObservationStore = TaskWatchdogObservationStore::new();

pub(crate) fn record_owner_phase(phase: TaskWatchdogOwnerPhase) {
    STORE.record_owner_phase(phase);
}

pub(crate) fn record_owner_progress(
    subphase: TaskWatchdogOwnerSubphase,
    maybe_observation: Option<TaskWatchdogObservation>,
) {
    STORE.record_owner_progress(subphase, maybe_observation);
}

pub(crate) fn record_owner_wait(maybe_deadline_millis: Option<u64>) {
    STORE.record_owner_wait(maybe_deadline_millis);
}

pub(crate) fn coherent_observation() -> TaskWatchdogObservationSnapshot {
    STORE.coherent_observation()
}

pub(crate) fn record(observation: TaskWatchdogObservation) {
    STORE.record(observation);
}

#[cfg(test)]
#[path = "task_watchdog_observation_test.rs"]
mod tests;
