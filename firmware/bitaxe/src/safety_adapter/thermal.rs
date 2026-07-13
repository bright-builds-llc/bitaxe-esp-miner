//! Observe-only thermal and fan safety adapter.
#![allow(dead_code)]

use bitaxe_safety::thermal::{ThermalObservation, ThermalReading};

pub const EMC2101_I2C_ADDRESS: u8 = 0x4C;

pub fn thermal_observation_from_raw(
    chip_temp_celsius: f64,
    board_temp_celsius: Option<f64>,
    vr_temp_celsius: Option<f64>,
) -> ThermalObservation {
    ThermalObservation::from_reading(Some(ThermalReading {
        chip_temp_celsius,
        maybe_board_temp_celsius: board_temp_celsius,
        maybe_vr_temp_celsius: vr_temp_celsius,
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
