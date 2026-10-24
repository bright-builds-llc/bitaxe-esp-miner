//! The Gamma 601's retained ASIC peripherals (ADR-0041): a UART and a reset, no enable line.

use super::{production_state, reset, uart};
use std::sync::atomic::{AtomicBool, Ordering};

/// Retains the Gamma 601's UART and reset: it has no ASIC enable line (ADR-0041).
pub fn store_board_601_peripherals(
    uart_driver: uart::AsicUart<'_>,
    reset_driver: reset::AsicReset<'_>,
) {
    // SAFETY: ESP-IDF singleton peripherals live for the firmware process lifetime.
    let uart_static: uart::AsicUart<'static> = unsafe { std::mem::transmute(uart_driver) };
    let reset_static: reset::AsicReset<'static> = unsafe { std::mem::transmute(reset_driver) };
    let Ok(mut state) = production_state().lock() else {
        log::warn!("asic_production_status=fail_closed reason=production_handle_lock_failed mining=disabled work_submission=disabled");
        // Dropping the reset driver would disable the pin and let the line float; keep it
        // driven low for the rest of the boot.
        std::mem::forget(reset_static);
        return;
    };
    state.maybe_uart = Some(uart_static);
    state.maybe_reset = Some(reset_static);
    state.production_ready = false;
    RESET_RETAINED.store(true, Ordering::Release);
}

/// Set just before the boot gate takes the reset pin; from then on the pin cannot be read back,
/// so only the executor's reset owner can say the reset is held.
static RESET_HANDOVER_STARTED: AtomicBool = AtomicBool::new(false);
/// Set once the executor owns the reset pin.
static RESET_RETAINED: AtomicBool = AtomicBool::new(false);

/// Marks the start of the reset pin's handover to the executor.
pub fn begin_reset_handover() {
    RESET_HANDOVER_STARTED.store(true, Ordering::Release);
}

/// Whether the executor's reset owner holds the ASIC in reset; `None` before the handover
/// starts. After it starts, anything but a retained owner confirming the hold reads as not
/// held: a failed handover, an owner not yet stored, a poisoned or busy lock.
pub fn executor_holds_reset() -> Option<bool> {
    if !RESET_HANDOVER_STARTED.load(Ordering::Acquire) {
        return None;
    }
    if !RESET_RETAINED.load(Ordering::Acquire) {
        return Some(false);
    }
    Some(production_state().try_lock().ok().is_some_and(|state| {
        state
            .maybe_reset
            .as_ref()
            .is_some_and(reset::AsicReset::holds_reset_low)
    }))
}
