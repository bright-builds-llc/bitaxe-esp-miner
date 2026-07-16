//! Watchdog-friendly firmware safety supervisor shell.
#![allow(dead_code)]

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use bitaxe_core::runtime_health::CheckpointObservation;
use bitaxe_safety::watchdog::{
    StepKind, StepProgress, StepSupervisor, WatchdogDecision, MAX_CONSECUTIVE_STEPS_BEFORE_YIELD,
    SAFETY_STEP_BUDGET_MS, WATCHDOG_YIELD_INTERVAL_MS,
};

const SUPERVISOR_THREAD_NAME: &str = "bitaxe-safety-supervisor";
const SUPERVISOR_CHECKPOINT_CATEGORY: &str = "telemetry";
static SUPERVISOR_CHECKPOINTS: OnceLock<Mutex<SupervisorCheckpointState>> = OnceLock::new();

/// Latest producer-owned checkpoint transition copied by the health reader.
#[derive(Debug, Clone, Default)]
pub struct SupervisorCheckpointHistory {
    pub maybe_previous: Option<CheckpointObservation>,
    pub maybe_latest: Option<CheckpointObservation>,
}

#[derive(Debug, Default)]
struct SupervisorCheckpointState {
    history: SupervisorCheckpointHistory,
    next_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupervisorStepLog {
    Yield { reason: &'static str },
    Watchdog { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckpointRecordFailure {
    SequenceExhausted,
    Invalid,
    Regression,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SupervisorStepOutcome {
    maybe_log: Option<SupervisorStepLog>,
    maybe_checkpoint_failure: Option<CheckpointRecordFailure>,
}

pub fn start_safety_supervisor_thread() -> std::io::Result<()> {
    let _handle = std::thread::Builder::new()
        .name(SUPERVISOR_THREAD_NAME.to_owned())
        .spawn(safety_supervisor_loop)?;
    Ok(())
}

fn safety_supervisor_loop() {
    debug_assert_eq!(WATCHDOG_YIELD_INTERVAL_MS, 100);
    log::info!("safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100");

    let mut logged_yield = false;
    loop {
        run_supervisor_step(&mut logged_yield, current_monotonic_millis());
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(not(test))]
fn current_monotonic_millis() -> u64 {
    crate::runtime_uptime::millis()
}

#[cfg(test)]
fn current_monotonic_millis() -> u64 {
    0
}

fn run_supervisor_step(logged_yield: &mut bool, observed_at_millis: u64) {
    let checkpoints =
        SUPERVISOR_CHECKPOINTS.get_or_init(|| Mutex::new(SupervisorCheckpointState::default()));
    let Ok(mut checkpoints) = checkpoints.lock() else {
        log::warn!("safety_supervisor_checkpoint=unavailable reason=mutex_poisoned");
        return;
    };

    let outcome = transition_supervisor_step(&mut checkpoints, logged_yield, observed_at_millis);
    match outcome.maybe_log {
        Some(SupervisorStepLog::Yield { reason }) => {
            log::info!("safety_supervisor_step=yield reason={reason}");
        }
        Some(SupervisorStepLog::Watchdog { reason }) => {
            log::warn!("safety_supervisor_step=watchdog reason={reason}");
        }
        None => {}
    }
    match outcome.maybe_checkpoint_failure {
        Some(CheckpointRecordFailure::SequenceExhausted) => {
            log::warn!("safety_supervisor_checkpoint=unavailable reason=sequence_exhausted");
        }
        Some(CheckpointRecordFailure::Invalid) => {
            log::warn!("safety_supervisor_checkpoint=unavailable reason=invalid");
        }
        Some(CheckpointRecordFailure::Regression) => {
            log::warn!("safety_supervisor_checkpoint=unavailable reason=regression");
        }
        None => {}
    }
}

fn transition_supervisor_step(
    checkpoints: &mut SupervisorCheckpointState,
    logged_yield: &mut bool,
    observed_at_millis: u64,
) -> SupervisorStepOutcome {
    let decision = StepSupervisor::decision(StepProgress {
        kind: StepKind::Telemetry,
        elapsed_ms: SAFETY_STEP_BUDGET_MS,
        consecutive_steps: MAX_CONSECUTIVE_STEPS_BEFORE_YIELD,
    });

    let maybe_log = match decision {
        WatchdogDecision::Continue => None,
        WatchdogDecision::YieldNow { reason } => {
            debug_assert_eq!(reason, "yield_interval_reached");
            if *logged_yield {
                None
            } else {
                *logged_yield = true;
                Some(SupervisorStepLog::Yield { reason })
            }
        }
        WatchdogDecision::ResetOrFeedWatchdog { reason } => {
            Some(SupervisorStepLog::Watchdog { reason })
        }
    };

    SupervisorStepOutcome {
        maybe_log,
        maybe_checkpoint_failure: record_supervisor_checkpoint(checkpoints, observed_at_millis),
    }
}

/// Returns a read-only copy of the latest accepted supervisor transition.
pub fn supervisor_checkpoint_history() -> SupervisorCheckpointHistory {
    let checkpoints =
        SUPERVISOR_CHECKPOINTS.get_or_init(|| Mutex::new(SupervisorCheckpointState::default()));
    let Ok(checkpoints) = checkpoints.lock() else {
        log::warn!("safety_supervisor_checkpoint=unavailable reason=mutex_poisoned");
        return SupervisorCheckpointHistory::default();
    };

    checkpoints.history.clone()
}

fn record_supervisor_checkpoint(
    checkpoints: &mut SupervisorCheckpointState,
    observed_at_millis: u64,
) -> Option<CheckpointRecordFailure> {
    let Some(sequence) = checkpoints.next_sequence.checked_add(1) else {
        checkpoints.history = SupervisorCheckpointHistory::default();
        return Some(CheckpointRecordFailure::SequenceExhausted);
    };
    let checkpoint = match CheckpointObservation::new(
        SUPERVISOR_CHECKPOINT_CATEGORY,
        sequence,
        observed_at_millis,
    ) {
        Ok(checkpoint) => checkpoint,
        Err(_) => {
            checkpoints.history = SupervisorCheckpointHistory::default();
            return Some(CheckpointRecordFailure::Invalid);
        }
    };
    if let Some(previous) = checkpoints.history.maybe_latest.as_ref() {
        if checkpoint.validate_after(previous).is_err() {
            checkpoints.history = SupervisorCheckpointHistory::default();
            return Some(CheckpointRecordFailure::Regression);
        }
    }

    checkpoints.next_sequence = sequence;
    checkpoints.history.maybe_previous = checkpoints.history.maybe_latest.take();
    checkpoints.history.maybe_latest = Some(checkpoint);
    None
}

#[cfg(test)]
mod tests {
    use bitaxe_core::runtime_health::{
        CheckpointHealth, PassiveSelfTestState, RuntimeHealthSnapshot,
    };

    use super::*;

    #[test]
    fn repeated_yields_publish_recurring_healthy_checkpoints() {
        // Arrange
        let mut checkpoints = SupervisorCheckpointState::default();
        let mut logged_yield = false;
        let mut log_count = 0;

        // Act
        for step in 1_u64..=12 {
            let observed_at_millis = step * u64::from(WATCHDOG_YIELD_INTERVAL_MS);
            let outcome =
                transition_supervisor_step(&mut checkpoints, &mut logged_yield, observed_at_millis);
            log_count += usize::from(outcome.maybe_log.is_some());

            // Assert
            assert_eq!(outcome.maybe_checkpoint_failure, None);
            let latest = checkpoints
                .history
                .maybe_latest
                .as_ref()
                .expect("every completed step should publish a checkpoint");
            assert_eq!(latest.sequence(), step);
            assert_eq!(latest.observed_at_millis(), observed_at_millis);
        }

        let latest = checkpoints
            .history
            .maybe_latest
            .as_ref()
            .expect("latest checkpoint should exist");
        let health = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            checkpoints.history.maybe_previous.as_ref(),
            Some(latest),
            latest.observed_at_millis() + u64::from(WATCHDOG_YIELD_INTERVAL_MS),
            500,
        );
        let frozen_health = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            Some(latest),
            latest.observed_at_millis() + 5_001,
            500,
        );

        // Assert
        assert_eq!(log_count, 1);
        assert_eq!(health.checkpoint_health(), CheckpointHealth::Healthy);
        assert_eq!(
            frozen_health.checkpoint_health(),
            CheckpointHealth::Unhealthy
        );
    }
}
