//! Firmware collection boundary for pure AxeOS API response snapshots.

use bitaxe_api::{ApiSnapshot, PlatformSnapshot};
use esp_idf_svc::sys;

/// Collects current firmware facts and overlays them on the safe Ultra 205 API
/// snapshot used by the pure contract mappers.
pub fn collect_api_snapshot() -> ApiSnapshot {
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.platform = collect_platform_snapshot(snapshot.platform);
    snapshot
}

fn collect_platform_snapshot(mut platform: PlatformSnapshot) -> PlatformSnapshot {
    platform.version = crate::firmware_commit().to_owned();
    platform.idf_version = crate::ESP_IDF_VERSION.to_owned();
    platform.reset_reason = crate::reset_reason().to_string();
    platform.running_partition = crate::partition_label();
    platform.psram_available = crate::psram_status() == "available";
    platform.free_heap = heap_free(sys::MALLOC_CAP_DEFAULT);
    platform.free_heap_internal = heap_free(sys::MALLOC_CAP_INTERNAL);
    platform.free_heap_spiram = heap_free(sys::MALLOC_CAP_SPIRAM);
    platform.min_free_heap = heap_min_free(sys::MALLOC_CAP_DEFAULT);
    platform.max_alloc_heap = heap_largest_free_block(sys::MALLOC_CAP_DEFAULT);
    platform.uptime_seconds = uptime_seconds();
    platform
}

fn heap_free(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_free_size(caps) as u64 }
}

fn heap_min_free(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_minimum_free_size(caps) as u64 }
}

fn heap_largest_free_block(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_largest_free_block(caps) as u64 }
}

fn uptime_seconds() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000_000
}
