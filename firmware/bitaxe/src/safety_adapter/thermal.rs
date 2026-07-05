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
    if let Some(report) = phase27_thermal_report() {
        return report;
    }

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
    if super::phase27_bring_up::phase27_bring_up_complete() {
        log::info!("safety_fan_effect=armed percent={percent}");
        return;
    }
    log::warn!("safety_fan_effect=suppressed percent={percent} reason={reason}");
}

fn phase27_thermal_report() -> Option<SafetyTelemetryReport> {
    let snapshot = super::phase27_bring_up::phase27_safety_snapshot();
    if !snapshot.bring_up_complete {
        return None;
    }

    let observation = snapshot.maybe_thermal?;
    if observation.reason().is_some() {
        return None;
    }

    let chip_temp = observation.chip_temp_celsius;
    Some(SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Fresh,
        evidence: SafetyCriticalEvidence::hardware_smoke("phase27-live-hardware-bridge-safe-stop"),
        chip_temp_celsius: chip_temp,
        fan_speed_percent: u16::from(snapshot.fan_duty_percent),
        fan_rpm: snapshot.fan_rpm,
        ..super::unavailable_report(THERMAL_HARDWARE_EVIDENCE_PENDING)
    })
}
