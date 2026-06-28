//! Observe-only thermal and fan safety adapter.
#![allow(dead_code)]

use bitaxe_api::{SafetyTelemetryReport, SafetyTelemetryStatus};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence,
    thermal::{ThermalObservation, ThermalReading},
};

pub const EMC2101_I2C_ADDRESS: u8 = 0x4C;

const THERMAL_HARDWARE_EVIDENCE_PENDING: &str = "thermal_hardware_evidence_pending";

pub fn collect_thermal_report() -> SafetyTelemetryReport {
    SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Unavailable {
            reason: THERMAL_HARDWARE_EVIDENCE_PENDING,
        },
        evidence: SafetyCriticalEvidence::Missing,
        ..super::unavailable_report(THERMAL_HARDWARE_EVIDENCE_PENDING)
    }
}

pub fn thermal_observation_from_raw(
    chip_temp_celsius: f64,
    board_temp_celsius: Option<f64>,
    vr_temp_celsius: Option<f64>,
) -> ThermalObservation {
    ThermalObservation::from_reading(Some(ThermalReading {
        chip_temp_celsius,
        board_temp_celsius,
        vr_temp_celsius,
    }))
}

pub fn unavailable_thermal_observation() -> ThermalObservation {
    ThermalObservation::from_reading(None)
}

pub fn suppress_fan_write(percent: u8, reason: &'static str) {
    log::warn!("safety_fan_effect=suppressed percent={percent} reason={reason}");
}
