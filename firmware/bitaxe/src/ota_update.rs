//! ESP-IDF firmware OTA streaming adapter.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - ESP-IDF OTA APIs through `esp_idf_svc::sys`

use std::ptr;

use esp_idf_svc::sys;

const OTA_CHUNK_BYTES: usize = 1000;
const YIELD_EVERY_CHUNKS: usize = 16;
const SHORT_YIELD_TICKS: sys::TickType_t = 10;

/// Status labels emitted while a firmware OTA request is processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareOtaStatus {
    /// Upload accepted and OTA initialization is starting.
    Starting,
    /// Upload bytes are being streamed into the OTA partition.
    Working { percent: u8 },
    /// HTTP upload stream failed.
    ProtocolError,
    /// ESP-IDF rejected a partition write.
    WriteError,
    /// ESP-IDF validation or boot partition activation failed.
    ValidationError,
    /// OTA was activated and reboot is scheduled.
    Rebooting,
}

impl FirmwareOtaStatus {
    /// Returns the upstream-compatible status text.
    #[must_use]
    pub fn status_text(self) -> String {
        match self {
            Self::Starting => "Starting...".to_owned(),
            Self::Working { percent } => format!("Working ({percent}%)"),
            Self::ProtocolError => "Protocol Error".to_owned(),
            Self::WriteError => "Write Error".to_owned(),
            Self::ValidationError => "Validation Error".to_owned(),
            Self::Rebooting => "Rebooting...".to_owned(),
        }
    }
}

/// Result of an attempted firmware OTA apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareOtaApplyResult {
    /// OTA bytes were written, validated, and selected for next boot.
    Complete { bytes_written: usize },
    /// HTTP request body streaming failed before all bytes were received.
    ProtocolError { code: i32 },
    /// ESP-IDF rejected a write and the OTA handle was aborted.
    WriteError { esp_err: sys::esp_err_t },
    /// ESP-IDF could not start, validate, or activate the OTA image.
    ValidationError { esp_err: sys::esp_err_t },
}

/// Streams the HTTP request body into the next ESP-IDF OTA app partition.
pub fn stream_firmware_ota(
    request: *mut sys::httpd_req_t,
    mut status_sink: impl FnMut(FirmwareOtaStatus),
) -> FirmwareOtaApplyResult {
    status_sink(FirmwareOtaStatus::Starting);

    let ota_partition = unsafe { sys::esp_ota_get_next_update_partition(ptr::null()) };
    if ota_partition.is_null() {
        status_sink(FirmwareOtaStatus::ValidationError);
        return FirmwareOtaApplyResult::ValidationError {
            esp_err: sys::ESP_ERR_NOT_FOUND,
        };
    }

    let mut ota_handle: sys::esp_ota_handle_t = 0;
    let begin_result = unsafe {
        sys::esp_ota_begin(
            ota_partition,
            sys::OTA_SIZE_UNKNOWN as usize,
            &mut ota_handle,
        )
    };
    if begin_result != sys::ESP_OK {
        status_sink(FirmwareOtaStatus::ValidationError);
        return FirmwareOtaApplyResult::ValidationError {
            esp_err: begin_result,
        };
    }

    match stream_request_body_to_ota(request, ota_handle, &mut status_sink) {
        Ok(bytes_written) => finish_ota(ota_handle, ota_partition, bytes_written, status_sink),
        Err(error) => error,
    }
}

fn stream_request_body_to_ota(
    request: *mut sys::httpd_req_t,
    ota_handle: sys::esp_ota_handle_t,
    status_sink: &mut impl FnMut(FirmwareOtaStatus),
) -> Result<usize, FirmwareOtaApplyResult> {
    let total = unsafe { (*request).content_len };
    let mut remaining = total;
    let mut bytes_written = 0;
    let mut chunks = 0;
    let mut buffer = [0_u8; OTA_CHUNK_BYTES];

    while remaining > 0 {
        let chunk_len = remaining.min(buffer.len());
        let recv_len =
            unsafe { sys::httpd_req_recv(request, buffer.as_mut_ptr().cast(), chunk_len) };
        if recv_len == sys::HTTPD_SOCK_ERR_TIMEOUT {
            continue;
        }
        if recv_len <= 0 {
            abort_ota(ota_handle);
            status_sink(FirmwareOtaStatus::ProtocolError);
            return Err(FirmwareOtaApplyResult::ProtocolError { code: recv_len });
        }

        let read = recv_len as usize;
        let write_result = unsafe { sys::esp_ota_write(ota_handle, buffer.as_ptr().cast(), read) };
        if write_result != sys::ESP_OK {
            abort_ota(ota_handle);
            status_sink(FirmwareOtaStatus::WriteError);
            return Err(FirmwareOtaApplyResult::WriteError {
                esp_err: write_result,
            });
        }

        let progress = progress_after_write(total, remaining, read);
        status_sink(FirmwareOtaStatus::Working {
            percent: progress.percent,
        });
        remaining = progress.remaining;
        bytes_written += read;
        chunks += 1;
        if chunks % YIELD_EVERY_CHUNKS == 0 {
            unsafe { sys::vTaskDelay(SHORT_YIELD_TICKS) };
        }
    }

    Ok(bytes_written)
}

fn finish_ota(
    ota_handle: sys::esp_ota_handle_t,
    ota_partition: *const sys::esp_partition_t,
    bytes_written: usize,
    mut status_sink: impl FnMut(FirmwareOtaStatus),
) -> FirmwareOtaApplyResult {
    let end_result = unsafe { sys::esp_ota_end(ota_handle) };
    if end_result != sys::ESP_OK {
        status_sink(FirmwareOtaStatus::ValidationError);
        return FirmwareOtaApplyResult::ValidationError {
            esp_err: end_result,
        };
    }

    let boot_partition_result = unsafe { sys::esp_ota_set_boot_partition(ota_partition) };
    if boot_partition_result != sys::ESP_OK {
        status_sink(FirmwareOtaStatus::ValidationError);
        return FirmwareOtaApplyResult::ValidationError {
            esp_err: boot_partition_result,
        };
    }

    status_sink(FirmwareOtaStatus::Rebooting);
    FirmwareOtaApplyResult::Complete { bytes_written }
}

fn abort_ota(ota_handle: sys::esp_ota_handle_t) {
    let abort_result = unsafe { sys::esp_ota_abort(ota_handle) };
    if abort_result != sys::ESP_OK {
        log::warn!("firmware_ota_abort=failed esp_err={abort_result}");
    }
}

fn progress_percent(total: usize, remaining: usize) -> u8 {
    if total == 0 {
        return 100;
    }

    let percent = 100usize.saturating_sub((remaining.saturating_mul(100)) / total);
    percent.min(100) as u8
}

struct OtaProgress {
    remaining: usize,
    percent: u8,
}

fn progress_after_write(total: usize, remaining: usize, written: usize) -> OtaProgress {
    let remaining = remaining.saturating_sub(written);
    OtaProgress {
        remaining,
        percent: progress_percent(total, remaining),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_after_write_reports_final_chunk_complete() {
        // Arrange
        let total = 1_000;
        let remaining = 1_000;
        let written = 1_000;

        // Act
        let progress = progress_after_write(total, remaining, written);

        // Assert
        assert_eq!(progress.remaining, 0);
        assert_eq!(progress.percent, 100);
    }
}
