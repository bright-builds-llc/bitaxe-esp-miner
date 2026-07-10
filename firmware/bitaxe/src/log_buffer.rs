//! Firmware-owned retained log buffer for Phase 05 API routes.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::RetainedLogBuffer;

static LOG_BUFFER: OnceLock<Mutex<RetainedLogBuffer>> = OnceLock::new();
const FALLBACK_LOG_RETENTION_BYTES: usize = 32 * 1024;

/// Appends one runtime log line to the API-visible retained buffer.
pub fn append_runtime_log_line(line: &str) {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(firmware_log_buffer()));
    let Ok(mut buffer) = buffer.lock() else {
        log::warn!("retained_log_buffer=unavailable reason=mutex_poisoned");
        return;
    };

    buffer.append(line);
    if !line.ends_with('\n') {
        buffer.append("\n");
    }
}

/// Returns retained log chunks for `/api/system/logs`.
#[must_use]
pub fn download_chunks() -> Vec<String> {
    retained_log_buffer().download_chunks()
}

/// Returns a point-in-time copy for WebSocket stream planning.
#[must_use]
pub fn retained_log_buffer() -> RetainedLogBuffer {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(firmware_log_buffer()));
    let Ok(buffer) = buffer.lock() else {
        log::warn!("retained_log_buffer=unavailable reason=mutex_poisoned");
        return RetainedLogBuffer::empty();
    };

    buffer.clone()
}

/// Returns complete retained accepted-state category markers for diagnostic replay.
#[must_use]
pub fn accepted_state_replay_lines() -> Vec<String> {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(firmware_log_buffer()));
    let Ok(buffer) = buffer.lock() else {
        log::warn!("retained_log_buffer=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    buffer.complete_lines_with_first_token("accepted_state_snapshot")
}

fn firmware_log_buffer() -> RetainedLogBuffer {
    match RetainedLogBuffer::try_new() {
        Ok(buffer) => buffer,
        Err(error) => {
            log::warn!(
                "retained_log_buffer=degraded reason=allocation_failed requested_bytes={} fallback_bytes={} error={error:?}",
                bitaxe_api::LOG_RETENTION_BYTES,
                FALLBACK_LOG_RETENTION_BYTES
            );
            fallback_log_buffer()
        }
    }
}

fn fallback_log_buffer() -> RetainedLogBuffer {
    match RetainedLogBuffer::try_with_capacity(FALLBACK_LOG_RETENTION_BYTES) {
        Ok(buffer) => buffer,
        Err(error) => {
            log::warn!(
                "retained_log_buffer=unavailable reason=fallback_allocation_failed requested_bytes={} error={error:?}",
                FALLBACK_LOG_RETENTION_BYTES
            );
            RetainedLogBuffer::empty()
        }
    }
}
