//! Exclusive SELF-001 access to the retained BM1366 transport.

use anyhow::Result;
use bitaxe_asic::bm1366::{
    command::Bm1366Command, production::ProductionAsicBlocker, result::Bm1366ValidJobIds,
};

use super::{
    execute_adapter_action_on_state, production_state, try_read_production_result_on_state,
    ProductionReadOutcome,
};

/// Executes one typed diagnostic command while the normal production owner is
/// absent. SELF-001 is the only production-firmware caller.
pub fn execute_self_test_command(command: Bm1366Command) -> Result<(), ProductionAsicBlocker> {
    let Ok(mut state) = production_state().lock() else {
        return Err(ProductionAsicBlocker::UartFailed);
    };
    if !state.production_ready || state.maybe_uart.is_none() {
        return Err(ProductionAsicBlocker::AsicInitFailed);
    }
    for action in command
        .adapter_actions()
        .map_err(|_| ProductionAsicBlocker::ResultMalformed)?
    {
        execute_adapter_action_on_state(action, &mut state)?;
    }
    Ok(())
}

/// Reads one bounded SELF-001 result from the retained UART.
pub fn try_read_self_test_result(
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
