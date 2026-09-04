//! Closed Worker CDC report; no raw logs or runtime network identifiers.

use bitaxe_api::panic_receipt::AllocationFailureContextMarker;

/// Builds one small line lazily, avoiding a report-sized retained-log clone.
pub(crate) fn maybe_worker_diagnostic_line(slot: usize) -> Option<String> {
    match slot {
        0 | 11 => Some(super::worker_usb_boot_marker()),
        1 => Some(format!(
            "usb_runtime_identity schema=v1 firmware_commit={} app_elf_sha256={} redacted=true",
            crate::firmware_commit(),
            crate::app_elf_sha256(),
        )),
        2 => super::worker_rust_panic_marker(),
        3 => super::worker_allocation_failure_marker(),
        4 => super::RESET_RECEIPTS
            .get()
            .and_then(|receipts| receipts.maybe_allocation_context)
            .map(AllocationFailureContextMarker::marker),
        5 => crate::log_buffer::maybe_worker_diagnostic_line(
            "usb_memory_checkpoint",
            "stage=worker_owner_prepare",
        ),
        6 => crate::log_buffer::maybe_worker_diagnostic_line(
            "usb_memory_checkpoint",
            "stage=usb_install",
        ),
        7 => crate::log_buffer::maybe_worker_diagnostic_line(
            "usb_memory_checkpoint",
            "stage=usb_installed",
        ),
        8 => crate::log_buffer::maybe_worker_diagnostic_line(
            "usb_memory_checkpoint",
            "stage=statistics_start",
        ),
        9 => crate::log_buffer::maybe_worker_diagnostic_line(
            "usb_memory_checkpoint",
            "stage=statistics_started",
        ),
        10 => crate::log_buffer::maybe_worker_diagnostic_line(
            "bwg_worker_start_failure",
            "category=startup_failed",
        ),
        _ => None,
    }
}
