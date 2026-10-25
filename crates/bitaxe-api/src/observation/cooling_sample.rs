//! The reading that says whether a board has cooled enough to restart mining.

use bitaxe_safety::observation::{Observation, UnavailableReason};
use bitaxe_safety::thermal::{SAFE_RESTART_TEMP_C, TPS546_VR_RESTART_TEMP_C};

use super::{ChipTemperatureSource, MiningSafetyLimits, TelemetryObservations};

/// One fresh cooling reading: when the sensor took it and whether it is cool enough.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoolingSample {
    pub acquired_at_ms: u64,
    pub cool: bool,
}

impl TelemetryObservations {
    /// The board's restart-cooling reading, or `None` without a fresh one. Callers judge its
    /// age and any time rule.
    /// - A board-sensor board: the chip temperature, cool at or below 45 C.
    /// - An ASIC-diode board, whose diode cannot be read with the chip down: the regulator
    ///   temperature, cool at or below 95 C and only while the chip is reported unpowered, as
    ///   upstream waits for 600-series boards (`power_management_task.c:183-207`).
    #[must_use]
    pub fn restart_cooling_sample(&self, limits: &MiningSafetyLimits) -> Option<CoolingSample> {
        match limits.chip_temperature_source() {
            ChipTemperatureSource::BoardSensor => {
                maybe_fresh_sample(&self.chip_temp_celsius, |celsius| {
                    celsius <= SAFE_RESTART_TEMP_C
                })
            }
            ChipTemperatureSource::AsicDiode => {
                let chip_unpowered = matches!(
                    self.chip_temp_celsius,
                    Observation::Unavailable {
                        reason: UnavailableReason::AsicUnpowered
                    }
                );
                maybe_fresh_sample(&self.vr_temp_celsius, |celsius| {
                    chip_unpowered && celsius <= TPS546_VR_RESTART_TEMP_C
                })
            }
        }
    }
}

fn maybe_fresh_sample(
    observation: &Observation<f64>,
    is_cool: impl Fn(f64) -> bool,
) -> Option<CoolingSample> {
    let Observation::Fresh { sample } = observation else {
        return None;
    };
    let celsius = *sample.value();
    Some(CoolingSample {
        acquired_at_ms: sample.acquired_at().get(),
        cool: celsius.is_finite() && is_cool(celsius),
    })
}

#[cfg(test)]
#[path = "cooling_sample_tests.rs"]
mod tests;
