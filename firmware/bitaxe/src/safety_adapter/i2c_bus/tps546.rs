//! TPS546 PMBus access for the Gamma 601 safe boot (ADR-0041).
//!
//! The bus offers the closed reads of `tps546::ReadCommand` and the writes of `OffWrite`, which
//! can only turn the output off. Nothing here can raise the output or store configuration.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c`

use super::*;
use bitaxe_safety::tps546::{OffWrite, ReadCommand, I2C_ADDRESS};

pub(crate) struct Tps546Bus<'bus, 'budget, 'd> {
    driver: &'bus mut I2cDriver<'d>,
    budget: &'budget mut RuntimeI2cBudget,
}

impl<'d> BitaxeI2cBus<'d> {
    /// The controller on the startup bus, so the safe boot can turn it off before anything else.
    pub(crate) fn tps546<'bus, 'budget>(
        &'bus mut self,
        budget: &'budget mut RuntimeI2cBudget,
    ) -> Tps546Bus<'bus, 'budget, 'd> {
        Tps546Bus {
            driver: &mut self.driver,
            budget,
        }
    }
}

impl<'d> RuntimeI2cOwner<'d> {
    pub(crate) fn tps546<'bus, 'budget>(
        &'bus mut self,
        budget: &'budget mut RuntimeI2cBudget,
    ) -> Tps546Bus<'bus, 'budget, 'd> {
        Tps546Bus {
            driver: &mut self.driver,
            budget,
        }
    }
}

impl Tps546Bus<'_, '_, '_> {
    /// Reads one command into `output`, which must be exactly the command's length.
    pub(crate) fn read(&mut self, command: ReadCommand, output: &mut [u8]) -> Result<()> {
        if output.len() != command.length() {
            anyhow::bail!(
                "tps546 read 0x{:02x} expects {} bytes",
                command.code(),
                command.length()
            );
        }
        retry_runtime_driver_transfer(self.budget, |timeout_ms| {
            self.driver.write_read(
                I2C_ADDRESS,
                &[command.code()],
                output,
                TickType::new_millis(timeout_ms).ticks(),
            )
        })
        .map_err(transfer_error)
        .with_context(|| format!("tps546 read 0x{:02x}", command.code()))
    }

    /// Sends one of the off writes.
    pub(crate) fn write_off(&mut self, write: OffWrite) -> Result<()> {
        let bytes = write.bytes();
        retry_runtime_driver_transfer(self.budget, |timeout_ms| {
            self.driver.write(
                I2C_ADDRESS,
                &bytes,
                TickType::new_millis(timeout_ms).ticks(),
            )
        })
        .map_err(transfer_error)
        .with_context(|| format!("tps546 write 0x{:02x}", bytes[0]))
    }
}

fn transfer_error(error: RuntimeI2cTransferError<esp_idf_sys::EspError>) -> anyhow::Error {
    match error {
        RuntimeI2cTransferError::Driver(error) => anyhow::Error::new(error),
        RuntimeI2cTransferError::BudgetExhausted => anyhow::anyhow!("runtime budget exhausted"),
    }
}
