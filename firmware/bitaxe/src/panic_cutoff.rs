//! Ultra 205 native-panic boundary. No scheduler, allocator, lock or driver is usable here.
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicU32, Ordering};

#[path = "panic_cutoff_model.rs"]
mod model;

// Internal DRAM remains accessible with flash caches disabled.
#[link_section = ".dram1.bitaxe_panic_cutoff"]
static CONFIGURED_OUTPUTS: AtomicU32 = AtomicU32::new(0);

// Exported fixed-width private core-dump receipt: magic, configured mask,
// enable register, output latch, generation, revoked state, self-test marker.
// Magic is committed last; values are evidence only when read from the core.
#[no_mangle]
// IDF includes this bounded user region even when bulk heap capture is disabled.
#[link_section = ".dram2.coredump.bitaxe_panic_cutoff"]
pub static mut BITAXE_PANIC_CUTOFF_RECEIPT: [u32; 7] = [0; 7];

#[link_section = ".dram1.bitaxe_core_self_test"]
static SELF_TEST_MARKER: AtomicU32 = AtomicU32::new(0);

pub(crate) fn mark_self_test() {
    SELF_TEST_MARKER.store(0x53544631, Ordering::Release);
}

pub(crate) fn mark_enable_configured() {
    CONFIGURED_OUTPUTS.fetch_or(model::ENABLE_MASK, Ordering::Release);
}
pub(crate) fn mark_reset_configured() {
    CONFIGURED_OUTPUTS.fetch_or(model::RESET_MASK, Ordering::Release);
}

/// Fresh output-latch proof; generation admission excludes concurrent ASIC effects.
pub(crate) fn asic_outputs_disabled() -> bool {
    let configured = CONFIGURED_OUTPUTS.load(Ordering::Acquire);
    let (enabled, output) = unsafe {
        (
            core::ptr::read_volatile(sys::GPIO_ENABLE_REG as *const u32),
            core::ptr::read_volatile(sys::GPIO_OUT_REG as *const u32),
        )
    };
    model::outputs_disabled(configured, enabled, output)
}

unsafe extern "C" {
    fn bitaxe_capture_original_panic(info: *const core::ffi::c_void);
    fn __real_esp_panic_handler(info: *mut core::ffi::c_void);
}

/// Linker-wrapped IDF panic entry after the port has stalled the other CPU.
///
/// # Safety
/// Only IDF calls this with its native panic-info pointer. The two pinned GPIOs
/// have normal output ownership before mining can begin. Before initialization,
/// safe output latches do not drive input-mode pads or change mux/direction.
/// Always cut the latches even when configuration evidence is absent/corrupt.
/// Preserve all standard IDF panic processing.
#[no_mangle]
#[link_section = ".iram1.bitaxe_panic_cutoff"]
#[inline(never)]
pub unsafe extern "C" fn __wrap_esp_panic_handler(info: *mut core::ffi::c_void) {
    // Separate W1TS/W1TC stores cannot disturb unrelated pins. Cut active-low
    // power first, then assert reset. No fallible code precedes either store.
    model::cut_latches(
        |mask| core::ptr::write_volatile(sys::GPIO_OUT_W1TS_REG as *mut u32, mask),
        |mask| core::ptr::write_volatile(sys::GPIO_OUT_W1TC_REG as *mut u32, mask),
    );
    let configured = CONFIGURED_OUTPUTS.load(Ordering::Relaxed);
    let revoked_state = crate::production_mining_session::revocation::panic_revoke_all();
    let receipt = core::ptr::addr_of_mut!(BITAXE_PANIC_CUTOFF_RECEIPT).cast::<u32>();
    core::ptr::write_volatile(receipt.add(1), configured);
    core::ptr::write_volatile(
        receipt.add(2),
        core::ptr::read_volatile(sys::GPIO_ENABLE_REG as *const u32),
    );
    core::ptr::write_volatile(
        receipt.add(3),
        core::ptr::read_volatile(sys::GPIO_OUT_REG as *const u32),
    );
    core::ptr::write_volatile(receipt.add(4), revoked_state >> 3);
    core::ptr::write_volatile(receipt.add(5), revoked_state);
    core::ptr::write_volatile(receipt.add(6), SELF_TEST_MARKER.load(Ordering::Relaxed));
    core::ptr::write_volatile(receipt, 0x50434f32);
    // Only after the physical cutoff and generation receipt are complete may diagnostics read panic pointers.
    bitaxe_capture_original_panic(info.cast_const());
    __real_esp_panic_handler(info);
}
