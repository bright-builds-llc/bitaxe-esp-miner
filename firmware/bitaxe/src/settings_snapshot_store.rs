//! Firmware-owned synchronization for the last storage-confirmed settings snapshot.

use std::fmt;
use std::sync::Mutex;

use bitaxe_config::{
    ConfirmedSnapshotPublicationFailure, ConfirmedSnapshotRead, ConfirmedSnapshotReadHealth,
    NvsSnapshot,
};

/// Process-lifetime store for the last atomically published settings snapshot.
#[derive(Default)]
pub(crate) struct ConfirmedSnapshotStore {
    snapshot: Mutex<NvsSnapshot>,
}

impl fmt::Debug for ConfirmedSnapshotStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let health = if self.snapshot.is_poisoned() {
            ConfirmedSnapshotReadHealth::PoisonRecovered
        } else {
            ConfirmedSnapshotReadHealth::Healthy
        };
        formatter
            .debug_struct("ConfirmedSnapshotStore")
            .field("health", &health)
            .finish_non_exhaustive()
    }
}

impl ConfirmedSnapshotStore {
    /// Creates a store with an initial confirmed snapshot.
    #[must_use]
    pub(crate) const fn new(snapshot: NvsSnapshot) -> Self {
        Self {
            snapshot: Mutex::new(snapshot),
        }
    }

    /// Clones the last confirmed snapshot, retaining the inner value after poison.
    #[must_use]
    pub(crate) fn read(&self) -> ConfirmedSnapshotRead {
        match self.snapshot.lock() {
            Ok(snapshot) => {
                ConfirmedSnapshotRead::new(snapshot.clone(), ConfirmedSnapshotReadHealth::Healthy)
            }
            Err(poisoned) => ConfirmedSnapshotRead::new(
                poisoned.into_inner().clone(),
                ConfirmedSnapshotReadHealth::PoisonRecovered,
            ),
        }
    }

    /// Atomically publishes a newly confirmed snapshot.
    pub(crate) fn publish(
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

    use bitaxe_config::{ConfirmedSnapshotReadHealth, NvsSnapshot, StoredValue};

    use super::ConfirmedSnapshotStore;

    #[test]
    fn publication_replaces_the_confirmed_snapshot_atomically() {
        // Arrange
        let initial = NvsSnapshot::from_values([StoredValue::string("hostname", "initial")]);
        let confirmed = NvsSnapshot::from_values([StoredValue::string("hostname", "confirmed")]);
        let store = ConfirmedSnapshotStore::new(initial);

        // Act
        store
            .publish(confirmed.clone())
            .expect("healthy store must accept a confirmed snapshot");
        let read = store.read();

        // Assert
        assert_eq!(read.health(), ConfirmedSnapshotReadHealth::Healthy);
        assert_eq!(read.into_snapshot(), confirmed);
    }

    #[test]
    fn poisoned_store_retains_the_inner_confirmed_snapshot() {
        // Arrange
        let expected = NvsSnapshot::from_values([StoredValue::string("hostname", "confirmed")]);
        let store = Arc::new(ConfirmedSnapshotStore::new(expected.clone()));
        let poisoner = Arc::clone(&store);
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
        let read = store.read();

        // Assert
        assert_eq!(read.health(), ConfirmedSnapshotReadHealth::PoisonRecovered);
        assert_eq!(read.into_snapshot(), expected);
    }

    #[test]
    fn public_wrapper_debug_exposes_only_health_after_poison() {
        // Arrange
        let secret_sentinel = "phase33-debug-secret-sentinel";
        let store = Arc::new(ConfirmedSnapshotStore::new(NvsSnapshot::from_values([
            StoredValue::string("wifiPass", secret_sentinel),
        ])));
        let poisoner = Arc::clone(&store);
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
        let read_debug = format!("{:?}", store.read());
        let store_debug = format!("{store:?}");

        // Assert
        for debug_text in [read_debug, store_debug] {
            assert!(debug_text.contains("PoisonRecovered"));
            assert!(!debug_text.contains(secret_sentinel));
            assert!(!debug_text.contains("wifiPass"));
        }
    }
}
