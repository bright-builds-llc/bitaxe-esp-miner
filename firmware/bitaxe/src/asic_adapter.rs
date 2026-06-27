//! Narrow firmware interpreter for typed BM1366 adapter actions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/serial.c`
//! - `reference/esp-miner/main/power/asic_reset.c`
//! - parity checklist rows `ASIC-005`, `ASIC-007`, and `ASIC-008`

use anyhow::{Context, Result};
use bitaxe_asic::bm1366::{
    adapter_gate::AsicAdapterMode,
    command::Bm1366AdapterAction,
    init_plan::{Bm1366InitPlan, Bm1366Preflight, BoardPreflightEvidence, ConfigPreflightEvidence},
    observation::AsicInitStatus,
};
use esp_idf_svc::hal::peripherals::Peripherals;

mod reset;
mod status;
mod uart;

pub fn run_boot_gate() -> Result<()> {
    match AsicAdapterMode::from_compile_env(
        option_env!("BITAXE_ASIC_DIAGNOSTIC"),
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK"),
    ) {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(())
        }
        AsicAdapterMode::ChipDetectOnly => run_chip_detect_only(),
    }
}

fn run_chip_detect_only() -> Result<()> {
    let peripherals = match Peripherals::take() {
        Ok(peripherals) => peripherals,
        Err(error) => {
            log::warn!("asic_status=fail_closed reason=peripherals_unavailable error={error}");
            status::publish_status(AsicInitStatus::FailClosed {
                reason: "peripherals_unavailable",
            });
            return Ok(());
        }
    };

    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );
    let decision = Bm1366InitPlan::chip_detect_only(preflight);
    let mut uart = uart::AsicUart::new(
        peripherals.uart1,
        peripherals.pins.gpio17,
        peripherals.pins.gpio18,
    )
    .context("initialize BM1366 UART1 adapter")?;
    let mut reset = reset::AsicReset::new(peripherals.pins.gpio1)
        .context("initialize ASIC reset GPIO adapter")?;

    for action in decision.actions() {
        interpret_action(action, &mut uart, &mut reset)?;
    }

    Ok(())
}

fn interpret_action(
    action: &Bm1366AdapterAction,
    uart: &mut uart::AsicUart<'_>,
    reset: &mut reset::AsicReset<'_>,
) -> Result<()> {
    match action {
        Bm1366AdapterAction::UseDefaultBaud { baud } | Bm1366AdapterAction::UseMaxBaud { baud } => {
            uart.change_baud(*baud)
        }
        Bm1366AdapterAction::WaitTxDone { timeout_ms } => uart.wait_tx_done(*timeout_ms),
        Bm1366AdapterAction::ClearRx => uart.clear_rx(),
        Bm1366AdapterAction::WriteFrame(frame) => uart.write_frame(frame.as_ref()),
        Bm1366AdapterAction::ReadExact { len, timeout_ms } => {
            let _ = uart.read_exact(*len, *timeout_ms)?;
            Ok(())
        }
        Bm1366AdapterAction::DelayMs(delay_ms) => {
            std::thread::sleep(std::time::Duration::from_millis(u64::from(*delay_ms)));
            Ok(())
        }
        Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => reset.reset_pulse(*low_ms, *high_ms),
        Bm1366AdapterAction::HoldResetLow => reset.hold_reset_low(),
        Bm1366AdapterAction::PublishStatus(init_status) => {
            status::publish_status(*init_status);
            Ok(())
        }
    }
}
