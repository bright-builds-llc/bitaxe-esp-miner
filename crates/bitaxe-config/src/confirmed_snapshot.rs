//! Thread-safe ownership of the last storage-confirmed settings snapshot.

use std::fmt;
use std::sync::Mutex;

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

/// Process-lifetime cell for the last atomically published settings snapshot.
#[derive(Default)]
pub struct ConfirmedSnapshotCell {
    snapshot: Mutex<NvsSnapshot>,
}

impl fmt::Debug for ConfirmedSnapshotCell {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let health = if self.snapshot.is_poisoned() {
            ConfirmedSnapshotReadHealth::PoisonRecovered
        } else {
            ConfirmedSnapshotReadHealth::Healthy
        };
        formatter
            .debug_struct("ConfirmedSnapshotCell")
            .field("health", &health)
            .finish_non_exhaustive()
    }
}

impl ConfirmedSnapshotCell {
    /// Creates a cell with an initial confirmed snapshot.
    #[must_use]
    pub const fn new(snapshot: NvsSnapshot) -> Self {
        Self {
            snapshot: Mutex::new(snapshot),
        }
    }

    /// Clones the last confirmed snapshot, retaining the inner value after poison.
    #[must_use]
    pub fn read(&self) -> ConfirmedSnapshotRead {
        match self.snapshot.lock() {
            Ok(snapshot) => ConfirmedSnapshotRead {
                snapshot: snapshot.clone(),
                health: ConfirmedSnapshotReadHealth::Healthy,
            },
            Err(poisoned) => ConfirmedSnapshotRead {
                snapshot: poisoned.into_inner().clone(),
                health: ConfirmedSnapshotReadHealth::PoisonRecovered,
            },
        }
    }

    /// Atomically publishes a newly confirmed snapshot.
    pub fn publish(
        &self,
        snapshot: NvsSnapshot,
    ) -> Result<(), ConfirmedSnapshotPublicationFailure> {
        let mut current = self
            .snapshot
            .lock()
            .map_err(|_| ConfirmedSnapshotPublicationFailure)?;
        *current = snapshot;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{ConfirmedSnapshotReadHealth, NvsSnapshot, StoredValue};

    use super::ConfirmedSnapshotCell;

    #[test]
    fn poisoned_cell_retains_the_inner_confirmed_snapshot() {
        // Arrange
        let expected = NvsSnapshot::from_values([StoredValue::string("hostname", "confirmed")]);
        let cell = Arc::new(ConfirmedSnapshotCell::new(expected.clone()));
        let poisoner = Arc::clone(&cell);
        let poison_result = std::thread::spawn(move || {
            let _guard = poisoner
                .snapshot
                .lock()
                .expect("test lock should start healthy");
            panic!("poison confirmed snapshot lock for regression coverage");
        })
        .join();
        assert!(poison_result.is_err());

        // Act
        let read = cell.read();

        // Assert
        assert_eq!(read.health(), ConfirmedSnapshotReadHealth::PoisonRecovered);
        assert_eq!(read.into_snapshot(), expected);
    }

    #[test]
    fn public_wrapper_debug_exposes_only_health_after_poison() {
        // Arrange
        let secret_sentinel = "phase33-debug-secret-sentinel";
        let cell = Arc::new(ConfirmedSnapshotCell::new(NvsSnapshot::from_values([
            StoredValue::string("wifiPass", secret_sentinel),
        ])));
        let poisoner = Arc::clone(&cell);
        let poison_result = std::thread::spawn(move || {
            let _guard = poisoner
                .snapshot
                .lock()
                .expect("test lock should start healthy");
            panic!("poison confirmed snapshot lock for debug regression coverage");
        })
        .join();
        assert!(poison_result.is_err());

        // Act
        let read_debug = format!("{:?}", cell.read());
        let cell_debug = format!("{cell:?}");

        // Assert
        for debug_text in [read_debug, cell_debug] {
            assert!(debug_text.contains("PoisonRecovered"));
            assert!(!debug_text.contains(secret_sentinel));
            assert!(!debug_text.contains("wifiPass"));
        }
    }
}
