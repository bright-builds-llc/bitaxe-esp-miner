//! Phase 27 chip-detect investigation compile-time plan selection.
//!
//! Controlled by `BITAXE_CHIP_DETECT_INVESTIGATION` action env at firmware build time.

use bitaxe_asic::bm1366::init_plan::{
    Bm1366InitDecision, Bm1366InitPlan, Bm1366Preflight, ChipDetectPlanOptions,
    PowerPreflightEvidence, SafetyPreflightEvidence, ThermalPreflightEvidence,
};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence, power::PowerEvidenceToken, status::SafetyStatus,
    thermal::ThermalEvidenceToken,
};

pub fn chip_detect_init_decision(preflight: Bm1366Preflight) -> Bm1366InitDecision {
    if investigation_mode() == Some(InvestigationMode::FullInitPrefix) {
        return Bm1366InitPlan::full_init(phase27_full_init_preflight(preflight));
    }

    Bm1366InitPlan::chip_detect_with_options(preflight, chip_detect_plan_options())
}

fn chip_detect_plan_options() -> ChipDetectPlanOptions {
    match investigation_mode() {
        Some(InvestigationMode::VersionMaskPrelude) => ChipDetectPlanOptions {
            skip_reset_pulse: false,
            version_mask_prelude_count: 3,
            wait_tx_done_after_chip_id_write: false,
        },
        Some(InvestigationMode::WaitTxClearRx) => ChipDetectPlanOptions {
            skip_reset_pulse: false,
            version_mask_prelude_count: 0,
            wait_tx_done_after_chip_id_write: true,
        },
        Some(InvestigationMode::SkipSecondReset) => ChipDetectPlanOptions {
            skip_reset_pulse: true,
            version_mask_prelude_count: 0,
            wait_tx_done_after_chip_id_write: false,
        },
        Some(InvestigationMode::FullInitPrefix) => {
            ChipDetectPlanOptions::chip_detect_only_baseline()
        }
        None if phase27_bridge_active() => {
            ChipDetectPlanOptions::upstream_aligned_after_safety_bring_up()
        }
        None => ChipDetectPlanOptions::chip_detect_only_baseline(),
    }
}

fn phase27_full_init_preflight(base: Bm1366Preflight) -> Bm1366Preflight {
    let snapshot = crate::safety_adapter::phase27_safety_snapshot();
    let mut preflight = base;
    if let Some(power) = snapshot.maybe_power {
        preflight = preflight.with_power(PowerPreflightEvidence::from_power_token(
            PowerEvidenceToken {
                bus_voltage_volts: power.bus_voltage_volts,
                current_amps: power.current_amps,
                power_watts: power.power_watts,
            },
        ));
    }
    if let Some(thermal) = snapshot.maybe_thermal {
        preflight = preflight.with_thermal(ThermalPreflightEvidence::from_thermal_token(
            ThermalEvidenceToken {
                chip_temp_celsius: thermal.chip_temp_celsius,
                evidence: SafetyCriticalEvidence::hardware_smoke(
                    "phase27-live-hardware-bridge-safe-stop",
                ),
            },
        ));
    }
    if snapshot.bring_up_complete {
        preflight = preflight.with_safety(SafetyPreflightEvidence::from_safety_status(
            SafetyCriticalEvidence::hardware_smoke("phase27-live-hardware-bridge-safe-stop"),
            SafetyStatus::Normal,
        ));
    }
    preflight
}

fn phase27_bridge_active() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvestigationMode {
    VersionMaskPrelude,
    WaitTxClearRx,
    SkipSecondReset,
    FullInitPrefix,
}

fn investigation_mode() -> Option<InvestigationMode> {
    match option_env!("BITAXE_CHIP_DETECT_INVESTIGATION") {
        Some("version_mask_prelude") => Some(InvestigationMode::VersionMaskPrelude),
        Some("wait_tx_clear_rx") => Some(InvestigationMode::WaitTxClearRx),
        Some("skip_second_reset") => Some(InvestigationMode::SkipSecondReset),
        Some("full_init_prefix") => Some(InvestigationMode::FullInitPrefix),
        _ => None,
    }
}
