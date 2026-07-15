//! The only unsafe boundary for the ESP32-S3 RTC no-init boot record.

use bitaxe_api::boot_identity::{
    transition_rtc_boot_record, ResetReasonCategory, RtcBootRecord, RtcBootTransition,
};
use esp_idf_svc::sys;

#[link_section = ".rtc_noinit"]
static mut RTC_BOOT_RECORD: RtcBootRecord = RtcBootRecord::ZERO;

pub fn initialize(reset_reason: ResetReasonCategory) -> RtcBootTransition {
    // SAFETY: this runs once in main before worker threads start. Volatile access
    // prevents the retained RTC record from being optimized into normal RAM state.
    let previous = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_BOOT_RECORD)) };
    let cold_start = matches!(
        reset_reason,
        ResetReasonCategory::PowerOn | ResetReasonCategory::Brownout
    );
    let transition = transition_rtc_boot_record(previous, cold_start);
    // SAFETY: same single-threaded startup boundary as the read above.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_BOOT_RECORD), transition.record);
    }
    transition
}

pub fn reset_reason_category() -> ResetReasonCategory {
    // SAFETY: ESP-IDF exposes the current reset reason as a side-effect-free query.
    let reason = unsafe { sys::esp_reset_reason() };
    match reason {
        sys::esp_reset_reason_t_ESP_RST_POWERON => ResetReasonCategory::PowerOn,
        sys::esp_reset_reason_t_ESP_RST_SW => ResetReasonCategory::SoftwareCpu,
        sys::esp_reset_reason_t_ESP_RST_PANIC => ResetReasonCategory::Panic,
        sys::esp_reset_reason_t_ESP_RST_BROWNOUT => ResetReasonCategory::Brownout,
        sys::esp_reset_reason_t_ESP_RST_INT_WDT
        | sys::esp_reset_reason_t_ESP_RST_TASK_WDT
        | sys::esp_reset_reason_t_ESP_RST_WDT => ResetReasonCategory::Watchdog,
        _ => ResetReasonCategory::Other,
    }
}
