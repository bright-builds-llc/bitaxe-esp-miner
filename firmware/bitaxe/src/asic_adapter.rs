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

mod chip_detect_investigation;
mod production;
mod reset;
mod status;
mod uart;
pub use uart::RESULT_WORK_TIMEOUT_MS;
mod work_result_investigation;

pub use production::{production_ready, ProductionAsicExecutor, ProductionReadOutcome};
pub use work_result_investigation::{
    continuous_result_task_enabled, job_redispatch_pump_enabled, JOB_REDISPATCH_INTERVAL_MS,
};

pub use status::{
    publish_mining_loop_blocked_status, publish_production_asic_blocked_status,
    publish_production_asic_status,
};

pub struct AsicBootPeripherals<UART, RESET, TX, RX> {
    pub uart: UART,
    pub reset: RESET,
    pub tx: TX,
    pub rx: RX,
}

pub struct Phase27SafetyPeripherals<I2C, SDA, SCL, ENABLE> {
    pub i2c: I2C,
    pub sda: SDA,
    pub scl: SCL,
    pub enable: ENABLE,
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
    match adapter_mode_from_firmware_compile_env() {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(())
        }
        AsicAdapterMode::ChipDetectOnly => run_chip_detect_only(peripherals),
        AsicAdapterMode::WorkResultDiagnostic => run_work_result_uart_bootstrap(peripherals, false),
        AsicAdapterMode::Phase27ProductionBridge => {
            run_work_result_uart_bootstrap(peripherals, true)
        }
    }
}

pub fn run_phase27_boot_gate_with_safety<UART, RESET, TX, RX, I2C, SDA, SCL, ENABLE>(
    peripherals: AsicBootPeripherals<UART, RESET, TX, RX>,
    safety: Phase27SafetyPeripherals<I2C, SDA, SCL, ENABLE>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
    I2C: esp_idf_svc::hal::i2c::I2c + 'static,
    SDA: InputPin + OutputPin + 'static,
    SCL: InputPin + OutputPin + 'static,
    ENABLE: OutputPin + 'static,
{
    match adapter_mode_from_firmware_compile_env() {
        AsicAdapterMode::Phase27ProductionBridge => {
            run_work_result_uart_bootstrap_with_phase27_safety(peripherals, safety)
        }
        _ => run_boot_gate_with_peripherals(peripherals),
    }
}

fn run_work_result_uart_bootstrap_with_phase27_safety<UART, RESET, TX, RX, I2C, SDA, SCL, ENABLE>(
    peripherals: AsicBootPeripherals<UART, RESET, TX, RX>,
    safety: Phase27SafetyPeripherals<I2C, SDA, SCL, ENABLE>,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
    I2C: esp_idf_svc::hal::i2c::I2c + 'static,
    SDA: InputPin + OutputPin + 'static,
    SCL: InputPin + OutputPin + 'static,
    ENABLE: OutputPin + 'static,
{
    let mut reset = match reset::AsicReset::new(peripherals.reset)
        .context("initialize ASIC reset GPIO adapter")
    {
        Ok(reset) => reset,
        Err(error) => {
            fail_closed_work_result_setup_error(None, &error);
            return Ok(());
        }
    };

    if let Err(error) = crate::safety_adapter::run_phase27_hardware_bring_up(
        safety.i2c,
        safety.sda,
        safety.scl,
        safety.enable,
        &mut reset,
    ) {
        log::warn!("phase27_safety_bring_up=failed error={error:#}");
    }

    run_work_result_uart_bootstrap_after_reset(
        peripherals.uart,
        peripherals.tx,
        peripherals.rx,
        reset,
        true,
    )
}

fn run_work_result_uart_bootstrap<UART, RESET, TX, RX>(
    peripherals: AsicBootPeripherals<UART, RESET, TX, RX>,
    retain_for_production: bool,
) -> Result<()>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    let mut reset = match reset::AsicReset::new(peripherals.reset)
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
        retain_for_production,
    )
}

pub fn run_boot_gate_without_peripherals(reason: &'static str) -> Result<()> {
    match adapter_mode_from_firmware_compile_env() {
        AsicAdapterMode::FailClosed => {
            status::publish_default_fail_closed_status();
            Ok(())
        }
        AsicAdapterMode::ChipDetectOnly => {
            log::warn!("asic_status=fail_closed reason={reason}");
            status::publish_status(AsicInitStatus::FailClosed { reason });
            Ok(())
        }
        AsicAdapterMode::WorkResultDiagnostic | AsicAdapterMode::Phase27ProductionBridge => {
            log::warn!("asic_status=fail_closed reason={reason}");
            status::publish_status(AsicInitStatus::FailClosed { reason });
            Ok(())
        }
    }
}

