//! Phase 27 work-result investigation compile-time plan selection.
//!
//! Controlled by `BITAXE_WORK_RESULT_INVESTIGATION` action env at firmware build time.
//! Supports comma-separated modes, e.g. `frequency_ramp,single_dispatch_bounded_read`.

use bitaxe_asic::{
    bm1366::{
        init_plan::{Bm1366InitDecision, Bm1366InitPlan, Bm1366Preflight},
        mining_ready::MiningReadyInitOptions,
    },
    work_result_investigation::investigation_modes_contain,
};

const INVESTIGATION_RAW: &str = match option_env!("BITAXE_WORK_RESULT_INVESTIGATION") {
    Some(raw) => raw,
    None => "",
};

pub fn mining_ready_init_decision(
    preflight: Bm1366Preflight,
    chip_count: u8,
) -> Option<Bm1366InitDecision> {
    if !phase27_bridge_active() {
        return None;
    }

    if has_investigation_mode("skip_mining_ready_init") {
        return None;
    }

    let options = mining_ready_init_options();
    Some(Bm1366InitPlan::mining_ready_init(
        preflight, chip_count, options,
    ))
}

pub fn mining_ready_init_options() -> MiningReadyInitOptions {
    let bridge_active = phase27_bridge_active();
    MiningReadyInitOptions {
        skip_max_baud: has_investigation_mode("skip_max_baud"),
        skip_asic_max_baud: has_investigation_mode("skip_asic_max_baud"),
        use_frequency_ramp: has_investigation_mode("frequency_ramp")
            || (bridge_active && bridge_default_frequency_ramp_enabled()),
        post_max_baud_delay_ms: if has_investigation_mode("post_max_baud_delay_2000") {
            2_000
        } else {
            0
        },
    }
}

/// Wave 3: upstream parity — stepped 50→485 MHz ramp is default on bridge packages unless
/// explicitly disabled via `skip_frequency_ramp` investigation mode.
fn bridge_default_frequency_ramp_enabled() -> bool {
    !has_investigation_mode("skip_frequency_ramp")
}

/// W13 rollback lever: compile-time opt-out that restores the old boot gate
/// requiring a diagnostic nonce before production peripheral retention.
pub fn require_diagnostic_nonce() -> bool {
    has_investigation_mode("require_diagnostic_nonce")
}

/// W13 rollback lever: compile-time opt-out that restores the old boot gate
/// requiring UART proof before production peripheral retention.
pub fn require_uart_proof_for_production() -> bool {
    has_investigation_mode("require_uart_proof_for_production")
}

pub fn clear_rx_before_production_work() -> bool {
    has_investigation_mode("clear_rx_before_production_work")
}

/// Phase 28.1 A/B control lever: compile-time opt-out that restores the
/// pre-28.1 pump behavior — one dispatch per queued pool work, bounded
/// result read, fail-closed `ResultTimeout` on chip silence.
pub fn single_dispatch_bounded_read_enabled() -> bool {
    has_investigation_mode("single_dispatch_bounded_read")
}

/// Phase 28.1.1.2 A/B: match upstream hashrate-monitor register-read poll
/// cadence (~1 Hz × REGISTER_MAP entries). Off by default; investigation only.
pub fn match_upstream_register_read_poll_enabled() -> bool {
    has_investigation_mode("match_upstream_register_read_poll")
}

/// Phase 28.1.1.3 A/B: continuous result poll uses upstream-like long-block
/// `RESULT_WORK_TIMEOUT_MS` (10000) instead of the 100 ms socket clamp.
/// Off by default; investigation only.
pub fn upstream_like_long_block_receive_enabled() -> bool {
    has_investigation_mode("upstream_like_long_block_receive")
}

fn phase27_bridge_active() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
}

fn has_investigation_mode(mode: &str) -> bool {
    investigation_modes_contain(INVESTIGATION_RAW, mode)
}
