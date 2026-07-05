//! Shared I2C0 bus helper for Ultra 205 safety peripherals.
//!
//! Reference: `reference/esp-miner/main/i2c_bitaxe.c`

use anyhow::{Context, Result};
use esp_idf_svc::hal::{
    gpio::{InputPin, OutputPin},
    i2c::{I2c, I2cConfig, I2cDriver},
    units::FromValueType,
};

pub const I2C_SDA_GPIO: i32 = 47;
pub const I2C_SCL_GPIO: i32 = 48;
pub const I2C_SPEED_KHZ: u32 = 400;
const I2C_TIMEOUT_MS: u32 = 1000;

pub struct BitaxeI2cBus<'d> {
    driver: I2cDriver<'d>,
}

impl<'d> BitaxeI2cBus<'d> {
    pub fn new<I2C, SDA, SCL>(i2c: I2C, sda: SDA, scl: SCL) -> Result<Self>
    where
        I2C: I2c + 'd,
        SDA: InputPin + OutputPin + 'd,
        SCL: InputPin + OutputPin + 'd,
    {
        debug_assert_eq!(I2C_SDA_GPIO, 47);
        debug_assert_eq!(I2C_SCL_GPIO, 48);
        debug_assert_eq!(I2C_SPEED_KHZ, 400);

        let config = I2cConfig::new().baudrate(I2C_SPEED_KHZ.kHz().into());
        let driver =
            I2cDriver::new(i2c, sda, scl, &config).context("initialize Ultra 205 I2C0 bus")?;
        Ok(Self { driver })
    }

    pub fn write_register(&mut self, device_addr: u8, register: u8, value: u8) -> Result<()> {
        self.driver
            .write(device_addr, &[register, value], I2C_TIMEOUT_MS)
            .with_context(|| format!("i2c write register 0x{register:02x} device 0x{device_addr:02x}"))
    }

    pub fn read_register(&mut self, device_addr: u8, register: u8, buf: &mut [u8]) -> Result<()> {
        self.driver
            .write(device_addr, &[register], I2C_TIMEOUT_MS)
            .with_context(|| format!("i2c write pointer 0x{register:02x} device 0x{device_addr:02x}"))?;
        self.driver
            .read(device_addr, buf, I2C_TIMEOUT_MS)
            .with_context(|| format!("i2c read device 0x{device_addr:02x}"))
    }
}
