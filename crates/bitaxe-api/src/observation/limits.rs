//! The ranges a board's mining-safety facts must hold, from its catalog entry.

use bitaxe_config::catalog::{ultra_205_catalog_entry, BoardCatalogEntry};
use bitaxe_safety::power::POWER_MARGIN_WATTS;
use bitaxe_safety::thermal::{MIN_PLAUSIBLE_TEMP_C, TPS546_VR_THROTTLE_TEMP_C};

/// What a board's mining-safety ranges add to the ones every modelled board shares (a 5 V
/// supply within 10 %, the chip below 75 C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningSafetyLimits {
    max_power_watts: f64,
    /// A TPS546 board's regulator temperature limit; such a board must also report it.
    maybe_vr_temp_max_celsius: Option<f64>,
}

impl MiningSafetyLimits {
    /// The board's limits:
    /// - power up to its catalog target plus the shared margin. A TPS546 board's power fact
    ///   must be what upstream reports, the regulator's VOUT × IOUT plus the family's power
    ///   offset (`power.c`). Its 22 W stop has no mining evidence yet (phase F3);
    /// - on a TPS546 board, the regulator temperature up to upstream's throttle.
    #[must_use]
    pub fn for_board(entry: BoardCatalogEntry) -> Self {
        Self {
            max_power_watts: f64::from(entry.power_consumption_target()) + POWER_MARGIN_WATTS,
            maybe_vr_temp_max_celsius: entry
                .capabilities()
                .tps546()
                .then_some(TPS546_VR_THROTTLE_TEMP_C),
        }
    }

    /// The Ultra 205's limits: 15 W, no regulator temperature.
    #[must_use]
    pub fn ultra_205() -> Self {
        Self::for_board(ultra_205_catalog_entry())
    }

    #[must_use]
    pub const fn max_power_watts(&self) -> f64 {
        self.max_power_watts
    }

    /// Whether the board must report a fresh regulator temperature to mine.
    #[must_use]
    pub const fn requires_vr_temperature(&self) -> bool {
        self.maybe_vr_temp_max_celsius.is_some()
    }

    pub(super) fn power_in_range(&self, watts: f64) -> bool {
        watts.is_finite() && (0.0..=self.max_power_watts).contains(&watts)
    }

    pub(super) fn vr_temperature_in_range(&self, celsius: f64) -> bool {
        self.maybe_vr_temp_max_celsius.is_none_or(|maximum| {
            celsius.is_finite() && (MIN_PLAUSIBLE_TEMP_C..=maximum).contains(&celsius)
        })
    }
}

#[cfg(test)]
#[path = "limits_tests.rs"]
mod tests;
