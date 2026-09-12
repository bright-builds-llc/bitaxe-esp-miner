//! Sole fixed USB Serial/JTAG driver; no descriptor, PHY, or reset ownership changes.

use esp_idf_sys as sys;
#[path = "usb_write_failure.rs"]
mod failure;
use failure::WriteStage;
pub(crate) use failure::{WriteFailure, WriteObservation, WriteObservationStage};
use std::sync::atomic::{AtomicBool, Ordering};
static PARTIAL_OUTPUT: AtomicBool = AtomicBool::new(false);

/// Only an unfinished record needs bootstrap resynchronization; queued complete
/// records retain their delimiter even if cancellation interrupts drain polling.
pub(crate) fn has_partial_output() -> bool {
    PARTIAL_OUTPUT.load(Ordering::Acquire)
}

pub(crate) fn install() -> anyhow::Result<()> {
    let mut config = sys::usb_serial_jtag_driver_config_t {
        tx_buffer_size: 2048,
        rx_buffer_size: 4096,
    };
    let result = unsafe { sys::usb_serial_jtag_driver_install(&mut config) };
    anyhow::ensure!(result == sys::ESP_OK, "serial_driver_install:{result}");
    Ok(())
}

pub(crate) fn read(bytes: &mut [u8]) -> anyhow::Result<usize> {
    let count = unsafe {
        sys::usb_serial_jtag_read_bytes(bytes.as_mut_ptr().cast(), bytes.len() as u32, 1)
    };
    anyhow::ensure!(count >= 0, "serial_read_failed");
    Ok(count as usize)
}

/// Only the serial writer task may call this; partial writes resume without interleaving.
pub(crate) fn write_if(bytes: &[u8], admitted: impl Fn() -> bool) -> anyhow::Result<()> {
    write_observed_if(bytes, admitted, |_| {})
}

/// Adds bounded observations without changing queue admission, drain polling, or deadlines.
pub(crate) fn write_observed_if<F: FnMut(WriteObservation)>(
    bytes: &[u8],
    admitted: impl Fn() -> bool,
    mut observe: F,
) -> anyhow::Result<()> {
    let started = crate::runtime_uptime::millis();
    let deadline = started.saturating_add(2000);
    let failure = |stage, queued_bytes, observe: &mut F| {
        if queued_bytes > 0 && queued_bytes < bytes.len() {
            PARTIAL_OUTPUT.store(true, Ordering::Release);
        }
        observe(WriteObservation {
            stage: WriteObservationStage::Abandoned,
            at_ms: crate::runtime_uptime::millis(),
            queued_bytes,
            record_bytes: bytes.len(),
        });
        WriteFailure {
            stage,
            queued_bytes,
            record_bytes: bytes.len(),
            elapsed_ms: crate::runtime_uptime::millis().saturating_sub(started),
        }
    };
    let mut remaining = bytes;
    while !remaining.is_empty() {
        if !admitted() {
            return Err(failure(
                WriteStage::Cancelled,
                bytes.len() - remaining.len(),
                &mut observe,
            )
            .into());
        }
        let count = unsafe {
            // Never wait for driver capacity after checking epoch admission.
            sys::usb_serial_jtag_write_bytes(remaining.as_ptr().cast(), remaining.len().min(512), 0)
        };
        if count < 0 {
            return Err(failure(
                WriteStage::Write,
                bytes.len() - remaining.len(),
                &mut observe,
            )
            .into());
        }
        remaining = &remaining[count as usize..];
        if count > 0 {
            observe(WriteObservation {
                stage: WriteObservationStage::Queued,
                at_ms: crate::runtime_uptime::millis(),
                queued_bytes: bytes.len() - remaining.len(),
                record_bytes: bytes.len(),
            });
        }
        if crate::runtime_uptime::millis() >= deadline {
            return Err(failure(
                WriteStage::WriteTimeout,
                bytes.len() - remaining.len(),
                &mut observe,
            )
            .into());
        }
        if count == 0 {
            std::thread::yield_now();
        }
    }
    // Already-queued bytes cannot be retracted. Poll their drain without holding
    // a revocation lock, and never admit another chunk after cancellation.
    loop {
        if !admitted() {
            return Err(failure(WriteStage::Cancelled, bytes.len(), &mut observe).into());
        }
        let ticks = (deadline.saturating_sub(crate::runtime_uptime::millis())
            * u64::from(sys::configTICK_RATE_HZ)
            / 1000)
            .min(1) as u32;
        let flushed = unsafe { sys::usb_serial_jtag_wait_tx_done(ticks) };
        if flushed == sys::ESP_OK && crate::runtime_uptime::millis() <= deadline {
            if !admitted() {
                return Err(failure(WriteStage::Cancelled, bytes.len(), &mut observe).into());
            }
            observe(WriteObservation {
                stage: WriteObservationStage::Completed,
                at_ms: crate::runtime_uptime::millis(),
                queued_bytes: bytes.len(),
                record_bytes: bytes.len(),
            });
            return Ok(());
        }
        if ticks == 0 || crate::runtime_uptime::millis() >= deadline {
            return Err(failure(WriteStage::FlushTimeout, bytes.len(), &mut observe).into());
        }
    }
}

/// Terminate an interrupted old line before the next fresh Hello acknowledgement.
pub(crate) fn resynchronize_if(admitted: impl Fn() -> bool) -> anyhow::Result<()> {
    if PARTIAL_OUTPUT.load(Ordering::Acquire) {
        write_if(b"\n", admitted)?;
        PARTIAL_OUTPUT.store(false, Ordering::Release);
    }
    Ok(())
}
