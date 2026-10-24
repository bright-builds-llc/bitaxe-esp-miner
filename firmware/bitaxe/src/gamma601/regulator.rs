//! The TPS546 steps the test variants share (ADR-0041): reading the controller, writing and
//! reading back the Gamma configuration, and turning the output off with confirmation. The
//! `gamma601-mining` variant adds the production core voltage: configured and turned on for
//! mining, and changed live.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c` (`TPS546_init`, `TPS546_set_vout`).

use crate::safety_adapter::{gamma601 as hardware, RuntimeI2cOwner};
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use bitaxe_safety::tps546::config::{self, Readback};
use bitaxe_safety::tps546::{OffState, ReadCommand, StatusWord};
use std::time::Duration;

/// The off command is repeated, this far apart, until the output confirms off.
const OFF_CONFIRM_PERIOD_MS: u64 = 250;
const OFF_CONFIRM_ATTEMPTS: u32 = 8;

pub(super) fn read_vout_mode(owner: &mut RuntimeI2cOwner<'_>) -> Option<u8> {
    let mut budget = hardware::budget();
    let mut mode = [0_u8; 1];
    owner
        .tps546(&mut budget)
        .read(ReadCommand::VoutMode, &mut mode)
        .ok()
        .map(|()| mode[0])
}

pub(super) fn read_status(owner: &mut RuntimeI2cOwner<'_>) -> Option<StatusWord> {
    let mut budget = hardware::budget();
    let mut word = [0_u8; 2];
    owner
        .tps546(&mut budget)
        .read(ReadCommand::StatusWord, &mut word)
        .ok()
        .map(|()| StatusWord(u16::from_le_bytes(word)))
}

/// Writes every register of the plan, then reads them all back.
pub(super) fn write_and_read_back(
    owner: &mut RuntimeI2cOwner<'_>,
    writes: &[config::ConfigWrite],
    vout_mode: u8,
) -> Result<Readback, &'static str> {
    for write in writes {
        let mut budget = hardware::budget();
        owner
            .tps546(&mut budget)
            .write_config(write)
            .map_err(|_| "config_write_failed")?;
    }
    read_back(owner, writes, vout_mode)
}

/// Reads every register back; the first one that differs from what was written is recorded.
fn read_back(
    owner: &mut RuntimeI2cOwner<'_>,
    writes: &[config::ConfigWrite],
    vout_mode: u8,
) -> Result<Readback, &'static str> {
    for write in writes {
        let mut budget = hardware::budget();
        let read = owner
            .tps546(&mut budget)
            .read_config(write)
            .map_err(|_| "config_readback_failed")?;
        if !config::readback_matches(write, read, vout_mode) {
            return Ok(Readback::Mismatch {
                command: write.command(),
                wrote: write.value(),
                read,
            });
        }
    }
    Ok(Readback::Matched)
}

/// Commands the output off, restoring upstream's on/off configuration, and repeats until it
/// confirms off or the attempts run out. `None` when no reading succeeded. Nothing here formats
/// an error: this path runs when the output may still be on, so it keeps its stack shallow.
pub(super) fn turn_off_and_confirm(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
) -> Option<OffState> {
    let mut maybe_state = None;
    for _ in 0..OFF_CONFIRM_ATTEMPTS {
        let mut budget = hardware::budget();
        if hardware::command_off(&mut owner.tps546(&mut budget)).is_err() {
            log::warn!("gamma601_regulator=fault category=off_command");
        }
        std::thread::sleep(Duration::from_millis(OFF_CONFIRM_PERIOD_MS));
        let mut reading = report;
        hardware::read_controller(owner, &mut reading);
        maybe_state = reading.maybe_off.or(maybe_state);
        if maybe_state == Some(OffState::Confirmed) {
            break;
        }
    }
    maybe_state
}

#[cfg(feature = "gamma601-mining")]
pub(crate) use production::{power_on, set_vout};

/// The production core voltage, only in the mining variant.
#[cfg(feature = "gamma601-mining")]
mod production {
    use super::*;
    use bitaxe_safety::tps546::enable::{self, EnableRefusal, SettleFailure, SETTLE_MS};
    use bitaxe_safety::tps546::{config::ConfigRefusal, ConfirmedOff};

