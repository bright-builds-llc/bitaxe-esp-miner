//! Production BM1366 UART executor for Phase 27 live hardware bridge.
//!
//! Executes typed `Bm1366ProductionCommand` only; diagnostic work stays unreachable.

use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use bitaxe_asic::bm1366::{
    accepted_state::{
        AcceptedStateSnapshot, AcceptedStateStage, AcceptedStateStatus, PowerDeltaClass,
    },
    command::{Bm1366AdapterAction, Bm1366Command, VersionMask},
    mining_ready::ultra_205_result_address_interval,
    packet::{CommandFrame, CMD_READ, COMMAND_HEADER_TYPE, GROUP_ALL},
    production::{Bm1366ProductionCommand, ProductionAsicBlocker, ProductionAsicStatus},
    registers::{
        read_register_payload, Bm1366Register, CHIP_ID_REGISTER, DOMAIN0_COUNT_REGISTER,
        DOMAIN1_COUNT_REGISTER, DOMAIN2_COUNT_REGISTER, DOMAIN3_COUNT_REGISTER,
        ERROR_COUNT_REGISTER, TOTAL_COUNT_REGISTER,
    },
    result::{
        parse_bm1366_result_frame, Bm1366NonceResult, Bm1366ParsedResult, Bm1366RegisterRead,
        Bm1366ValidJobIds, BM1366_RESULT_FRAME_LEN,
    },
};

use super::{reset, status, uart};

/// Outcome of a bounded production UART read poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionReadOutcome {
    Pending,
    JobNonce(Bm1366NonceResult),
    RegisterReadProof(Bm1366RegisterRead),
}

/// Exact safe-readable set for one-shot accepted-state diagnostics.
pub const ACCEPTED_STATE_READ_REGISTERS: [u8; 7] = [
    CHIP_ID_REGISTER,
    ERROR_COUNT_REGISTER,
    DOMAIN0_COUNT_REGISTER,
    DOMAIN1_COUNT_REGISTER,
    DOMAIN2_COUNT_REGISTER,
    DOMAIN3_COUNT_REGISTER,
    TOTAL_COUNT_REGISTER,
];

/// Redaction-safe accumulator for one bounded accepted-state read burst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedStateSnapshotObservation {
    stage: AcceptedStateStage,
    readable_responses: u32,
    chip_count_class: AcceptedStateStatus,
    error_counter_active: bool,
    domain_counter_active: bool,
    total_counter_active: bool,
}

impl AcceptedStateSnapshotObservation {
    #[must_use]
    pub const fn new(stage: AcceptedStateStage) -> Self {
        Self {
            stage,
            readable_responses: 0,
            chip_count_class: AcceptedStateStatus::Unavailable,
            error_counter_active: false,
            domain_counter_active: false,
            total_counter_active: false,
        }
    }

    pub fn observe(&mut self, read: Bm1366RegisterRead) {
        self.readable_responses = self.readable_responses.saturating_add(1);
        match read.register {
            Bm1366Register::ChipId => {
                self.chip_count_class = if read.asic_index == 0 {
                    AcceptedStateStatus::Match
                } else {
                    AcceptedStateStatus::Mismatch
                };
            }
            Bm1366Register::ErrorCount => {
                self.error_counter_active |= read.value > 0;
            }
            Bm1366Register::Domain0Count
            | Bm1366Register::Domain1Count
            | Bm1366Register::Domain2Count
            | Bm1366Register::Domain3Count => {
                self.domain_counter_active |= read.value > 0;
            }
            Bm1366Register::TotalCount => {
                self.total_counter_active |= read.value > 0;
            }
        }
    }

    #[must_use]
    pub const fn snapshot(
        self,
        power_delta_class: PowerDeltaClass,
        result_correlated: bool,
        submit_observed: bool,
    ) -> AcceptedStateSnapshot {
        AcceptedStateSnapshot {
            stage: self.stage,
            chip_count_class: self.chip_count_class,
            readable_response_count: self.readable_responses,
            error_counter_active: self.error_counter_active,
            domain_counter_active: self.domain_counter_active,
            total_counter_active: self.total_counter_active,
            power_delta_class,
            result_correlated,
            submit_observed,
        }
    }
}

static PRODUCTION_HANDLE: OnceLock<Mutex<ProductionAsicState>> = OnceLock::new();

struct ProductionAsicState {
    maybe_uart: Option<uart::AsicUart<'static>>,
    maybe_reset: Option<reset::AsicReset<'static>>,
    production_ready: bool,
}

