//! Numeric RTC checkpoints around pinned native store seams; formatting stays outside panic.
use bitaxe_api::core_dump_receipt::{self as receipt, ReceiptWords, COMMIT, WORDS};
use bitaxe_api::panic_receipt::allocation_source_hash;
use core::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

const SOURCE_HASH: u64 = allocation_source_hash(env!("BITAXE_FIRMWARE_COMMIT"));
#[no_mangle]
#[cfg_attr(target_os = "espidf", link_section = ".rtc_noinit")]
static BITAXE_CORE_DUMP_RTC: [AtomicU32; WORDS] = [const { AtomicU32::new(0) }; WORDS];
#[no_mangle]
static BITAXE_CORE_DUMP_CURRENT: [AtomicU32; WORDS] = [const { AtomicU32::new(0) }; WORDS];
static PREVIOUS: OnceLock<ReceiptWords> = OnceLock::new();

/// Cache partition capacity and a fresh source/boot binding in ordinary boot context.
pub(crate) fn initialize(boot_ordinal: u64) {
    if PREVIOUS.get().is_some() {
        return;
    }
    let previous = std::array::from_fn(|index| BITAXE_CORE_DUMP_RTC[index].load(Ordering::Acquire));
    if PREVIOUS.set(previous).is_err() {
        return;
    }
    let partition = unsafe {
        esp_idf_svc::sys::esp_partition_find_first(
            esp_idf_svc::sys::esp_partition_type_t_ESP_PARTITION_TYPE_DATA,
            esp_idf_svc::sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_COREDUMP,
            core::ptr::null(),
        )
    };
    let capacity = if partition.is_null() {
        0
    } else {
        unsafe { (*partition).size }
    };
    let words = receipt::ready(SOURCE_HASH, boot_ordinal, capacity);
    receipt::commit_words(&words, |index, value| {
        BITAXE_CORE_DUMP_CURRENT[index].store(value, Ordering::Release);
        BITAXE_CORE_DUMP_RTC[index].store(value, Ordering::Release);
    });
}

/// Called before abort while effects are still confirmed off.
pub(crate) fn mark_self_test() {
    bitaxe_core_dump_receipt_update(0, 16, 1, 0);
}

// Literal fixed indices avoid dynamic indexing and zero-fill helpers on the fatal stack.
#[inline(always)]
fn current_words() -> ReceiptWords {
    macro_rules! read {
        ($i:expr) => {
            BITAXE_CORE_DUMP_CURRENT[$i].load(Ordering::Relaxed)
        };
    }
    [
        read!(0),
        read!(1),
        read!(2),
        read!(3),
        read!(4),
        read!(5),
        read!(6),
        read!(7),
        read!(8),
        read!(9),
        read!(10),
        read!(11),
        read!(12),
        read!(13),
        read!(14),
        read!(15),
        read!(16),
        read!(17),
        read!(18),
        read!(19),
    ]
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_receipt")]
pub(crate) extern "C" fn bitaxe_core_dump_receipt_update(
    stage: u32,
    field: u32,
    value: u32,
    result: i32,
) {
    if BITAXE_CORE_DUMP_CURRENT[0].load(Ordering::Acquire) != COMMIT
        || stage > 10
        || !matches!(field, 0 | 9 | 10 | 16)
        || (field == 16 && value > 1)
    {
        return;
    }
    let mut words = current_words();
    receipt::checkpoint(&mut words, stage, field, value, result);
    receipt::commit_words(&words, |index, value| {
        BITAXE_CORE_DUMP_CURRENT[index].store(value, Ordering::Release);
        BITAXE_CORE_DUMP_RTC[index].store(value, Ordering::Release);
    });
}

extern "C" {
    fn __real_esp_core_dump_store() -> i32;
    fn __real_esp_core_dump_write_init() -> i32;
    fn __real_esp_core_dump_write_prepare(data: *mut c_void, length: *mut u32) -> i32;
    fn __real_esp_core_dump_write_start(data: *mut c_void) -> i32;
    fn __real_esp_core_dump_write_end(data: *mut c_void) -> i32;
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_store")]
pub(crate) unsafe extern "C" fn __wrap_esp_core_dump_store() -> i32 {
    bitaxe_core_dump_receipt_update(1, 0, 0, 0);
    let result = __real_esp_core_dump_store();
    bitaxe_core_dump_receipt_update(10, 0, 0, result);
    result
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_init")]
pub(crate) unsafe extern "C" fn __wrap_esp_core_dump_write_init() -> i32 {
    bitaxe_core_dump_receipt_update(2, 0, 0, 0);
    let result = __real_esp_core_dump_write_init();
    bitaxe_core_dump_receipt_update(3, 0, 0, result);
    result
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_prepare")]
pub(crate) unsafe extern "C" fn __wrap_esp_core_dump_write_prepare(
    data: *mut c_void,
    length: *mut u32,
) -> i32 {
    if length.is_null() {
        bitaxe_core_dump_receipt_update(4, 0, 0, 0);
    } else {
        bitaxe_core_dump_receipt_update(4, 9, core::ptr::read_volatile(length), 0);
    }
    let result = __real_esp_core_dump_write_prepare(data, length);
    if length.is_null() {
        bitaxe_core_dump_receipt_update(5, 0, 0, result);
    } else {
        bitaxe_core_dump_receipt_update(5, 10, core::ptr::read_volatile(length), result);
    }
    result
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_start")]
pub(crate) unsafe extern "C" fn __wrap_esp_core_dump_write_start(data: *mut c_void) -> i32 {
    bitaxe_core_dump_receipt_update(6, 0, 0, 0);
    let result = __real_esp_core_dump_write_start(data);
    bitaxe_core_dump_receipt_update(7, 0, 0, result);
    result
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(target_os = "espidf", link_section = ".iram1.core_dump_end")]
pub(crate) unsafe extern "C" fn __wrap_esp_core_dump_write_end(data: *mut c_void) -> i32 {
    bitaxe_core_dump_receipt_update(8, 0, 0, 0);
    let result = __real_esp_core_dump_write_end(data);
    bitaxe_core_dump_receipt_update(9, 0, 0, result);
    result
}

/// Only the regular diagnostic writer calls this allocating renderer.
pub(crate) fn marker(previous: bool) -> String {
    let boot = crate::boot_evidence::operator_snapshot_boot_ordinal();
    if previous {
        receipt::marker(
            PREVIOUS.get().unwrap_or(&[0; WORDS]),
            SOURCE_HASH,
            boot.saturating_sub(1),
            receipt::Origin::PreviousBoot,
        )
    } else {
        receipt::marker(
            &current_words(),
            SOURCE_HASH,
            boot,
            receipt::Origin::CurrentBoot,
        )
    }
}
