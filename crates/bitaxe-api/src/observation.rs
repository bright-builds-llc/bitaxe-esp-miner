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

    /// Returns unavailable truth for a retained source that has no valid
    /// producer-owned session, sequence, or acquisition time.
    #[must_use]
    pub const fn unavailable_from_unstamped_legacy_source() -> Self {
        Self::unavailable(UnavailableReason::ProducerUnavailable)
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

/// Projects one fact out of an observation without changing producer metadata.
#[must_use]
pub fn project_observation<T, U: Copy>(
    observation: &Observation<T>,
    project: impl Fn(&T) -> Option<U> + Copy,
    missing_reason: UnavailableReason,
) -> Observation<U> {
    match observation {
        Observation::Fresh { sample } => project_sample(sample, project).map_or_else(
            || Observation::unavailable(missing_reason),
            |sample| Observation::Fresh { sample },
        ),
        Observation::Stale { last_good, reason } => project_sample(last_good, project).map_or_else(
            || Observation::unavailable(missing_reason),
            |last_good| Observation::Stale {
                last_good,
                reason: *reason,
            },
        ),
        Observation::Unavailable { reason } => Observation::unavailable(*reason),
        Observation::Fault {
            reason,
            maybe_last_good,
        } => Observation::Fault {
            reason: *reason,
            maybe_last_good: maybe_last_good
                .as_ref()
                .and_then(|sample| project_sample(sample, project)),
        },
    }
}

fn project_sample<T, U>(
    sample: &StampedSample<T>,
    project: impl Fn(&T) -> Option<U>,
) -> Option<StampedSample<U>> {
    let maybe_value = project(sample.value());
    maybe_value.map(|value| {
        StampedSample::new(
            value,
            sample.boot_session(),
            sample.sequence(),
            sample.acquired_at(),
        )
    })
}

#[cfg(test)]
mod tests {
    use bitaxe_safety::observation::{BootSessionId, MonotonicMillis, ObservationSequence};
    use serde_json::json;

    use super::*;
    use crate::{
        ApiSnapshot, LiveTelemetryPlanner, SafeTelemetrySnapshot, StatisticsSample, SystemInfoWire,
    };

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

    fn fresh_u16(value: u16) -> Observation<u16> {
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

    #[test]
    fn unstamped_legacy_source_cannot_publish_fresh_operator_truth() {
        // Arrange
        let observations = TelemetryObservations::unavailable_from_unstamped_legacy_source();

        // Act
        let truths = [
            observations.power_watts.state_label(),
            observations.bus_voltage_volts.state_label(),
            observations.current_amps.state_label(),
            observations.chip_temp_celsius.state_label(),
            observations.vr_temp_celsius.state_label(),
            observations.fan_rpm.state_label(),
        ];
        let stamps = [
            observations.power_watts.maybe_last_good(),
            observations.bus_voltage_volts.maybe_last_good(),
            observations.current_amps.maybe_last_good(),
            observations.chip_temp_celsius.maybe_last_good(),
            observations.vr_temp_celsius.maybe_last_good(),
        ];

        // Assert
        assert_eq!(truths, ["unavailable"; 6]);
        assert!(stamps.into_iter().all(|maybe_stamp| maybe_stamp.is_none()));
        assert!(observations.fan_rpm.maybe_last_good().is_none());
    }

    #[test]
    fn projection_repeated_consumer_reads_leave_store_and_stamps_unchanged() {
        // Arrange
        let observations = TelemetryObservations {
            power_watts: fresh(10.0),
            bus_voltage_volts: fresh(5.0),
            current_amps: fresh(2.0),
            chip_temp_celsius: fresh(55.0),
            vr_temp_celsius: fresh(42.0),
            fan_rpm: fresh_u16(3_200),
        };
        let store = ObservationStore::new(observations);
        let before = store.read();

        // Act
        let mut first_snapshot = ApiSnapshot::safe_ultra_205();
        first_snapshot.safe_telemetry = SafeTelemetrySnapshot::from_observations(&store.read());
        let first_system = SystemInfoWire::from_snapshot(&first_snapshot);
        let first_payload = serde_json::to_value(&first_system).expect("wire should serialize");
        let first_statistics = StatisticsSample::from_snapshot(&first_snapshot, 1, 0.0);
        let first_projection_bytes = serde_json::to_vec(&[
            first_system.power_status,
            first_system.voltage_status,
            first_system.current_status,
            first_system.chip_temp_status,
            first_system.vr_temp_status,
            first_system.fan_rpm_status,
        ])
        .expect("truth projection should serialize");
        let mut websocket = LiveTelemetryPlanner::default();
        websocket.set_active_client_count(1);
        websocket.seed_cadence_baseline(first_payload.clone());
        let websocket_read = websocket.cadence_frame(first_payload);

        let mut second_snapshot = ApiSnapshot::safe_ultra_205();
        second_snapshot.safe_telemetry = SafeTelemetrySnapshot::from_observations(&store.read());
        let second_system = SystemInfoWire::from_snapshot(&second_snapshot);
        let second_statistics = StatisticsSample::from_snapshot(&second_snapshot, 1, 0.0);
        let second_projection_bytes = serde_json::to_vec(&[
            second_system.power_status,
            second_system.voltage_status,
            second_system.current_status,
            second_system.chip_temp_status,
            second_system.vr_temp_status,
            second_system.fan_rpm_status,
        ])
        .expect("truth projection should serialize");
        let after = store.read();

        // Assert
        assert_eq!(before, observations);
        assert_eq!(after, observations);
        assert_eq!(before, after);
        assert_eq!(first_system, second_system);
        assert_eq!(first_statistics, second_statistics);
        assert_eq!(first_projection_bytes, second_projection_bytes);
        assert!(websocket_read.is_none());
    }

    #[test]
    fn projection_store_advances_only_metadata_supplied_by_producer() {
        // Arrange
        let initial = TelemetryObservations {
            power_watts: fresh(10.0),
            ..TelemetryObservations::default()
        };
        let mut store = ObservationStore::new(initial);
        let (next_power, _) = Observation::record_success(
            11.0,
            BootSessionId::new(7),
            ObservationSequence::new(10),
            MonotonicMillis::new(500),
        )
        .expect("fixture sequence should advance");
        let replacement = TelemetryObservations {
            power_watts: next_power,
            ..initial
        };

        // Act
        store.replace(replacement);
        let stored = store.read();

        // Assert
        assert_eq!(
            stored
                .power_watts
                .maybe_last_good()
                .expect("power should be fresh")
                .sequence()
                .get(),
            11
        );
        assert_eq!(stored.current_amps, initial.current_amps);
        assert_eq!(stored.chip_temp_celsius, initial.chip_temp_celsius);
    }

    #[test]
    fn phase32_consumer_reads_preserve_failed_source_and_unaffected_fresh_facts() {
        // Arrange
        let failed_temperature = fresh(55.0).record_fault(FaultReason::ReadFailed);
        let observations = TelemetryObservations {
            power_watts: fresh(10.0),
            bus_voltage_volts: fresh(5.0),
            current_amps: fresh(2.0),
            chip_temp_celsius: failed_temperature,
            vr_temp_celsius: Observation::unavailable(UnavailableReason::ThermalReadingUnavailable),
            fan_rpm: fresh_u16(3_200),
        };
        let mut store = ObservationStore::new(observations);

        // Act
        let first = store.read();
        let second = store.read();
        let mut first_snapshot = ApiSnapshot::safe_ultra_205();
        first_snapshot.safe_telemetry = SafeTelemetrySnapshot::from_observations(&first);
        let first_wire = SystemInfoWire::from_snapshot(&first_snapshot);
        let (next_temperature, _) = Observation::record_success(
            56.0,
            BootSessionId::new(7),
            ObservationSequence::new(10),
            MonotonicMillis::new(500),
        )
        .expect("producer replacement sequence should advance");
        store.replace(TelemetryObservations {
            chip_temp_celsius: next_temperature,
            ..second
        });
        let replaced = store.read();

        // Assert
        assert_eq!(first, observations);
        assert_eq!(second, observations);
        assert_eq!(first.chip_temp_celsius.state_label(), "fault");
        assert!(first.power_watts.is_fresh());
        assert!(first.fan_rpm.is_fresh());
        assert_eq!(
            first_wire.chip_temp_status.state,
            ObservationStateWire::Fault
        );
        assert_eq!(first_wire.power_status.state, ObservationStateWire::Fresh);
        assert_eq!(first_wire.fan_rpm_status.state, ObservationStateWire::Fresh);
        assert_eq!(replaced.power_watts, observations.power_watts);
        assert_eq!(replaced.fan_rpm, observations.fan_rpm);
        assert_eq!(
            replaced
                .chip_temp_celsius
                .maybe_last_good()
                .expect("producer replacement should be fresh")
                .sequence()
                .get(),
            11
        );
    }

    #[test]
    fn projection_mapping_copies_state_and_stamp_without_advancing_metadata() {
        // Arrange
        let source = fresh(5.0)
            .mark_stale(StaleReason::ProducerTimeout)
            .expect("fresh fixture can become stale");
        let expected_stamp = ObservationTruthWire::from(&source).stamp;

        // Act
        let projected = project_observation(
            &source,
            |value| Some(*value * 2.0),
            UnavailableReason::ProducerUnavailable,
        );

        // Assert
        assert_eq!(projected.state_label(), "stale");
        assert_eq!(
            projected.maybe_last_good().map(StampedSample::value),
            Some(&10.0)
        );
        assert_eq!(ObservationTruthWire::from(&projected).stamp, expected_stamp);
    }
}
