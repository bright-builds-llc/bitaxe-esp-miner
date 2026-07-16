//! Serialized completion and issuance for operator-visible snapshots.

use std::cell::Cell;
use std::fmt;
use std::sync::Mutex;

use crate::{BootSessionId, OperatorSnapshotIdentity, OperatorSnapshotSequence};

thread_local! {
    static PUBLICATION_DEPTH: Cell<u8> = const { Cell::new(0) };
}

/// Boot-lifetime authority for completion-ordered operator snapshot publication.
#[derive(Debug, Default)]
pub struct OperatorSnapshotPublisher {
    sequence: Mutex<OperatorSnapshotSequence>,
}

/// Health of the ordering mutex acquired for a publication attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorSnapshotLockHealth {
    /// The ordering mutex had not been poisoned.
    Healthy,
    /// A prior unwind poisoned the mutex; its existing sequence was recovered.
    RecoveredPoison,
}

/// Successful publication output together with ordering-lock health.
#[derive(Debug, Eq, PartialEq)]
pub struct OperatorSnapshotPublication<T> {
    /// Value returned by the final issuance adapter.
    pub output: T,
    /// Health observed while acquiring the ordering mutex.
    pub lock_health: OperatorSnapshotLockHealth,
}

/// Fail-closed publication failure classified by stage.
#[derive(Debug, Eq, PartialEq)]
pub enum OperatorSnapshotPublishError<RetentionError, IssueError> {
    /// Same-thread recursion was rejected before candidate collection.
    Reentrant,
    /// The within-boot revision sequence cannot advance without wrapping.
    SequenceExhausted {
        /// Health observed while acquiring the ordering mutex.
        lock_health: OperatorSnapshotLockHealth,
    },
    /// Retained chronology could not be appended, so issuance was skipped.
    Retention {
        /// Adapter-local failure.
        source: RetentionError,
        /// Health observed while acquiring the ordering mutex.
        lock_health: OperatorSnapshotLockHealth,
    },
    /// Final external issuance failed after retained chronology was appended.
    Issuance {
        /// Adapter-local failure.
        source: IssueError,
        /// Health observed while acquiring the ordering mutex.
        lock_health: OperatorSnapshotLockHealth,
    },
}

impl<RetentionError, IssueError> OperatorSnapshotPublishError<RetentionError, IssueError> {
    /// Returns ordering-lock health when the failing attempt acquired the lock.
    #[must_use]
    pub const fn maybe_lock_health(&self) -> Option<OperatorSnapshotLockHealth> {
        match self {
            Self::Reentrant => None,
            Self::SequenceExhausted { lock_health }
            | Self::Retention { lock_health, .. }
            | Self::Issuance { lock_health, .. } => Some(*lock_health),
        }
    }
}

impl<RetentionError: fmt::Display, IssueError: fmt::Display> fmt::Display
    for OperatorSnapshotPublishError<RetentionError, IssueError>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reentrant => formatter.write_str("operator snapshot publication is reentrant"),
            Self::SequenceExhausted { .. } => {
                formatter.write_str("operator snapshot revision sequence exhausted")
            }
            Self::Retention { source, .. } => {
                write!(formatter, "operator snapshot retention failed: {source}")
            }
            Self::Issuance { source, .. } => {
                write!(formatter, "operator snapshot issuance failed: {source}")
            }
        }
    }
}

