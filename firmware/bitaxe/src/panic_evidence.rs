//! The only unsafe boundary for reset-retained Rust panic evidence.

use std::sync::atomic::{AtomicBool, Ordering};

use bitaxe_api::boot_identity::ResetReasonCategory;
use bitaxe_api::panic_receipt::{
    AllocationFailureMarker, RtcAllocationFailureReceipt, RtcPanicReceipt, RustPanicMarker,
};
use esp_idf_svc::sys;

#[link_section = ".rtc_noinit"]
static mut RTC_PANIC_RECEIPT: RtcPanicReceipt = RtcPanicReceipt::ZERO;
#[link_section = ".rtc_noinit"]
static mut RTC_ALLOCATION_FAILURE_RECEIPT: RtcAllocationFailureReceipt =
    RtcAllocationFailureReceipt::ZERO;
static PANIC_RECORDED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
pub(crate) struct ResetReceipts {
    pub(crate) rust_panic: Option<RustPanicMarker>,
    pub(crate) allocation_failure: Option<AllocationFailureMarker>,
}

pub(crate) fn initialize(reset_reason: ResetReasonCategory) -> ResetReceipts {
    let previous_panic =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_PANIC_RECEIPT)) };
    let previous_allocation =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_ALLOCATION_FAILURE_RECEIPT)) };
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_PANIC_RECEIPT),
            RtcPanicReceipt::ZERO,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_FAILURE_RECEIPT),
            RtcAllocationFailureReceipt::ZERO,
        );
    }
    install_hook();
    let _registration =
        unsafe { sys::heap_caps_register_failed_alloc_callback(Some(record_allocation_failure)) };
    if reset_reason != ResetReasonCategory::Panic {
        return ResetReceipts {
            rust_panic: None,
            allocation_failure: None,
        };
    }
    ResetReceipts {
        rust_panic: RustPanicMarker::from_receipt(previous_panic),
        allocation_failure: AllocationFailureMarker::from_receipt(previous_allocation),
    }
}

unsafe extern "C" fn record_allocation_failure(
    requested_bytes: usize,
    capabilities: u32,
    _function_name: *const core::ffi::c_char,
) {
    let receipt = RtcAllocationFailureReceipt::new(requested_bytes, capabilities);
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_FAILURE_RECEIPT),
            receipt,
        );
    }
}

fn install_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |information| {
        if PANIC_RECORDED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            if let Some(location) = information.location() {
                let receipt = RtcPanicReceipt::new(location.file(), location.line());
                unsafe {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_PANIC_RECEIPT), receipt);
                }
            }
        }
        previous(information);
    }));
}
