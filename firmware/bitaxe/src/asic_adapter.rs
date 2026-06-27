//! Narrow firmware interpreter for typed BM1366 adapter actions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/serial.c`
//! - `reference/esp-miner/main/power/asic_reset.c`
//! - parity checklist rows `ASIC-005`, `ASIC-007`, and `ASIC-008`

use anyhow::{Context, Result};
use bitaxe_asic::bm1366::{
    adapter_gate::AsicAdapterMode,
    chip_detect::{
        self, Bm1366AdapterIoFault, Bm1366AdapterSetupFault, CHIP_DETECT_ADAPTER_ERROR,
        CHIP_DETECT_RESPONSE_INVALID, RESET_ADAPTER_UNAVAILABLE, UART_ADAPTER_UNAVAILABLE,
    },
    command::Bm1366AdapterAction,
    init_plan::{Bm1366InitPlan, Bm1366Preflight, BoardPreflightEvidence, ConfigPreflightEvidence},
    observation::AsicInitStatus,
    BM1366_RESULT_FRAME_LEN,
};
use esp_idf_svc::hal::{
    gpio::{InputPin, OutputPin},
    uart::Uart,
};

mod reset;
mod status;
mod uart;

pub struct AsicBootPeripherals<UART, RESET, TX, RX> {
    pub uart: UART,
    pub reset: RESET,
    pub tx: TX,
    pub rx: RX,
}

pub fn run_boot_gate_with_peripherals<UART, RESET, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, TX, RX>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    match AsicAdapterMode::from_compile_env(
        option_env!("BITAXE_ASIC_DIAGNOSTIC"),
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK"),
    ) {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(())
        }
        AsicAdapterMode::ChipDetectOnly => run_chip_detect_only(peripherals),
    }
}

pub fn run_boot_gate_without_peripherals(reason: &'static str) -> Result<()> {
    match AsicAdapterMode::from_compile_env(
        option_env!("BITAXE_ASIC_DIAGNOSTIC"),
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK"),
    ) {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(())
        }
        AsicAdapterMode::ChipDetectOnly => {
            log::warn!("asic_status=fail_closed reason={reason}");
            status::publish_status(AsicInitStatus::FailClosed { reason });
            Ok(())
        }
    }
}

fn run_chip_detect_only<UART, RESET, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, TX, RX>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );
    let decision = Bm1366InitPlan::chip_detect_only(preflight);
    let mut reset = match reset::AsicReset::new(peripherals.reset)
        .context("initialize ASIC reset GPIO adapter")
    {
        Ok(reset) => reset,
        Err(error) => {
            fail_closed_setup_error(Bm1366AdapterSetupFault::ResetUnavailable, None, &error);
            return Ok(());
        }
    };
    let mut uart = match uart::AsicUart::new(peripherals.uart, peripherals.tx, peripherals.rx)
        .context("initialize BM1366 UART1 adapter")
    {
        Ok(uart) => uart,
        Err(error) => {
            fail_closed_setup_error(
                Bm1366AdapterSetupFault::UartUnavailable,
                Some(&mut reset),
                &error,
            );
            return Ok(());
        }
    };

    for action in decision.actions() {
        match interpret_action(action, &mut uart, &mut reset) {
            Ok(ActionOutcome::Continue) => {}
            Ok(ActionOutcome::Stop) => return Ok(()),
            Err(error) => {
                fail_closed_adapter_error(&mut reset, &error);
                return Ok(());
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionOutcome {
    Continue,
    Stop,
}

fn interpret_action(
    action: &Bm1366AdapterAction,
    uart: &mut uart::AsicUart<'_>,
    reset: &mut reset::AsicReset<'_>,
) -> Result<ActionOutcome> {
    match action {
        Bm1366AdapterAction::UseDefaultBaud { baud } | Bm1366AdapterAction::UseMaxBaud { baud } => {
            uart.change_baud(*baud)?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::WaitTxDone { timeout_ms } => {
            uart.wait_tx_done(*timeout_ms)?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::ClearRx => {
            uart.clear_rx()?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::WriteFrame(frame) => {
            uart.write_frame(frame.as_ref())?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::ReadExact { len, timeout_ms } => {
            let _ = uart.read_exact(*len, *timeout_ms)?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::ReadChipId {
            expected_chips,
            timeout_ms,
        } => {
            let response = uart.read_exact(BM1366_RESULT_FRAME_LEN, *timeout_ms)?;
            match chip_detect::validate_single_chip_detect_response(&response, *expected_chips) {
                Ok(chips) => {
                    status::publish_status(AsicInitStatus::ChipDetectedNoMining { chips });
                    Ok(ActionOutcome::Continue)
                }
                Err(fault) => {
                    log::warn!(
                        "asic_status=fail_closed reason={CHIP_DETECT_RESPONSE_INVALID} error={fault}"
                    );
                    best_effort_hold_reset_low(reset, CHIP_DETECT_RESPONSE_INVALID);
                    status::publish_status(AsicInitStatus::FailClosed {
                        reason: CHIP_DETECT_RESPONSE_INVALID,
                    });
                    Ok(ActionOutcome::Stop)
                }
            }
        }
        Bm1366AdapterAction::DelayMs(delay_ms) => {
            std::thread::sleep(std::time::Duration::from_millis(u64::from(*delay_ms)));
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => {
            reset.reset_pulse(*low_ms, *high_ms)?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::HoldResetLow => {
            reset.hold_reset_low()?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::PublishStatus(init_status) => {
            status::publish_status(*init_status);
            Ok(ActionOutcome::Continue)
        }
    }
}

fn best_effort_hold_reset_low(reset: &mut reset::AsicReset<'_>, reason: &'static str) {
    if let Err(error) = reset.hold_reset_low() {
        log::warn!("asic_status=fail_closed reason={reason} hold_reset_low_error={error}");
    }
}

fn fail_closed_adapter_error(reset: &mut reset::AsicReset<'_>, error: &anyhow::Error) {
    log::warn!("asic_status=fail_closed reason={CHIP_DETECT_ADAPTER_ERROR} error={error:#}");
    for action in chip_detect::adapter_io_failure_actions(Bm1366AdapterIoFault::AdapterError) {
        match action {
            Bm1366AdapterAction::HoldResetLow => {
                best_effort_hold_reset_low(reset, CHIP_DETECT_ADAPTER_ERROR);
            }
            Bm1366AdapterAction::PublishStatus(init_status) => {
                status::publish_status(init_status);
            }
            _ => {}
        }
    }
}

fn fail_closed_setup_error(
    fault: Bm1366AdapterSetupFault,
    mut maybe_reset: Option<&mut reset::AsicReset<'_>>,
    error: &anyhow::Error,
) {
    let reason = match fault {
        Bm1366AdapterSetupFault::ResetUnavailable => RESET_ADAPTER_UNAVAILABLE,
        Bm1366AdapterSetupFault::UartUnavailable => UART_ADAPTER_UNAVAILABLE,
    };
    log::warn!("asic_status=fail_closed reason={reason} error={error:#}");
    for action in chip_detect::adapter_setup_failure_actions(fault) {
        match action {
            Bm1366AdapterAction::HoldResetLow => {
                if let Some(reset) = maybe_reset.as_deref_mut() {
                    best_effort_hold_reset_low(reset, reason);
                }
            }
            Bm1366AdapterAction::PublishStatus(init_status) => {
                status::publish_status(init_status);
            }
            _ => {}
        }
    }
}
