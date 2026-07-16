//! Firmware-owned retained log buffer for Phase 05 API routes.

use std::sync::{Mutex, OnceLock};

use std::{error::Error, fmt};

use bitaxe_api::{
    logs::{RetainedPair, RetainedPairError},
    RetainedLogBuffer,
};

static LOG_BUFFER: OnceLock<Mutex<RetainedLogBuffer>> = OnceLock::new();
const FALLBACK_LOG_RETENTION_BYTES: usize = 32 * 1024;

/// Closed, redaction-safe production retained-pair storage failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetainedPairStorageError {
    MutexPoisoned,
    InvalidPair,
    StorageUnavailable,
    PairExceedsCapacity,
    CounterOverflow,
}

impl fmt::Display for RetainedPairStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self {
            Self::MutexPoisoned => "mutex_poisoned",
            Self::InvalidPair => "invalid_pair",
            Self::StorageUnavailable => "storage_unavailable",
            Self::PairExceedsCapacity => "pair_exceeds_capacity",
            Self::CounterOverflow => "counter_overflow",
        };
        write!(formatter, "retained_pair_storage={category}")
    }
}

impl Error for RetainedPairStorageError {}

impl From<RetainedPairError> for RetainedPairStorageError {
    fn from(error: RetainedPairError) -> Self {
        match error {
            RetainedPairError::EmptyRecord
            | RetainedPairError::EmbeddedLineBreak
            | RetainedPairError::SizeOverflow => Self::InvalidPair,
            RetainedPairError::StorageUnavailable => Self::StorageUnavailable,
            RetainedPairError::PairExceedsCapacity => Self::PairExceedsCapacity,
            RetainedPairError::CounterOverflow => Self::CounterOverflow,
        }
    }
}

/// Retains one complete operator-snapshot correlation pair under one mutex acquisition.
pub fn retain_operator_snapshot_pair(
    marker: &str,
    runtime_health: &str,
) -> Result<(), RetainedPairStorageError> {
    let pair = RetainedPair::try_new(marker, runtime_health)?;
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(firmware_log_buffer()));
    let mut buffer = buffer
        .lock()
        .map_err(|_| RetainedPairStorageError::MutexPoisoned)?;
    buffer.try_append_pair(&pair).map_err(Into::into)
}

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

/// Returns complete allowlisted Plan 13 markers for diagnostic replay.
#[must_use]
pub fn accepted_state_replay_lines() -> Vec<String> {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(firmware_log_buffer()));
    let Ok(buffer) = buffer.lock() else {
        log::warn!("retained_log_buffer=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    plan13_replay_lines(&buffer)
}

fn plan13_replay_lines(buffer: &RetainedLogBuffer) -> Vec<String> {
    ["plan13_boot_evidence", "accepted_state_snapshot"]
        .into_iter()
        .flat_map(|token| buffer.complete_lines_with_first_token(token))
        .collect()
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

#[cfg(test)]
pub fn install_retained_log_buffer_for_test(replacement: RetainedLogBuffer) {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(RetainedLogBuffer::empty()));
    let mut current = match buffer.lock() {
        Ok(current) => current,
        Err(poisoned) => poisoned.into_inner(),
    };
    *current = replacement;
    buffer.clear_poison();
}

#[cfg(test)]
#[must_use]
pub fn retained_text_for_test() -> String {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(RetainedLogBuffer::empty()));
    let current = match buffer.lock() {
        Ok(current) => current,
        Err(poisoned) => poisoned.into_inner(),
    };
    current.download_chunks().concat()
}

#[cfg(test)]
pub fn poison_retained_log_buffer_for_test() {
    let buffer = LOG_BUFFER.get_or_init(|| Mutex::new(RetainedLogBuffer::empty()));
    let result = std::thread::spawn(move || {
        let _guard = buffer
            .lock()
            .expect("test retained-log mutex should start healthy");
        panic!("poison retained-log mutex for production-path regression");
    })
    .join();
    assert!(result.is_err(), "poisoning thread should panic");
}

#[cfg(test)]
pub fn reset_retained_log_buffer_after_poison_for_test() {
    install_retained_log_buffer_for_test(RetainedLogBuffer::empty());
}

#[cfg(test)]
mod tests {
    use super::plan13_replay_lines;
    use bitaxe_api::RetainedLogBuffer;

    #[test]
    fn plan13_replay_selects_only_allowlisted_complete_lines() {
        // Arrange
        let mut buffer = RetainedLogBuffer::with_capacity(1024);
        buffer.append("plan13_boot_evidence session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa state=booted redacted=true\n");
        buffer.append("secret=value\n");
        buffer.append("accepted_state_snapshot stage=post_enumerate redacted=true\n");
        buffer.append("plan13_boot_evidence partial=true");

        // Act
        let lines = plan13_replay_lines(&buffer);

        // Assert
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("plan13_boot_evidence "));
        assert!(lines[1].starts_with("accepted_state_snapshot "));
    }
}
