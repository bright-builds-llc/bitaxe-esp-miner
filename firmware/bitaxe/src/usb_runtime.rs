//! Sole fixed USB Serial/JTAG driver; no descriptor, PHY, or reset ownership changes.

use esp_idf_sys as sys;

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
    let deadline = crate::runtime_uptime::millis().saturating_add(2000);
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let count = unsafe {
            sys::usb_serial_jtag_write_bytes(remaining.as_ptr().cast(), remaining.len().min(512), 1)
        };
        anyhow::ensure!(count >= 0, "serial_write_failed");
        anyhow::ensure!(
            crate::runtime_uptime::millis() < deadline,
            "serial_write_timeout"
        );
        remaining = &remaining[count as usize..];
    }
    let flushed = unsafe { sys::usb_serial_jtag_wait_tx_done(1) };
    anyhow::ensure!(flushed == sys::ESP_OK, "serial_flush_timeout");
    Ok(())
}
