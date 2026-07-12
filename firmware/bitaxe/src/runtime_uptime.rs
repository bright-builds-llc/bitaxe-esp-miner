//! Canonical ESP-IDF monotonic uptime boundary.

use esp_idf_svc::sys;

/// Returns monotonic milliseconds since this boot.
pub(crate) fn millis() -> u64 {
    micros().map_or(0, |uptime_micros| uptime_micros / 1_000)
}

/// Returns monotonic seconds since this boot.
pub(crate) fn seconds() -> u64 {
    micros().map_or(0, |uptime_micros| uptime_micros / 1_000_000)
}

fn micros() -> Option<u64> {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    (uptime_micros > 0).then_some(uptime_micros as u64)
}
