//! Closed device-side qualification facts; no credential or protocol payload projection.

use super::revocation::{self, WorkerGeneration};
use bitaxe_core::runtime_health::TaskWatchdogObservation;
use bitaxe_safety::observation::Observation;
use bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS;

pub(crate) fn status_evidence(
    _maybe_generation: Option<WorkerGeneration>,
) -> Option<serde_json::Value> {
    let now_ms = crate::runtime_uptime::millis();
    let timing = revocation::timing(now_ms)?;
    let (budget_reserved_ms, budget_complete) =
        crate::worker_acceptance_budget::diagnostic_snapshot();
    let observations = crate::safety_adapter::observation_snapshot();
    let voltage =
        fresh_value(&observations.bus_voltage_volts, now_ms).filter(|value| value.is_finite());
    let power = fresh_value(&observations.power_watts, now_ms).filter(|value| value.is_finite());
    let temperature =
        fresh_value(&observations.chip_temp_celsius, now_ms).filter(|value| value.is_finite());
    let rpm = fresh_value(&observations.fan_rpm, now_ms);
    let watchdog = crate::task_watchdog_observation::coherent_observation();
    let watchdog_alive = matches!(watchdog.maybe_latest,
        Some(TaskWatchdogObservation::Fed { observed_at_millis, .. })
            if now_ms >= observed_at_millis && now_ms - observed_at_millis <= 1_000);
    Some(serde_json::json!({
        "schema": "worker-qualification-v1",
        "generation": timing.generation,
        "revocation_reason": timing.revocation_reason.label(),
        "active_ms": timing.active_ms,
        "generation_elapsed_ms": timing.generation_elapsed_ms,
        "active_limit_ms": timing.active_limit_ms,
        "shutdown_budget_ms": timing.shutdown_budget_ms,
        "work_gate_remaining_ms": timing.work_gate_remaining_ms,
        "budget_reserved_ms": budget_reserved_ms,
        "budget_complete": budget_complete,
        "submitted": timing.submitted,
        "accepted": timing.accepted,
        "rejected": timing.rejected,
        "nonce_work_correlations": timing.nonce_work_correlations,
        "work_dispatched": timing.work_dispatched,
        "last_valid_heartbeat_ms": timing.last_valid_heartbeat_ms,
        "gate_closed_ms": timing.maybe_gate_closed_ms,
        "shutdown_started_ms": timing.maybe_shutdown_started_ms,
        "safe_stop_stage": shutdown_stage(timing.shutdown_stage),
        "safe_stop_complete": timing.shutdown_complete,
        "voltage_volts": voltage,
        "power_watts": power,
        "chip_temp_celsius": temperature,
        "fan_rpm": rpm,
        "voltage_fresh": voltage.is_some(),
        "power_fresh": power.is_some(),
        "temperature_fresh": temperature.is_some(),
        "fan_fresh": rpm.is_some(),
        "watchdog_alive": watchdog_alive,
        "mine_on_boot": crate::settings_adapter::start_mining_on_boot(),
    }))
}

fn fresh_value<T: Copy>(observation: &Observation<T>, now_ms: u64) -> Option<T> {
    let Observation::Fresh { sample } = observation else {
        return None;
    };
    (now_ms >= sample.acquired_at().get()
        && now_ms - sample.acquired_at().get() <= u64::from(POWER_SAMPLE_STALE_AFTER_MS))
    .then_some(*sample.value())
}

fn shutdown_stage(stage: u32) -> &'static str {
    match stage {
        1 => "stop_dispatch",
        2 => "reduce_frequency_and_reset_nonce",
        3 => "hold_reset_low",
        4 => "disable_core_voltage",
        5 => "disable_asic",
        6 => "fan_full",
        7 => "cooling_proof",
        8 => "fan_paused",
        _ => "not_started",
    }
}
