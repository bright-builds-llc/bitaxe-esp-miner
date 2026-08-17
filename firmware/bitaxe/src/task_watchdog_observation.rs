#![cfg_attr(test, allow(dead_code))]

//! Shared observation store for producer-owned ESP task-watchdog facts.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;

use bitaxe_core::runtime_health::{
    TaskWatchdogObservation, TaskWatchdogOwnerPhase, TaskWatchdogWaitObservation,
};

const COHERENT_READ_ATTEMPTS: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
struct TaskWatchdogObservationHistory {
    maybe_previous: Option<TaskWatchdogObservation>,
    maybe_latest: Option<TaskWatchdogObservation>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskWatchdogObservationSnapshot {
    pub(crate) maybe_previous: Option<TaskWatchdogObservation>,
    pub(crate) maybe_latest: Option<TaskWatchdogObservation>,
    pub(crate) owner_phase: TaskWatchdogOwnerPhase,
    pub(crate) owner_wait: TaskWatchdogWaitObservation,
}

impl Default for TaskWatchdogObservationSnapshot {
    fn default() -> Self {
        Self {
            maybe_previous: None,
            maybe_latest: None,
            owner_phase: TaskWatchdogOwnerPhase::Unavailable,
            owner_wait: TaskWatchdogWaitObservation::NotWaiting,
        }
    }
}

struct TaskWatchdogObservationStore {
    history: Mutex<TaskWatchdogObservationHistory>,
    publication_sequence: AtomicU32,
    owner_phase: AtomicU8,
    owner_wait_deadline_millis: AtomicU32,
    owner_wait_deadline_valid: AtomicBool,
}

impl TaskWatchdogObservationStore {
    const fn new() -> Self {
        Self {
            history: Mutex::new(TaskWatchdogObservationHistory {
                maybe_previous: None,
                maybe_latest: None,
            }),
            publication_sequence: AtomicU32::new(0),
            owner_phase: AtomicU8::new(TaskWatchdogOwnerPhase::Unavailable as u8),
            owner_wait_deadline_millis: AtomicU32::new(0),
            owner_wait_deadline_valid: AtomicBool::new(false),
        }
    }

    fn record_owner_phase(&self, phase: TaskWatchdogOwnerPhase) {
        let _publication = self.begin_publication();
        self.owner_phase.store(phase as u8, Ordering::Relaxed);
    }

    fn record_owner_wait(&self, maybe_deadline_millis: Option<u64>) {
        let _publication = self.begin_publication();
        self.owner_wait_deadline_millis
            .store(maybe_deadline_millis.unwrap_or(0) as u32, Ordering::Relaxed);
        self.owner_wait_deadline_valid
            .store(maybe_deadline_millis.is_some(), Ordering::Relaxed);
        self.owner_phase.store(
            TaskWatchdogOwnerPhase::WaitingInbox as u8,
            Ordering::Relaxed,
        );
    }

    fn record(&self, observation: TaskWatchdogObservation) {
        let _publication = self.begin_publication();
        let Ok(mut history) = self.history.lock() else {
            log::error!("task_watchdog_observation=unavailable reason=mutex_poisoned");
            return;
        };
        if history.maybe_latest == Some(observation) {
            return;
        }
        history.maybe_previous = history.maybe_latest;
        history.maybe_latest = Some(observation);
    }

    fn coherent_observation(&self) -> TaskWatchdogObservationSnapshot {
        self.coherent_observation_with(|| {})
    }

    fn coherent_observation_with(
        &self,
        mut after_history_copy: impl FnMut(),
    ) -> TaskWatchdogObservationSnapshot {
        for _ in 0..COHERENT_READ_ATTEMPTS {
            let start_sequence = self.publication_sequence.load(Ordering::Acquire);
            if start_sequence & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }

            let history = match self.history.lock() {
                Ok(history) => *history,
                Err(_) => return TaskWatchdogObservationSnapshot::default(),
            };
            after_history_copy();
            let owner_phase =
                TaskWatchdogOwnerPhase::from_u8(self.owner_phase.load(Ordering::Relaxed));
            let owner_wait = self.owner_wait(owner_phase);
            let end_sequence = self.publication_sequence.load(Ordering::Acquire);

            if start_sequence == end_sequence && end_sequence & 1 == 0 {
                return TaskWatchdogObservationSnapshot {
                    maybe_previous: history.maybe_previous,
                    maybe_latest: history.maybe_latest,
                    owner_phase,
                    owner_wait,
                };
            }
            std::hint::spin_loop();
        }

        TaskWatchdogObservationSnapshot::default()
    }

    fn owner_wait(&self, owner_phase: TaskWatchdogOwnerPhase) -> TaskWatchdogWaitObservation {
        if owner_phase != TaskWatchdogOwnerPhase::WaitingInbox {
            return TaskWatchdogWaitObservation::NotWaiting;
        }

        let deadline_millis_low = self.owner_wait_deadline_millis.load(Ordering::Relaxed);
        let deadline_valid = self.owner_wait_deadline_valid.load(Ordering::Relaxed);
        TaskWatchdogWaitObservation::waiting_until(
            deadline_valid.then_some(u64::from(deadline_millis_low)),
        )
    }

    fn begin_publication(&self) -> TaskWatchdogPublication<'_> {
        let previous_sequence = self.publication_sequence.fetch_add(1, Ordering::AcqRel);
        debug_assert_eq!(
            previous_sequence & 1,
            0,
            "task-watchdog observation store requires one writer"
        );
        TaskWatchdogPublication {
            sequence: &self.publication_sequence,
        }
    }
}

struct TaskWatchdogPublication<'a> {
    sequence: &'a AtomicU32,
}

impl Drop for TaskWatchdogPublication<'_> {
    fn drop(&mut self) {
        self.sequence.fetch_add(1, Ordering::Release);
    }
}

static STORE: TaskWatchdogObservationStore = TaskWatchdogObservationStore::new();

pub(crate) fn record_owner_phase(phase: TaskWatchdogOwnerPhase) {
    STORE.record_owner_phase(phase);
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
