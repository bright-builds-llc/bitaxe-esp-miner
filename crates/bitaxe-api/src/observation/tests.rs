use bitaxe_safety::observation::{BootSessionId, MonotonicMillis, ObservationSequence};
use serde_json::json;

use super::*;
use crate::{
    ApiSnapshot, LiveTelemetryPlanner, SafeTelemetrySnapshot, SafetyTelemetryStatus,
    StatisticsSample, SystemInfoWire,
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

fn safe_mining_observations() -> TelemetryObservations {
    TelemetryObservations {
        power_watts: fresh(15.0),
        bus_voltage_volts: fresh(5.5),
        current_amps: fresh(3.0),
        core_voltage_actual_mv: fresh(1_200.0),
        chip_temp_celsius: fresh(74.0),
        vr_temp_celsius: fresh(45.0),
        fan_rpm: fresh_u16(0),
    }
}

#[test]
fn mining_safety_requires_every_supported_ultra205_observation() {
    // Arrange
    let safe = TelemetryObservations {
        vr_temp_celsius: Observation::unavailable(UnavailableReason::ThermalReadingUnavailable),
        ..safe_mining_observations()
    };
    let faulted_fan = safe.fan_rpm.record_fault(FaultReason::ReadFailed);
    let cases = [
        TelemetryObservations {
            power_watts: Observation::unavailable(UnavailableReason::PowerSampleUnavailable),
            ..safe
        },
        TelemetryObservations {
            bus_voltage_volts: Observation::unavailable(UnavailableReason::PowerSampleUnavailable),
            ..safe
        },
        TelemetryObservations {
            current_amps: Observation::unavailable(UnavailableReason::PowerSampleUnavailable),
            ..safe
        },
        TelemetryObservations {
            chip_temp_celsius: Observation::unavailable(
                UnavailableReason::ThermalReadingUnavailable,
            ),
            ..safe
        },
        TelemetryObservations {
            fan_rpm: faulted_fan,
            ..safe
        },
    ];

    // Act / Assert
    assert!(safe.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250)));
    assert_eq!(
        SafeTelemetrySnapshot::from_observations(&safe).status,
        SafetyTelemetryStatus::Fresh
    );
    assert!(
        cases
            .into_iter()
            .all(|observations| !observations
                .is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250)))
    );
}

#[test]
fn mining_safety_enforces_voltage_power_temperature_and_numeric_limits() {
    // Arrange
    let safe = safe_mining_observations();
    let lower_voltage_boundary = TelemetryObservations {
        bus_voltage_volts: fresh(4.5),
        ..safe
    };
    let unsafe_cases = [
        TelemetryObservations {
            bus_voltage_volts: fresh(4.499),
            ..safe
        },
        TelemetryObservations {
            bus_voltage_volts: fresh(5.501),
            ..safe
        },
        TelemetryObservations {
            power_watts: fresh(15.001),
            ..safe
        },
        TelemetryObservations {
            chip_temp_celsius: fresh(75.0),
            ..safe
        },
        TelemetryObservations {
            current_amps: fresh(f64::NAN),
            ..safe
        },
    ];

    // Act / Assert
    assert!(safe.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250)));
    assert!(lower_voltage_boundary.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250)));
    assert!(
        unsafe_cases
            .into_iter()
            .all(|observations| !observations
                .is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250)))
    );
}

