//! Stored observation boundary for request-side firmware consumers.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::{ObservationStore, TelemetryObservations};

static OBSERVATION_STORE: OnceLock<Mutex<ObservationStore>> = OnceLock::new();

fn store() -> &'static Mutex<ObservationStore> {
    OBSERVATION_STORE.get_or_init(|| Mutex::new(ObservationStore::default()))
}

pub(crate) fn observation_snapshot() -> TelemetryObservations {
    let Ok(store) = store().lock() else {
        log::warn!("observation_store=unavailable category=mutex_poisoned");
        return TelemetryObservations::default();
    };

    store.read()
}

pub(crate) fn replace_observations_from_producer(observations: TelemetryObservations) {
    let Ok(mut store) = store().lock() else {
        log::warn!("observation_store=unavailable category=mutex_poisoned");
        return;
    };

    store.replace(observations);
}

#[cfg(test)]
mod tests {
    #[test]
    fn observation_store_source_has_no_acquisition_or_control_boundary() {
        // Arrange
        let source = include_str!("observation_store.rs");
        let forbidden_terms = [
            ["esp", "_idf"].concat(),
            ["i", "2c"].concat(),
            ["g", "pio"].concat(),
            ["f", "an"].concat(),
            ["volt", "age"].concat(),
            ["re", "set"].concat(),
        ];

        // Act / Assert
        for forbidden_term in forbidden_terms {
            assert!(!source.contains(&forbidden_term));
        }
    }
}
