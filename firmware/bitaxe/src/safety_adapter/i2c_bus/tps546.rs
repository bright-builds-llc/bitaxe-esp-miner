//! TPS546 PMBus access for the Gamma 601 safe boot (ADR-0041).
//!
//! The bus offers the closed reads of `tps546::ReadCommand`, the off writes of `OffWrite`, and the
//! Gamma configuration, `CLEAR_FAULTS` and `OPERATION` on: every 601 build may power the ASIC
//! since the promotion (ADR-0041 item 15), and the 205 compiles none of this. No build can store
//! configuration.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c`

use super::*;
use bitaxe_safety::tps546::{
    config::{self, ConfigWrite},
    enable,
};
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

/// The writes that configure the controller and turn its output on (ADR-0041 phase D and items 7
/// and 15).
impl Tps546Bus<'_, '_, '_> {
    /// Writes one entry of the Gamma plan. Only `gamma_config_plan` makes a `ConfigWrite`; a
    /// command outside the plan, or an NVM command, is still refused before the bus.
    pub(crate) fn write_config(&mut self, write: &ConfigWrite) -> Result<()> {
        let command = write.command();
        if !config::is_plan_command(command) || config::is_nvm_command(command) {
            anyhow::bail!("tps546 write 0x{command:02x} is not a configuration write");
        }
        let [low, high] = write.value().to_le_bytes();
        let word = [command, low, high];
        let bytes = if write.word() { &word[..] } else { &word[..2] };
        self.send(bytes)
    }

    /// Reads back a configuration register, as a byte or a word.
    pub(crate) fn read_config(&mut self, write: &ConfigWrite) -> Result<u16> {
        let command = write.command();
        let mut word = [0_u8; 2];
        let length = if write.word() { 2 } else { 1 };
        let output = &mut word[..length];
        retry_runtime_driver_transfer(self.budget, |timeout_ms| {
            self.driver.write_read(
                I2C_ADDRESS,
                &[command],
                output,
                TickType::new_millis(timeout_ms).ticks(),
            )
        })
        .map_err(transfer_error)
        .with_context(|| format!("tps546 read 0x{command:02x}"))?;
        Ok(u16::from_le_bytes(word))
    }

    pub(crate) fn clear_faults(&mut self) -> Result<()> {
        self.send(&enable::CLEAR_FAULTS)
    }

    /// Turns the output on; the armed evidence is spent. The context literal is how the
    /// variant-isolation gate finds the enable in an ELF, whatever its symbols are called.
    pub(crate) fn operation_on(&mut self, _armed: enable::EnableArmed) -> Result<()> {
        self.send(&enable::OPERATION_ON)
            .context("tps546_operation_on_enable")
    }

    fn send(&mut self, bytes: &[u8]) -> Result<()> {
        retry_runtime_driver_transfer(self.budget, |timeout_ms| {
            self.driver
                .write(I2C_ADDRESS, bytes, TickType::new_millis(timeout_ms).ticks())
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
