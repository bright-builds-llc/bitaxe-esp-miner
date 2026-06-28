//! Watchdog-friendly firmware safety supervisor shell.
#![allow(dead_code)]

use std::time::Duration;

use bitaxe_safety::watchdog::{
    StepKind, StepProgress, StepSupervisor, WatchdogDecision, MAX_CONSECUTIVE_STEPS_BEFORE_YIELD,
    SAFETY_STEP_BUDGET_MS, WATCHDOG_YIELD_INTERVAL_MS,
};

const SUPERVISOR_THREAD_NAME: &str = "bitaxe-safety-supervisor";

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
}