    /// Configures the controller for `target_mv` and turns its output on, as upstream's
    /// `TPS546_init` then `TPS546_set_vout`:
    /// 1. commanded off until it confirms off;
    /// 2. configured and read back, still off, then re-confirmed off;
    /// 3. faults cleared and a clean, off baseline read;
    /// 4. `OPERATION` on through a token armed from those checks and a held ASIC reset;
    /// 5. after `SETTLE_MS`, `OPERATION` read back on, VOUT near the target and no new fault.
    ///
    /// An `Err` before the enable write leaves the output off. After the write the output may
    /// be on even when it reports an error, so the caller turns it off on any `Err`.
    pub(crate) fn power_on(
        owner: &mut RuntimeI2cOwner<'_>,
        report: SafeBootReport,
        target_mv: u16,
    ) -> Result<(), &'static str> {
        let target_volts = f64::from(target_mv) / 1000.0;
        let off = turn_off_and_confirm(owner, report)
            .and_then(OffState::confirmed)
            .ok_or("off_unconfirmed")?;
        let vout_mode = read_vout_mode(owner).ok_or("vout_mode_unread")?;
        let plan = config::gamma_config_plan(off, vout_mode, target_volts)
            .map_err(ConfigRefusal::label)?;
        if write_and_read_back(owner, plan.writes(), vout_mode)? != Readback::Matched {
            return Err("config_readback_mismatch");
        }
        let off_after_config =
            confirmed_off(owner, report).ok_or("off_unconfirmed_after_config")?;
        let mut budget = hardware::budget();
        owner
            .tps546(&mut budget)
            .clear_faults()
            .map_err(|_| "clear_faults_failed")?;
        let baseline = read_status(owner).ok_or("baseline_unread")?;
        let armed = enable::arm_production(
            target_volts,
            baseline,
            Readback::Matched,
            off_after_config,
            crate::gamma601::reset_held(),
        )
        .map_err(EnableRefusal::label)?;
        let mut budget = hardware::budget();
        owner
            .tps546(&mut budget)
            .operation_on(armed)
            .map_err(|_| "enable_write_failed")?;
        std::thread::sleep(Duration::from_millis(u64::from(SETTLE_MS)));
        let operation = read_operation(owner).ok_or("operation_unread")?;
        let mut reading = report;
        hardware::read_controller(owner, &mut reading);
        let status = reading
            .maybe_status
            .map(StatusWord)
            .ok_or("status_unread")?;
        let vout_volts = reading.maybe_vout_volts.ok_or("vout_unread")?;
        enable::judge_settled(target_volts, baseline, operation, status, vout_volts)
            .map_err(SettleFailure::label)
    }

    /// Changes the output to `target_mv` while it is on (upstream `TPS546_set_vout`), only while
    /// the controller still holds this boot's `VOUT_MAX` and loop scale, then reads the new
    /// `VOUT_COMMAND` back. A readback `Err` comes after the write has landed; the clamp and the
    /// confirmed 1.25 V `VOUT_MAX` still bound it.
    pub(crate) fn set_vout(
        owner: &mut RuntimeI2cOwner<'_>,
        target_mv: u16,
    ) -> Result<(), &'static str> {
        let vout_mode = read_vout_mode(owner).ok_or("vout_mode_unread")?;
        for guard in config::live_guard_writes(vout_mode).map_err(ConfigRefusal::label)? {
            if !register_reads_as_written(owner, &guard, vout_mode)? {
                return Err("configuration_lost");
            }
        }
        let write = config::vout_command_write(vout_mode, f64::from(target_mv) / 1000.0)
            .map_err(ConfigRefusal::label)?;
        let mut budget = hardware::budget();
        owner
            .tps546(&mut budget)
            .write_config(&write)
            .map_err(|_| "vout_write_failed")?;
        if register_reads_as_written(owner, &write, vout_mode)? {
            Ok(())
        } else {
            Err("vout_readback_mismatch")
        }
    }

    fn register_reads_as_written(
        owner: &mut RuntimeI2cOwner<'_>,
        write: &config::ConfigWrite,
        vout_mode: u8,
    ) -> Result<bool, &'static str> {
        let mut budget = hardware::budget();
        let read = owner
            .tps546(&mut budget)
            .read_config(write)
            .map_err(|_| "config_readback_failed")?;
        Ok(config::readback_matches(write, read, vout_mode))
    }

    fn read_operation(owner: &mut RuntimeI2cOwner<'_>) -> Option<u8> {
        let mut budget = hardware::budget();
        let mut operation = [0_u8; 1];
        owner
            .tps546(&mut budget)
            .read(ReadCommand::Operation, &mut operation)
            .ok()
            .map(|()| operation[0])
    }

    fn confirmed_off(
        owner: &mut RuntimeI2cOwner<'_>,
        report: SafeBootReport,
    ) -> Option<ConfirmedOff> {
        let mut reading = report;
        hardware::read_controller(owner, &mut reading);
        reading.maybe_off.and_then(OffState::confirmed)
    }
}
