//! Read-only ESP-IDF adapter for one coherent running-platform candidate.

use std::ffi::CStr;

use bitaxe_api::{
    PlatformAsic, PlatformBoard, PlatformFact, PlatformIdentity, PlatformResetReason,
    PlatformUnavailableReason,
};
use esp_idf_svc::sys;
use serde::Deserialize;

const EMBEDDED_STATIC_ASSET_RELEASE: &str = include_str!("../static/www/assets/release.json");

#[derive(Debug, Deserialize)]
struct EmbeddedStaticAssetRelease {
    name: String,
}

/// Captures each read-only running-platform fact once for one operator snapshot.
pub fn collect() -> PlatformIdentity {
    PlatformIdentity {
        esp_idf_version: esp_idf_version(),
        axe_os_static_asset: axe_os_static_asset(),
        board: PlatformFact::available(PlatformBoard::Ultra205),
        asic: PlatformFact::available(PlatformAsic::Bm1366),
        running_partition: running_partition(),
        reset_reason: reset_reason(),
        uptime_milliseconds: uptime_milliseconds(),
        internal_heap_free_bytes: nonzero_heap_fact(unsafe {
            sys::heap_caps_get_free_size(sys::MALLOC_CAP_INTERNAL)
        }),
        internal_heap_minimum_free_bytes: nonzero_heap_fact(unsafe {
            sys::heap_caps_get_minimum_free_size(sys::MALLOC_CAP_INTERNAL)
        }),
        internal_heap_largest_free_block_bytes: nonzero_heap_fact(unsafe {
            sys::heap_caps_get_largest_free_block(sys::MALLOC_CAP_INTERNAL)
        }),
        psram_available: PlatformFact::available(
            unsafe { sys::heap_caps_get_total_size(sys::MALLOC_CAP_SPIRAM) } > 0,
        ),
    }
}

fn esp_idf_version() -> PlatformFact<String> {
    let maybe_version = c_string(unsafe { sys::esp_get_idf_version() });
    maybe_version.map_or_else(
        || PlatformFact::unavailable(PlatformUnavailableReason::EspIdfUnavailable),
        PlatformFact::available,
    )
}

fn axe_os_static_asset() -> PlatformFact<String> {
    let maybe_name =
        serde_json::from_str::<EmbeddedStaticAssetRelease>(EMBEDDED_STATIC_ASSET_RELEASE)
            .ok()
            .map(|release| release.name)
            .filter(|name| !name.trim().is_empty());

    maybe_name.map_or_else(
        || PlatformFact::unavailable(PlatformUnavailableReason::StaticAssetUnavailable),
        PlatformFact::available,
    )
}

fn running_partition() -> PlatformFact<String> {
    let maybe_partition = unsafe { sys::esp_ota_get_running_partition().as_ref() };
    let maybe_label = maybe_partition.and_then(|partition| c_string(partition.label.as_ptr()));
    maybe_label.map_or_else(
        || PlatformFact::unavailable(PlatformUnavailableReason::RunningPartitionUnavailable),
        PlatformFact::available,
    )
}

fn reset_reason() -> PlatformFact<PlatformResetReason> {
    PlatformResetReason::decode(unsafe { sys::esp_reset_reason() as i32 })
}

fn uptime_milliseconds() -> PlatformFact<u64> {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    let maybe_uptime_milliseconds = u64::try_from(uptime_micros)
        .ok()
        .map(|micros| micros / 1_000)
        .filter(|milliseconds| *milliseconds > 0);
    maybe_uptime_milliseconds.map_or_else(
        || PlatformFact::unavailable(PlatformUnavailableReason::UptimeUnavailable),
        PlatformFact::available,
    )
}

fn nonzero_heap_fact(bytes: usize) -> PlatformFact<u64> {
    u64::try_from(bytes)
        .ok()
        .filter(|bytes| *bytes > 0)
        .map_or_else(
            || PlatformFact::unavailable(PlatformUnavailableReason::HeapUnavailable),
            PlatformFact::available,
        )
}

fn c_string(value: *const core::ffi::c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(value) }.to_str().ok()?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
