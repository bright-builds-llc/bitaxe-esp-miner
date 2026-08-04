//! Shared I2C0 bus owner for Ultra 205 startup display and safety sensors.
//!
//! Reference: `reference/esp-miner/main/i2c_bitaxe.c`

use anyhow::{Context, Result};
use embedded_hal::i2c::{
    Error as EmbeddedHalI2cError, ErrorKind, ErrorType, I2c as EmbeddedHalI2c, Operation,
};
use esp_idf_svc::hal::{
    delay::{FreeRtos, TickType},
    gpio::{InputPin, OutputPin},
    i2c::{I2c, I2cConfig, I2cDriver, I2cError},
    units::FromValueType,
};

use super::{
    ds4432u::{Ds4432uRegisterWriter, Ds4432uWriteRegister},
    emc2101::{
        Emc2101ReadRegister, Emc2101RegisterReader, Emc2101RegisterWriter, Emc2101WriteRegister,
    },
    i2c_retry::{retry_transfer, I2C_TRANSACTION_TIMEOUT_MS},
};

pub const I2C_SDA_GPIO: i32 = 47;
pub const I2C_SCL_GPIO: i32 = 48;
pub const I2C_SPEED_KHZ: u32 = 400;

const INA260_I2C_ADDRESS: u8 = 0x40;
const EMC2101_I2C_ADDRESS: u8 = 0x4c;
const DS4432U_I2C_ADDRESS: u8 = 0x48;
const SSD1306_I2C_ADDRESS: u8 = 0x3c;

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

        let config = I2cConfig::new()
            .baudrate(I2C_SPEED_KHZ.kHz().into())
            .sda_enable_pullup(true)
            .scl_enable_pullup(true);
        let driver =
            I2cDriver::new(i2c, sda, scl, &config).context("initialize Ultra 205 I2C0 bus")?;
        Ok(Self { driver })
    }

    pub(crate) fn startup_display(&mut self) -> DisplayBus<'_, 'd> {
        DisplayBus {
            driver: &mut self.driver,
        }
    }

    pub(crate) fn into_runtime(self) -> RuntimeI2cOwner<'d> {
        RuntimeI2cOwner {
            driver: self.driver,
        }
    }
}

fn transaction_timeout_ticks() -> esp_idf_sys::TickType_t {
    TickType::new_millis(I2C_TRANSACTION_TIMEOUT_MS).ticks()
}

fn retry_driver_transfer<T, E>(transfer: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    retry_transfer(transfer, FreeRtos::delay_ms)
}

pub(crate) struct DisplayBus<'bus, 'd> {
    driver: &'bus mut I2cDriver<'d>,
}

#[derive(Debug)]
pub(crate) enum DisplayI2cError {
    Driver(I2cError),
    RestrictedAddress,
}

impl EmbeddedHalI2cError for DisplayI2cError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Driver(error) => error.kind(),
            Self::RestrictedAddress => ErrorKind::Other,
        }
    }
}

fn restrict_display_address(address: u8) -> Result<(), DisplayI2cError> {
    if address != SSD1306_I2C_ADDRESS {
        return Err(DisplayI2cError::RestrictedAddress);
    }
    Ok(())
}

/// Runtime owner exposing only display writes and closed sensor reads.
pub(crate) struct RuntimeI2cOwner<'d> {
    driver: I2cDriver<'d>,
}

impl<'d> RuntimeI2cOwner<'d> {
    pub(crate) fn display(&mut self) -> DisplayBus<'_, 'd> {
        DisplayBus {
            driver: &mut self.driver,
        }
    }

    pub(super) fn sensors(&mut self) -> ReadOnlySensorBus<'_, 'd> {
        ReadOnlySensorBus {
            driver: &mut self.driver,
        }
    }

    pub(super) fn actuators(&mut self) -> ActuationBus<'_, 'd> {
        ActuationBus {
            driver: &mut self.driver,
        }
    }
}