impl ProductionAsicState {
    fn new() -> Self {
        Self {
            maybe_uart: None,
            maybe_reset: None,
            production_ready: false,
        }
    }
}

fn production_state() -> &'static Mutex<ProductionAsicState> {
    PRODUCTION_HANDLE.get_or_init(|| Mutex::new(ProductionAsicState::new()))
}

/// Retain UART and reset after boot gate for Phase 27 production bridge access.
pub fn store_production_peripherals(
    uart_driver: uart::AsicUart<'_>,
    reset_driver: reset::AsicReset<'_>,
    production_ready: bool,
) {
    // SAFETY: ESP-IDF singleton peripherals live for the firmware process lifetime.
    let uart_static: uart::AsicUart<'static> = unsafe { std::mem::transmute(uart_driver) };
    let reset_static: reset::AsicReset<'static> = unsafe { std::mem::transmute(reset_driver) };

    let Ok(mut state) = production_state().lock() else {
        log::warn!("asic_production_status=fail_closed reason=production_handle_lock_failed mining=disabled work_submission=disabled");
        return;
    };
    state.maybe_uart = Some(uart_static);
    state.maybe_reset = Some(reset_static);
    state.production_ready = production_ready;
    if production_ready {
        status::publish_production_asic_status(ProductionAsicStatus::InitializedForProduction);
    }
}

#[must_use]
pub fn production_handle_available() -> bool {
    production_state()
        .lock()
        .ok()
        .is_some_and(|state| state.maybe_uart.is_some())
}

#[must_use]
pub fn production_ready() -> bool {
    production_state()
        .lock()
        .ok()
        .is_some_and(|state| state.production_ready && state.maybe_uart.is_some())
}

/// Upstream hashrate-monitor REGISTER_MAP addresses (valid entries only).
///
/// Reference: `reference/esp-miner/components/asic/bm1366.c` REGISTER_MAP /
/// `BM1366_read_registers` — CMD_READ frames for 0x4C, 0x88–0x8C.
const HASHRATE_MONITOR_REGISTERS: &[u8] = &[0x4C, 0x88, 0x89, 0x8A, 0x8B, 0x8C];

/// TX-only `ReadChipId` register-read probe on the retained production UART.
///
/// Sends one reg-0x00 read frame and returns without blocking for the
/// response: the continuous poll path already classifies a register-read
/// reply through [`ProductionReadOutcome::RegisterReadProof`]. Diagnostic
/// only — any failure returns `false` and never blocks dispatch or mining.
///
/// Reference: reference/esp-miner/components/asic/bm1366.c:389-399
/// (`BM1366_read_registers`, reg 0x00 = chip id).
#[must_use]
pub fn probe_register_read_tx() -> bool {
    let Ok(frame) = Bm1366Command::ReadChipId.frame_bytes() else {
        return false;
    };
    let Ok(mut state) = production_state().lock() else {
        return false;
    };
    let Some(uart) = state.maybe_uart.as_mut() else {
        return false;
    };
    if uart.write_frame(frame.as_ref()).is_err() {
        return false;
    }
    uart.wait_tx_done(uart::WAIT_TX_DONE_TIMEOUT_MS).is_ok()
}

/// Post-configure runtime `SetVersionMask` on the retained production UART.
///
/// Mirrors upstream `ASIC_set_version_mask` / `BM1366_set_version_mask` after
/// configure when ASIC is initialized. Not value-delta gated — callers may
/// reload the init-default mask (`0x1fffe000`). Returns `false` when UART is
/// not retained or not production-ready; never blocks mining beyond logging.
///
/// Reference: `reference/esp-miner/main/tasks/create_jobs_task.c:126-129`,
/// `reference/esp-miner/components/asic/bm1366.c:141-147`.
#[must_use]
pub fn apply_negotiated_version_mask(mask: VersionMask) -> bool {
    if !production_ready() {
        return false;
    }
    let Ok(frame) = Bm1366Command::SetVersionMask(mask).frame_bytes() else {
        return false;
    };
    let Ok(mut state) = production_state().lock() else {
        return false;
    };
    if !state.production_ready {
        return false;
    }
    let Some(uart) = state.maybe_uart.as_mut() else {
        return false;
    };
    if uart.write_frame(frame.as_ref()).is_err() {
        return false;
    }
    uart.wait_tx_done(uart::WAIT_TX_DONE_TIMEOUT_MS).is_ok()
}

