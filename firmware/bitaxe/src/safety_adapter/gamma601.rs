//! Gamma 601 safe-boot hardware (ADR-0041): the TPS546 kept off, and the EMC2101 diode
//! configuration and fixed fan duty.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c`, `main/thermal/EMC2101.c` and
//! `main/thermal/thermal.c`.

use super::emc2101::{self, Emc2101RegisterWriter, Emc2101WriteRegister};
use super::i2c_bus::Tps546Bus;
use super::{RuntimeI2cBudget, RuntimeI2cOwner};
use anyhow::Result;
use bitaxe_safety::gamma601_safe_boot::{Controller, SafeBootReport};
use bitaxe_safety::sensor_acquisition::AcquisitionOutcome;
use bitaxe_safety::tps546::{self, ReadCommand, StatusWord, OFF_SEQUENCE};

/// The Gamma's external-diode ideality factor and beta compensation (`device_config.h`).
const EMC2101_IDEALITY: u8 = 0x24;
const EMC2101_BETA: u8 = 0x00;
/// Each group of transfers gets this long before its budget is exhausted.
const TRANSFER_BUDGET_MS: u64 = 400;

/// A fresh budget for one group of transfers.
pub(crate) fn budget() -> RuntimeI2cBudget {
    RuntimeI2cBudget::new(crate::runtime_uptime::millis().saturating_add(TRANSFER_BUDGET_MS))
}

/// Reads `IC_DEVICE_ID`; only a recognised TPS546 is ever written.
pub(crate) fn identify_controller(bus: &mut Tps546Bus<'_, '_, '_>) -> Controller {
    let mut block = [0_u8; 7];
    match bus.read(ReadCommand::DeviceId, &mut block) {
        Ok(()) => {
            tps546::classify_device_id(&block).map_or(Controller::Unrecognised, Controller::Found)
        }
        Err(_) => Controller::Unread,
    }
}

/// Sends the off writes, in upstream's order.
pub(crate) fn command_off(bus: &mut Tps546Bus<'_, '_, '_>) -> Result<()> {
    for write in OFF_SEQUENCE {
        bus.write_off(write)?;
    }
    Ok(())
}

/// Sets the fan duty and the Gamma's external-diode configuration.
pub(crate) fn configure_thermal(owner: &mut RuntimeI2cOwner<'_>, duty_percent: u8) -> Result<()> {
    let mut budget = budget();
    let mut bus = owner.actuators(&mut budget);
    emc2101::write_fan_duty_percent(&mut bus, duty_percent)?;
    bus.write_emc2101(Emc2101WriteRegister::ExternalIdeality, EMC2101_IDEALITY)?;
    bus.write_emc2101(Emc2101WriteRegister::BetaCompensation, EMC2101_BETA)
}

/// Reads the controller's status and telemetry into `report`.
pub(crate) fn read_controller(owner: &mut RuntimeI2cOwner<'_>, report: &mut SafeBootReport) {
    let mut budget = budget();
    let mut bus = owner.tps546(&mut budget);
    let mut word = |command| {
        let mut bytes = [0_u8; 2];
        bus.read(command, &mut bytes)
            .ok()
            .map(|()| u16::from_le_bytes(bytes))
    };
    let maybe_status = word(ReadCommand::StatusWord).map(StatusWord);
    let maybe_raw_vout = word(ReadCommand::ReadVout);
    let maybe_vin = word(ReadCommand::ReadVin).map(tps546::decode_linear11);
    let maybe_iout = word(ReadCommand::ReadIout).map(tps546::decode_linear11);
    let maybe_temperature = word(ReadCommand::ReadTemperature).map(tps546::decode_linear11);
    let mut mode = [0_u8; 1];
    let maybe_mode = bus
        .read(ReadCommand::VoutMode, &mut mode)
        .ok()
        .map(|()| mode[0]);
    let maybe_vout = maybe_raw_vout
        .zip(maybe_mode)
        .and_then(|(raw, mode)| tps546::decode_vout(raw, mode));
    report.maybe_faults = maybe_status.map(StatusWord::faults);
    report.maybe_off = maybe_status
        .zip(maybe_vout)
        .map(|(status, vout)| tps546::confirm_off(status, vout));
    report.maybe_vout_volts = maybe_vout;
    report.maybe_vin_volts = maybe_vin;
    report.maybe_iout_amps = maybe_iout;
    report.maybe_vr_temperature_celsius = maybe_temperature;
}

/// Reads the board temperature and fan speed into `report`.
pub(crate) fn read_thermal(owner: &mut RuntimeI2cOwner<'_>, report: &mut SafeBootReport) {
    let mut budget = budget();
    let mut bus = owner.sensors(&mut budget);
    report.maybe_board_temperature_celsius =
        success(emc2101::read_internal_temperature_acquisition(&mut bus));
    report.maybe_fan_rpm = success(emc2101::read_tachometer_acquisition(&mut bus)).map(u32::from);
}

fn success<T>(outcome: AcquisitionOutcome<T>) -> Option<T> {
    match outcome {
        AcquisitionOutcome::Success(value) => Some(value),
        _ => None,
    }
}