impl ErrorType for DisplayBus<'_, '_> {
    type Error = DisplayI2cError;
}

impl EmbeddedHalI2c for DisplayBus<'_, '_> {
    fn read(&mut self, address: u8, output: &mut [u8]) -> Result<(), Self::Error> {
        restrict_display_address(address)?;
        retry_driver_transfer(|| {
            self.driver
                .read(address, output, transaction_timeout_ticks())
        })
        .map_err(|error| DisplayI2cError::Driver(I2cError::from(error)))
    }

    fn write(&mut self, address: u8, input: &[u8]) -> Result<(), Self::Error> {
        restrict_display_address(address)?;
        retry_driver_transfer(|| {
            self.driver
                .write(address, input, transaction_timeout_ticks())
        })
        .map_err(|error| DisplayI2cError::Driver(I2cError::from(error)))
    }

    fn write_read(
        &mut self,
        address: u8,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error> {
        restrict_display_address(address)?;
        retry_driver_transfer(|| {
            self.driver
                .write_read(address, input, output, transaction_timeout_ticks())
        })
        .map_err(|error| DisplayI2cError::Driver(I2cError::from(error)))
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        restrict_display_address(address)?;
        retry_driver_transfer(|| {
            self.driver
                .transaction(address, operations, transaction_timeout_ticks())
        })
        .map_err(|error| DisplayI2cError::Driver(I2cError::from(error)))
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

pub(crate) struct ReadOnlySensorBus<'bus, 'd> {
    driver: &'bus mut I2cDriver<'d>,
}

impl ReadOnlySensorBus<'_, '_> {
    pub(crate) fn read_ina260(
        &mut self,
        register: Ina260ReadRegister,
        output: &mut [u8; 2],
    ) -> Result<()> {
        self.read_register(INA260_I2C_ADDRESS, register.address(), output)
    }

    fn read_register(&mut self, device_addr: u8, register: u8, output: &mut [u8]) -> Result<()> {
        retry_driver_transfer(|| {
            self.driver.write_read(
                device_addr,
                &[register],
                output,
                transaction_timeout_ticks(),
            )
        })
        .with_context(|| format!("i2c read register 0x{register:02x} device 0x{device_addr:02x}"))
    }
}

impl Emc2101RegisterReader for ReadOnlySensorBus<'_, '_> {
    type Error = anyhow::Error;

    fn read_emc2101(
        &mut self,
        register: Emc2101ReadRegister,
        output: &mut [u8; 1],
    ) -> Result<(), Self::Error> {
        self.read_register(EMC2101_I2C_ADDRESS, register.address(), output)
    }
}

pub(super) struct ActuationBus<'bus, 'd> {
    driver: &'bus mut I2cDriver<'d>,
}

impl ActuationBus<'_, '_> {
    fn write_register(&mut self, device_addr: u8, register: u8, value: u8) -> Result<()> {
        retry_driver_transfer(|| {
            self.driver
                .write(device_addr, &[register, value], transaction_timeout_ticks())
        })
        .with_context(|| format!("i2c write register 0x{register:02x} device 0x{device_addr:02x}"))
    }
}

impl Emc2101RegisterWriter for ActuationBus<'_, '_> {
    type Error = anyhow::Error;

    fn write_emc2101(
        &mut self,
        register: Emc2101WriteRegister,
        value: u8,
    ) -> Result<(), Self::Error> {
        self.write_register(EMC2101_I2C_ADDRESS, register.address(), value)
    }
}

impl Ds4432uRegisterWriter for ActuationBus<'_, '_> {
    type Error = anyhow::Error;

    fn write_ds4432u(
        &mut self,
        register: Ds4432uWriteRegister,
        value: u8,
    ) -> Result<(), Self::Error> {
        self.write_register(DS4432U_I2C_ADDRESS, register.address(), value)
    }
}
