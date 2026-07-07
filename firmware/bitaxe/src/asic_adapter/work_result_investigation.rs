//! Phase 27 work-result investigation compile-time plan selection.
//!
//! Controlled by `BITAXE_WORK_RESULT_INVESTIGATION` action env at firmware build time.
//! Supports comma-separated modes, e.g. `frequency_ramp,initialized_no_mining_gate`.

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

/// Phase 27 bootstrap: retain production UART when mining-ready init completed but
/// the bounded diagnostic read does not return proof within 10s.
pub fn phase27_initialized_no_mining_bootstrap(mining_ready_completed: bool) -> bool {
    if !phase27_bridge_active() || !mining_ready_completed {
        return false;
    }

    if require_diagnostic_nonce() || require_uart_proof_for_production() {
        return false;
    }

    has_investigation_mode("initialized_no_mining_gate")
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

pub fn skip_boot_diagnostic_work() -> bool {
    has_investigation_mode("skip_boot_diagnostic_work")
}

pub fn initialized_no_mining_bootstrap_gate() -> bool {
    has_investigation_mode("initialized_no_mining_gate")
}

pub fn clear_rx_before_production_work() -> bool {
    has_investigation_mode("clear_rx_before_production_work")
}

/// H4: emulate upstream `ASIC_result_task` — continuous UART listen with non-fatal timeouts.
pub fn continuous_result_task_enabled() -> bool {
    has_investigation_mode("continuous_result_task")
}

/// H4: emulate upstream `create_jobs_task` re-feed — periodic re-dispatch while work active.
pub fn job_redispatch_pump_enabled() -> bool {
    has_investigation_mode("job_redispatch_pump")
}

/// Upstream create_jobs_task re-send interval when holding current work (ms).
pub const JOB_REDISPATCH_INTERVAL_MS: u64 = 2000;

fn phase27_bridge_active() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
}

fn has_investigation_mode(mode: &str) -> bool {
    investigation_modes_contain(INVESTIGATION_RAW, mode)
}
