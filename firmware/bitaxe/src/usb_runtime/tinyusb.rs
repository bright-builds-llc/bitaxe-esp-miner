use std::ffi::c_char;
use std::fmt;

use bitaxe_core::usb_worker_diagnostics::{
    CdcEvidenceTransport, CdcEvidenceWriter, DIAGNOSTIC_LINE_BYTES,
    MAINTENANCE_RECEIPT_RESERVE_BYTES,
};

use bitaxe_core::usb_worker::{
    UsbWriteFailure, VendorWriteProgress, VendorWriteStep, WORKER_CONFIGURATION_DESCRIPTOR,
    WORKER_DEVICE_DESCRIPTOR, WORKER_STRING_DESCRIPTORS,
};
use esp_idf_sys as sys;

const _: () = assert!(
    sys::CFG_TUD_CDC_TX_BUFSIZE as usize
        >= DIAGNOSTIC_LINE_BYTES + MAINTENANCE_RECEIPT_RESERVE_BYTES
);

const WORKER_INTERFACE: u8 = 0;
const EVIDENCE_INTERFACE: u8 = 0;
const _: () = assert!(sys::CONFIG_TINYUSB_TASK_STACK_SIZE == 3_072);

struct StaticStringPointers([*const c_char; 5]);

// SAFETY: every pointer targets immutable static language/string bytes, and
// esp_tinyusb only reads this table for the boot lifetime.
unsafe impl Sync for StaticStringPointers {}

static STRING_POINTERS: StaticStringPointers = StaticStringPointers([
    WORKER_STRING_DESCRIPTORS[0].as_ptr().cast(),
    WORKER_STRING_DESCRIPTORS[1].as_ptr().cast(),
    WORKER_STRING_DESCRIPTORS[2].as_ptr().cast(),
    WORKER_STRING_DESCRIPTORS[3].as_ptr().cast(),
    WORKER_STRING_DESCRIPTORS[4].as_ptr().cast(),
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UsbRuntimeFailure {
    UnavailableTransport,
    Disconnected,
    PartialWrite,
    Timeout,
    Install(i32),
    Handoff(i32),
}

impl fmt::Display for UsbRuntimeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnavailableTransport => "unavailable_transport",
            Self::Disconnected => "disconnected",
            Self::PartialWrite => "partial_write",
            Self::Timeout => "timeout",
            Self::Install(_) => "install",
            Self::Handoff(_) => "handoff",
        })
    }
}

impl std::error::Error for UsbRuntimeFailure {}

pub(super) fn install_worker_runtime() -> Result<(), UsbRuntimeFailure> {
    let config = sys::tinyusb_config_t {
        __bindgen_anon_1: sys::tinyusb_config_t__bindgen_ty_1 {
            device_descriptor: WORKER_DEVICE_DESCRIPTOR
                .as_ptr()
                .cast::<sys::tusb_desc_device_t>(),
        },
        string_descriptor: STRING_POINTERS.0.as_ptr().cast_mut(),
        string_descriptor_count: i32::try_from(STRING_POINTERS.0.len())
            .map_err(|_| UsbRuntimeFailure::Install(sys::ESP_ERR_INVALID_SIZE))?,
        external_phy: false,
        __bindgen_anon_2: sys::tinyusb_config_t__bindgen_ty_2 {
            __bindgen_anon_1: sys::tinyusb_config_t__bindgen_ty_2__bindgen_ty_1 {
                configuration_descriptor: WORKER_CONFIGURATION_DESCRIPTOR.as_ptr(),
            },
        },
        self_powered: false,
        vbus_monitor_io: -1,
    };
    let result = unsafe { sys::tinyusb_driver_install(&config) };
    if result != sys::ESP_OK {
        return Err(UsbRuntimeFailure::Install(result));
    }
    Ok(())
}

pub(super) fn send_worker_frame(bytes: &[u8]) -> Result<(), UsbRuntimeFailure> {
    let mut progress = VendorWriteProgress::new(bytes.len()).map_err(map_write_failure)?;
    loop {
        let mounted = unsafe { sys::tud_vendor_n_mounted(WORKER_INTERFACE) };
        let available = if mounted {
            usize::try_from(unsafe { sys::tud_vendor_n_write_available(WORKER_INTERFACE) })
                .map_err(|_| UsbRuntimeFailure::PartialWrite)?
        } else {
            0
        };
        match progress.next(mounted, available) {
            VendorWriteStep::Complete => return Ok(()),
            VendorWriteStep::Write { offset, length } => {
                let length_u32 =
                    u32::try_from(length).map_err(|_| UsbRuntimeFailure::PartialWrite)?;
                let written = unsafe {
                    sys::tud_vendor_n_write(
                        WORKER_INTERFACE,
                        bytes[offset..].as_ptr().cast(),
                        length_u32,
                    )
                };
                let written =
                    usize::try_from(written).map_err(|_| UsbRuntimeFailure::PartialWrite)?;
                handle_write_step(progress.record_write(length, written))?;
            }
            VendorWriteStep::Wait => wait_for_vendor_progress(),
            VendorWriteStep::Continue => {}
            VendorWriteStep::Failed(failure) => {
                clear_vendor_write();
                return Err(map_write_failure(failure));
            }
        }
    }
}

