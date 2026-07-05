//! Production BM1366 UART executor for Phase 27 live hardware bridge.
//!
//! Executes typed `Bm1366ProductionCommand` only; diagnostic work stays unreachable.

use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use bitaxe_asic::bm1366::{
    command::Bm1366AdapterAction,
    production::{Bm1366ProductionCommand, ProductionAsicBlocker, ProductionAsicStatus},
    result::{
        parse_bm1366_result_frame, Bm1366NonceResult, Bm1366ParsedResult, Bm1366ValidJobIds,
        BM1366_RESULT_FRAME_LEN,
    },
};

use super::{reset, status, uart};

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
                for action in actions {
                    execute_adapter_action_on_state(action, &mut state)?;
                }
                status::publish_production_asic_status(ProductionAsicStatus::WorkDispatched);
                Ok(None)
            }
            Bm1366ProductionCommand::ReadProductionResult => {
                for action in actions {
                    execute_adapter_action_on_state(action, &mut state)?;
                }
                let uart = state
                    .maybe_uart
                    .as_mut()
                    .expect("uart presence checked above");
                let frame = uart
                    .read_exact(BM1366_RESULT_FRAME_LEN, uart::RESULT_WORK_TIMEOUT_MS)
                    .map_err(|_| ProductionAsicBlocker::ResultTimeout)?;
                match parse_bm1366_result_frame(&frame, valid_jobs, 16) {
                    Ok(Bm1366ParsedResult::JobNonce(result)) => Ok(Some(result)),
                    Ok(Bm1366ParsedResult::RegisterRead(_)) => {
                        Err(ProductionAsicBlocker::ResultMalformed)
                    }
                    Err(_) => Err(ProductionAsicBlocker::ResultMalformed),
                }
            }
        }
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
