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
        self, chip_detect_drain_budget_exhausted, Bm1366AdapterIoFault, Bm1366AdapterSetupFault,
        ChipIdCountFrameDisposition, CHIP_DETECT_ADAPTER_ERROR, CHIP_DETECT_RESPONSE_INVALID,
        COUNT_ASIC_CHIPS_IDLE_TIMEOUT_MS, COUNT_ASIC_CHIPS_TOTAL_TIMEOUT_MS,
        RESET_ADAPTER_UNAVAILABLE, UART_ADAPTER_UNAVAILABLE,
    },
    command::Bm1366AdapterAction,
    init_plan::{Bm1366InitPlan, Bm1366Preflight, BoardPreflightEvidence, ConfigPreflightEvidence},
    mining_ready::ultra_205_result_address_interval,
    observation::AsicInitStatus,
    result::{
        parse_bm1366_result_frame, Bm1366ParsedResult, Bm1366ValidJobIds, BM1366_RESULT_FRAME_LEN,
    },
    work::{diagnostic_job_frame, Bm1366JobId, Bm1366WorkFields},
};
use esp_idf_svc::hal::{
    gpio::{InputPin, OutputPin},
    uart::Uart,
};
#[allow(dead_code)]
pub(crate) mod production;
mod reset;
#[allow(dead_code)]
mod status;
mod uart;
#[allow(dead_code)]
mod work_result_investigation;

pub use work_result_investigation::accepted_state_snapshot_enabled;

pub struct AsicBootPeripherals<UART, RESET, ENABLE, TX, RX> {
    pub uart: UART,
    pub reset: RESET,
    pub enable: ENABLE,
    pub tx: TX,
    pub rx: RX,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootMiningBaseline {
    Confirmed,
    Unconfirmed,
}

pub fn run_boot_gate_with_peripherals<UART, RESET, ENABLE, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, ENABLE, TX, RX>,
) -> Result<BootMiningBaseline>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    ENABLE: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    match adapter_mode_from_firmware_compile_env() {
        AsicAdapterMode::FailClosed => retain_safe_production_peripherals(peripherals),
        AsicAdapterMode::ChipDetectOnly => {
            run_chip_detect_only(peripherals).map(|()| BootMiningBaseline::Unconfirmed)
        }
        AsicAdapterMode::WorkResultDiagnostic => {
            run_work_result_uart_bootstrap(peripherals).map(|()| BootMiningBaseline::Unconfirmed)
        }
    }
}

fn retain_safe_production_peripherals<UART, RESET, ENABLE, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, ENABLE, TX, RX>,
) -> Result<BootMiningBaseline>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    ENABLE: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    status::publish_default_fail_closed_status();
    let mut enable = match reset::AsicEnable::new(peripherals.enable)
        .context("initialize ASIC enable GPIO adapter")
    {
        Ok(enable) => enable,
        Err(error) => {
            log::warn!(
                "asic_production_status=fail_closed reason=enable_unavailable error={error:#}"
            );
            return Ok(BootMiningBaseline::Unconfirmed);
        }
    };
    if let Err(error) = enable.disable() {
        log::warn!(
            "asic_production_status=fail_closed reason=enable_disable_failed error={error:#}"
        );
        return Ok(BootMiningBaseline::Unconfirmed);
    }
    let mut reset = match reset::AsicReset::new(peripherals.reset)
        .context("initialize ASIC reset GPIO adapter")
    {
        Ok(reset) => reset,
        Err(error) => {
            log::warn!(
                "asic_production_status=fail_closed reason=reset_unavailable error={error:#}"
            );
            return Ok(BootMiningBaseline::Unconfirmed);
        }
    };
    if let Err(error) = reset.hold_reset_low() {
        log::warn!("asic_production_status=fail_closed reason=reset_hold_failed error={error:#}");
        return Ok(BootMiningBaseline::Unconfirmed);
    }
    let uart = match uart::AsicUart::new(peripherals.uart, peripherals.tx, peripherals.rx)
        .context("initialize retained BM1366 UART1 adapter")
    {
        Ok(uart) => uart,
        Err(error) => {
            log::warn!(
                "asic_production_status=fail_closed reason=uart_unavailable error={error:#}"
            );
            return Ok(BootMiningBaseline::Unconfirmed);
        }
    };
    production::store_production_peripherals(uart, reset, enable, false);
    Ok(BootMiningBaseline::Confirmed)
}