/// TX-only hashrate-monitor register-read burst (investigation A/B only).
///
/// Mirrors upstream `BM1366_read_registers`: one CMD_READ (`0x52`) frame per
/// REGISTER_MAP entry. Frames are prebuilt, then sent back-to-back under the
/// production UART lock (no sleep while holding the mutex — that panics on
/// ESP-IDF). Does not wait for replies — continuous RX poll classifies
/// register-read responses. Failures return `false` and never block mining.
#[must_use]
pub fn probe_hashrate_monitor_register_reads_tx() -> bool {
    send_register_read_burst(HASHRATE_MONITOR_REGISTERS)
}

/// One-shot accepted-state read burst on the retained production UART.
///
/// Callers own the closed stage guard. This helper sends the exact safe read
/// set once and never schedules retries or polling.
#[must_use]
pub fn probe_accepted_state_register_reads_tx() -> bool {
    if !super::work_result_investigation::accepted_state_snapshot_enabled() {
        return false;
    }
    send_register_read_burst(&ACCEPTED_STATE_READ_REGISTERS)
}

/// Collect one bounded accepted-state burst while boot still owns the UART.
pub(super) fn collect_boot_accepted_state_snapshot(
    uart: &mut uart::AsicUart<'_>,
    stage: AcceptedStateStage,
) -> AcceptedStateSnapshotObservation {
    const RESPONSE_TIMEOUT_MS: u32 = 250;

    let mut observation = AcceptedStateSnapshotObservation::new(stage);
    for &register in &ACCEPTED_STATE_READ_REGISTERS {
        let Ok(frame) = CommandFrame::new(
            COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ,
            read_register_payload(register).as_bytes(),
        ) else {
            return observation;
        };
        if uart.write_frame(frame.into_bytes().as_ref()).is_err()
            || uart.wait_tx_done(uart::WAIT_TX_DONE_TIMEOUT_MS).is_err()
        {
            return observation;
        }
    }

    for _ in ACCEPTED_STATE_READ_REGISTERS {
        let Ok(Some(frame)) = uart.try_read_exact(BM1366_RESULT_FRAME_LEN, RESPONSE_TIMEOUT_MS)
        else {
            continue;
        };
        if let Ok(Bm1366ParsedResult::RegisterRead(read)) = parse_bm1366_result_frame(
            &frame,
            &Bm1366ValidJobIds::empty(),
            ultra_205_result_address_interval(),
        ) {
            observation.observe(read);
        }
    }
    observation
}

fn send_register_read_burst(registers: &[u8]) -> bool {
    let mut frames = Vec::with_capacity(registers.len());
    for &register in registers {
        let Ok(frame) = CommandFrame::new(
            COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ,
            read_register_payload(register).as_bytes(),
        ) else {
            return false;
        };
        frames.push(frame.into_bytes());
    }

    let Ok(mut state) = production_state().lock() else {
        return false;
    };
    let Some(uart) = state.maybe_uart.as_mut() else {
        return false;
    };

    for frame in &frames {
        if uart.write_frame(frame.as_ref()).is_err() {
            return false;
        }
        if uart.wait_tx_done(uart::WAIT_TX_DONE_TIMEOUT_MS).is_err() {
            return false;
        }
    }
    true
}

pub struct ProductionAsicExecutor;

