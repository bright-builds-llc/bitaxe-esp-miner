//! Sole native-USB PHY owner with private TinyUSB and callback Adapters.

use bitaxe_core::usb_worker_diagnostics::CdcEvidenceWriter;

mod callbacks;
mod tinyusb;

pub(crate) use bitaxe_core::usb_maintenance::{
    MaintenanceAction, MaintenanceEvent, UsbMaintenanceState,
};
pub(crate) use tinyusb::UsbRuntimeFailure;

unsafe extern "C" {
    fn bitaxe_usb_restart_bootloader() -> i32;
}

pub(crate) fn install_worker_runtime() -> Result<(), UsbRuntimeFailure> {
    tinyusb::install_worker_runtime()
}

pub(crate) fn send_worker_frame(bytes: &[u8]) -> Result<(), UsbRuntimeFailure> {
    tinyusb::send_worker_frame(bytes)
}

pub(crate) fn emit_evidence(
    writer: &mut CdcEvidenceWriter,
    bytes: &[u8],
) -> Result<(), UsbRuntimeFailure> {
    tinyusb::emit_evidence(writer, bytes)
}

pub(crate) fn emit_diagnostic(
    writer: &mut CdcEvidenceWriter,
    bytes: &[u8],
) -> Result<(), UsbRuntimeFailure> {
    tinyusb::emit_diagnostic(writer, bytes)
}

pub(crate) fn worker_observer_state() -> (u32, bool) {
    tinyusb::worker_observer_state()
}

pub(crate) fn restart_into_rom_downloader() -> Result<(), UsbRuntimeFailure> {
    let result = unsafe { bitaxe_usb_restart_bootloader() };
    if result != esp_idf_sys::ESP_OK {
        return Err(UsbRuntimeFailure::Handoff(result));
    }
    Ok(())
}
