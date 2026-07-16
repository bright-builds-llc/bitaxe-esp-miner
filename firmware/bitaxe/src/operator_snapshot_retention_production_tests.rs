#[path = "log_buffer.rs"]
mod log_buffer;
#[path = "operator_snapshot_retention.rs"]
mod operator_snapshot_retention;

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, OnceLock};

use bitaxe_api::{
    BootSessionId, OperatorSnapshotIdentity, OperatorSnapshotLockHealth,
    OperatorSnapshotPublishError, OperatorSnapshotPublisher, RetainedLogBuffer,
};

const MARKER: &str = "operator_snapshot session=opaque revision=1 redacted=true";
const RUNTIME_HEALTH: &str = "runtime_health status=healthy redacted=true";

#[derive(Debug, Eq, PartialEq)]
struct DistinctIssueError;

impl std::fmt::Display for DistinctIssueError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("distinct_issue")
    }
}

#[derive(Debug)]
struct CompletedPublication {
    identity: OperatorSnapshotIdentity,
    marker: String,
    runtime_health: String,
}

fn complete_publication(_: (), identity: OperatorSnapshotIdentity) -> CompletedPublication {
    CompletedPublication {
        identity,
        marker: identity.retained_marker(),
        runtime_health: format!(
            "runtime_health boot_session={} operator_snapshot_revision={} redacted=true",
            identity.boot_session(),
            identity.revision()
        ),
    }
}

fn publish_with_production_retention(
    publisher: &OperatorSnapshotPublisher,
    issue_called: &Cell<bool>,
) -> Result<
    bitaxe_api::OperatorSnapshotPublication<u64>,
    OperatorSnapshotPublishError<log_buffer::RetainedPairStorageError, DistinctIssueError>,
> {
    publisher.publish(
        BootSessionId::from_words([1, 2, 3, 4]),
        || (),
        complete_publication,
        |publication| {
            operator_snapshot_retention::retain_completed_operator_snapshot(
                &publication.marker,
                &publication.runtime_health,
            )
        },
        |publication| {
            issue_called.set(true);
            Ok(publication.identity.revision().get())
        },
    )
}

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

#[test]
fn publisher_skips_issue_on_unavailable_retention_and_consumes_revision() {
    // Arrange
    let _guard = serial_test_guard();
    let publisher = OperatorSnapshotPublisher::new();
    let first_issue_called = Cell::new(false);
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::empty());

    // Act
    let error = publish_with_production_retention(&publisher, &first_issue_called)
        .expect_err("unavailable retention must fail before issue");
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(1024));
    let second_issue_called = Cell::new(false);
    let next = publish_with_production_retention(&publisher, &second_issue_called)
        .expect("usable retention must permit the next publication");

    // Assert
    assert!(matches!(
        error,
        OperatorSnapshotPublishError::Retention {
            source: log_buffer::RetainedPairStorageError::StorageUnavailable,
            lock_health: OperatorSnapshotLockHealth::Healthy,
        }
    ));
    assert!(!first_issue_called.get());
    assert!(second_issue_called.get());
    assert_eq!(next.output, 2);
    let retained = log_buffer::retained_text_for_test();
    assert!(!retained.contains("revision=1"));
    assert!(retained.contains("revision=2"));
}

#[test]
fn publisher_skips_issue_on_poisoned_retention() {
    // Arrange
    let _guard = serial_test_guard();
    let publisher = OperatorSnapshotPublisher::new();
    let issue_called = Cell::new(false);
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(1024));
    log_buffer::poison_retained_log_buffer_for_test();

    // Act
    let error = publish_with_production_retention(&publisher, &issue_called)
        .expect_err("poisoned retention must fail before issue");

    // Assert
    assert!(matches!(
        error,
        OperatorSnapshotPublishError::Retention {
            source: log_buffer::RetainedPairStorageError::MutexPoisoned,
            lock_health: OperatorSnapshotLockHealth::Healthy,
        }
    ));
    assert!(!issue_called.get());
    log_buffer::reset_retained_log_buffer_after_poison_for_test();
}

#[test]
fn publisher_skips_issue_on_insufficient_retention_capacity() {
    // Arrange
    let _guard = serial_test_guard();
    let publisher = OperatorSnapshotPublisher::new();
    let issue_called = Cell::new(false);
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::with_capacity(1));

    // Act
    let error = publish_with_production_retention(&publisher, &issue_called)
        .expect_err("undersized retention must fail before issue");

    // Assert
    assert!(matches!(
        error,
        OperatorSnapshotPublishError::Retention {
            source: log_buffer::RetainedPairStorageError::PairExceedsCapacity,
            lock_health: OperatorSnapshotLockHealth::Healthy,
        }
    ));
    assert!(!issue_called.get());
    assert!(log_buffer::retained_text_for_test().is_empty());
}

#[test]
fn publisher_retention_error_rendering_contains_only_closed_categories() {
    // Arrange
    let _guard = serial_test_guard();
    let publisher = OperatorSnapshotPublisher::new();
    let issue_called = Cell::new(false);
    log_buffer::install_retained_log_buffer_for_test(RetainedLogBuffer::empty());

    // Act
    let error = publish_with_production_retention(&publisher, &issue_called)
        .expect_err("unavailable retention must fail before issue");
    let debug = format!("{error:?}");
    let display = format!("{error}");

    // Assert
    for secret in [
        "00000001000000020000000300000004",
        "revision=1",
        "runtime_health",
        "/dev/",
        "http://",
        "password",
    ] {
        assert!(!debug.contains(secret), "debug leaked {secret}");
        assert!(!display.contains(secret), "display leaked {secret}");
    }
    assert!(display.contains("retained_pair_storage=storage_unavailable"));
    assert!(!issue_called.get());
}
