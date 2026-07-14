//! DS4432U voltage DAC adapter for Ultra 205 core voltage.
//!
//! Reference: `reference/esp-miner/main/power/DS4432U.c`

use anyhow::{ensure, Result};

use super::{i2c_bus::ActiveI2cBus, power::DS4432U_I2C_ADDRESS};

pub const DS4432U_OUT0_REG: u8 = 0xF8;

const BITAXE_IFS: f64 = 0.000_098_921;
const BITAXE_RA: f64 = 4750.0;
const BITAXE_RB: f64 = 3320.0;
const BITAXE_VNOM: f64 = 1.451;
const BITAXE_VMAX: f64 = 2.39;
const BITAXE_VMIN: f64 = 0.046;
const TPS40305_VFB: f64 = 0.6;

/// Compute DS4432U OUT0 register value for requested core voltage in volts.
#[must_use]
pub fn register_for_voltage_v(vout: f64) -> Option<u8> {
    if !vout.is_finite() || vout > BITAXE_VMAX || vout < BITAXE_VMIN {
        return None;
    }

    let change = (((TPS40305_VFB / BITAXE_RB) - ((vout - TPS40305_VFB) / BITAXE_RA)) / BITAXE_IFS)
        .abs()
        * 127.0;
    let mut reg = change.ceil() as u8;
    if vout < BITAXE_VNOM {
        reg |= 0x80;
    }
    Some(reg)
}

pub fn set_core_voltage_v(bus: &mut ActiveI2cBus<'_, '_>, vout: f64) -> Result<()> {
    let reg = register_for_voltage_v(vout)
        .ok_or_else(|| anyhow::anyhow!("ds4432u voltage out of range: {vout}"))?;
    bus.write_register(DS4432U_I2C_ADDRESS, DS4432U_OUT0_REG, reg)?;
    log::info!("safety_voltage_effect=write setpoint_v={vout:.3} register=0x{reg:02x}");
    Ok(())
}

pub fn set_core_voltage_mv(bus: &mut ActiveI2cBus<'_, '_>, setpoint_mv: u16) -> Result<()> {
    let vout = f64::from(setpoint_mv) / 1000.0;
    ensure!(setpoint_mv > 0, "ds4432u setpoint must be positive");
    set_core_voltage_v(bus, vout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_for_default_ultra_205_voltage_is_nonzero() {
        // Arrange
        let vout = 1.2;

        // Act
        let reg = register_for_voltage_v(vout);

        // Assert
        assert!(reg.is_some());
        assert_ne!(reg.expect("valid voltage"), 0);
    }
}
