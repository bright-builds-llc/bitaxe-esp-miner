#![allow(dead_code)]
extern crate self as esp_idf_svc;
#[path = "preparation_evidence.rs"]
mod preparation_evidence;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
thread_local! {static TRACK:Cell<bool>=const{Cell::new(false)};static ALLOCATIONS:Cell<u32>=const{Cell::new(0)};}
struct CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK.with(|track| {
            if track.get() {
                ALLOCATIONS.with(|count| count.set(count.get() + 1));
            }
        });
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout)
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
pub mod sys {
    use std::sync::atomic::{AtomicU32, Ordering};
    pub static QUERIES: AtomicU32 = AtomicU32::new(0);
    pub const MALLOC_CAP_INTERNAL: u32 = 1;
    pub const MALLOC_CAP_8BIT: u32 = 2;
    pub unsafe fn heap_caps_get_free_size(_: u32) -> usize {
        QUERIES.fetch_add(1, Ordering::SeqCst);
        10_000
    }
    pub unsafe fn heap_caps_get_largest_free_block(_: u32) -> usize {
        QUERIES.fetch_add(1, Ordering::SeqCst);
        5_000
    }
    #[allow(non_snake_case)]
    pub unsafe fn uxTaskGetStackHighWaterMark(_: *mut core::ffi::c_void) -> u32 {
        QUERIES.fetch_add(1, Ordering::SeqCst);
        1_000
    }
}
mod boot_evidence {
    pub fn operator_snapshot_boot_ordinal() -> u64 {
        7
    }
}
mod runtime_uptime {
    pub fn millis() -> u64 {
        1000
    }
}
