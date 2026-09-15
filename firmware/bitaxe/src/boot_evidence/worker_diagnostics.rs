//! Closed fixed Serial/JTAG report; no raw logs or runtime network identifiers.

use crate::boot_diagnostic_cache::{Checkpoint, CACHE};
use bitaxe_api::panic_receipt::AllocationFailureContextMarker;

/// Replays typed boot observations without touching the general retained-log buffer.
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
        5 => CACHE.maybe_checkpoint_marker(Checkpoint::WorkerOwnerPrepare),
        6 => CACHE.maybe_checkpoint_marker(Checkpoint::UsbInstall),
        7 => CACHE.maybe_checkpoint_marker(Checkpoint::UsbInstalled),
        8 => CACHE.maybe_checkpoint_marker(Checkpoint::StatisticsStart),
        9 => CACHE.maybe_checkpoint_marker(Checkpoint::StatisticsStarted),
        10 => CACHE.maybe_failure_marker(),
        12 => CACHE.maybe_checkpoint_marker(Checkpoint::WifiDriverPrepare),
        13 => CACHE.maybe_checkpoint_marker(Checkpoint::WifiDriverPrepared),
        16 => crate::storage_http_diagnostics::maybe_failure_marker(),
        17 => crate::storage_http_diagnostics::maybe_status_marker(),
        19 => Some(crate::preparation_evidence::marker(true)),
        20 => Some(crate::preparation_evidence::marker(false)),
        _ => None,
    }
}
