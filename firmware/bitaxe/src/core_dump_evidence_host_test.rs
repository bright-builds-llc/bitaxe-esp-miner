#![allow(dead_code, non_upper_case_globals)]
extern crate self as esp_idf_svc;
#[path = "core_dump_evidence.rs"]
mod core_dump_evidence;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU32, Ordering};
thread_local! { static TRACK: Cell<bool> = const { Cell::new(false) }; static ALLOCATIONS: Cell<u32> = const { Cell::new(0) }; }
struct Counter;
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK.with(|track| {
            if track.get() {
                ALLOCATIONS.with(|n| n.set(n.get() + 1));
            }
        });
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        System.dealloc(p, layout);
    }
}
#[global_allocator]
static ALLOCATOR: Counter = Counter;
pub mod sys {
    use super::*;
    pub const esp_partition_type_t_ESP_PARTITION_TYPE_DATA: u32 = 1;
    pub const esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_COREDUMP: u32 = 3;
    pub struct Partition {
        pub size: u32,
    }
    pub static QUERIES: AtomicU32 = AtomicU32::new(0);
    static PARTITION: Partition = Partition { size: 974848 };
    pub unsafe fn esp_partition_find_first(
        kind: u32,
        subtype: u32,
        label: *const core::ffi::c_char,
    ) -> *const Partition {
        assert_eq!((kind, subtype), (1, 3));
        assert!(label.is_null());
        QUERIES.fetch_add(1, Ordering::Relaxed);
        &PARTITION
    }
}
mod boot_evidence {
    pub fn operator_snapshot_boot_ordinal() -> u64 {
        7
    }
}
#[no_mangle]
unsafe extern "C" fn __real_esp_core_dump_store() -> i32 {
    use core_dump_evidence::*;
    let data = core::ptr::null_mut();
    let mut length = 100;
    assert_eq!(__wrap_esp_core_dump_write_init(), 0);
    assert_eq!(__wrap_esp_core_dump_write_prepare(data, &mut length), 0);
    assert_eq!(__wrap_esp_core_dump_write_start(data), 0);
    assert_eq!(__wrap_esp_core_dump_write_end(data), -1);
    -2
}
#[no_mangle]
extern "C" fn __real_esp_core_dump_write_init() -> i32 {
    0
}
#[no_mangle]
unsafe extern "C" fn __real_esp_core_dump_write_prepare(
    _: *mut core::ffi::c_void,
    length: *mut u32,
) -> i32 {
    *length = 160;
    0
}
#[no_mangle]
extern "C" fn __real_esp_core_dump_write_start(_: *mut core::ffi::c_void) -> i32 {
    0
}
#[no_mangle]
extern "C" fn __real_esp_core_dump_write_end(_: *mut core::ffi::c_void) -> i32 {
    -1
}

#[test]
fn actual_wrappers_retain_numeric_results_without_allocation_or_partition_queries() {
    // Arrange: early fatal calls cannot borrow an uninitialized source/boot binding.
    core_dump_evidence::bitaxe_core_dump_receipt_update(1, 0, 0, 0);
    assert!(core_dump_evidence::marker(false).contains("status=unavailable"));
    core_dump_evidence::initialize(7);
    core_dump_evidence::mark_self_test();
    let queries = sys::QUERIES.load(Ordering::Relaxed);
    ALLOCATIONS.with(|n| n.set(0));
    TRACK.with(|v| v.set(true));
    // Act
    let result = unsafe { core_dump_evidence::__wrap_esp_core_dump_store() };
    TRACK.with(|v| v.set(false));
    let marker = core_dump_evidence::marker(false);
    // Assert
    assert_eq!(result, -2);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(sys::QUERIES.load(Ordering::Relaxed), queries);
    for field in [
        "status=valid",
        "boot_ordinal=7",
        "stage=store_returned",
        "capacity_bytes=974848",
        "requested_bytes=100",
        "prepared_bytes=160",
        "init_result=0",
        "prepare_result=0",
        "start_result=0",
        "end_result=-1",
        "store_result=-2",
        "self_test_marked=true",
    ] {
        assert!(marker.contains(field), "missing {field}");
    }
    assert!(core_dump_evidence::marker(true).contains("status=unavailable"));
}
