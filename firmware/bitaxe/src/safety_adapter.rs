//! Observe-only firmware safety adapter facade.
//!
//! This module bridges pure Phase 6 safety decisions into firmware
//! observability without enabling voltage, fan, ASIC reset, or mining effects.
#![allow(dead_code)]

mod power;
mod thermal;

use bitaxe_api::{SafetyTelemetryReport, SafetyTelemetryStatus};
use bitaxe_safety::{
    effects::SafetyEffect, evidence::SafetyCriticalEvidence, status::SafetyStatus,
};

pub fn collect_safety_report() -> SafetyTelemetryReport {
    let mut report = power::collect_power_report();
    let thermal = thermal::collect_thermal_report();

    report.chip_temp_celsius = thermal.chip_temp_celsius;
    report.chip_temp2_celsius = thermal.chip_temp2_celsius;
    report.vr_temp_celsius = thermal.vr_temp_celsius;
    report
}

pub fn interpret_safety_effects(effects: &[SafetyEffect]) {
    for effect in effects {
        interpret_safety_effect(effect);
    }
}

fn interpret_safety_effect(effect: &SafetyEffect) {
    match *effect {
        SafetyEffect::HoldResetLow => {
            log::warn!(
                "safety_effect=hold_reset_low status=unavailable reason=peripherals_unavailable"
            );
        }
        SafetyEffect::DisableAsicEnable => {
            log::warn!("safety_effect=disable_asic_enable status=suppressed");
        }
        SafetyEffect::SuppressVoltageWrite => {
            power::suppress_voltage_write("safety_effect");
        }
        SafetyEffect::SetFanDutyPercent { percent } => {
            thermal::suppress_fan_write(percent, "hardware_evidence_pending");
        }
        SafetyEffect::BlockWorkSubmission { reason } => {
            crate::asic_adapter::publish_mining_loop_blocked_status(reason);
        }
        SafetyEffect::PublishStatus(status) => {
            publish_safety_status(status);
        }
        SafetyEffect::YieldWatchdog { after_ms } => {
            log::warn!("safety_effect=yield_watchdog after_ms={after_ms}");
        }
    }
}

fn publish_safety_status(status: SafetyStatus) {
    log::warn!(
        "safety_status={} reason={}",
        status_label(status),
        status.public_reason()
    );
}

fn status_label(status: SafetyStatus) -> &'static str {
    match status {
        SafetyStatus::Normal => "normal",
        SafetyStatus::Unavailable { .. } => "unavailable",
        SafetyStatus::SafeBlocked { .. } => "safe_blocked",
        SafetyStatus::PowerFault { .. } => "power_fault",
        SafetyStatus::ThermalFault { .. } => "thermal_fault",
        SafetyStatus::FanFault { .. } => "fan_fault",
        SafetyStatus::SelfTestRunning => "self_test_running",
        SafetyStatus::SelfTestPassed => "self_test_passed",
        SafetyStatus::SelfTestFailed { .. } => "self_test_failed",
    }
}

pub(crate) fn unavailable_report(reason: &'static str) -> SafetyTelemetryReport {
    SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Unavailable { reason },
        evidence: SafetyCriticalEvidence::Missing,
        power_watts: 0.0,
        voltage_volts: 0.0,
        current_amps: 0.0,
        chip_temp_celsius: 0.0,
        chip_temp2_celsius: 0.0,
        vr_temp_celsius: 0.0,
        core_voltage_actual_mv: 0.0,
        actual_frequency_mhz: 0.0,
        expected_hashrate_ghs: 0.0,
        fan_speed_percent: 0,
        fan_rpm: 0,
        fan2_rpm: 0,
        wifi_rssi_dbm: -90,
    }
}
