//! `worker-qualification-v1` status evidence, projected as firmware projects it
//! (`production_mining_session/qualification.rs`) but from the virtual board: the real
//! revocation gate's timing, the modeled sensor sample, the qualification ledger and
//! modeled owner resources. It is evidence for the Gate's checks, never authority.

use super::Session;
use bitaxe_safety::observation::Observation;
use bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS;
use bitaxe_virtual_board::memory::Capability;
use serde_json::{json, Value};

/// The virtual board has no task stacks. This models the 24 KiB mining owner stack with
/// a fixed headroom above the Gate's 4,096-byte floor; it is not a measurement.
const MODELED_OWNER_STACK_FREE_BYTES: u32 = 8192;

pub(super) fn qualification(session: &Session) -> Option<Value> {
    let actuation = &session.actuation;
    let now_ms = actuation.now_ms();
    let timing = actuation
        .gate
        .timing(now_ms)
        .or_else(|| actuation.gate.idle_timing())?;
    let observations = actuation.observations().ok()?;
    let voltage = fresh(&observations.bus_voltage_volts, now_ms).filter(|value| value.is_finite());
    let power = fresh(&observations.power_watts, now_ms).filter(|value| value.is_finite());
    let temperature =
        fresh(&observations.chip_temp_celsius, now_ms).filter(|value| value.is_finite());
    let rpm = fresh(&observations.fan_rpm, now_ms);
    let mut value = json!({
        "schema": "worker-qualification-v1",
        "generation": timing.generation,
        "revocation_reason": timing.revocation_reason.label(),
        "active_ms": timing.active_ms,
        "generation_elapsed_ms": timing.generation_elapsed_ms,
        "active_limit_ms": timing.active_limit_ms,
        "shutdown_budget_ms": timing.shutdown_budget_ms,
        "work_gate_remaining_ms": timing.work_gate_remaining_ms,
        // The legacy acceptance campaign budget, which the virtual device never reserves.
        "budget_reserved_ms": 0,
        "budget_complete": false,
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
        // No task watchdog runs on the virtual board.
        "watchdog_alive": false,
        "mine_on_boot": false,
    });
    if timing.generation == 0 {
        return Some(value);
    }
    if session.maybe_attempt_generation == Some(timing.generation) {
        if let Some(allowance) = session.ledger.maybe_last_allowance() {
            let maximum = allowance.maximum_active_milliseconds();
            value["attempt"] = json!({"schema":"worker-qualification-observation-v1",
                "ordinal":allowance.ordinal(),"purpose":allowance.purpose().label(),
                "maximum_active_ms":maximum,"reserved_ms":maximum,
                "complete":!session.ledger.pending(),"active_ms":timing.active_ms});
        }
    }
    let heap = actuation.board.borrow().heap.snapshot(Capability::Internal);
    value["owner_resources"] = json!({"schema":"worker-owner-resources-v1",
        "generation":timing.generation,
        "phase":if timing.shutdown_complete { "shutdown_complete" } else { "active" },
        "observed_at_ms":now_ms.to_string(),"heap_free_bytes":heap.free_bytes,
        "heap_largest_bytes":heap.largest_block,"stack_free_bytes":MODELED_OWNER_STACK_FREE_BYTES});
    Some(value)
}

fn fresh<T: Copy>(observation: &Observation<T>, now_ms: u64) -> Option<T> {
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