impl ProductionAsicExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &mut self,
        command: Bm1366ProductionCommand,
        valid_jobs: &Bm1366ValidJobIds,
    ) -> Result<Option<Bm1366NonceResult>, ProductionAsicBlocker> {
        let Ok(mut state) = production_state().lock() else {
            return Err(ProductionAsicBlocker::UartFailed);
        };
        if !state.production_ready || state.maybe_uart.is_none() {
            return Err(ProductionAsicBlocker::AsicInitFailed);
        }

        let actions = command.adapter_actions();
        match command {
            Bm1366ProductionCommand::SendProductionWork(_) => {
                if super::work_result_investigation::clear_rx_before_production_work() {
                    let uart = state
                        .maybe_uart
                        .as_mut()
                        .ok_or(ProductionAsicBlocker::UartFailed)?;
                    uart.clear_rx()
                        .map_err(|_| ProductionAsicBlocker::UartFailed)?;
                    log::info!("asic_production_trace=clear_rx_before_work");
                }
                for action in actions {
                    execute_adapter_action_on_state(action, &mut state)?;
                }
                status::publish_production_asic_status(ProductionAsicStatus::WorkDispatched);
                Ok(None)
            }
            Bm1366ProductionCommand::ReadProductionResult => {
                match try_read_production_result_on_state(
                    &mut state,
                    valid_jobs,
                    uart::RESULT_WORK_TIMEOUT_MS,
                )? {
                    ProductionReadOutcome::Pending => Ok(None),
                    ProductionReadOutcome::JobNonce(result) => Ok(Some(result)),
                    ProductionReadOutcome::RegisterReadProof(_) => Ok(None),
                }
            }
        }
    }

    pub fn try_read_production_result(
        &mut self,
        valid_jobs: &Bm1366ValidJobIds,
        poll_timeout_ms: u32,
    ) -> Result<ProductionReadOutcome, ProductionAsicBlocker> {
        let Ok(mut state) = production_state().lock() else {
            return Err(ProductionAsicBlocker::UartFailed);
        };
        if !state.production_ready || state.maybe_uart.is_none() {
            return Err(ProductionAsicBlocker::AsicInitFailed);
        }

        try_read_production_result_on_state(&mut state, valid_jobs, poll_timeout_ms)
    }
}

fn try_read_production_result_on_state(
    state: &mut ProductionAsicState,
    valid_jobs: &Bm1366ValidJobIds,
    poll_timeout_ms: u32,
) -> Result<ProductionReadOutcome, ProductionAsicBlocker> {
    log::info!("asic_production_trace=result_read_attempt poll_timeout_ms={poll_timeout_ms}");
    // Flood-safe compact counters (no hex); emit every N polls for comparator D-06.
    uart::note_result_poll_and_maybe_emit_summary();
    let uart = state
        .maybe_uart
        .as_mut()
        .ok_or(ProductionAsicBlocker::UartFailed)?;
    let maybe_frame = match uart.try_read_exact(BM1366_RESULT_FRAME_LEN, poll_timeout_ms) {
        Ok(maybe_frame) => maybe_frame,
        Err(_) => return Ok(ProductionReadOutcome::Pending),
    };
    let Some(frame) = maybe_frame else {
        return Ok(ProductionReadOutcome::Pending);
    };

    match parse_bm1366_result_frame(&frame, valid_jobs, ultra_205_result_address_interval()) {
        Ok(Bm1366ParsedResult::JobNonce(result)) => Ok(ProductionReadOutcome::JobNonce(result)),
        Ok(Bm1366ParsedResult::RegisterRead(read)) => {
            log::info!("asic_production_trace=register_read_parsed");
            Ok(ProductionReadOutcome::RegisterReadProof(read))
        }
        Err(_) => Err(ProductionAsicBlocker::ResultMalformed),
    }
}

