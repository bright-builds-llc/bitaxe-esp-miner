//! ESP-IDF OTA rollback boot-validation adapter.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - ESP-IDF OTA rollback APIs through `esp_idf_svc::sys`

use esp_idf_svc::sys;

use crate::log_buffer;

/// Validates a newly booted OTA image once startup diagnostics have completed.
pub fn validate_boot(startup_diagnostics_passed: bool) -> anyhow::Result<()> {
    let state = running_partition_state()?;
    if !state.requires_validation() {
        info_retained(&format!(
            "ota_boot_validation=not_pending state={}",
            state.as_str()
        ));
        return Ok(());
    }

    if startup_diagnostics_passed {
        mark_running_slot_valid()?;
        info_retained("ota_boot_validation=marked_valid");
        return Ok(());
    }

    info_retained("ota_boot_validation=marked_invalid_reboot");
    mark_running_slot_invalid_and_reboot()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtaBootState {
    New,
    PendingVerify,
    Valid,
    Invalid,
    Aborted,
    Undefined,
    Factory,
    Unknown,
    RunningPartitionUnavailable,
}

impl OtaBootState {
    const fn requires_validation(self) -> bool {
        matches!(self, Self::New | Self::PendingVerify)
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::PendingVerify => "pending_verify",
            Self::Valid => "valid",
            Self::Invalid => "invalid",
            Self::Aborted => "aborted",
            Self::Undefined => "undefined",
            Self::Factory => "factory",
            Self::Unknown => "unknown",
            Self::RunningPartitionUnavailable => "running_partition_unavailable",
        }
    }
}

fn running_partition_state() -> anyhow::Result<OtaBootState> {
    let partition = unsafe { sys::esp_ota_get_running_partition() };
    if partition.is_null() {
        return Ok(OtaBootState::RunningPartitionUnavailable);
    }

    let mut state: sys::esp_ota_img_states_t = sys::esp_ota_img_states_t_ESP_OTA_IMG_UNDEFINED;
    let result = unsafe { sys::esp_ota_get_state_partition(partition, &mut state) };
    match result {
        sys::ESP_OK => Ok(ota_state_from_raw(state)),
        sys::ESP_ERR_NOT_FOUND => Ok(OtaBootState::Unknown),
        sys::ESP_ERR_NOT_SUPPORTED => Ok(OtaBootState::Factory),
        error => Err(anyhow::anyhow!(
            "esp_ota_get_state_partition failed: esp_err={error}"
        )),
    }
}

fn ota_state_from_raw(state: sys::esp_ota_img_states_t) -> OtaBootState {
    #[allow(non_upper_case_globals)]
    match state {
        sys::esp_ota_img_states_t_ESP_OTA_IMG_NEW => OtaBootState::New,
        sys::esp_ota_img_states_t_ESP_OTA_IMG_PENDING_VERIFY => OtaBootState::PendingVerify,
        sys::esp_ota_img_states_t_ESP_OTA_IMG_VALID => OtaBootState::Valid,
        sys::esp_ota_img_states_t_ESP_OTA_IMG_INVALID => OtaBootState::Invalid,
        sys::esp_ota_img_states_t_ESP_OTA_IMG_ABORTED => OtaBootState::Aborted,
        sys::esp_ota_img_states_t_ESP_OTA_IMG_UNDEFINED => OtaBootState::Undefined,
        _ => OtaBootState::Unknown,
    }
}

fn mark_running_slot_valid() -> anyhow::Result<()> {
    let result = unsafe { sys::esp_ota_mark_app_valid_cancel_rollback() };
    if result == sys::ESP_OK {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "esp_ota_mark_app_valid_cancel_rollback failed: esp_err={result}"
    ))
}

fn mark_running_slot_invalid_and_reboot() -> anyhow::Result<()> {
    let result = unsafe { sys::esp_ota_mark_app_invalid_rollback_and_reboot() };
    if result == sys::ESP_OK {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "esp_ota_mark_app_invalid_rollback_and_reboot failed: esp_err={result}"
    ))
}

fn info_retained(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}
