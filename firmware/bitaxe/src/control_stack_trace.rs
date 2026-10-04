//! Control owner diagnostics for the idle-review panic.
//!
//! Each control command checks internal-heap integrity before and after it runs,
//! and its stack high-water mark lands in the captured core-dump region.
//! A failed check aborts with a named message that the dump's panic-details note keeps.
use esp_idf_svc::sys;

#[path = "control_stack_model.rs"]
mod model;

// IDF includes this bounded user region even when bulk heap capture is disabled.
// Layout: magic, commands, min free, command at min, last free, checks, stack, version.
#[no_mangle]
#[link_section = ".dram2.coredump.bitaxe_control_stack"]
pub static mut BITAXE_CONTROL_STACK_TRACE: [u32; model::WORDS] = [0; model::WORDS];

const INTERNAL: u32 = sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT;

/// Owned by the single control owner thread; dropping it closes the command.
pub(crate) struct CommandTrace {
    stack_bytes: u32,
}

/// Checks the heap before a command starts; the returned guard checks it again on drop.
pub(crate) fn command(stack_bytes: usize) -> CommandTrace {
    require_intact(c"bitaxe_control_heap_corrupt_before_command");
    CommandTrace {
        stack_bytes: u32::try_from(stack_bytes).unwrap_or(u32::MAX),
    }
}

fn require_intact(message: &'static core::ffi::CStr) {
    // SAFETY: the SDK walks its own heap metadata under the heap lock.
    if !unsafe { sys::heap_caps_check_integrity(INTERNAL, false) } {
        // SAFETY: a static NUL-terminated message outlives the panic handler.
        unsafe { sys::esp_system_abort(message.as_ptr()) }
    }
    CHECKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

static CHECKS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

impl Drop for CommandTrace {
    fn drop(&mut self) {
        require_intact(c"bitaxe_control_heap_corrupt_after_command");
        // SAFETY: a null handle selects the calling task; the mark is in bytes on this port.
        let free_bytes = unsafe { sys::uxTaskGetStackHighWaterMark(core::ptr::null_mut()) };
        let checks = CHECKS.load(core::sync::atomic::Ordering::Relaxed);
        // SAFETY: only the control owner writes the record; the panic path only reads it.
        unsafe {
            let record = core::ptr::addr_of_mut!(BITAXE_CONTROL_STACK_TRACE);
            let next = model::record_command(
                core::ptr::read_volatile(record),
                free_bytes,
                self.stack_bytes,
                checks,
            );
            let words = record.cast::<u32>();
            core::ptr::write_volatile(words, 0);
            for (index, word) in next.iter().enumerate().skip(1) {
                core::ptr::write_volatile(words.add(index), *word);
            }
            core::ptr::write_volatile(words, next[0]);
        }
    }
}
