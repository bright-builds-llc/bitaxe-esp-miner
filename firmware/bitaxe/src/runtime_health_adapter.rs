//! Read-only firmware adapter for passive runtime-health projection.

use bitaxe_api::LIVE_TELEMETRY_CADENCE_MS;
use bitaxe_core::runtime_health::{
    PassiveSelfTestState, RuntimeHealthSnapshot, RuntimeHealthTiming,
};

const MILLIS_PER_SECOND: u64 = 1_000;

fn task_watchdog_timeout_millis() -> u64 {
    u64::from(esp_idf_svc::sys::CONFIG_ESP_TASK_WDT_TIMEOUT_S)
        .checked_mul(MILLIS_PER_SECOND)
        .expect("u32 watchdog seconds always fit in u64 milliseconds")
}

/// Copies already-observed lifecycle and supervisor facts into the pure evaluator.
pub(crate) fn collect(current_monotonic_millis: u64) -> RuntimeHealthSnapshot {
    let checkpoints = crate::safety_adapter::supervisor_checkpoint_history();
    let task_watchdog = crate::task_watchdog_observation::observation_history();
    RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
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
}
