//! Pure result and failure models for storage-confirmed settings snapshots.

use std::fmt;

use crate::NvsSnapshot;

/// Health of a confirmed-snapshot read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmedSnapshotReadHealth {
    /// The snapshot lock was acquired normally.
    Healthy,
    /// The lock was poisoned, but its last inner snapshot was retained.
    PoisonRecovered,
}

/// A cloned confirmed snapshot plus its lock-health classification.
#[derive(Clone, PartialEq, Eq)]
pub struct ConfirmedSnapshotRead {
    snapshot: NvsSnapshot,
    health: ConfirmedSnapshotReadHealth,
}

impl fmt::Debug for ConfirmedSnapshotRead {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfirmedSnapshotRead")
            .field("health", &self.health)
            .finish_non_exhaustive()
    }
}

impl ConfirmedSnapshotRead {
    /// Builds a read result at the firmware synchronization boundary.
    #[must_use]
    pub const fn new(snapshot: NvsSnapshot, health: ConfirmedSnapshotReadHealth) -> Self {
        Self { snapshot, health }
    }

    /// Returns the lock-health classification without exposing raw failure details.
    #[must_use]
    pub const fn health(&self) -> ConfirmedSnapshotReadHealth {
        self.health
    }

    /// Consumes the read result and returns the retained confirmed snapshot.
    #[must_use]
    pub fn into_snapshot(self) -> NvsSnapshot {
        self.snapshot
    }
}

/// Failure to publish a new confirmed snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmedSnapshotPublicationFailure;
