//! The only unsafe boundary for reset-retained Rust panic evidence.

use std::sync::atomic::{AtomicBool, Ordering};

use bitaxe_api::boot_identity::ResetReasonCategory;
use bitaxe_api::panic_receipt::{RtcPanicReceipt, RustPanicMarker};

#[link_section = ".rtc_noinit"]
static mut RTC_PANIC_RECEIPT: RtcPanicReceipt = RtcPanicReceipt::ZERO;
static PANIC_RECORDED: AtomicBool = AtomicBool::new(false);

pub(crate) fn initialize(reset_reason: ResetReasonCategory) -> Option<RustPanicMarker> {
    let previous = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_PANIC_RECEIPT)) };
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_PANIC_RECEIPT),
            RtcPanicReceipt::ZERO,
        );
    }
    install_hook();
    if reset_reason != ResetReasonCategory::Panic {
        return None;
    }
    RustPanicMarker::from_receipt(previous)
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
