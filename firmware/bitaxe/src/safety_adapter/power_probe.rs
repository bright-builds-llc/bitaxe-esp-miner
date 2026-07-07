//! Mid-bridge INA260 power sampling for the Phase 28.1 hashing-discrimination probe.
//! Reference: reference/esp-miner/main/power/INA260.c (sampling semantics via ina260.rs).

use std::sync::{Mutex, OnceLock};

use super::{i2c_bus::BitaxeI2cBus, ina260};

static POWER_PROBE_BUS: OnceLock<Mutex<PowerProbeState>> = OnceLock::new();

struct PowerProbeState {
    maybe_bus: Option<BitaxeI2cBus<'static>>,
}

fn probe_state() -> &'static Mutex<PowerProbeState> {
    POWER_PROBE_BUS.get_or_init(|| Mutex::new(PowerProbeState { maybe_bus: None }))
}

/// Retain the bring-up I2C bus so mid-bridge power sampling stays possible.
///
/// Returns `false` (without panicking) when a bus is already stored or the
/// probe state lock is unavailable.
#[must_use]
pub fn store_power_probe_bus(bus: BitaxeI2cBus<'_>) -> bool {
    // SAFETY: The ESP-IDF I2C singleton peripheral lives for the firmware
    // process lifetime; the bus is stored once at bring-up and never
    // reconstructed, so extending its lifetime to 'static cannot dangle.
    let bus_static: BitaxeI2cBus<'static> = unsafe { std::mem::transmute(bus) };

    let Ok(mut state) = probe_state().lock() else {
        return false;
    };
    if state.maybe_bus.is_some() {
        return false;
    }
    state.maybe_bus = Some(bus_static);
    true
}

/// Sample INA260 board power in milliwatts.
///
/// Diagnostic only, never a gate: any read error or missing retained bus
/// yields `None`.
#[must_use]
pub fn sample_power_mw() -> Option<u32> {
    let Ok(mut state) = probe_state().lock() else {
        return None;
    };
    let bus = state.maybe_bus.as_mut()?;
    let sample = ina260::read_sample(bus).ok()?;
    Some(sample.power_mw as u32)
}
