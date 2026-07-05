//! Observe-only Ultra 205 power safety adapter.
#![allow(dead_code)]

use bitaxe_api::{SafetyTelemetryReport, SafetyTelemetryStatus};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence,
    power::VoltageEffectPlan,
};

pub const DS4432U_I2C_ADDRESS: u8 = 0x48;
pub const DS4432U_OUTPUT0_REGISTER: u8 = 0xF8;
pub const DS4432U_OUTPUT1_REGISTER: u8 = 0xF9;
pub const INA260_I2C_ADDRESS: u8 = 0x40;
pub const INA260_CURRENT_REGISTER: u8 = 0x01;
pub const INA260_BUS_VOLTAGE_REGISTER: u8 = 0x02;
pub const INA260_POWER_REGISTER: u8 = 0x03;

const HARDWARE_EVIDENCE_PENDING: &str = "hardware_evidence_pending";

pub fn collect_power_report() -> SafetyTelemetryReport {
    if let Some(report) = phase27_power_report() {
        return report;
    }

    match option_env!("BITAXE_SAFETY_TELEMETRY") {
        Some("observe-only") => observe_only_unavailable_report(),
        _ => observe_only_unavailable_report(),
    }
}

pub fn interpret_voltage_effect(plan: VoltageEffectPlan) {
    if crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        if let VoltageEffectPlan::WriteDs4432u { setpoint_mv, .. } = plan {
            log::info!("safety_voltage_effect=armed setpoint_mv={setpoint_mv}");
            return;
        }
    }

    match plan {
        VoltageEffectPlan::NoWrite { reason } | VoltageEffectPlan::SuppressWrite { reason } => {
            log::warn!("safety_voltage_effect=suppressed reason={reason}");
        }
        VoltageEffectPlan::WriteDs4432u {
            i2c_address,
            output_registers,
            setpoint_mv,
        } => {
            log::warn!(
                "safety_voltage_effect=write_suppressed reason=hardware_evidence_pending i2c_address=0x{i2c_address:02x} output0=0x{:02x} output1=0x{:02x} setpoint_mv={setpoint_mv}",
                output_registers[0],
                output_registers[1]
            );
        }
    }
}

pub fn suppress_voltage_write(reason: &'static str) {
    log::warn!("safety_voltage_effect=suppressed reason={reason}");
}

fn phase27_power_report() -> Option<SafetyTelemetryReport> {
    let snapshot = super::phase27_bring_up::phase27_safety_snapshot();
    if !snapshot.bring_up_complete {
        return None;
    }

    let observation = snapshot.maybe_power?;
    if observation.reason().is_some() {
        return None;
    }

    Some(SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Fresh,
        evidence: SafetyCriticalEvidence::hardware_smoke("phase27-live-hardware-bridge-safe-stop"),
        power_watts: observation.power_watts,
        voltage_volts: observation.bus_voltage_volts,
        current_amps: observation.current_amps,
        core_voltage_actual_mv: 0.0,
        ..super::unavailable_report(HARDWARE_EVIDENCE_PENDING)
    })
}

fn observe_only_unavailable_report() -> SafetyTelemetryReport {
    SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Unavailable {
            reason: HARDWARE_EVIDENCE_PENDING,
        },
        evidence: SafetyCriticalEvidence::Missing,
        ..super::unavailable_report(HARDWARE_EVIDENCE_PENDING)
    }
}
