//! Pure result and failure models for operator-visible snapshot publication.

use std::fmt;

/// Health of the firmware-owned ordering mutex acquired for a publication attempt.
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
