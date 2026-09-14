//! Runs the production pthread-capability and heap observation adapter against closed FFI fakes.
#![allow(dead_code)]
extern crate self as esp_idf_svc;
#[path = "statistics_runtime/diagnostics.rs"]
mod diagnostics;
#[path = "statistics_runtime/native.rs"]
mod native;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::Mutex;
static LOCK: Mutex<()> = Mutex::new(());
static CONFIG_RESULT: AtomicI32 = AtomicI32::new(0);
static CONFIG_CAPS: AtomicU32 = AtomicU32::new(2052);
static HEAP_CALLS: AtomicU32 = AtomicU32::new(0);
static LAST_HEAP_CAPS: AtomicU32 = AtomicU32::new(0);

#[allow(non_camel_case_types)]
pub mod sys {
    use super::*;
    pub const ESP_OK: i32 = 0;
    pub const ESP_ERR_NOT_FOUND: i32 = 261;
    pub const MALLOC_CAP_INTERNAL: u32 = 2048;
    pub const MALLOC_CAP_8BIT: u32 = 4;
    pub struct esp_pthread_cfg_t {
        pub stack_alloc_caps: u32,
    }
    pub struct multi_heap_info_t {
        pub total_free_bytes: usize,
        pub largest_free_block: usize,
    }
    /// # Safety
    /// Mirrors the pinned zero-input configuration function.
    pub unsafe fn esp_pthread_get_default_config() -> esp_pthread_cfg_t {
        esp_pthread_cfg_t {
            stack_alloc_caps: MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT,
        }
    }
    /// # Safety
    /// Caller supplies a valid mutable configuration output.
    pub unsafe fn esp_pthread_get_cfg(output: *mut esp_pthread_cfg_t) -> i32 {
        (*output).stack_alloc_caps = CONFIG_CAPS.load(Ordering::Relaxed);
        CONFIG_RESULT.load(Ordering::Relaxed)
    }
    /// # Safety
    /// Caller supplies a writable complete heap-info output.
    pub unsafe fn heap_caps_get_info(output: *mut multi_heap_info_t, caps: u32) {
        HEAP_CALLS.fetch_add(1, Ordering::Relaxed);
        LAST_HEAP_CAPS.store(caps, Ordering::Relaxed);
        *output = multi_heap_info_t {
            total_free_bytes: 16000,
            largest_free_block: 10000,
        };
    }
}

#[test]
fn absent_pthread_configuration_uses_the_actual_esp_default_class() {
    // Arrange
    let _exclusive = LOCK.lock().expect("fixture lock");
    CONFIG_RESULT.store(sys::ESP_ERR_NOT_FOUND, Ordering::Relaxed);
    CONFIG_CAPS.store(0, Ordering::Relaxed);
    // Act / Assert
    assert_eq!(native::stack_capabilities().expect("default config"), 2052);
}

#[test]
fn explicit_pthread_capabilities_are_preserved_and_zero_uses_the_documented_default() {
    // Arrange
    let _exclusive = LOCK.lock().expect("fixture lock");
    CONFIG_RESULT.store(sys::ESP_OK, Ordering::Relaxed);
    CONFIG_CAPS.store(1028, Ordering::Relaxed);
    // Act / Assert
    assert_eq!(native::stack_capabilities().expect("configured mask"), 1028);
    CONFIG_CAPS.store(0, Ordering::Relaxed);
    assert_eq!(
        native::stack_capabilities().expect("normalized default"),
        2052
    );
}

#[test]
fn an_unknown_configuration_error_is_not_treated_as_default_configuration() {
    // Arrange
    let _exclusive = LOCK.lock().expect("fixture lock");
    CONFIG_RESULT.store(-1, Ordering::Relaxed);
    // Act / Assert
    assert!(native::stack_capabilities().is_none());
}

#[test]
fn heap_pair_is_read_once_using_the_resolved_capabilities_without_dma_substitution() {
    // Arrange
    let _exclusive = LOCK.lock().expect("fixture lock");
    HEAP_CALLS.store(0, Ordering::Relaxed);
    // Act
    let observation = native::heap(2052);
    // Assert
    assert_eq!(HEAP_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(LAST_HEAP_CAPS.load(Ordering::Relaxed), 2052);
    assert_eq!(
        observation,
        diagnostics::HeapObservation {
            free_bytes: 16000,
            largest_block_bytes: 10000
        }
    );
}
