//! Firmware-owned retained log buffer for Phase 05 API routes.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::RetainedLogBuffer;

static LOG_BUFFER: OnceLock<Mutex<RetainedLogBuffer>> = OnceLock::new();

/// Appends one runtime log line to the API-visible retained buffer.
pub fn append_runtime_log_line(line: &str) {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(RetainedLogBuffer::new()));
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
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(RetainedLogBuffer::new()));
    let Ok(buffer) = buffer.lock() else {
        log::warn!("retained_log_buffer=unavailable reason=mutex_poisoned");
        return RetainedLogBuffer::new();
    };

    buffer.clone()
}
