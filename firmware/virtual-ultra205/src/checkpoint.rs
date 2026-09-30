//! Allocation-free diagnostic journal. Raw pointers/register facts stay in private GDB evidence.
use esp_idf_svc::sys;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HeapCheckpoint {
    pub magic: u32,
    pub phase: u32,
    pub stage: u32,
    pub integrity: u32,
    pub core: u32,
    pub stack_base: u32,
    pub stack_bytes: u32,
    pub stack_pointer: u32,
    pub internal_free: u32,
    pub internal_largest: u32,
    pub stack_low_water: u32,
    pub stack_pointer_inside: u32,
}
const EMPTY: HeapCheckpoint = HeapCheckpoint {
    magic: 0,
    phase: 0,
    stage: 0,
    integrity: 0,
    core: 0,
    stack_base: 0,
    stack_bytes: 0,
    stack_pointer: 0,
    internal_free: 0,
    internal_largest: 0,
    stack_low_water: 0,
    stack_pointer_inside: 0,
};
const _: () = assert!(std::mem::size_of::<HeapCheckpoint>() == 48);
#[no_mangle]
#[used]
#[link_section = ".dram1.coredump"]
pub static mut BITAXE_VIRTUAL_HEAP_CHECKPOINTS: [HeapCheckpoint; 8] = [EMPTY; 8];
#[no_mangle]
#[used]
#[link_section = ".dram1.coredump"]
pub static mut BITAXE_VIRTUAL_CHECKPOINT_COUNT: u32 = 0;

/// Exact known scenario phases only; no format, allocation, logging or changed admission.
#[no_mangle]
#[inline(never)]
pub extern "C" fn bitaxe_virtual_heap_checkpoint(phase: u32) {
    unsafe {
        let index = std::ptr::read_volatile(std::ptr::addr_of!(BITAXE_VIRTUAL_CHECKPOINT_COUNT));
        if index >= 8 || !(1..=4).contains(&phase) {
            return;
        }
        let destination = std::ptr::addr_of_mut!(BITAXE_VIRTUAL_HEAP_CHECKPOINTS)
            .cast::<HeapCheckpoint>()
            .add(index as usize);
        let mut record = EMPTY;
        record.magic = 0x56484331;
        record.phase = phase;
        record.stage = 1;
        std::ptr::write_volatile(destination, record);
        std::ptr::write_volatile(
            std::ptr::addr_of_mut!(BITAXE_VIRTUAL_CHECKPOINT_COUNT),
            index + 1,
        );
        record.integrity = u32::from(sys::heap_caps_check_integrity_all(false));
        let task = sys::xTaskGetCurrentTaskHandle();
        let base = sys::pxTaskGetStackStart(task);
        record.stack_base = base as u32;
        record.stack_bytes = sys::heap_caps_get_allocated_size(base.cast()) as u32;
        core::arch::asm!("mov {0}, a1", out(reg) record.stack_pointer);
        record.core = i32::from(esp_idf_svc::hal::cpu::core()) as u32;
        record.internal_free =
            sys::heap_caps_get_free_size(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT) as u32;
        record.internal_largest =
            sys::heap_caps_get_largest_free_block(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT)
                as u32;
        record.stack_low_water = sys::uxTaskGetStackHighWaterMark(task);
        record.stack_pointer_inside = u32::from(
            record.stack_pointer >= record.stack_base
                && record.stack_pointer < record.stack_base.saturating_add(record.stack_bytes),
        );
        record.stage = 2;
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Release);
        std::ptr::write_volatile(destination, record);
        bitaxe_virtual_checkpoint_complete(phase);
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn bitaxe_virtual_checkpoint_complete(phase: u32) {
    std::hint::black_box(phase);
}
