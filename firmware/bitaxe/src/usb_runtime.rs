//! Sole fixed USB Serial/JTAG driver; no descriptor, PHY, or reset ownership changes.

use esp_idf_sys as sys;
#[path = "usb_write_failure.rs"]
mod failure;
use failure::WriteStage;
#[path = "usb_tx_measurement.rs"]
pub(crate) mod measurement;
pub(crate) use failure::{WriteFailure, WriteObservation, WriteObservationStage};
use measurement::{Category, Measurement, RecordKind, Retained};
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
#[cfg(test)]
pub(crate) fn write_if(bytes: &[u8], admitted: impl Fn() -> bool) -> anyhow::Result<()> {
    write_observed_if(bytes, admitted, |_| {})
}

/// Adds bounded observations without changing queue admission, drain polling, or deadlines.
#[cfg(test)]
pub(crate) fn write_observed_if<F: FnMut(WriteObservation)>(
    bytes: &[u8],
    admitted: impl Fn() -> bool,
    observe: F,
) -> anyhow::Result<()> {
    write_measured_if(
        &mut Retained::new(),
        bytes,
        admitted,
        observe,
        Category::ControlReply,
        RecordKind::Protocol,
    )
}

pub(crate) fn write_measured_if<F: FnMut(WriteObservation)>(
    retained: &mut Retained,
    bytes: &[u8],
    admitted: impl Fn() -> bool,
    mut observe: F,
    category: Category,
    kind: RecordKind,
) -> anyhow::Result<()> {
    retained.begin(kind);
    let started = crate::runtime_uptime::millis();
    let mut measurement = Measurement::new(category, kind, bytes.len(), started);
    let deadline = started.saturating_add(2000);
    let mut failure =
        |stage: WriteStage, queued_bytes, observe: &mut F, measurement: Measurement| {
            let terminal_at = crate::runtime_uptime::millis();
            measurement.finish(retained, stage.label(), terminal_at);
            if queued_bytes > 0 && queued_bytes < bytes.len() {
                PARTIAL_OUTPUT.store(true, Ordering::Release);
            }
            observe(WriteObservation {
                stage: WriteObservationStage::Abandoned,
                at_ms: terminal_at,
                queued_bytes,
                record_bytes: bytes.len(),
            });
            WriteFailure {
                stage,
                queued_bytes,
                record_bytes: bytes.len(),
                elapsed_ms: terminal_at.saturating_sub(started),
            }
        };
    let mut remaining = bytes;
    while !remaining.is_empty() {
        measurement.time(crate::runtime_uptime::millis());
        if measurement.invalid() || crate::runtime_uptime::millis() >= deadline {
            return Err(failure(
                WriteStage::WriteTimeout,
                bytes.len() - remaining.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        if !admitted() {
            return Err(failure(
                WriteStage::Cancelled,
                bytes.len() - remaining.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        let count = unsafe {
            // Never wait for driver capacity after checking epoch admission.
            sys::usb_serial_jtag_write_bytes(remaining.as_ptr().cast(), remaining.len().min(512), 0)
        };
        measurement.queue(
            count,
            remaining.len().min(512),
            crate::runtime_uptime::millis(),
        );
        if count < 0 || count as usize > remaining.len().min(512) {
            return Err(failure(
                WriteStage::Write,
                bytes.len() - remaining.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        remaining = &remaining[count as usize..];
        if measurement.invalid() {
            return Err(failure(
                WriteStage::Write,
                bytes.len() - remaining.len(),
                &mut observe,
                measurement,
            )
            .into());
        }

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
                measurement,
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
            return Err(failure(
                WriteStage::Cancelled,
                bytes.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        measurement.time(crate::runtime_uptime::millis());
        if measurement.invalid() {
            return Err(failure(
                WriteStage::FlushTimeout,
                bytes.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        let ticks = (deadline.saturating_sub(crate::runtime_uptime::millis())
            * u64::from(sys::configTICK_RATE_HZ)
            / 1000)
            .min(1) as u32;
        let drain_start = crate::runtime_uptime::millis();
        let flushed = unsafe { sys::usb_serial_jtag_wait_tx_done(ticks) };
        let terminal_at = crate::runtime_uptime::millis();
        measurement.drain(flushed, drain_start, terminal_at);
        if measurement.invalid() {
            return Err(failure(
                WriteStage::FlushTimeout,
                bytes.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
        if flushed == sys::ESP_OK && terminal_at <= deadline {
            if !admitted() {
                return Err(failure(
                    WriteStage::Cancelled,
                    bytes.len(),
                    &mut observe,
                    measurement,
                )
                .into());
            }
            observe(WriteObservation {
                stage: WriteObservationStage::Completed,
                at_ms: terminal_at,
                queued_bytes: bytes.len(),
                record_bytes: bytes.len(),
            });
            measurement.finish(retained, "completed", terminal_at);
            return Ok(());
        }
        if crate::runtime_uptime::millis() >= deadline {
            return Err(failure(
                WriteStage::FlushTimeout,
                bytes.len(),
                &mut observe,
                measurement,
            )
            .into());
        }
    }
}

/// Terminate an interrupted old line before the next fresh Hello acknowledgement.
pub(crate) fn resynchronize_if(
    retained: &mut Retained,
    admitted: impl Fn() -> bool,
) -> anyhow::Result<()> {
    if PARTIAL_OUTPUT.load(Ordering::Acquire) {
        write_measured_if(
            retained,
            b"\n",
            admitted,
            |_| {},
            Category::Resynchronization,
            RecordKind::Resynchronization,
        )?;
        PARTIAL_OUTPUT.store(false, Ordering::Release);
    }
    Ok(())
}