fn run_work_result_uart_bootstrap<UART, RESET, ENABLE, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, ENABLE, TX, RX>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    ENABLE: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    let _enable = peripherals.enable;
    let reset = match reset::AsicReset::new(peripherals.reset)
        .context("initialize ASIC reset GPIO adapter")
    {
        Ok(reset) => reset,
        Err(error) => {
            fail_closed_work_result_setup_error(None, &error);
            return Ok(());
        }
    };

    run_work_result_uart_bootstrap_after_reset(
        peripherals.uart,
        peripherals.tx,
        peripherals.rx,
        reset,
    )
}

pub fn run_boot_gate_without_peripherals(reason: &'static str) -> Result<BootMiningBaseline> {
    match adapter_mode_from_firmware_compile_env() {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(BootMiningBaseline::Unconfirmed)
        }
        AsicAdapterMode::ChipDetectOnly => {
            log::warn!("asic_status=fail_closed reason={reason}");
            status::publish_status(AsicInitStatus::FailClosed { reason });
            Ok(BootMiningBaseline::Unconfirmed)
        }
        AsicAdapterMode::WorkResultDiagnostic => {
            log::warn!("asic_status=fail_closed reason={reason}");
            status::publish_status(AsicInitStatus::FailClosed { reason });
            Ok(BootMiningBaseline::Unconfirmed)
        }
    }
}

fn adapter_mode_from_firmware_compile_env() -> AsicAdapterMode {
    AsicAdapterMode::from_compile_env(
        option_env!("BITAXE_ASIC_DIAGNOSTIC"),
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK"),
    )
}

fn run_chip_detect_only<UART, RESET, ENABLE, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, ENABLE, TX, RX>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    ENABLE: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    let _enable = peripherals.enable;
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

fn run_work_result_uart_bootstrap_after_reset<UART, TX, RX>(
    uart_peripheral: UART,
    tx: TX,
    rx: RX,
    mut reset: reset::AsicReset<'_>,
) -> Result<()>
where
    UART: Uart + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    let mut uart = match uart::AsicUart::new(uart_peripheral, tx, rx)
        .context("initialize BM1366 UART1 adapter")
    {
        Ok(uart) => uart,
        Err(error) => {
            fail_closed_work_result_setup_error(Some(&mut reset), &error);
            return Ok(());
        }
    };

    status::publish_work_result_diagnostic_started_status();
    let job_id = Bm1366JobId::new(0x28);
    let work_frame = match diagnostic_job_frame(job_id, diagnostic_work_fields()) {
        Ok(work_frame) => work_frame,
        Err(error) => {
            fail_closed_work_result_invalid(&mut reset, &error);
            return Ok(());
        }
    };

    if let Err(error) = uart.write_frame(work_frame.bytes()) {
        fail_closed_work_result_invalid(&mut reset, &error);
        return Ok(());
    }
    if let Err(error) = uart.wait_tx_done(uart::WAIT_TX_DONE_TIMEOUT_MS) {
        fail_closed_work_result_invalid(&mut reset, &error);
        return Ok(());
    }
    if uart_trace_enabled() {
        log::info!("asic_work_result_trace=work_tx_done elapsed_from_dispatch=0");
    }
    status::publish_work_result_dispatched_status(job_id.raw(), work_frame.bytes().len());

    let valid_jobs = Bm1366ValidJobIds::single(job_id);
    let address_interval = ultra_205_result_address_interval();
    if uart_trace_enabled() {
        log::info!(
            "asic_work_result_trace=result_read_start address_interval={address_interval} timeout_ms={}",
            uart::RESULT_WORK_TIMEOUT_MS
        );
    }
    let frame = match uart.read_exact(BM1366_RESULT_FRAME_LEN, uart::RESULT_WORK_TIMEOUT_MS) {
        Ok(frame) => frame,
        Err(error) => {
            fail_closed_work_result_read(&mut reset, &error);
            return Ok(());
        }
    };

    match parse_bm1366_result_frame(&frame, &valid_jobs, address_interval) {
        Ok(Bm1366ParsedResult::JobNonce(_result)) => {
            status::publish_work_result_parsed_status(job_id.raw());
        }
        Ok(Bm1366ParsedResult::RegisterRead(_read)) => {
            log::info!("asic_work_result_trace=register_read_parsed");
            status::publish_work_result_parsed_status(job_id.raw());
        }
        Err(error) => {
            fail_closed_work_result_invalid(&mut reset, &error);
        }
    }

    Ok(())
}

