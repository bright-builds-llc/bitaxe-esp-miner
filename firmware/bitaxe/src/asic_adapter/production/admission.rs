//! Generation checks at retained ASIC admission and control boundaries.

use super::*;

/// Stops new production UART work before any safe-shutdown command is sent.
pub fn block_production_dispatch() -> Result<(), ProductionAsicBlocker> {
    crate::production_mining_session::revocation::block_work();
    let Ok(mut state) = production_state().lock() else {
        return Err(ProductionAsicBlocker::UartFailed);
    };
    state.production_ready = false;
    Ok(())
}

pub fn set_production_work_permit(
    permit: crate::production_mining_session::revocation::WorkPermit,
) -> bool {
    let Ok(mut state) = production_state().lock() else {
        return false;
    };
    if !state.production_ready
        || !crate::production_mining_session::revocation::permits_work(permit)
    {
        return false;
    }
    let Some(uart) = state.maybe_uart.as_mut() else {
        return false;
    };
    uart.set_work_permit(permit);
    true
}

/// Applies the active-low Ultra 205 ASIC-enable line through its retained owner.
pub fn set_asic_power_enabled(enabled: bool) -> Result<(), ProductionAsicBlocker> {
    set_asic_power_enabled_guarded(enabled, None)
}

pub fn set_asic_power_enabled_guarded(
    enabled: bool,
    maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
) -> Result<(), ProductionAsicBlocker> {
    let Ok(mut state) = production_state().lock() else {
        return Err(ProductionAsicBlocker::UartFailed);
    };
    if enabled && !crate::production_mining_session::revocation::permits(maybe_generation) {
        return Err(ProductionAsicBlocker::AsicInitFailed);
    }
    if !enabled {
        state.production_ready = false;
    }
    let enable = state
        .maybe_enable
        .as_mut()
        .ok_or(ProductionAsicBlocker::AsicInitFailed)?;
    if enabled {
        enable
            .enable()
            .map_err(|_| ProductionAsicBlocker::AsicInitFailed)
    } else {
        enable
            .disable()
            .map_err(|_| ProductionAsicBlocker::AsicInitFailed)
    }
}

/// Executes reset and exact chip-count proof without admitting production work.
pub fn execute_chip_detection_actions(
    actions: &[Bm1366AdapterAction],
) -> Result<(), ProductionAsicBlocker> {
    execute_chip_detection_actions_guarded(actions, None)
}

pub fn execute_chip_detection_actions_guarded(
    actions: &[Bm1366AdapterAction],
    maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
) -> Result<(), ProductionAsicBlocker> {
    let Ok(mut state) = production_state().lock() else {
        return Err(ProductionAsicBlocker::UartFailed);
    };
    state.production_ready = false;
    if !crate::production_mining_session::revocation::permits(maybe_generation) {
        return Err(ProductionAsicBlocker::AsicInitFailed);
    }
    if let Some(uart) = state.maybe_uart.as_mut() {
        uart.set_worker_generation(maybe_generation);
    }
    execute_adapter_actions_on_state(actions, &mut state)
}

/// Executes mining-ready initialization and admits UART work only after every
/// typed action succeeds.
pub fn execute_mining_ready_actions(
    actions: &[Bm1366AdapterAction],
) -> Result<(), ProductionAsicBlocker> {
    execute_mining_ready_actions_guarded(actions, None)
}

pub fn execute_mining_ready_actions_guarded(
    actions: &[Bm1366AdapterAction],
    maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
) -> Result<(), ProductionAsicBlocker> {
    let Ok(mut state) = production_state().lock() else {
        return Err(ProductionAsicBlocker::UartFailed);
    };
    state.production_ready = false;
    if !crate::production_mining_session::revocation::permits(maybe_generation) {
        return Err(ProductionAsicBlocker::AsicInitFailed);
    }
    if let Some(uart) = state.maybe_uart.as_mut() {
        uart.set_worker_generation(maybe_generation);
    }
    execute_adapter_actions_on_state(actions, &mut state)?;
    if !crate::production_mining_session::revocation::permits(maybe_generation) {
        return Err(ProductionAsicBlocker::AsicInitFailed);
    }
    state.production_ready = true;
    status::publish_production_asic_status(ProductionAsicStatus::InitializedForProduction);
    Ok(())
}
