//! EMC2101 thermal and fan adapter.
//!
//! Reference: `reference/esp-miner/main/thermal/EMC2101.c`

use anyhow::Result;
use bitaxe_safety::sensor_acquisition::{
    decode_emc2101_external_temperature, decode_emc2101_tachometer, AcquisitionOutcome,
};

use super::{
    i2c_bus::{ActiveI2cBus, BitaxeI2cBus, Emc2101ReadRegister, ReadOnlySensorBus},
    thermal::EMC2101_I2C_ADDRESS,
};

const EMC2101_REG_CONFIG: u8 = 0x03;
const EMC2101_FAN_CONFIG: u8 = 0x4A;
const EMC2101_REG_FAN_SETTING: u8 = 0x4C;
const EMC2101_EXTERNAL_TEMP_MSB: u8 = 0x01;
const EMC2101_EXTERNAL_TEMP_LSB: u8 = 0x10;
const EMC2101_TACH_LSB: u8 = 0x46;
const EMC2101_TACH_MSB: u8 = 0x47;
const EMC2101_FAN_CONFIG_VALUE: u8 = 0b0010_0011;
const EMC2101_TACH_INPUT_CONFIG: u8 = 0x04;
const EMC2101_FAN_RPM_NUMERATOR: u32 = 5_400_000;

pub fn read_external_temperature_acquisition(
    bus: &mut ReadOnlySensorBus<'_, '_>,
) -> AcquisitionOutcome<f64> {
    let mut msb = [0_u8; 1];
    let mut lsb = [0_u8; 1];

    if bus
        .read_emc2101(Emc2101ReadRegister::ExternalTemperatureMsb, &mut msb)
        .is_err()
        || bus
            .read_emc2101(Emc2101ReadRegister::ExternalTemperatureLsb, &mut lsb)
            .is_err()
    {
        return AcquisitionOutcome::ReadFailed;
    }

    match decode_emc2101_external_temperature([msb[0], lsb[0]]) {
        Ok(temperature) => AcquisitionOutcome::Success(temperature),
        Err(_) => AcquisitionOutcome::InvalidSample,
    }
}

pub fn read_tachometer_acquisition(bus: &mut ReadOnlySensorBus<'_, '_>) -> AcquisitionOutcome<u16> {
    let mut lsb = [0_u8; 1];
    let mut msb = [0_u8; 1];

    if bus
        .read_emc2101(Emc2101ReadRegister::TachometerLsb, &mut lsb)
        .is_err()
        || bus
            .read_emc2101(Emc2101ReadRegister::TachometerMsb, &mut msb)
            .is_err()
    {
        return AcquisitionOutcome::ReadFailed;
    }

    match decode_emc2101_tachometer([lsb[0], msb[0]]) {
        Ok(rpm) => AcquisitionOutcome::Success(rpm),
        Err(_) => AcquisitionOutcome::InvalidSample,
    }
}

pub fn init(bus: &mut ActiveI2cBus<'_, '_>) -> Result<()> {
    bus.write_register(
        EMC2101_I2C_ADDRESS,
        EMC2101_REG_CONFIG,
        EMC2101_TACH_INPUT_CONFIG,
    )?;
    bus.write_register(
        EMC2101_I2C_ADDRESS,
        EMC2101_FAN_CONFIG,
        EMC2101_FAN_CONFIG_VALUE,
    )?;
    Ok(())
}

pub fn set_fan_duty_percent(bus: &mut ActiveI2cBus<'_, '_>, percent: u8) -> Result<()> {
    let clamped = percent.min(100);
    let register_value = ((f64::from(clamped) / 100.0) * 63.0).round() as u8;
    bus.write_register(EMC2101_I2C_ADDRESS, EMC2101_REG_FAN_SETTING, register_value)?;
    log::info!("safety_fan_effect=write percent={clamped} register=0x{register_value:02x}");
    Ok(())
}

pub fn read_external_temp_celsius(bus: &mut BitaxeI2cBus<'_>) -> Result<f64> {
    let mut msb = [0_u8];
    let mut lsb = [0_u8];
    bus.read_register(EMC2101_I2C_ADDRESS, EMC2101_EXTERNAL_TEMP_MSB, &mut msb)?;
    bus.read_register(EMC2101_I2C_ADDRESS, EMC2101_EXTERNAL_TEMP_LSB, &mut lsb)?;

    let mut reading = u16::from(msb[0]) << 8 | u16::from(lsb[0]);
    reading >>= 5;
    let mut signed_reading = reading as i16;
    if signed_reading & 0x0400 != 0 {
        signed_reading |= 0xF800u16 as i16;
    }
    Ok(f64::from(signed_reading) / 8.0)
}

pub fn read_fan_rpm(bus: &mut BitaxeI2cBus<'_>) -> Result<u16> {
    let mut tach_lsb = [0_u8];
    let mut tach_msb = [0_u8];
    bus.read_register(EMC2101_I2C_ADDRESS, EMC2101_TACH_LSB, &mut tach_lsb)?;
    bus.read_register(EMC2101_I2C_ADDRESS, EMC2101_TACH_MSB, &mut tach_msb)?;
    let reading = u16::from(tach_lsb[0]) | (u16::from(tach_msb[0]) << 8);
    if reading == 0 {
        return Ok(0);
    }
    let rpm = EMC2101_FAN_RPM_NUMERATOR / u32::from(reading);
    if rpm == 82 {
        return Ok(0);
    }
    Ok(rpm as u16)
}
