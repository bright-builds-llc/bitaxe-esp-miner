//! Read-only API projection of producer-owned observation truth.

use bitaxe_safety::observation::{
    FaultReason, Observation, StaleReason, StampedSample, UnavailableReason,
};
use serde::{Deserialize, Serialize};

/// Stable public state labels for one observed fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStateWire {
    Fresh,
    Stale,
    Unavailable,
    Fault,
}

/// Producer-owned provenance copied to the public contract without mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationStampWire {
    pub boot_session: u64,
    pub sequence: u64,
    pub acquired_at_ms: u64,
}

impl<T> From<&StampedSample<T>> for ObservationStampWire {
    fn from(sample: &StampedSample<T>) -> Self {
        Self {
            boot_session: sample.boot_session().get(),
            sequence: sample.sequence().get(),
            acquired_at_ms: sample.acquired_at().get(),
        }
    }
}

/// Typed and redaction-safe reason attached to a non-fresh observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "code", rename_all = "snake_case")]
pub enum ObservationReasonWire {
    Stale(StaleReason),
    Unavailable(UnavailableReason),
    Fault(FaultReason),
}

/// Truth-only wire projection. Numeric compatibility values live elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationTruthWire {
    pub state: ObservationStateWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamp: Option<ObservationStampWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ObservationReasonWire>,
}

impl<T> From<&Observation<T>> for ObservationTruthWire {
    fn from(observation: &Observation<T>) -> Self {
        match observation {
            Observation::Fresh { sample } => Self {
                state: ObservationStateWire::Fresh,
                stamp: Some(sample.into()),
                reason: None,
            },
            Observation::Stale { last_good, reason } => Self {
                state: ObservationStateWire::Stale,
                stamp: Some(last_good.into()),
                reason: Some(ObservationReasonWire::Stale(*reason)),
            },
            Observation::Unavailable { reason } => Self {
                state: ObservationStateWire::Unavailable,
                stamp: None,
                reason: Some(ObservationReasonWire::Unavailable(*reason)),
            },
            Observation::Fault {
                reason,
                maybe_last_good,
            } => Self {
                state: ObservationStateWire::Fault,
                stamp: maybe_last_good.as_ref().map(Into::into),
                reason: Some(ObservationReasonWire::Fault(*reason)),
            },
        }
    }
}

/// Complete stored observation state consumed by operator projections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetryObservations {
    pub power_watts: Observation<f64>,
    pub bus_voltage_volts: Observation<f64>,
    pub current_amps: Observation<f64>,
    pub chip_temp_celsius: Observation<f64>,
    pub vr_temp_celsius: Observation<f64>,
    pub fan_rpm: Observation<u16>,
}

impl TelemetryObservations {
    #[must_use]
    pub const fn unavailable(reason: UnavailableReason) -> Self {
        Self {
            power_watts: Observation::unavailable(reason),
            bus_voltage_volts: Observation::unavailable(reason),
            current_amps: Observation::unavailable(reason),
            chip_temp_celsius: Observation::unavailable(reason),
            vr_temp_celsius: Observation::unavailable(reason),
            fan_rpm: Observation::unavailable(reason),
        }
    }
}

impl Default for TelemetryObservations {
    fn default() -> Self {
        Self::unavailable(UnavailableReason::NotYetObserved)
    }
}

/// Host-testable store whose reads only copy already-stamped observations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObservationStore {
    observations: TelemetryObservations,
}

impl ObservationStore {
    #[must_use]
    pub const fn new(observations: TelemetryObservations) -> Self {
        Self { observations }
    }

    #[must_use]
    pub const fn read(&self) -> TelemetryObservations {
        self.observations
    }

    pub fn replace(&mut self, observations: TelemetryObservations) {
        self.observations = observations;
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_safety::observation::{BootSessionId, MonotonicMillis, ObservationSequence};
    use serde_json::json;

    use super::*;

    fn fresh(value: f64) -> Observation<f64> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(9),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    #[test]
    fn safety_telemetry_truth_serializes_exact_state_and_stamp_names() {
        // Arrange
        let fresh = ObservationTruthWire::from(&fresh(0.0));

        // Act
        let value = serde_json::to_value(fresh).expect("truth should serialize");

        // Assert
        assert_eq!(value["state"], "fresh");
        assert_eq!(value["stamp"]["bootSession"], 7);
        assert_eq!(value["stamp"]["sequence"], 10);
        assert_eq!(value["stamp"]["acquiredAtMs"], 250);
        assert!(value.get("reason").is_none());
    }

    #[test]
    fn safety_telemetry_truth_serializes_exact_four_state_vocabulary() {
        // Arrange
        let fresh = fresh(1.0);
        let stale = fresh
            .mark_stale(StaleReason::ProducerTimeout)
            .expect("fresh fixture can become stale");
        let unavailable = Observation::<f64>::unavailable(UnavailableReason::NotYetObserved);
        let fault = unavailable.record_fault(FaultReason::ReadFailed);

        // Act
        let states = [fresh, stale, unavailable, fault]
            .iter()
            .map(|observation| {
                serde_json::to_value(ObservationTruthWire::from(observation))
                    .expect("truth should serialize")["state"]
                    .clone()
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            states,
            [
                json!("fresh"),
                json!("stale"),
                json!("unavailable"),
                json!("fault")
            ]
        );
    }

    #[test]
    fn safety_telemetry_truth_preserves_stale_and_fault_last_good_stamps() {
        // Arrange
        let fresh = fresh(4.5);
        let stale = fresh
            .mark_stale(StaleReason::ProducerTimeout)
            .expect("fresh fixture can become stale");
        let fault = stale.record_fault(FaultReason::ReadFailed);
        let expected_stamp = ObservationTruthWire::from(&fresh).stamp;

        // Act
        let stale_truth = ObservationTruthWire::from(&stale);
        let fault_truth = ObservationTruthWire::from(&fault);

        // Assert
        assert_eq!(stale_truth.state, ObservationStateWire::Stale);
        assert_eq!(fault_truth.state, ObservationStateWire::Fault);
        assert_eq!(stale_truth.stamp, expected_stamp);
        assert_eq!(fault_truth.stamp, expected_stamp);
    }

    #[test]
    fn safety_telemetry_truth_fault_without_last_good_has_no_stamp() {
        // Arrange
        let unavailable = Observation::<f64>::unavailable(UnavailableReason::NotYetObserved);

        // Act
        let truth = ObservationTruthWire::from(&unavailable.record_fault(FaultReason::ReadFailed));
        let value = serde_json::to_value(truth).expect("truth should serialize");

        // Assert
        assert_eq!(value["state"], "fault");
        assert!(value.get("stamp").is_none());
        assert_eq!(
            value["reason"],
            json!({ "kind": "fault", "code": "read_failed" })
        );
    }

    #[test]
    fn projection_store_repeated_reads_preserve_supplied_observations() {
        // Arrange
        let observations = TelemetryObservations {
            power_watts: fresh(0.0),
            ..TelemetryObservations::default()
        };
        let store = ObservationStore::new(observations);

        // Act
        let first = store.read();
        let second = store.read();

        // Assert
        assert_eq!(first, observations);
        assert_eq!(second, observations);
        assert_eq!(first, second);
    }

    #[test]
    fn projection_store_replacement_uses_complete_supplied_snapshot() {
        // Arrange
        let mut store = ObservationStore::default();
        let replacement = TelemetryObservations {
            current_amps: fresh(2.0),
            ..TelemetryObservations::default()
        };

        // Act
        store.replace(replacement);

        // Assert
        assert_eq!(store.read(), replacement);
    }
}
