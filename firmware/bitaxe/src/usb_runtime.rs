//! Sole native-USB PHY owner around the pure maintenance-handoff reducer.

pub(crate) use bitaxe_core::usb_maintenance::{
    MaintenanceAction, MaintenanceEvent, UsbMaintenanceState,
};

unsafe extern "C" {
    fn bwg_usb_install() -> i32;
    fn bwg_usb_restart_bootloader() -> i32;
}

pub(crate) fn install_worker_runtime() -> anyhow::Result<()> {
    let result = unsafe { bwg_usb_install() };
    if result != esp_idf_sys::ESP_OK {
        anyhow::bail!("TinyUSB install failed: {result}");
    }
    Ok(())
}

pub(crate) fn restart_into_rom_downloader() -> anyhow::Result<()> {
    let result = unsafe { bwg_usb_restart_bootloader() };
    if result != esp_idf_sys::ESP_OK {
        anyhow::bail!("USB ROM handoff failed: {result}");
    }
    Ok(())
}
