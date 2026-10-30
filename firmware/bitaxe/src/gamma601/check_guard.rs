//! What a bounded hardware check on the 601 requires of the boot it runs in (ADR-0041).

use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use esp_idf_svc::sys;

/// Only a power-on, a software restart, a USB or an external reset admits a check. Any other
/// reset, such as a panic, a watchdog or a brown-out, may have cut an earlier check short.
pub(super) fn reset_reason_admitted() -> Result<(), &'static str> {
    // SAFETY: ESP-IDF exposes the current reset reason as a side-effect-free query.
    let reason = unsafe { sys::esp_reset_reason() };
    match reason {
        sys::esp_reset_reason_t_ESP_RST_POWERON
        | sys::esp_reset_reason_t_ESP_RST_SW
        | sys::esp_reset_reason_t_ESP_RST_USB
        | sys::esp_reset_reason_t_ESP_RST_EXT => Ok(()),
        sys::esp_reset_reason_t_ESP_RST_PANIC => Err("previous_reset_panic"),
        sys::esp_reset_reason_t_ESP_RST_BROWNOUT => Err("previous_reset_brownout"),
        sys::esp_reset_reason_t_ESP_RST_INT_WDT
        | sys::esp_reset_reason_t_ESP_RST_TASK_WDT
        | sys::esp_reset_reason_t_ESP_RST_WDT => Err("previous_reset_watchdog"),
        _ => Err("previous_reset_other"),
    }
}

/// Whether this boot followed a power-on, after which retained RTC memory holds no value.
pub(super) fn power_on_reset() -> bool {
    // SAFETY: as `reset_reason_admitted`.
    unsafe { sys::esp_reset_reason() == sys::esp_reset_reason_t_ESP_RST_POWERON }
}

/// The safe boot's thread is busy for a check's window: its replayed record must not keep
/// claiming the pre-check readings. Only the boot's own facts stay; the readings read
/// unavailable.
pub(super) fn publish_window_report(report: &SafeBootReport) {
    super::publish(SafeBootReport {
        reset_held: report.reset_held,
        controller: report.controller,
        maybe_pre_id_status: report.maybe_pre_id_status,
        maybe_pre_id_cml: report.maybe_pre_id_cml,
        maybe_start_status: report.maybe_start_status,
        maybe_start_cml: report.maybe_start_cml,
        maybe_pre_render_cml: report.maybe_pre_render_cml,
        maybe_post_render_cml: report.maybe_post_render_cml,
        ..SafeBootReport::initial(report.reset_held)
    });
}
