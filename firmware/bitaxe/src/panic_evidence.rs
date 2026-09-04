//! The only unsafe boundary for reset-retained Rust panic evidence.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use bitaxe_api::boot_identity::ResetReasonCategory;
use bitaxe_api::panic_receipt::{
    allocation_source_hash, AllocationFailureContextMarker, AllocationFailureMarker,
    RtcAllocationContextReceipt, RtcAllocationFailureReceipt, RtcPanicReceipt, RustPanicMarker,
    StartupStage,
};
use esp_idf_svc::sys;

#[link_section = ".rtc_noinit"]
static mut RTC_PANIC_RECEIPT: RtcPanicReceipt = RtcPanicReceipt::ZERO;
#[link_section = ".rtc_noinit"]
static mut RTC_ALLOCATION_FAILURE_RECEIPT: RtcAllocationFailureReceipt =
    RtcAllocationFailureReceipt::ZERO;
#[link_section = ".rtc_noinit"]
static mut RTC_ALLOCATION_CONTEXT: RtcAllocationContextReceipt = RtcAllocationContextReceipt::ZERO;
static PANIC_RECORDED: AtomicBool = AtomicBool::new(false);
static ALLOCATION_RECORDED: AtomicBool = AtomicBool::new(false);
static STARTUP_STAGE: AtomicU32 = AtomicU32::new(StartupStage::EarlyIdentity as u32);
const SOURCE_HASH: u64 = allocation_source_hash(env!("BITAXE_FIRMWARE_COMMIT"));

#[derive(Clone, Copy)]
pub(crate) struct ResetReceipts {
    pub(crate) rust_panic: Option<RustPanicMarker>,
    pub(crate) allocation_failure: Option<AllocationFailureMarker>,
    pub(crate) maybe_allocation_context: Option<AllocationFailureContextMarker>,
}

/// Records the global startup boundary, not the identity of an allocating task.
pub(crate) fn enter_stage(stage: StartupStage) {
    STARTUP_STAGE.store(stage as u32, Ordering::Release);
}

pub(crate) fn initialize(reset_reason: ResetReasonCategory) -> ResetReceipts {
    let previous_panic =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_PANIC_RECEIPT)) };
    let previous_allocation =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_ALLOCATION_FAILURE_RECEIPT)) };
    let previous_context =
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_ALLOCATION_CONTEXT)) };
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_PANIC_RECEIPT),
            RtcPanicReceipt::ZERO,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_FAILURE_RECEIPT),
            RtcAllocationFailureReceipt::ZERO,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_CONTEXT),
            RtcAllocationContextReceipt::ZERO,
        );
    }
    let _registration =
        unsafe { sys::heap_caps_register_failed_alloc_callback(Some(record_allocation_failure)) };
    install_hook();
    if reset_reason != ResetReasonCategory::Panic {
        return ResetReceipts {
            rust_panic: None,
            allocation_failure: None,
            maybe_allocation_context: None,
        };
    }
    ResetReceipts {
        rust_panic: RustPanicMarker::from_receipt(previous_panic),
        allocation_failure: AllocationFailureMarker::from_receipt(previous_allocation),
        maybe_allocation_context: AllocationFailureContextMarker::maybe_from_receipts(
            previous_allocation,
            previous_context,
        ),
    }
}

unsafe extern "C" fn record_allocation_failure(
    requested_bytes: usize,
    capabilities: u32,
    _function_name: *const core::ffi::c_char,
) {
    // Keep the first failure, including recoverable failures. A later panic is
    // chronology only; this receipt does not establish its cause. The one-shot
    // atomic claim also prevents concurrent callbacks from racing RTC writes.
    if ALLOCATION_RECORDED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let receipt = RtcAllocationFailureReceipt::new(requested_bytes, capabilities);
    let maybe_stage = StartupStage::maybe_from_raw(STARTUP_STAGE.load(Ordering::Acquire));
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_FAILURE_RECEIPT),
            receipt,
        );
        if let Some(stage) = maybe_stage {
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!(RTC_ALLOCATION_CONTEXT),
                RtcAllocationContextReceipt::new(receipt, SOURCE_HASH, stage),
            );
        }
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