#[test]
fn mining_safety_rejects_fresh_state_after_the_one_second_sample_window() {
    // Arrange
    let observations = safe_mining_observations();

    // Act
    let at_boundary = observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_250));
    let beyond_boundary = observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_251));

    // Assert
    assert!(at_boundary);
    assert!(!beyond_boundary);
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
        observations.core_voltage_actual_mv.state_label(),
        observations.chip_temp_celsius.state_label(),
        observations.vr_temp_celsius.state_label(),
        observations.fan_rpm.state_label(),
    ];
    let stamps = [
        observations.power_watts.maybe_last_good(),
        observations.bus_voltage_volts.maybe_last_good(),
        observations.current_amps.maybe_last_good(),
        observations.core_voltage_actual_mv.maybe_last_good(),
        observations.chip_temp_celsius.maybe_last_good(),
        observations.vr_temp_celsius.maybe_last_good(),
    ];

    // Assert
    assert_eq!(truths, ["unavailable"; 7]);
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
        core_voltage_actual_mv: fresh(1_198.0),
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
    let websocket_connect = websocket.connect_frame(first_payload.clone());
    websocket.seed_cadence_baseline(first_payload.clone());
    let websocket_read = websocket.maybe_cadence_frame(first_payload.clone());

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
    assert_eq!(first_payload["vrTemp"], json!(42.0));
    assert_eq!(first_payload["vrTempStatus"]["state"], json!("fresh"));
    assert_eq!(websocket_connect["data"]["vrTemp"], json!(42.0));
    assert_eq!(
        websocket_connect["data"]["vrTempStatus"]["state"],
        json!("fresh")
    );
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
        core_voltage_actual_mv: fresh(1_198.0),
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
fn phase32_consumer_failure_isolation_covers_each_sensor_source() {
    // Arrange
    #[derive(Clone, Copy)]
    enum FailedSource {
        Power,
        AsicTemperature,
        VrTemperature,
        Tachometer,
    }

    for failed_source in [
        FailedSource::Power,
        FailedSource::AsicTemperature,
        FailedSource::VrTemperature,
        FailedSource::Tachometer,
    ] {
        let power_watts = if matches!(failed_source, FailedSource::Power) {
            fresh(10.0).record_fault(FaultReason::ReadFailed)
        } else {
            fresh(10.0)
        };
        let bus_voltage_volts = if matches!(failed_source, FailedSource::Power) {
            fresh(5.0).record_fault(FaultReason::ReadFailed)
        } else {
            fresh(5.0)
        };
        let current_amps = if matches!(failed_source, FailedSource::Power) {
            fresh(2.0).record_fault(FaultReason::ReadFailed)
        } else {
            fresh(2.0)
        };
        let chip_temp_celsius = if matches!(failed_source, FailedSource::AsicTemperature) {
            fresh(55.0).record_fault(FaultReason::ReadFailed)
        } else {
            fresh(55.0)
        };
        let vr_temp_celsius = if matches!(failed_source, FailedSource::VrTemperature) {
            fresh(45.0).record_fault(FaultReason::ReadFailed)
        } else {
            fresh(45.0)
        };
        let fan_rpm = if matches!(failed_source, FailedSource::Tachometer) {
            fresh_u16(3_200).record_fault(FaultReason::ReadFailed)
        } else {
            fresh_u16(3_200)
        };
        let observations = TelemetryObservations {
            power_watts,
            bus_voltage_volts,
            current_amps,
            core_voltage_actual_mv: fresh(1_198.0),
            chip_temp_celsius,
            vr_temp_celsius,
            fan_rpm,
        };
        let store = ObservationStore::new(observations);

        // Act
        let first = store.read();
        let second = store.read();
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.safe_telemetry = SafeTelemetrySnapshot::from_observations(&first);
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(first, observations);
        assert_eq!(second, observations);
        match failed_source {
            FailedSource::Power => {
                assert_eq!(wire.power_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.voltage_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.current_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.chip_temp_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.fan_rpm_status.state, ObservationStateWire::Fresh);
            }
            FailedSource::AsicTemperature => {
                assert_eq!(wire.chip_temp_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.vr_temp_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.power_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.fan_rpm_status.state, ObservationStateWire::Fresh);
            }
            FailedSource::VrTemperature => {
                assert_eq!(wire.vr_temp_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.chip_temp_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.power_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.fan_rpm_status.state, ObservationStateWire::Fresh);
            }
            FailedSource::Tachometer => {
                assert_eq!(wire.fan_rpm_status.state, ObservationStateWire::Fault);
                assert_eq!(wire.power_status.state, ObservationStateWire::Fresh);
                assert_eq!(wire.chip_temp_status.state, ObservationStateWire::Fresh);
            }
        }
    }
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
