//! ESP-IDF SPIFFS mount adapter for the firmware `www` partition.
//!
//! Reference breadcrumb: `reference/esp-miner/main/filesystem.c`.

use std::ffi::{CStr, CString};

use esp_idf_svc::sys;

const WWW_BASE_PATH: &str = "/www";
const WWW_MAX_FILES: usize = 5;

/// Public SPIFFS mount status for HTTP static serving decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemStatus {
    /// The `www` SPIFFS partition is mounted and its size information is visible.
    Available {
        /// Total partition bytes reported by ESP-IDF.
        total_bytes: usize,
        /// Used partition bytes reported by ESP-IDF.
        used_bytes: usize,
    },
    /// The `www` SPIFFS partition cannot be used for static serving.
    Unavailable {
        /// Reason stable enough for logs, evidence, and route behavior.
        reason: FilesystemUnavailableReason,
        /// Raw ESP-IDF error value that produced this status.
        esp_err: sys::esp_err_t,
    },
}

/// Stable unavailable reasons for SPIFFS mount/status failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemUnavailableReason {
    /// ESP-IDF reported a mount failure.
    MountFailed,
    /// ESP-IDF could not find the named `www` partition.
    PartitionNotFound,
    /// The mount succeeded but partition information could not be read.
    InfoFailed,
    /// ESP-IDF returned a different error.
    EspErr,
}

impl FilesystemUnavailableReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MountFailed => "mount_failed",
            Self::PartitionNotFound => "partition_not_found",
            Self::InfoFailed => "info_failed",
            Self::EspErr => "esp_err",
        }
    }
}

/// Mounts the `www` SPIFFS partition at `/www` without formatting on failure.
#[must_use]
pub fn mount_www_spiffs() -> FilesystemStatus {
    let base_path = match CString::new(WWW_BASE_PATH) {
        Ok(base_path) => base_path,
        Err(error) => {
            log::warn!("spiffs_mount=unavailable partition=www reason=esp_err error={error}");
            return FilesystemStatus::Unavailable {
                reason: FilesystemUnavailableReason::EspErr,
                esp_err: sys::ESP_FAIL,
            };
        }
    };
    let partition_label = match CString::new("www") {
        Ok(partition_label) => partition_label,
        Err(error) => {
            log::warn!("spiffs_mount=unavailable partition=www reason=esp_err error={error}");
            return FilesystemStatus::Unavailable {
                reason: FilesystemUnavailableReason::EspErr,
                esp_err: sys::ESP_FAIL,
            };
        }
    };

    let config = sys::esp_vfs_spiffs_conf_t {
        base_path: base_path.as_ptr(),
        partition_label: partition_label.as_ptr(),
        max_files: WWW_MAX_FILES,
        format_if_mount_failed: false,
    };

    let result = unsafe { sys::esp_vfs_spiffs_register(&config) };
    if result != sys::ESP_OK {
        return unavailable_status(map_mount_error(result), result);
    }

    let mut total_bytes = 0;
    let mut used_bytes = 0;
    let info_result = unsafe {
        sys::esp_spiffs_info(partition_label.as_ptr(), &mut total_bytes, &mut used_bytes)
    };
    if info_result != sys::ESP_OK {
        return unavailable_status(FilesystemUnavailableReason::InfoFailed, info_result);
    }

    log::info!(
        "spiffs_mount=available partition=www total_bytes={total_bytes} used_bytes={used_bytes}"
    );
    FilesystemStatus::Available {
        total_bytes,
        used_bytes,
    }
}

fn map_mount_error(error: sys::esp_err_t) -> FilesystemUnavailableReason {
    match error {
        sys::ESP_FAIL => FilesystemUnavailableReason::MountFailed,
        sys::ESP_ERR_NOT_FOUND => FilesystemUnavailableReason::PartitionNotFound,
        _ => FilesystemUnavailableReason::EspErr,
    }
}

fn unavailable_status(
    reason: FilesystemUnavailableReason,
    esp_err: sys::esp_err_t,
) -> FilesystemStatus {
    log::warn!(
        "spiffs_mount=unavailable partition=www reason={} esp_err={} esp_err_name={}",
        reason.as_str(),
        esp_err,
        esp_err_name(esp_err)
    );
    FilesystemStatus::Unavailable { reason, esp_err }
}

fn esp_err_name(error: sys::esp_err_t) -> String {
    let name = unsafe { sys::esp_err_to_name(error) };
    if name.is_null() {
        return "unknown".to_owned();
    }

    unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned()
}
