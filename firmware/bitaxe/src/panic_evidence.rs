//! The only unsafe boundary for reset-retained Rust panic evidence.

use std::sync::atomic::{AtomicBool, Ordering};

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
const SOURCE_HASH: u64 = allocation_source_hash(env!("BITAXE_FIRMWARE_COMMIT"));
// The native callback writes the existing repr(C) RTC records as fixed u32 words.
const _: () = {
    assert!(core::mem::size_of::<RtcAllocationFailureReceipt>() == 16);
    assert!(core::mem::size_of::<RtcAllocationContextReceipt>() == 32);
};

#[derive(Clone, Copy)]
pub(crate) struct ResetReceipts {
    pub(crate) rust_panic: Option<RustPanicMarker>,
    pub(crate) allocation_failure: Option<AllocationFailureMarker>,
    pub(crate) maybe_allocation_context: Option<AllocationFailureContextMarker>,
}

/// Records the global startup boundary, not the identity of an allocating task.
pub(crate) fn enter_stage(stage: StartupStage) {
    unsafe { bitaxe_allocation_set_stage(stage as u32) };
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
    unsafe {
        bitaxe_allocation_set_legacy(
            core::ptr::addr_of_mut!(RTC_ALLOCATION_FAILURE_RECEIPT).cast(),
            core::ptr::addr_of_mut!(RTC_ALLOCATION_CONTEXT).cast(),
            SOURCE_HASH as u32,
            (SOURCE_HASH >> 32) as u32,
        );
    }
    let registration = unsafe {
        sys::heap_caps_register_failed_alloc_callback(Some(bitaxe_allocation_failure_record))
    };
    assert!(
        registration == sys::ESP_OK,
        "allocation_failure_hook_registration"
    );
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

/// Immutable ELF source binding for offline private provenance validation.
#[no_mangle]
#[used]
pub static BITAXE_FAULT_COMPILED_SOURCE: [u32; 2] =
    [SOURCE_HASH as u32, (SOURCE_HASH >> 32) as u32];

extern "C" {
    fn bitaxe_allocation_set_legacy(
        allocation: *mut core::ffi::c_void,
        context: *mut core::ffi::c_void,
        source_lo: u32,
        source_hi: u32,
    );
    fn bitaxe_allocation_failure_record(size: usize, caps: u32, name: *const core::ffi::c_char);
    fn bitaxe_allocation_set_stage(stage: u32);
    fn bitaxe_allocation_set_identity(source_lo: u32, source_hi: u32, boot_lo: u32, boot_hi: u32);
    fn bitaxe_fault_set_identity(source_lo: u32, source_hi: u32, boot_lo: u32, boot_hi: u32);
    fn bitaxe_fault_owner_begin();
    fn bitaxe_fault_owner_end();
    fn bitaxe_fault_command_begin();
    fn bitaxe_fault_enter_phase(phase: u32);
}

/// Bind both private records after the real boot ordinal has been established.
pub(crate) fn initialize_identity(ordinal: u64) {
    unsafe {
        bitaxe_allocation_set_identity(
            SOURCE_HASH as u32,
            (SOURCE_HASH >> 32) as u32,
            ordinal as u32,
            (ordinal >> 32) as u32,
        );
        bitaxe_fault_set_identity(
            SOURCE_HASH as u32,
            (SOURCE_HASH >> 32) as u32,
            ordinal as u32,
            (ordinal >> 32) as u32,
        );
    }
}
#[inline(always)]
pub(crate) fn owner_begin() {
    unsafe { bitaxe_fault_owner_begin() };
}
#[inline(always)]
pub(crate) fn owner_end() {
    unsafe { bitaxe_fault_owner_end() };
}
#[inline(always)]
pub(crate) fn command_begin() {
    unsafe { bitaxe_fault_command_begin() };
}
#[inline(always)]
pub(crate) fn enter_control_phase(phase: u32) {
    unsafe { bitaxe_fault_enter_phase(phase) };
}
