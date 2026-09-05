//! Sole fixed USB Serial/JTAG driver; no descriptor, PHY, or reset ownership changes.

use esp_idf_sys as sys;
#[path = "usb_write_failure.rs"]
mod failure;
pub(crate) use failure::WriteFailure;
use failure::WriteStage;

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
pub(crate) fn write(bytes: &[u8]) -> anyhow::Result<()> {
    let started = crate::runtime_uptime::millis();
    let deadline = started.saturating_add(2000);
    let failure = |stage, queued_bytes| WriteFailure {
        stage,
        queued_bytes,
        record_bytes: bytes.len(),
        elapsed_ms: crate::runtime_uptime::millis().saturating_sub(started),
    };
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let count = unsafe {
            sys::usb_serial_jtag_write_bytes(remaining.as_ptr().cast(), remaining.len().min(512), 1)
        };
        if count < 0 {
            return Err(failure(WriteStage::Write, bytes.len() - remaining.len()).into());
        }
        remaining = &remaining[count as usize..];
        if crate::runtime_uptime::millis() >= deadline {
            return Err(failure(WriteStage::WriteTimeout, bytes.len() - remaining.len()).into());
        }
    }
    // The ring buffer may still contain data. Drain within the same total record budget.
    let ticks = (deadline.saturating_sub(crate::runtime_uptime::millis())
        * u64::from(sys::configTICK_RATE_HZ)
        / 1000) as u32;
    let flushed = unsafe { sys::usb_serial_jtag_wait_tx_done(ticks) };
    if flushed != sys::ESP_OK || crate::runtime_uptime::millis() > deadline {
        return Err(failure(WriteStage::FlushTimeout, bytes.len()).into());
    }
    Ok(())
}
