//! The Gamma 601's ASIC boot gate (ADR-0041).

use super::{production, reset, status, uart, BootMiningBaseline};
use anyhow::Result;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::uart::Uart;

/// The Gamma 601's boot gate: the ASIC reset and UART retained for the executor, with the reset
/// held low; the 601 has no enable line. Its baseline is confirmed only with both retained; the
/// caller also requires the TPS546 output confirmed off before it lets the Worker start
/// (`gamma601::initialize_hardware`, ADR-0041 item 10).
pub fn retain_board_601_peripherals<UART, RESET, TX, RX>(
    uart: UART,
    reset_pin: RESET,
    tx: TX,
    rx: RX,
) -> Result<BootMiningBaseline>
where
    UART: Uart + 'static,
    RESET: OutputPin + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    status::publish_default_fail_closed_status();
    production::begin_reset_handover();
    let mut reset = match reset::AsicReset::new(reset_pin) {
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
        // Dropping the driver would disable the pin; keep whatever drive it has.
        std::mem::forget(reset);
        return Ok(BootMiningBaseline::Unconfirmed);
    }
    match uart::AsicUart::new(uart, tx, rx) {
        Ok(uart) => {
            production::store_board_601_peripherals(uart, reset);
            // A poisoned executor lock stores nothing; only a retained owner holding the reset
            // confirms.
            if production::executor_holds_reset() == Some(true) {
                Ok(BootMiningBaseline::Confirmed)
            } else {
                log::warn!("asic_production_status=fail_closed reason=reset_not_retained");
                Ok(BootMiningBaseline::Unconfirmed)
            }
        }
        Err(error) => {
            log::warn!(
                "asic_production_status=fail_closed reason=uart_unavailable error={error:#}"
            );
            // Without a UART the executor never runs; the reset stays driven low.
            std::mem::forget(reset);
            Ok(BootMiningBaseline::Unconfirmed)
        }
    }
}