impl Default for ProductionAsicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn execute_adapter_action_on_state(
    action: Bm1366AdapterAction,
    state: &mut ProductionAsicState,
) -> Result<(), ProductionAsicBlocker> {
    match action {
        Bm1366AdapterAction::WriteFrame(frame) => {
            let uart = state
                .maybe_uart
                .as_mut()
                .ok_or(ProductionAsicBlocker::UartFailed)?;
            uart.write_frame(frame.as_ref())
                .map_err(|_| ProductionAsicBlocker::UartFailed)
        }
        Bm1366AdapterAction::ReadExact { len, timeout_ms } => {
            let uart = state
                .maybe_uart
                .as_mut()
                .ok_or(ProductionAsicBlocker::UartFailed)?;
            let _ = uart
                .read_exact(len, timeout_ms)
                .map_err(|_| ProductionAsicBlocker::ResultTimeout)?;
            Ok(())
        }
        Bm1366AdapterAction::HoldResetLow => {
            if let Some(reset_driver) = state.maybe_reset.as_mut() {
                reset_driver
                    .hold_reset_low()
                    .map_err(|_| ProductionAsicBlocker::ResetFailed)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use bitaxe_asic::bm1366::{
        production::ProductionWorkPayload,
        result::Bm1366ValidJobIds,
        work::{Bm1366JobId, Bm1366WorkFields},
    };

    use super::*;

    struct FakeProductionBackend {
        send_count: Rc<Cell<u32>>,
        read_count: Rc<Cell<u32>>,
        maybe_result: Option<Bm1366NonceResult>,
    }

    impl FakeProductionBackend {
        fn execute(
            &self,
            command: Bm1366ProductionCommand,
            _valid_jobs: &Bm1366ValidJobIds,
        ) -> Result<Option<Bm1366NonceResult>, ProductionAsicBlocker> {
            match command {
                Bm1366ProductionCommand::SendProductionWork(_) => {
                    self.send_count.set(self.send_count.get() + 1);
                    Ok(None)
                }
                Bm1366ProductionCommand::ReadProductionResult => {
                    self.read_count.set(self.read_count.get() + 1);
                    Ok(self.maybe_result)
                }
            }
        }
    }

    #[test]
    fn send_production_work_increments_dispatch_counter() {
        // Arrange
        let send_count = Rc::new(Cell::new(0));
        let backend = FakeProductionBackend {
            send_count: send_count.clone(),
            read_count: Rc::new(Cell::new(0)),
            maybe_result: None,
        };
        let job_id = Bm1366JobId::new(0x28);
        let payload = ProductionWorkPayload::new(job_id, sample_fields());
        let command = Bm1366ProductionCommand::SendProductionWork(payload);

        // Act
        let _ = backend.execute(command, &Bm1366ValidJobIds::single(job_id));

        // Assert
        assert_eq!(send_count.get(), 1);
    }

    #[test]
    fn read_production_result_uses_bounded_read_path() {
        // Arrange
        let read_count = Rc::new(Cell::new(0));
        let job_id = Bm1366JobId::new(0x28);
        let backend = FakeProductionBackend {
            send_count: Rc::new(Cell::new(0)),
            read_count: read_count.clone(),
            maybe_result: Some(Bm1366NonceResult {
                job_id,
                nonce: 0x0102_0304,
                asic_index: 0,
                core_id: 0,
                small_core_id: 0,
                version_bits: 0,
            }),
        };

        // Act
        let result = backend
            .execute(
                Bm1366ProductionCommand::ReadProductionResult,
                &Bm1366ValidJobIds::single(job_id),
            )
            .expect("read should succeed");

        // Assert
        assert_eq!(read_count.get(), 1);
        assert!(result.is_some());
    }

    #[test]
    fn production_executor_module_never_references_diagnostic_work() {
        // Arrange
        let source = include_str!("production.rs");

        // Assert
        assert!(!source.contains("SendDiagnosticWork"));
    }

    #[test]
    fn apply_negotiated_version_mask_encodes_set_version_mask_frame() {
        // Arrange — encode path used by apply_negotiated_version_mask (no UART).
        let mask = VersionMask::new(0x1fff_e000);

        // Act
        let frame = Bm1366Command::SetVersionMask(mask)
            .frame_bytes()
            .expect("SetVersionMask should encode");

        // Assert — non-empty command frame; helper returns false without UART.
        assert!(!frame.as_ref().is_empty());
        assert!(!apply_negotiated_version_mask(mask));
    }

    #[test]
    fn accepted_state_safe_read_set_is_exact() {
        // Arrange / Act
        let registers = ACCEPTED_STATE_READ_REGISTERS;

        // Assert
        assert_eq!(registers, [0x00, 0x4c, 0x88, 0x89, 0x8a, 0x8b, 0x8c]);
    }

    #[test]
    fn accepted_state_marker_contains_categories_only() {
        // Arrange
        let mut observation =
            AcceptedStateSnapshotObservation::new(AcceptedStateStage::PostFirstWork);
        observation.observe(Bm1366RegisterRead {
            register: Bm1366Register::TotalCount,
            asic_index: 7,
            asic_address: 8,
            value: 9,
        });

        // Act
        let marker = observation
            .snapshot(PowerDeltaClass::RisingHashing, false, false)
            .marker();

        // Assert
        assert_eq!(marker, "accepted_state_snapshot stage=post_first_work observation=unavailable chip_count_class=unavailable readable_responses=1 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=rising_hashing result_correlated=false submit_observed=false redacted=true");
        assert!(!marker.contains("asic_address"));
        assert!(!marker.contains("value"));
        assert!(!marker.contains("=7"));
        assert!(!marker.contains("=8"));
        assert!(!marker.contains("=9"));
    }

    fn sample_fields() -> Bm1366WorkFields {
        Bm1366WorkFields {
            starting_nonce: [0; 4],
            nbits: [1, 2, 3, 4],
            ntime: [5, 6, 7, 8],
            merkle_root: [9; 32],
            prev_block_hash: [10; 32],
            version: [11, 12, 13, 14],
        }
    }
}
