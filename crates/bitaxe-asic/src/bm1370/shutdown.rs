//! The BM1370's safe shutdown, before its reset is held: ramp down to the reset frequency, then
//! size the nonce space for it.
//!
//! Reference: `reference/esp-miner/main/tasks/power_management_task.c` (`mining_stop`: the
//! frequency goes to 50 MHz and the nonce space follows, before the core voltage is cut).

use crate::{
    bm1366::command::Bm1366AdapterAction,
    bm1370::{
        init::{
            frequency_plan, frequency_ramp_commands, hash_counting_number,
            RESET_FREQUENCY_QUARTER_MHZ,
        },
        protocol::Bm1370Command,
        Bm1370ProtocolFault,
    },
};

/// The shutdown commands from a mining frequency: the stepped ramp down to 50 MHz, then the
/// nonce space for the achieved 50 MHz.
pub fn safe_shutdown_commands(
    frequency_mhz: u16,
    chip_count: u8,
    core_count: u16,
) -> Result<Vec<Bm1370Command>, Bm1370ProtocolFault> {
    let mut commands =
        frequency_ramp_commands(u32::from(frequency_mhz) * 4, RESET_FREQUENCY_QUARTER_MHZ)?;
    let reset_frequency_mhz = crate::bm1366::frequency_voltage::actual_frequency_mhz(
        frequency_plan(RESET_FREQUENCY_QUARTER_MHZ)?,
    );
    commands.push(Bm1370Command::SetNonceSpace(hash_counting_number(
        1.0,
        reset_frequency_mhz,
        u16::from(chip_count),
        core_count,
    )?));
    Ok(commands)
}

/// The shutdown as adapter actions, ending once the last frame has left the UART. Holding the
/// reset is the caller's separate step.
pub fn safe_shutdown_command_actions(
    frequency_mhz: u16,
    chip_count: u8,
    core_count: u16,
) -> Result<Vec<Bm1366AdapterAction>, Bm1370ProtocolFault> {
    let mut actions = lower_commands(&safe_shutdown_commands(
        frequency_mhz,
        chip_count,
        core_count,
    )?)?;
    actions.push(Bm1366AdapterAction::WAIT_TX_DONE);
    Ok(actions)
}

/// Lowers commands into the adapter's vocabulary: each frame a write, each delay a delay.
pub fn lower_commands(
    commands: &[Bm1370Command],
) -> Result<Vec<Bm1366AdapterAction>, Bm1370ProtocolFault> {
    commands
        .iter()
        .map(|command| match command {
            Bm1370Command::DelayMs(milliseconds) => Ok(Bm1366AdapterAction::DelayMs(*milliseconds)),
            _ => command
                .maybe_frame_bytes()?
                .map(Bm1366AdapterAction::WriteFrame)
                .ok_or(Bm1370ProtocolFault::InvalidLength {
                    expected: 1,
                    actual: 0,
                }),
        })
        .collect()
}

#[cfg(test)]
mod tests;
