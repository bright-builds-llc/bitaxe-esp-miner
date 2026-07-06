//! Phase 27 work-result investigation compile-time plan selection.
//!
//! Controlled by `BITAXE_WORK_RESULT_INVESTIGATION` action env at firmware build time.

use bitaxe_asic::bm1366::{
    init_plan::{Bm1366InitDecision, Bm1366InitPlan, Bm1366Preflight},
    mining_ready::MiningReadyInitOptions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkResultInvestigationMode {
    SkipMiningReadyInit,
    SkipMaxBaudBeforeWork,
    SkipAsicMaxBaud,
    FrequencyRamp,
    RequireDiagnosticNonce,
    InitializedNoMiningGate,
    PostMaxBaudDelay2000,
    ClearRxBeforeProductionWork,
}

pub fn mining_ready_init_decision(
    preflight: Bm1366Preflight,
    chip_count: u8,
) -> Option<Bm1366InitDecision> {
    if !phase27_bridge_active() {
        return None;
    }

    if investigation_mode() == Some(WorkResultInvestigationMode::SkipMiningReadyInit) {
        return None;
    }

    let options = mining_ready_init_options();
    Some(Bm1366InitPlan::mining_ready_init(
        preflight,
        chip_count,
        options,
    ))
}

pub fn mining_ready_init_options() -> MiningReadyInitOptions {
    let mode = investigation_mode();
    MiningReadyInitOptions {
        skip_max_baud: mode == Some(WorkResultInvestigationMode::SkipMaxBaudBeforeWork),
        skip_asic_max_baud: mode == Some(WorkResultInvestigationMode::SkipAsicMaxBaud),
        use_frequency_ramp: mode == Some(WorkResultInvestigationMode::FrequencyRamp),
        post_max_baud_delay_ms: if mode == Some(WorkResultInvestigationMode::PostMaxBaudDelay2000) {
            2_000
        } else {
            0
        },
    }
}

/// Phase 27 bootstrap: retain production UART when mining-ready init completed but
/// the bounded diagnostic read does not return proof within 10s.
pub fn phase27_initialized_no_mining_bootstrap(mining_ready_completed: bool) -> bool {
    if !phase27_bridge_active() || !mining_ready_completed {
        return false;
    }

    match investigation_mode() {
        Some(WorkResultInvestigationMode::RequireDiagnosticNonce) => false,
        Some(WorkResultInvestigationMode::InitializedNoMiningGate) => true,
        Some(WorkResultInvestigationMode::PostMaxBaudDelay2000)
        | Some(WorkResultInvestigationMode::ClearRxBeforeProductionWork)
        | Some(WorkResultInvestigationMode::SkipMiningReadyInit)
        | Some(WorkResultInvestigationMode::SkipMaxBaudBeforeWork)
        | Some(WorkResultInvestigationMode::SkipAsicMaxBaud)
        | Some(WorkResultInvestigationMode::FrequencyRamp)
        | None => false,
    }
}

pub fn initialized_no_mining_bootstrap_gate() -> bool {
    investigation_mode() == Some(WorkResultInvestigationMode::InitializedNoMiningGate)
}

pub fn clear_rx_before_production_work() -> bool {
    investigation_mode() == Some(WorkResultInvestigationMode::ClearRxBeforeProductionWork)
}

fn phase27_bridge_active() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
}

fn investigation_mode() -> Option<WorkResultInvestigationMode> {
    match option_env!("BITAXE_WORK_RESULT_INVESTIGATION") {
        Some("skip_mining_ready_init") => Some(WorkResultInvestigationMode::SkipMiningReadyInit),
        Some("skip_max_baud") => Some(WorkResultInvestigationMode::SkipMaxBaudBeforeWork),
        Some("skip_asic_max_baud") => Some(WorkResultInvestigationMode::SkipAsicMaxBaud),
        Some("frequency_ramp") => Some(WorkResultInvestigationMode::FrequencyRamp),
        Some("require_diagnostic_nonce") => Some(WorkResultInvestigationMode::RequireDiagnosticNonce),
        Some("initialized_no_mining_gate") => {
            Some(WorkResultInvestigationMode::InitializedNoMiningGate)
        }
        Some("post_max_baud_delay_2000") => Some(WorkResultInvestigationMode::PostMaxBaudDelay2000),
        Some("clear_rx_before_production_work") => {
            Some(WorkResultInvestigationMode::ClearRxBeforeProductionWork)
        }
        _ => None,
    }
}
