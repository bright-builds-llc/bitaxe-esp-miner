//! Read-only firmware adapter for passive runtime-health projection.

use bitaxe_api::LIVE_TELEMETRY_CADENCE_MS;
use bitaxe_core::runtime_health::{RuntimeHealthSnapshot, RuntimeHealthTiming};

const MILLIS_PER_SECOND: u64 = 1_000;

fn task_watchdog_timeout_millis() -> u64 {
    u64::from(esp_idf_svc::sys::CONFIG_ESP_TASK_WDT_TIMEOUT_S)
        .checked_mul(MILLIS_PER_SECOND)
        .expect("u32 watchdog seconds always fit in u64 milliseconds")
}

/// Copies already-observed lifecycle and supervisor facts into the pure evaluator.
pub(crate) fn collect() -> RuntimeHealthSnapshot {
    let checkpoints = crate::safety_adapter::supervisor_checkpoint_history();
    let task_watchdog = crate::task_watchdog_observation::coherent_observation();
    let current_monotonic_millis = crate::runtime_uptime::millis();
    RuntimeHealthSnapshot::evaluate(
        crate::self_test_runtime::passive_state(),
        checkpoints.maybe_previous.as_ref(),
        checkpoints.maybe_latest.as_ref(),
        task_watchdog.maybe_previous,
        task_watchdog.maybe_latest,
        RuntimeHealthTiming::new(
            current_monotonic_millis,
            LIVE_TELEMETRY_CADENCE_MS,
            task_watchdog_timeout_millis(),
        ),
    )
    .with_task_watchdog_read_outcome(task_watchdog.read_outcome)
    .with_task_watchdog_owner_phase(task_watchdog.owner_phase)
    .with_task_watchdog_owner_subphase(task_watchdog.owner_subphase)
    .with_task_watchdog_wait_state(task_watchdog.owner_wait.state_at(current_monotonic_millis))
}