impl OperatorSnapshotPublisher {
    /// Starts a publisher before revision one.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sequence: Mutex::new(OperatorSnapshotSequence::new()),
        }
    }

    /// Collects outside ordering, then serializes identity, retention, and issuance.
    pub fn publish<Candidate, Publication, T, RetentionError, IssueError>(
        &self,
        boot_session: BootSessionId,
        collect: impl FnOnce() -> Candidate,
        complete: impl FnOnce(Candidate, OperatorSnapshotIdentity) -> Publication,
        retain: impl FnOnce(&Publication) -> Result<(), RetentionError>,
        issue: impl FnOnce(Publication) -> Result<T, IssueError>,
    ) -> Result<
        OperatorSnapshotPublication<T>,
        OperatorSnapshotPublishError<RetentionError, IssueError>,
    > {
        let Some(_depth_guard) = PublicationDepthGuard::enter() else {
            return Err(OperatorSnapshotPublishError::Reentrant);
        };
        let candidate = collect();
        let (mut sequence, lock_health) = match self.sequence.lock() {
            Ok(sequence) => (sequence, OperatorSnapshotLockHealth::Healthy),
            Err(poisoned) => {
                let sequence = poisoned.into_inner();
                self.sequence.clear_poison();
                (sequence, OperatorSnapshotLockHealth::RecoveredPoison)
            }
        };
        let identity = sequence
            .next_identity(boot_session)
            .map_err(|_| OperatorSnapshotPublishError::SequenceExhausted { lock_health })?;
        let publication = complete(candidate, identity);
        retain(&publication).map_err(|source| OperatorSnapshotPublishError::Retention {
            source,
            lock_health,
        })?;
        let output =
            issue(publication).map_err(|source| OperatorSnapshotPublishError::Issuance {
                source,
                lock_health,
            })?;

        Ok(OperatorSnapshotPublication {
            output,
            lock_health,
        })
    }

    #[cfg(test)]
    fn exhausted_for_test() -> Self {
        Self {
            sequence: Mutex::new(OperatorSnapshotSequence::with_last_revision_for_test(
                u64::MAX,
            )),
        }
    }
}

struct PublicationDepthGuard;

impl PublicationDepthGuard {
    fn enter() -> Option<Self> {
        PUBLICATION_DEPTH.with(|depth| {
            if depth.get() != 0 {
                return None;
            }
            depth.set(1);
            Some(Self)
        })
    }
}

impl Drop for PublicationDepthGuard {
    fn drop(&mut self) {
        PUBLICATION_DEPTH.with(|depth| depth.set(0));
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;

    use super::*;

    fn session() -> BootSessionId {
        BootSessionId::from_words([1, 2, 3, 4])
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Completed {
        candidate: &'static str,
        identity: OperatorSnapshotIdentity,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct DistinctRetentionError;

    impl fmt::Display for DistinctRetentionError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("retention")
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct DistinctIssueError;

    impl fmt::Display for DistinctIssueError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("issuance")
        }
    }

    fn complete(candidate: &'static str, identity: OperatorSnapshotIdentity) -> Completed {
        Completed {
            candidate,
            identity,
        }
    }

    #[test]
    fn retention_and_issuance_preserve_distinct_error_sources() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::new();

        // Act
        let retention_error = publisher
            .publish(
                session(),
                || "retention",
                complete,
                |_| Err(DistinctRetentionError),
                |_| Ok::<(), DistinctIssueError>(()),
            )
            .expect_err("retention failure must be preserved");
        let issuance_error = publisher
            .publish(
                session(),
                || "issuance",
                complete,
                |_| Ok::<(), DistinctRetentionError>(()),
                |_| Err::<(), DistinctIssueError>(DistinctIssueError),
            )
            .expect_err("issuance failure must be preserved");

        // Assert
        assert!(matches!(
            retention_error,
            OperatorSnapshotPublishError::Retention {
                source: DistinctRetentionError,
                lock_health: OperatorSnapshotLockHealth::Healthy,
            }
        ));
        assert!(matches!(
            issuance_error,
            OperatorSnapshotPublishError::Issuance {
                source: DistinctIssueError,
                lock_health: OperatorSnapshotLockHealth::Healthy,
            }
        ));
    }

