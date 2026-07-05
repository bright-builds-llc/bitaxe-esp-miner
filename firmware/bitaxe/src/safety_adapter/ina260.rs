//! INA260 power telemetry adapter.
//!
//! Reference: `reference/esp-miner/main/power/INA260.c`

use anyhow::Result;
use bitaxe_safety::power::{Ina260RawSample, PowerObservation, PowerSampleAgeMs};

use super::{i2c_bus::BitaxeI2cBus, power::INA260_I2C_ADDRESS};

const INA260_REG_CURRENT: u8 = 0x01;
const INA260_REG_BUS_VOLTAGE: u8 = 0x02;
const INA260_REG_POWER: u8 = 0x03;

pub struct Ina260Sample {
    pub current_ma: f64,
    pub bus_voltage_mv: f64,
    pub power_mw: f64,
}

pub fn read_sample(bus: &mut BitaxeI2cBus<'_>) -> Result<Ina260Sample> {
    let mut data = [0_u8; 2];

    bus.read_register(INA260_I2C_ADDRESS, INA260_REG_CURRENT, &mut data)?;
    let current_ma = f64::from(u16::from_be_bytes(data)) * 1.25;

    bus.read_register(INA260_I2C_ADDRESS, INA260_REG_BUS_VOLTAGE, &mut data)?;
    let bus_voltage_mv = f64::from(u16::from_be_bytes(data)) * 1.25;

    bus.read_register(INA260_I2C_ADDRESS, INA260_REG_POWER, &mut data)?;
    let power_mw = f64::from(u16::from_be_bytes(data)) * 10.0;

    Ok(Ina260Sample {
        current_ma,
        bus_voltage_mv,
        power_mw,
    })
}

const BOARD_POWER_TARGET_WATTS: f64 = 12.0;

pub fn power_observation_from_sample(sample: Ina260Sample) -> PowerObservation {
    PowerObservation::from_ina260_sample(
        Some(Ina260RawSample {
            bus_voltage_volts: sample.bus_voltage_mv / 1000.0,
            current_amps: sample.current_ma / 1000.0,
            power_watts: sample.power_mw / 1000.0,
            read_failed: false,
        }),
        PowerSampleAgeMs(0),
        BOARD_POWER_TARGET_WATTS,
    )
}
