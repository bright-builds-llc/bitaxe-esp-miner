#![allow(dead_code)]
extern crate self as esp_idf_svc;
#[path = "production_mining_session/owner_resources.rs"]
mod owner_resources;
pub mod sys {
    use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
    pub static TASK: AtomicUsize = AtomicUsize::new(1);
    pub static QUERIES: AtomicU32 = AtomicU32::new(0);
    pub static STACK: AtomicU32 = AtomicU32::new(8_220);
    pub const MALLOC_CAP_INTERNAL: u32 = 1;
    pub const MALLOC_CAP_8BIT: u32 = 2;
    #[allow(non_snake_case)]
    pub unsafe fn xTaskGetCurrentTaskHandle() -> *mut core::ffi::c_void {
        TASK.load(Ordering::SeqCst) as *mut core::ffi::c_void
    }
    #[allow(non_camel_case_types)]
    pub struct multi_heap_info_t {
        pub total_free_bytes: usize,
        pub largest_free_block: usize,
    }
    pub unsafe fn heap_caps_get_info(info: *mut multi_heap_info_t, _: u32) {
        QUERIES.fetch_add(1, Ordering::SeqCst);
        info.write(multi_heap_info_t {
            total_free_bytes: 15_807,
            largest_free_block: 8_192,
        });
    }
    #[allow(non_snake_case)]
    pub unsafe fn uxTaskGetStackHighWaterMark(_: *mut core::ffi::c_void) -> u32 {
        QUERIES.fetch_add(1, Ordering::SeqCst);
        STACK.load(Ordering::SeqCst)
    }
}
mod runtime_uptime {
    use std::sync::atomic::{AtomicU64, Ordering};
    pub static NOW: AtomicU64 = AtomicU64::new(1000);
    pub fn millis() -> u64 {
        NOW.load(Ordering::SeqCst)
    }
}