    #[test]
    fn reverse_collection_completion_publishes_direct_revisions_in_order() {
        // Arrange
        let publisher = Arc::new(OperatorSnapshotPublisher::new());
        let issued = Arc::new(Mutex::new(Vec::new()));
        let (first_collect_entered_tx, first_collect_entered_rx) = mpsc::channel();
        let (release_first_tx, release_first_rx) = mpsc::channel();
        let first_publisher = Arc::clone(&publisher);
        let first_issued = Arc::clone(&issued);

        // Act
        let first = thread::spawn(move || {
            first_publisher.publish(
                session(),
                || {
                    first_collect_entered_tx
                        .send(())
                        .expect("first collection entry must be observable");
                    release_first_rx
                        .recv()
                        .expect("first collection release must arrive");
                    "capture-1"
                },
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| {
                    first_issued
                        .lock()
                        .expect("issued history must be available")
                        .push((publication.candidate, publication.identity.revision().get()));
                    Ok::<(), &'static str>(())
                },
            )
        });
        first_collect_entered_rx
            .recv()
            .expect("first collection must enter");
        let second_publisher = Arc::clone(&publisher);
        let second_issued = Arc::clone(&issued);
        let second = thread::spawn(move || {
            second_publisher.publish(
                session(),
                || "capture-2",
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| {
                    second_issued
                        .lock()
                        .expect("issued history must be available")
                        .push((publication.candidate, publication.identity.revision().get()));
                    Ok::<(), &'static str>(())
                },
            )
        });
        second
            .join()
            .expect("second publisher thread must not panic")
            .expect("second publication must succeed");
        release_first_tx
            .send(())
            .expect("first collection must be releasable");
        first
            .join()
            .expect("first publisher thread must not panic")
            .expect("first publication must succeed");

        // Assert
        assert_eq!(
            *issued.lock().expect("issued history must be available"),
            [("capture-2", 1), ("capture-1", 2)]
        );
    }

    #[test]
    fn second_completion_waits_until_first_issue_returns() {
        // Arrange
        let publisher = Arc::new(OperatorSnapshotPublisher::new());
        let (issue_entered_tx, issue_entered_rx) = mpsc::channel();
        let (release_issue_tx, release_issue_rx) = mpsc::channel();
        let (second_collected_tx, second_collected_rx) = mpsc::channel();
        let (second_completed_tx, second_completed_rx) = mpsc::channel();
        let first_publisher = Arc::clone(&publisher);

        // Act
        let first = thread::spawn(move || {
            first_publisher.publish(
                session(),
                || "first",
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| {
                    issue_entered_tx
                        .send(publication.identity.revision().get())
                        .expect("issue entry must be observable");
                    release_issue_rx.recv().expect("issue release must arrive");
                    Ok::<(), &'static str>(())
                },
            )
        });
        assert_eq!(issue_entered_rx.recv().expect("issue must enter"), 1);
        let second_publisher = Arc::clone(&publisher);
        let second = thread::spawn(move || {
            second_publisher.publish(
                session(),
                || {
                    second_collected_tx
                        .send(())
                        .expect("collection must be observable");
                    "second"
                },
                |candidate, identity| {
                    second_completed_tx
                        .send(())
                        .expect("completion must be observable");
                    complete(candidate, identity)
                },
                |_| Ok::<(), &'static str>(()),
                |_| Ok::<(), &'static str>(()),
            )
        });
        second_collected_rx
            .recv()
            .expect("second collection must finish outside the ordering mutex");

        // Assert
        assert!(second_completed_rx.try_recv().is_err());
        release_issue_tx.send(()).expect("issue must be releasable");
        first
            .join()
            .expect("first thread must not panic")
            .expect("first publication must succeed");
        second
            .join()
            .expect("second thread must not panic")
            .expect("second publication must succeed");
        second_completed_rx
            .recv()
            .expect("second completion must enter after issue returns");
    }

    #[test]
    fn retention_failure_skips_issue_and_consumes_revision() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::new();
        let issued = Cell::new(false);

