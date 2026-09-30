//! Owner-backed ESP-IDF capability reservations for shared runtime resource tests.
//! This adapter does not replace or reconfigure the global production allocator.

use std::alloc::Layout;
use std::ptr::NonNull;

#[cfg(test)]
use crate::runtime_allocation_test_sys as sys;
use bitaxe_runtime::allocation::{
    AllocationClass, AllocationRequest, AllocationSnapshot, CapabilityAllocation,
};
#[cfg(not(test))]
use esp_idf_svc::sys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    InvalidLayout,
    Exhausted,
}

/// Owns an actual allocation and its matching deallocator until release/drop.
pub struct EspAllocation {
    pointer: NonNull<u8>,
    maybe_default_layout: Option<Layout>,
}

impl Drop for EspAllocation {
    fn drop(&mut self) {
        // The token is constructed only from a successful matching allocator;
        // it is neither Clone nor Copy, so each pointer is released exactly once.
        unsafe {
            if let Some(layout) = self.maybe_default_layout {
                std::alloc::dealloc(self.pointer.as_ptr(), layout);
            } else {
                sys::heap_caps_free(self.pointer.as_ptr().cast());
            }
        }
    }
}

/// Actual target heap observations and owner-backed allocation tokens.
#[derive(Default)]
pub struct EspCapabilityAllocation;

fn capabilities(class: AllocationClass) -> u32 {
    match class {
        AllocationClass::Default8Bit => sys::MALLOC_CAP_DEFAULT,
        AllocationClass::Internal8Bit => sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT,
        AllocationClass::InternalDma8Bit => {
            sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_DMA | sys::MALLOC_CAP_8BIT
        }
        AllocationClass::Psram8Bit => sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT,
    }
}

impl CapabilityAllocation for EspCapabilityAllocation {
    type Token = EspAllocation;
    type Error = AllocationError;

    fn allocate(&mut self, request: AllocationRequest) -> Result<Self::Token, Self::Error> {
        let layout = Layout::from_size_align(request.bytes, request.alignment)
            .map_err(|_| AllocationError::InvalidLayout)?;
        if request.bytes == 0 {
            return Err(AllocationError::InvalidLayout);
        }
        let default = request.class == AllocationClass::Default8Bit;
        // Default requests use the existing Rust allocator and its resolved SDK
        // routing preference. Forced capabilities use the SDK's explicit API.
        let pointer = unsafe {
            if default {
                std::alloc::alloc(layout)
            } else {
                sys::heap_caps_aligned_alloc(
                    request.alignment,
                    request.bytes,
                    capabilities(request.class),
                )
                .cast()
            }
        };
        Ok(EspAllocation {
            pointer: NonNull::new(pointer).ok_or(AllocationError::Exhausted)?,
            maybe_default_layout: default.then_some(layout),
        })
    }

    fn release(&mut self, token: Self::Token) -> Result<(), Self::Error> {
        drop(token);
        Ok(())
    }

    fn snapshot(&self, class: AllocationClass) -> AllocationSnapshot {
        let caps = capabilities(class);
        // SDK observations allocate no memory and do not alter the heap.
        unsafe {
            AllocationSnapshot {
                free_bytes: sys::heap_caps_get_free_size(caps),
                largest_block_bytes: sys::heap_caps_get_largest_free_block(caps),
                minimum_free_bytes: sys::heap_caps_get_minimum_free_size(caps),
            }
        }
    }
}