fn uart_trace_enabled() -> bool {
    option_env!("BITAXE_ASIC_UART_TRACE") == Some("1")
}

fn diagnostic_work_fields() -> Bm1366WorkFields {
    Bm1366WorkFields {
        starting_nonce: [1, 2, 3, 4],
        nbits: [5, 6, 7, 8],
        ntime: [9, 10, 11, 12],
        merkle_root: [17; 32],
        prev_block_hash: [34; 32],
        version: [51, 52, 53, 54],
    }
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
    uart.check_cancellation()?;
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
        } => count_asic_chips_rx_loop(uart, reset, *expected_chips, *timeout_ms),
        Bm1366AdapterAction::DelayMs(delay_ms) => {
            uart.cancellable_delay(*delay_ms)?;
            Ok(ActionOutcome::Continue)
        }
        Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => {
            reset.reset_pulse_cancellable(*low_ms, *high_ms, &mut || uart.check_cancellation())?;
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

/// Upstream-like `count_asic_chips` RX loop: drain until idle timeout, soft-retry
/// preamble/CRC/chip-id mismatch, hard-fail on wrong length / UART error, then
/// require counted == expected. Stores counted chips for mining-ready handoff.
fn count_asic_chips_rx_loop(
    uart: &mut uart::AsicUart<'_>,
    reset: &mut reset::AsicReset<'_>,
    expected_chips: u8,
    timeout_ms: u32,
) -> Result<ActionOutcome> {
    let idle_timeout_ms = if timeout_ms == 0 {
        COUNT_ASIC_CHIPS_IDLE_TIMEOUT_MS
    } else {
        timeout_ms
    };
    let mut counted_chips = 0_u8;
    let mut frames_seen = 0_u32;
    let started = std::time::Instant::now();

    if uart_trace_enabled() {
        log::info!(
            "asic_uart_trace=count_asic_chips_rx_loop start expected_chips={expected_chips} idle_timeout_ms={idle_timeout_ms}"
        );
    }

    loop {
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if chip_detect_drain_budget_exhausted(elapsed_ms, frames_seen) {
            log::warn!(
                "asic_status=fail_closed reason={CHIP_DETECT_RESPONSE_INVALID} category=drain_budget_exhausted frames_seen={frames_seen} elapsed_ms={elapsed_ms}"
            );
            best_effort_hold_reset_low(reset, CHIP_DETECT_RESPONSE_INVALID);
            status::publish_status(AsicInitStatus::FailClosed {
                reason: CHIP_DETECT_RESPONSE_INVALID,
            });
            return Ok(ActionOutcome::Stop);
        }
        let remaining_total_ms =
            u64::from(COUNT_ASIC_CHIPS_TOTAL_TIMEOUT_MS).saturating_sub(elapsed_ms);
        let read_timeout_ms =
            idle_timeout_ms.min(u32::try_from(remaining_total_ms).unwrap_or(u32::MAX).max(1));
        match uart.maybe_try_read_exact(BM1366_RESULT_FRAME_LEN, read_timeout_ms) {
            Ok(None) => {
                // Idle / received==0 — exit drain loop.
                if uart_trace_enabled() {
                    log::info!(
                        "asic_uart_trace=count_asic_chips_rx_loop idle counted={counted_chips}"
                    );
                }
                break;
            }
            Ok(Some(response)) => {
                frames_seen = frames_seen.saturating_add(1);
                match chip_detect::classify_chip_id_count_frame(&response) {
                    Ok(ChipIdCountFrameDisposition::Accept) => {
                        counted_chips = counted_chips.saturating_add(1);
                        if uart_trace_enabled() {
                            log::info!(
                                "asic_uart_trace=count_asic_chips_rx_loop accept counted={counted_chips}"
                            );
                        }
                    }
                    Ok(ChipIdCountFrameDisposition::SoftRetry) => {
                        if uart_trace_enabled() {
                            log::info!(
                                "asic_uart_trace=count_asic_chips_rx_loop soft_retry counted={counted_chips}"
                            );
                        }
                    }
                    Err(fault) => {
                        log::warn!(
                            "asic_status=fail_closed reason={CHIP_DETECT_RESPONSE_INVALID} error={fault}"
                        );
                        best_effort_hold_reset_low(reset, CHIP_DETECT_RESPONSE_INVALID);
                        status::publish_status(AsicInitStatus::FailClosed {
                            reason: CHIP_DETECT_RESPONSE_INVALID,
                        });
                        return Ok(ActionOutcome::Stop);
                    }
                }
            }
            Err(error) => {
                // Empty-buffer UART timeout is idle (upstream SERIAL_rx==0). Partial
                // frames / other UART errors remain fail-closed.
                let message = format!("{error:#}");
                if message.contains("ESP_ERR_TIMEOUT") || message.contains("timeout") {
                    if uart_trace_enabled() {
                        log::info!(
                            "asic_uart_trace=count_asic_chips_rx_loop idle_via_timeout counted={counted_chips}"
                        );
                    }
                    break;
                }
                log::warn!(
                    "asic_status=fail_closed reason={CHIP_DETECT_RESPONSE_INVALID} error={error:#}"
                );
                best_effort_hold_reset_low(reset, CHIP_DETECT_RESPONSE_INVALID);
                status::publish_status(AsicInitStatus::FailClosed {
                    reason: CHIP_DETECT_RESPONSE_INVALID,
                });
                return Ok(ActionOutcome::Stop);
            }
        }
    }

    match chip_detect::finalize_counted_chip_detect(counted_chips, expected_chips) {
        Ok(chips) => {
            status::publish_status(AsicInitStatus::ChipDetectedNoMining { chips });
            if uart_trace_enabled() {
                log::info!(
                    "asic_uart_trace=count_asic_chips_rx_loop complete chip_count_source=counted_rx chips={chips}"
                );
            }
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

fn fail_closed_work_result_setup_error(
    maybe_reset: Option<&mut reset::AsicReset<'_>>,
    error: &anyhow::Error,
) {
    if let Some(reset) = maybe_reset {
        best_effort_hold_reset_low(reset, "work_result_diagnostic_setup_error");
    }
    status::publish_work_result_invalid_status(format_args!("{error:#}"));
    status::publish_status(AsicInitStatus::FailClosed {
        reason: "work_result_diagnostic_setup_error",
    });
}

fn fail_closed_work_result_read(reset: &mut reset::AsicReset<'_>, error: &anyhow::Error) {
    best_effort_hold_reset_low(reset, "work_result_diagnostic_read_error");
    if error_is_timeout(error) {
        status::publish_work_result_timeout_status();
        status::publish_status(AsicInitStatus::FailClosed {
            reason: "work_result_diagnostic_timeout",
        });
        return;
    }

    status::publish_work_result_invalid_status(format_args!("{error:#}"));
    status::publish_status(AsicInitStatus::FailClosed {
        reason: "work_result_diagnostic_invalid",
    });
}

fn fail_closed_work_result_invalid(
    reset: &mut reset::AsicReset<'_>,
    error: impl std::fmt::Display,
) {
    best_effort_hold_reset_low(reset, "work_result_diagnostic_invalid");
    status::publish_work_result_invalid_status(error);
    status::publish_status(AsicInitStatus::FailClosed {
        reason: "work_result_diagnostic_invalid",
    });
}

fn error_is_timeout(error: &anyhow::Error) -> bool {
    let rendered = format!("{error:#}").to_ascii_lowercase();
    rendered.contains("timeout") || rendered.contains("timed out")
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
