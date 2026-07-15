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
        run_supervisor_step(&mut logged_yield);
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn run_supervisor_step(logged_yield: &mut bool) {
    let decision = StepSupervisor::decision(StepProgress {
        kind: StepKind::Telemetry,
        elapsed_ms: SAFETY_STEP_BUDGET_MS,
        consecutive_steps: MAX_CONSECUTIVE_STEPS_BEFORE_YIELD,
    });

    match decision {
        WatchdogDecision::Continue => {}
        WatchdogDecision::YieldNow { reason } => {
            debug_assert_eq!(reason, "yield_interval_reached");
            if *logged_yield {
                return;
            }

            log::info!("safety_supervisor_step=yield reason={reason}");
            *logged_yield = true;
        }
        WatchdogDecision::ResetOrFeedWatchdog { reason } => {
            log::warn!("safety_supervisor_step=watchdog reason={reason}");
        }
    }

    record_supervisor_checkpoint(crate::runtime_uptime::millis());
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

fn record_supervisor_checkpoint(observed_at_millis: u64) {
    let checkpoints =
        SUPERVISOR_CHECKPOINTS.get_or_init(|| Mutex::new(SupervisorCheckpointState::default()));
    let Ok(mut checkpoints) = checkpoints.lock() else {
        log::warn!("safety_supervisor_checkpoint=unavailable reason=mutex_poisoned");
        return;
    };

    let Some(sequence) = checkpoints.next_sequence.checked_add(1) else {
        checkpoints.history = SupervisorCheckpointHistory::default();
        log::warn!("safety_supervisor_checkpoint=unavailable reason=sequence_exhausted");
        return;
    };
    let checkpoint = match CheckpointObservation::new(
        SUPERVISOR_CHECKPOINT_CATEGORY,
        sequence,
        observed_at_millis,
    ) {
        Ok(checkpoint) => checkpoint,
        Err(error) => {
            checkpoints.history = SupervisorCheckpointHistory::default();
            log::warn!("safety_supervisor_checkpoint=unavailable reason=invalid error={error:?}");
            return;
        }
    };
    if let Some(previous) = checkpoints.history.maybe_latest.as_ref() {
        if let Err(error) = checkpoint.validate_after(previous) {
            checkpoints.history = SupervisorCheckpointHistory::default();
            log::warn!(
                "safety_supervisor_checkpoint=unavailable reason=regression error={error:?}"
            );
            return;
        }
    }

    checkpoints.next_sequence = sequence;
    checkpoints.history.maybe_previous = checkpoints.history.maybe_latest.take();
    checkpoints.history.maybe_latest = Some(checkpoint);
}