pub(super) fn emit_evidence(
    writer: &mut CdcEvidenceWriter,
    bytes: &[u8],
) -> Result<(), UsbRuntimeFailure> {
    try_emit_evidence(writer, bytes, false)
}

pub(super) fn emit_diagnostic(
    writer: &mut CdcEvidenceWriter,
    bytes: &[u8],
) -> Result<(), UsbRuntimeFailure> {
    try_emit_evidence(writer, bytes, true)
}

fn try_emit_evidence(
    writer: &mut CdcEvidenceWriter,
    bytes: &[u8],
    diagnostic: bool,
) -> Result<(), UsbRuntimeFailure> {
    if bytes.is_empty() {
        return Err(UsbRuntimeFailure::UnavailableTransport);
    }
    if !unsafe { sys::tud_mounted() } {
        return Err(UsbRuntimeFailure::Disconnected);
    }
    let accepted = if diagnostic {
        writer.try_emit_diagnostic(&mut TinyUsbEvidenceTransport, bytes)
    } else {
        writer.try_emit(&mut TinyUsbEvidenceTransport, bytes)
    };
    if !accepted {
        return Err(UsbRuntimeFailure::PartialWrite);
    }
    Ok(())
}

pub(super) fn worker_observer_state() -> (u32, bool) {
    if !unsafe { sys::tud_mounted() } {
        return (0, false);
    }
    let mut coding = sys::cdc_line_coding_t::default();
    unsafe {
        sys::tud_cdc_n_get_line_coding(EVIDENCE_INTERFACE, &mut coding);
    }
    (coding.bit_rate, unsafe {
        sys::tud_cdc_n_connected(EVIDENCE_INTERFACE)
    })
}

struct TinyUsbEvidenceTransport;

impl CdcEvidenceTransport for TinyUsbEvidenceTransport {
    fn available(&self) -> usize {
        unsafe { sys::tud_cdc_n_write_available(EVIDENCE_INTERFACE) as usize }
    }

    fn write(&mut self, bytes: &[u8]) -> usize {
        let Ok(length) = u32::try_from(bytes.len()) else {
            return 0;
        };
        unsafe { sys::tud_cdc_n_write(EVIDENCE_INTERFACE, bytes.as_ptr().cast(), length) as usize }
    }

    fn flush(&mut self) {
        unsafe {
            sys::tud_cdc_n_write_flush(EVIDENCE_INTERFACE);
        }
    }
}

pub(super) fn vendor_available() -> usize {
    usize::try_from(unsafe { sys::tud_vendor_n_available(WORKER_INTERFACE) }).unwrap_or(0)
}

pub(super) fn read_vendor(buffer: &mut [u8]) -> usize {
    let Ok(length) = u32::try_from(buffer.len()) else {
        return 0;
    };
    usize::try_from(unsafe {
        sys::tud_vendor_n_read(WORKER_INTERFACE, buffer.as_mut_ptr().cast(), length)
    })
    .unwrap_or(0)
}

pub(super) fn discard_cdc(interface: u8, buffer: &mut [u8]) {
    let Ok(length) = u32::try_from(buffer.len()) else {
        return;
    };
    while unsafe { sys::tud_cdc_n_available(interface) } > 0 {
        let received =
            unsafe { sys::tud_cdc_n_read(interface, buffer.as_mut_ptr().cast(), length) };
        if received == 0 {
            return;
        }
    }
}

fn handle_write_step(step: VendorWriteStep) -> Result<(), UsbRuntimeFailure> {
    match step {
        VendorWriteStep::Complete | VendorWriteStep::Continue => Ok(()),
        VendorWriteStep::Wait => {
            wait_for_vendor_progress();
            Ok(())
        }
        VendorWriteStep::Failed(failure) => {
            clear_vendor_write();
            Err(map_write_failure(failure))
        }
        VendorWriteStep::Write { .. } => Err(UsbRuntimeFailure::PartialWrite),
    }
}

fn wait_for_vendor_progress() {
    unsafe {
        sys::tud_vendor_n_write_flush(WORKER_INTERFACE);
        sys::vTaskDelay(0);
    }
}

fn clear_vendor_write() {
    unsafe {
        sys::tud_vendor_n_write_clear(WORKER_INTERFACE);
    }
}

fn map_write_failure(failure: UsbWriteFailure) -> UsbRuntimeFailure {
    match failure {
        UsbWriteFailure::UnavailableTransport => UsbRuntimeFailure::UnavailableTransport,
        UsbWriteFailure::PartialWrite => UsbRuntimeFailure::PartialWrite,
        UsbWriteFailure::Timeout => UsbRuntimeFailure::Timeout,
    }
}
