//! Shared I2C0 bus owner for Ultra 205 startup display and safety sensors.
//!
//! Reference: `reference/esp-miner/main/i2c_bitaxe.c`

use anyhow::{Context, Result};
use embedded_hal::i2c::{ErrorType, I2c as EmbeddedHalI2c, Operation};
use esp_idf_svc::hal::{
    delay::TickType,
    gpio::{InputPin, OutputPin},
    i2c::{I2c, I2cConfig, I2cDriver, I2cError},
    units::FromValueType,
};

pub const I2C_SDA_GPIO: i32 = 47;
pub const I2C_SCL_GPIO: i32 = 48;
pub const I2C_SPEED_KHZ: u32 = 400;
pub const I2C_TRANSACTION_TIMEOUT_MS: u64 = 50;

const INA260_I2C_ADDRESS: u8 = 0x40;
const EMC2101_I2C_ADDRESS: u8 = 0x4c;

pub(crate) struct BitaxeI2cBus<'d> {
    driver: I2cDriver<'d>,
}

impl<'d> BitaxeI2cBus<'d> {
    pub(crate) fn new<I2C, SDA, SCL>(i2c: I2C, sda: SDA, scl: SCL) -> Result<Self>
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

    pub(crate) fn startup_display(&mut self) -> StartupDisplayBus<'_, 'd> {
        StartupDisplayBus { bus: self }
    }

    pub(crate) fn read_only_sensors(&mut self) -> ReadOnlySensorBus<'_, 'd> {
        ReadOnlySensorBus { bus: self }
    }

    pub(super) fn active_for_phase27(
        &mut self,
        _token: &super::phase27_bring_up::Phase27ActiveI2cToken,
    ) -> ActiveI2cBus<'_, 'd> {
        ActiveI2cBus { bus: self }
    }

    /// Generic read retained only for the explicitly gated Phase 27 path.
    pub(super) fn read_register(
        &mut self,
        device_addr: u8,
        register: u8,
        output: &mut [u8],
    ) -> Result<()> {
        self.driver
            .write_read(
                device_addr,
                &[register],
                output,
                transaction_timeout_ticks(),
            )
            .with_context(|| {
                format!("i2c read register 0x{register:02x} device 0x{device_addr:02x}")
            })
    }
}

pub(super) struct ActiveI2cBus<'bus, 'd> {
    bus: &'bus mut BitaxeI2cBus<'d>,
}

impl ActiveI2cBus<'_, '_> {
    pub(super) fn write_register(
        &mut self,
        device_addr: u8,
        register: u8,
        value: u8,
    ) -> Result<()> {
        self.bus
            .driver
            .write(device_addr, &[register, value], transaction_timeout_ticks())
            .with_context(|| {
                format!("i2c write register 0x{register:02x} device 0x{device_addr:02x}")
            })
    }
}

fn transaction_timeout_ticks() -> esp_idf_sys::TickType_t {
    TickType::new_millis(I2C_TRANSACTION_TIMEOUT_MS).ticks()
}

pub(crate) struct StartupDisplayBus<'bus, 'd> {
    bus: &'bus mut BitaxeI2cBus<'d>,
}

impl ErrorType for StartupDisplayBus<'_, '_> {
    type Error = I2cError;
}

impl EmbeddedHalI2c for StartupDisplayBus<'_, '_> {
    fn read(&mut self, address: u8, output: &mut [u8]) -> Result<(), Self::Error> {
        self.bus
            .driver
            .read(address, output, transaction_timeout_ticks())
            .map_err(I2cError::from)
    }

    fn write(&mut self, address: u8, input: &[u8]) -> Result<(), Self::Error> {
        self.bus
            .driver
            .write(address, input, transaction_timeout_ticks())
            .map_err(I2cError::from)
    }

    fn write_read(
        &mut self,
        address: u8,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.bus
            .driver
            .write_read(address, input, output, transaction_timeout_ticks())
            .map_err(I2cError::from)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.bus
            .driver
            .transaction(address, operations, transaction_timeout_ticks())
            .map_err(I2cError::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ina260ReadRegister {
    Current,
    BusVoltage,
    Power,
}

impl Ina260ReadRegister {
    const fn address(self) -> u8 {
        match self {
            Self::Current => 0x01,
            Self::BusVoltage => 0x02,
            Self::Power => 0x03,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Emc2101ReadRegister {
    ExternalTemperatureMsb,
    ExternalTemperatureLsb,
    TachometerLsb,
    TachometerMsb,
}

impl Emc2101ReadRegister {
    const fn address(self) -> u8 {
        match self {
            Self::ExternalTemperatureMsb => 0x01,
            Self::ExternalTemperatureLsb => 0x10,
            Self::TachometerLsb => 0x46,
            Self::TachometerMsb => 0x47,
        }
    }
}

pub(crate) struct ReadOnlySensorBus<'bus, 'd> {
    bus: &'bus mut BitaxeI2cBus<'d>,
}

impl ReadOnlySensorBus<'_, '_> {
    pub(crate) fn read_ina260(
        &mut self,
        register: Ina260ReadRegister,
        output: &mut [u8; 2],
    ) -> Result<()> {
        self.read_register(INA260_I2C_ADDRESS, register.address(), output)
    }

    pub(crate) fn read_emc2101(
        &mut self,
        register: Emc2101ReadRegister,
        output: &mut [u8; 1],
    ) -> Result<()> {
        self.read_register(EMC2101_I2C_ADDRESS, register.address(), output)
    }

    fn read_register(&mut self, device_addr: u8, register: u8, output: &mut [u8]) -> Result<()> {
        self.bus
            .driver
            .write_read(
                device_addr,
                &[register],
                output,
                transaction_timeout_ticks(),
            )
            .with_context(|| {
                format!("i2c read register 0x{register:02x} device 0x{device_addr:02x}")
            })
    }
}
