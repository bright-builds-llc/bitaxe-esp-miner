#[path = "log_buffer.rs"]
mod log_buffer;
#[path = "operator_snapshot_retention.rs"]
mod operator_snapshot_retention;

use std::sync::{Mutex, MutexGuard, OnceLock};

use bitaxe_api::RetainedLogBuffer;

const MARKER: &str = "operator_snapshot session=opaque revision=1 redacted=true";
const RUNTIME_HEALTH: &str = "runtime_health status=healthy redacted=true";

fn serial_test_guard() -> MutexGuard<'static, ()> {
    static SERIAL: OnceLock<Mutex<()>> = OnceLock::new();
    SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn production_retention_rejects_unavailable_storage_without_partial_pair() {
    // Arrange
    let _guard = serial_test_guard();
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::empty());

    // Act
    let result = operator_snapshot_retention::retain_completed_operator_snapshot(
        MARKER,
        RUNTIME_HEALTH,
    );

    // Assert
    assert_eq!(result, Err(log_buffer::RetainedPairStorageError::StorageUnavailable));
    let retained = log_buffer::retained_text_for_test();
    assert!(!retained.contains("operator_snapshot"));
    assert!(!retained.contains("runtime_health"));
}

#[test]
fn production_retention_rejects_insufficient_capacity_without_partial_pair() {
    // Arrange
    let _guard = serial_test_guard();
    let required_bytes = MARKER.len() + 1 + RUNTIME_HEALTH.len() + 1;
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(
        required_bytes - 1,
    ));

    // Act
    let result = operator_snapshot_retention::retain_completed_operator_snapshot(
        MARKER,
        RUNTIME_HEALTH,
    );

    // Assert
    assert_eq!(result, Err(log_buffer::RetainedPairStorageError::PairExceedsCapacity));
    let retained = log_buffer::retained_text_for_test();
    assert!(!retained.contains("operator_snapshot"));
    assert!(!retained.contains("runtime_health"));
}

#[test]
fn production_retention_appends_complete_pair_in_order() {
    // Arrange
    let _guard = serial_test_guard();
    let required_bytes = MARKER.len() + 1 + RUNTIME_HEALTH.len() + 1;
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(
        required_bytes,
    ));

    // Act
    let result = operator_snapshot_retention::retain_completed_operator_snapshot(
        MARKER,
        RUNTIME_HEALTH,
    );

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        log_buffer::retained_text_for_test(),
        format!("{MARKER}\n{RUNTIME_HEALTH}\n")
    );
}

#[test]
fn production_retention_reports_poison_without_recovery_or_partial_pair() {
    // Arrange
    let _guard = serial_test_guard();
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(1024));
    log_buffer::poison_retained_log_buffer_for_test();

    // Act
    let result = operator_snapshot_retention::retain_completed_operator_snapshot(
        MARKER,
        RUNTIME_HEALTH,
    );

    // Assert
    assert_eq!(result, Err(log_buffer::RetainedPairStorageError::MutexPoisoned));
    assert_eq!(format!("{result:?}"), "Err(MutexPoisoned)");
    log_buffer::reset_retained_log_buffer_after_poison_for_test();
}
