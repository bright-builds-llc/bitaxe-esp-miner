//! Read-only firmware adapter for passive runtime-health projection.

use bitaxe_api::LIVE_TELEMETRY_CADENCE_MS;
use bitaxe_core::runtime_health::{PassiveSelfTestState, RuntimeHealthSnapshot};

/// Copies already-observed lifecycle and supervisor facts into the pure evaluator.
pub(crate) fn collect(current_monotonic_millis: u64) -> RuntimeHealthSnapshot {
    let checkpoints = crate::safety_adapter::supervisor_checkpoint_history();
    RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        checkpoints.maybe_previous.as_ref(),
        checkpoints.maybe_latest.as_ref(),
        current_monotonic_millis,
        LIVE_TELEMETRY_CADENCE_MS,
    )
}