fn adapter_mode_from_firmware_compile_env() -> AsicAdapterMode {
    AsicAdapterMode::from_compile_env(
        option_env!("BITAXE_ASIC_DIAGNOSTIC"),
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK"),
        option_env!("BITAXE_MINING_EVIDENCE_MODE"),
    )
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

fn run_work_result_uart_bootstrap_after_reset<UART, TX, RX>(
    uart_peripheral: UART,
    tx: TX,
    rx: RX,
    mut reset: reset::AsicReset<'_>,
    retain_for_production: bool,
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

    if retain_for_production && !run_chip_detect_actions(&mut uart, &mut reset) {
        return Ok(());
    }

    let mining_ready_completed =
        !retain_for_production || run_mining_ready_init_actions(&mut uart, &mut reset);
    if retain_for_production && !mining_ready_completed {
        return Ok(());
    }

    // Reference: reference/esp-miner/main/power/asic_init.c:58-61 — upstream sets
    // ASIC_initalized = true unconditionally after init; no boot diagnostic nonce
    // gate exists upstream (W13). Retention after chip detect + mining-ready init
    // is the bridge-mode default; the levers below restore the old boot gate.
    if retain_for_production
        && !work_result_investigation::require_diagnostic_nonce()
        && !work_result_investigation::require_uart_proof_for_production()
    {
        log::info!(
            "asic_work_result_trace=skip_boot_diagnostic_work bootstrap=initialized_no_mining"
        );
        status::publish_work_result_bootstrap_initialized_status();
        production::store_production_peripherals(uart, reset, true);
        return Ok(());
    }

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
            if retain_for_production
                && work_result_investigation::phase27_initialized_no_mining_bootstrap(
                    mining_ready_completed,
                )
                && error_is_timeout(&error)
            {
                log::info!("asic_work_result_trace=initialized_no_mining_bootstrap timeout");
                status::publish_work_result_bootstrap_initialized_status();
                production::store_production_peripherals(uart, reset, true);
                return Ok(());
            }
            fail_closed_work_result_read(&mut reset, &error);
            return Ok(());
        }
    };

    match parse_bm1366_result_frame(&frame, &valid_jobs, address_interval) {
        Ok(Bm1366ParsedResult::JobNonce(_result)) => {
            status::publish_work_result_parsed_status(job_id.raw());
            if retain_for_production {
                production::store_production_peripherals(uart, reset, true);
            }
        }
        Ok(Bm1366ParsedResult::RegisterRead(_read)) => {
            log::info!("asic_work_result_trace=register_read_parsed");
            status::publish_work_result_parsed_status(job_id.raw());
            if retain_for_production {
                production::store_production_peripherals(uart, reset, true);
            }
        }
        Err(error) => {
            if retain_for_production
                && work_result_investigation::phase27_initialized_no_mining_bootstrap(
                    mining_ready_completed,
                )
            {
                log::info!("asic_work_result_trace=initialized_no_mining_bootstrap parse_error");
                status::publish_work_result_bootstrap_initialized_status();
                production::store_production_peripherals(uart, reset, true);
                return Ok(());
            }
            fail_closed_work_result_invalid(&mut reset, &error);
        }
    }

    Ok(())
}

fn run_chip_detect_actions(
    uart: &mut uart::AsicUart<'_>,
    reset: &mut reset::AsicReset<'_>,
) -> bool {
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );
    let decision = chip_detect_investigation::chip_detect_init_decision(preflight);

    for action in decision.actions() {
        match interpret_action(action, uart, reset) {
            Ok(ActionOutcome::Continue) => {}
            Ok(ActionOutcome::Stop) => return false,
            Err(error) => {
                fail_closed_adapter_error(reset, &error);
                return false;
            }
        }
    }

    true
}

fn run_mining_ready_init_actions(
    uart: &mut uart::AsicUart<'_>,
    reset: &mut reset::AsicReset<'_>,
) -> bool {
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );
    let chip_count = preflight.expected_chips();
    let Some(decision) =
        work_result_investigation::mining_ready_init_decision(preflight, chip_count)
    else {
        return true;
    };

    if uart_trace_enabled() {
        log::info!(
            "asic_work_result_trace=mining_ready_init_started chip_count={chip_count} actions={}",
            decision.actions().len()
        );
    }

    for action in decision.actions() {
        trace_init_action(action);
        match interpret_action(action, uart, reset) {
            Ok(ActionOutcome::Continue) => {}
            Ok(ActionOutcome::Stop) => return false,
            Err(error) => {
                fail_closed_adapter_error(reset, &error);
                return false;
            }
        }
    }

    if uart_trace_enabled() {
        log::info!("asic_work_result_trace=mining_ready_init_complete");
    }

    true
}

fn uart_trace_enabled() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
        || option_env!("BITAXE_ASIC_UART_TRACE") == Some("1")
}

fn trace_init_action(action: &Bm1366AdapterAction) {
    if !uart_trace_enabled() {
        return;
    }

    match action {
        Bm1366AdapterAction::WriteFrame(frame) => {
            log::info!(
                "asic_work_result_trace=init_action kind=write_frame len={}",
                frame.as_ref().len()
            );
        }
        Bm1366AdapterAction::UseMaxBaud { baud } => {
            log::info!("asic_work_result_trace=init_action kind=use_max_baud baud={baud}");
        }
        Bm1366AdapterAction::UseDefaultBaud { baud } => {
            log::info!("asic_work_result_trace=init_action kind=use_default_baud baud={baud}");
        }
        Bm1366AdapterAction::ClearRx => {
            log::info!("asic_work_result_trace=init_action kind=clear_rx");
        }
        Bm1366AdapterAction::PublishStatus(status) => {
            log::info!("asic_work_result_trace=init_action kind=publish_status status={status:?}");
        }
        Bm1366AdapterAction::WaitTxDone { timeout_ms } => {
            log::info!(
                "asic_work_result_trace=init_action kind=wait_tx_done timeout_ms={timeout_ms}"
            );
        }
        _ => {}
    }
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
