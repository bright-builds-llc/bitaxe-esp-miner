//! The BM1370's bring-up as the adapter's actions, in the two phases the firmware runs: chip
//! detection after the reset, then the mining-ready init and the baud switch.
//!
//! Reference: `reference/esp-miner/main/tasks/power_management_task.c` (`mining_start`: the
//! UART flush and 100 ms before the init), `main/power/asic_init.c` (`asic_initialize`: the
//! reset, the baud back to 115200 and 100 ms, `ASIC_init`, then
//! `SERIAL_set_baud(BM1370_set_max_baud())` and the buffer clear) and
//! `components/asic/bm1370.c` (`BM1370_init`, `count_asic_chips`). Upstream's 2000 ms
//! stabilization after the init is the caller's, outside these actions.

use crate::{
    asic::ChipIdCountFrameDisposition,
    bm1366::{command::Bm1366AdapterAction, observation::AsicInitStatus},
    bm1370::{
        init::{initialization_plan, Bm1370InitConfig},
        protocol::{Bm1370Command, DEFAULT_BAUD, MAX_BAUD},
        result::parse_chip_id_response,
        shutdown::lower_commands,
        Bm1370ProtocolFault,
    },
};

/// How long the count loop waits for the line to go idle, as the BM1366's.
const CHIP_ID_IDLE_TIMEOUT_MS: u32 = 1_000;
/// Upstream's settle time after the UART flush and after the baud reset.
const UART_SETTLE_MS: u32 = 100;

/// The UART flushed, the reset, the default baud, the version masks and the `CHIP_ID` read,
/// ending in the count of exactly `config.chip_count` chips.
pub fn chip_detect_actions(
    config: Bm1370InitConfig,
) -> Result<Vec<Bm1366AdapterAction>, Bm1370ProtocolFault> {
    let mut actions = vec![
        Bm1366AdapterAction::PublishStatus(AsicInitStatus::ChipDetectOnly),
        Bm1366AdapterAction::ClearRx,
        Bm1366AdapterAction::DelayMs(UART_SETTLE_MS),
        Bm1366AdapterAction::RESET_PULSE,
        Bm1366AdapterAction::UseDefaultBaud { baud: DEFAULT_BAUD },
        Bm1366AdapterAction::DelayMs(UART_SETTLE_MS),
    ];
    actions.extend(lower_commands(
        &initialization_plan(config)?.before_chip_detect,
    )?);
    actions.push(Bm1366AdapterAction::WAIT_TX_DONE);
    actions.push(Bm1366AdapterAction::ReadChipId {
        expected_chips: config.chip_count,
        timeout_ms: CHIP_ID_IDLE_TIMEOUT_MS,
    });
    Ok(actions)
}

/// The init after detection, with its frequency ramp and nonce space, then the switch to the
/// maximum baud and the cleared receive buffer.
pub fn mining_ready_actions(
    config: Bm1370InitConfig,
) -> Result<Vec<Bm1366AdapterAction>, Bm1370ProtocolFault> {
    let mut actions = lower_commands(&initialization_plan(config)?.after_chip_detect)?;
    actions.extend(lower_commands(&[Bm1370Command::SetMaxBaud])?);
    actions.extend([
        Bm1366AdapterAction::WAIT_TX_DONE,
        Bm1366AdapterAction::UseMaxBaud { baud: MAX_BAUD },
        Bm1366AdapterAction::ClearRx,
        Bm1366AdapterAction::PublishStatus(AsicInitStatus::InitializedNoMining),
    ]);
    Ok(actions)
}

/// One frame of the chip count: a wrong length fails; a bad preamble, CRC or chip id is
/// skipped, as upstream's `count_asic_chips` skips it.
pub fn classify_chip_id_count_frame(
    bytes: &[u8],
) -> Result<ChipIdCountFrameDisposition, Bm1370ProtocolFault> {
    // `parse_chip_id_response` refuses a wrong length first, which stays a failed count.
    match parse_chip_id_response(bytes) {
        Ok(_) => Ok(ChipIdCountFrameDisposition::Accept),
        Err(
            Bm1370ProtocolFault::BadPreamble { .. }
            | Bm1370ProtocolFault::BadCrc
            | Bm1370ProtocolFault::UnexpectedChipId { .. },
        ) => Ok(ChipIdCountFrameDisposition::SoftRetry),
        Err(fault) => Err(fault),
    }
}
