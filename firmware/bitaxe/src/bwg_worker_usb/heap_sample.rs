//! Reads the internal heap for the periodic `internal_heap_sample` serial line.

use bitaxe_core::usb_diagnostics::{internal_heap_sample_marker, InternalHeapSample};
use esp_idf_svc::sys;

/// Samples `MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT`, the region whose loss preceded the
/// restart006 allocation failure, and renders it as one closed numeric line.
pub(super) fn marker(uptime_ms: u64) -> String {
    let mut info = std::mem::MaybeUninit::<sys::multi_heap_info_t>::uninit();
    // SAFETY: `heap_caps_get_info` fully initializes the provided structure.
    let info = unsafe {
        sys::heap_caps_get_info(
            info.as_mut_ptr(),
            sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT,
        );
        info.assume_init()
    };
    let bytes = |value: usize| u64::try_from(value).unwrap_or(u64::MAX);
    internal_heap_sample_marker(&InternalHeapSample {
        uptime_ms,
        free_bytes: bytes(info.total_free_bytes),
        allocated_bytes: bytes(info.total_allocated_bytes),
        largest_block_bytes: bytes(info.largest_free_block),
        minimum_free_bytes: bytes(info.minimum_free_bytes),
        allocated_blocks: bytes(info.allocated_blocks),
        free_blocks: bytes(info.free_blocks),
        revoked: crate::production_mining_session::revocation::maybe_revoked().is_some(),
    })
}