        // Act
        let error = publisher
            .publish(
                session(),
                || "failed-retention",
                complete,
                |_| Err("retain"),
                |_| {
                    issued.set(true);
                    Ok::<(), &'static str>(())
                },
            )
            .expect_err("retention failure must fail publication");
        let next = publisher
            .publish(
                session(),
                || "next",
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| Ok::<u64, &'static str>(publication.identity.revision().get()),
            )
            .expect("next publication must succeed");

        // Assert
        assert!(matches!(
            error,
            OperatorSnapshotPublishError::Retention {
                source: "retain",
                lock_health: OperatorSnapshotLockHealth::Healthy
            }
        ));
        assert!(!issued.get());
        assert_eq!(next.output, 2);
    }

    #[test]
    fn issuance_failure_releases_lock_and_consumes_revision() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::new();

        // Act
        let error = publisher
            .publish(
                session(),
                || "failed-issue",
                complete,
                |_| Ok::<(), &'static str>(()),
                |_| Err::<(), &'static str>("issue"),
            )
            .expect_err("issuance failure must fail publication");
        let next = publisher
            .publish(
                session(),
                || "next",
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| Ok::<u64, &'static str>(publication.identity.revision().get()),
            )
            .expect("next publication must succeed");

        // Assert
        assert!(matches!(
            error,
            OperatorSnapshotPublishError::Issuance {
                source: "issue",
                lock_health: OperatorSnapshotLockHealth::Healthy
            }
        ));
        assert_eq!(next.output, 2);
    }

    #[test]
    fn poison_recovery_preserves_sequence_and_reports_health() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::new();
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = publisher.publish(
                session(),
                || "poison",
                complete,
                |_| Ok::<(), &'static str>(()),
                |_| -> Result<(), &'static str> { panic!("poison publication mutex") },
            );
        }));

        // Act
        let next = publisher
            .publish(
                session(),
                || "next",
                complete,
                |_| Ok::<(), &'static str>(()),
                |publication| Ok::<u64, &'static str>(publication.identity.revision().get()),
            )
            .expect("poison recovery must succeed");

        // Assert
        assert!(panic_result.is_err());
        assert_eq!(next.output, 2);
        assert_eq!(
            next.lock_health,
            OperatorSnapshotLockHealth::RecoveredPoison
        );
    }

    #[test]
    fn reentrant_publication_fails_before_nested_collection() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::new();
        let nested_collected = Cell::new(false);

        // Act
        let outer = publisher.publish(
            session(),
            || "outer",
            complete,
            |_| Ok::<(), &'static str>(()),
            |_| {
                let nested = publisher.publish(
                    session(),
                    || {
                        nested_collected.set(true);
                        "nested"
                    },
                    complete,
                    |_| Ok::<(), &'static str>(()),
                    |_| Ok::<(), &'static str>(()),
                );
                assert!(matches!(
                    nested,
                    Err(OperatorSnapshotPublishError::Reentrant)
                ));
                Ok::<(), &'static str>(())
            },
        );

        // Assert
        outer.expect("outer publication must succeed");
        assert!(!nested_collected.get());
    }

    #[test]
    fn sequence_exhaustion_fails_without_running_completion_or_issue() {
        // Arrange
        let publisher = OperatorSnapshotPublisher::exhausted_for_test();
        let completed = Cell::new(false);
        let issued = Cell::new(false);

        // Act
        let error = publisher
            .publish(
                session(),
                || "candidate",
                |candidate, identity| {
                    completed.set(true);
                    complete(candidate, identity)
                },
                |_| Ok::<(), &'static str>(()),
                |_| {
                    issued.set(true);
                    Ok::<(), &'static str>(())
                },
            )
            .expect_err("exhausted sequence must fail");

        // Assert
        assert!(matches!(
            error,
            OperatorSnapshotPublishError::SequenceExhausted {
                lock_health: OperatorSnapshotLockHealth::Healthy
            }
        ));
        assert!(!completed.get());
        assert!(!issued.get());
    }
}
