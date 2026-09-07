//! Fresh stack/heap observations captured only by the sole production owner.
#[path = "owner_resources/cache.rs"]
mod cache;
pub(crate) use cache::Phase;
use cache::{Cache, Snapshot};
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
static OWNER: AtomicUsize = AtomicUsize::new(0);
static RETIRED: AtomicU32 = AtomicU32::new(0);
static CACHE: Cache = Cache::new();
fn current_owner() -> usize {
    unsafe { sys::xTaskGetCurrentTaskHandle() as usize }
}
pub(crate) fn bind_owner_thread() {
    let owner = current_owner();
    if owner != 0 {
        let _ = OWNER.compare_exchange(0, owner, Ordering::AcqRel, Ordering::Acquire);
    }
}
pub(crate) fn capture(generation: u32, phase: Phase) {
    let owner = OWNER.load(Ordering::Acquire);
    if generation == 0 || owner == 0 || current_owner() != owner {
        return;
    }
    let now = crate::runtime_uptime::millis();
    if CACHE
        .read(generation, now)
        .is_some_and(|value| value.phase == phase && now - value.observed_at_ms < 250)
    {
        return;
    }
    let caps = sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT;
    let mut info = std::mem::MaybeUninit::<sys::multi_heap_info_t>::uninit();
    let (info, stack) = unsafe {
        sys::heap_caps_get_info(info.as_mut_ptr(), caps);
        (
            info.assume_init(),
            sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()),
        )
    };
    let (Ok(heap_free_bytes), Ok(heap_largest_bytes)) = (
        u32::try_from(info.total_free_bytes),
        u32::try_from(info.largest_free_block),
    ) else {
        return;
    };
    CACHE.publish(Snapshot {
        generation,
        phase,
        observed_at_ms: crate::runtime_uptime::millis(),
        heap_free_bytes,
        heap_largest_bytes,
        stack_free_bytes: stack,
    });
    if phase == Phase::ShutdownComplete {
        RETIRED.store(generation, Ordering::Release);
    }
}
pub(crate) fn refresh_retired() {
    capture(RETIRED.load(Ordering::Acquire), Phase::ShutdownComplete);
}
/// Formatting is on the ordinary status path; the coherent cache read itself allocates nothing.
pub(crate) fn observation(generation: u32) -> Option<serde_json::Value> {
    let value = CACHE.read_now(generation, crate::runtime_uptime::millis)?;
    Some(
        serde_json::json!({"schema":"worker-owner-resources-v1","generation":value.generation,"phase":value.phase.label(),"observed_at_ms":value.observed_at_ms.to_string(),"heap_free_bytes":value.heap_free_bytes,"heap_largest_bytes":value.heap_largest_bytes,"stack_free_bytes":value.stack_free_bytes}),
    )
}
#[cfg(test)]
#[path = "owner_resources/tests.rs"]
mod tests;
