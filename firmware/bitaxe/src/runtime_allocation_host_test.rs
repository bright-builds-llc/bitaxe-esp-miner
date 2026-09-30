#![allow(dead_code)]

#[path = "runtime_allocation_adapter.rs"]
mod runtime_allocation_adapter;

use bitaxe_runtime::allocation::{AllocationClass, AllocationRequest, CapabilityAllocation};
use runtime_allocation_adapter::{AllocationError, EspCapabilityAllocation};

mod runtime_allocation_test_sys {
    use std::alloc::{alloc, dealloc, Layout};
    use std::cell::RefCell;
    use std::ffi::c_void;

    pub const MALLOC_CAP_DEFAULT: u32 = 4096;
    pub const MALLOC_CAP_INTERNAL: u32 = 2048;
    pub const MALLOC_CAP_8BIT: u32 = 4;
    pub const MALLOC_CAP_DMA: u32 = 8;
    pub const MALLOC_CAP_SPIRAM: u32 = 1024;

    #[derive(Default)]
    pub struct State {
        pub maybe_layout: Option<Layout>,
        pub caps: u32,
        pub allocations: usize,
        pub releases: usize,
        pub fail: bool,
    }
    thread_local! { pub static STATE: RefCell<State> = RefCell::new(State::default()); }

    pub unsafe fn heap_caps_aligned_alloc(
        alignment: usize,
        bytes: usize,
        caps: u32,
    ) -> *mut c_void {
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.caps = caps;
            state.allocations += 1;
            if state.fail {
                return std::ptr::null_mut();
            }
            let layout = Layout::from_size_align(bytes, alignment).expect("validated layout");
            state.maybe_layout = Some(layout);
            alloc(layout).cast()
        })
    }
    pub unsafe fn heap_caps_free(pointer: *mut c_void) {
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.releases += 1;
            dealloc(
                pointer.cast(),
                state.maybe_layout.take().expect("owned allocation"),
            );
        });
    }
    pub unsafe fn heap_caps_get_free_size(_caps: u32) -> usize {
        3451
    }
    pub unsafe fn heap_caps_get_largest_free_block(_caps: u32) -> usize {
        2176
    }
    pub unsafe fn heap_caps_get_minimum_free_size(_caps: u32) -> usize {
        2000
    }
}

fn request(class: AllocationClass) -> AllocationRequest {
    AllocationRequest {
        bytes: 128,
        alignment: 8,
        class,
        phase: "test",
    }
}

#[test]
fn internal_dma_token_releases_exactly_once_on_drop() {
    // Arrange
    let mut allocator = EspCapabilityAllocation;
    // Act
    let token = allocator
        .allocate(request(AllocationClass::InternalDma8Bit))
        .expect("allocation");
    drop(token);
    // Assert
    runtime_allocation_test_sys::STATE.with(|cell| {
        let state = cell.borrow();
        assert_eq!(state.caps, 0x80c);
        assert_eq!(state.allocations, 1);
        assert_eq!(state.releases, 1);
    });
}

#[test]
fn refused_allocation_does_not_create_a_token_or_release() {
    // Arrange
    runtime_allocation_test_sys::STATE.with(|cell| cell.borrow_mut().fail = true);
    let mut allocator = EspCapabilityAllocation;
    // Act
    let result = allocator.allocate(request(AllocationClass::Internal8Bit));
    // Assert
    assert!(matches!(result, Err(AllocationError::Exhausted)));
    runtime_allocation_test_sys::STATE.with(|cell| assert_eq!(cell.borrow().releases, 0));
}

#[test]
fn invalid_layout_is_rejected_before_ffi_access() {
    // Arrange
    let mut allocator = EspCapabilityAllocation;
    let mut allocation = request(AllocationClass::Psram8Bit);
    allocation.alignment = 3;
    // Act
    let result = allocator.allocate(allocation);
    // Assert
    assert!(matches!(result, Err(AllocationError::InvalidLayout)));
    runtime_allocation_test_sys::STATE.with(|cell| assert_eq!(cell.borrow().allocations, 0));
}

#[test]
fn heap_snapshot_preserves_actual_fragmentation_fields() {
    // Arrange
    let allocator = EspCapabilityAllocation;
    // Act
    let snapshot = allocator.snapshot(AllocationClass::Internal8Bit);
    // Assert
    assert_eq!(snapshot.free_bytes, 3451);
    assert_eq!(snapshot.largest_block_bytes, 2176);
    assert_eq!(snapshot.minimum_free_bytes, 2000);
    runtime_allocation_test_sys::STATE.with(|cell| assert_eq!(cell.borrow().allocations, 0));
}
