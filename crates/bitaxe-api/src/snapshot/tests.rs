use bitaxe_safety::evidence::SafetyCriticalEvidence;
use bitaxe_safety::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
    UnavailableReason,
};

use crate::{
    ApiSnapshot, ConfigSnapshot, ObservationStateWire, SafeTelemetrySnapshot,
    SafetyTelemetryReport, SafetyTelemetryStatus, TelemetryObservations,
};

#[test]
fn api_snapshot_contains_typed_input_fields_without_platform_sdk_dependencies() {
    // Arrange
    let snapshot = ApiSnapshot::safe_ultra_205();

    // Act
    let config = snapshot.config;
    let catalog = snapshot.catalog;
    let mining = snapshot.mining;
    let block_found = snapshot.block_found;
    let asic = snapshot.asic;
    let platform = snapshot.platform;

    // Assert
    assert_eq!(config.defaults.asic_model(), "BM1366");
    assert_eq!(catalog.board_version(), "205");
    assert_eq!(mining.counters.accepted, 0);
    assert_eq!(block_found.block_found, 0);
    assert!(!block_found.show_new_block);
    assert_eq!(asic.maybe_detected_chips, Some(1));
    assert_eq!(platform.hostname, "bitaxe");
}

#[test]
fn config_snapshot_uses_ultra_205_defaults() {
    // Arrange
    let config = ConfigSnapshot::ultra_205();

    // Act
    let defaults = config.defaults;

    // Assert
    assert_eq!(defaults.board_version(), "205");
    assert_eq!(defaults.asic_frequency_mhz(), 485);
    assert_eq!(defaults.asic_voltage_mv(), 1200);
}

#[test]
fn safety_telemetry_model_safe_ultra_205_is_explicit_unavailable() {
    // Arrange
    let snapshot = ApiSnapshot::safe_ultra_205();

    // Act
    let telemetry = snapshot.safe_telemetry;

    // Assert
    assert_eq!(
        telemetry.status,
        SafetyTelemetryStatus::Unavailable {
            reason: "safety_telemetry_unavailable"
        }
    );
    assert_eq!(telemetry.evidence, SafetyCriticalEvidence::Missing);
    assert_eq!(telemetry.power_watts, 0.0);
    assert_eq!(telemetry.fan_rpm, 0);
}

#[test]
fn safety_telemetry_model_fresh_legacy_report_cannot_publish_unstamped_values() {
    // Arrange
    let report = fresh_report(SafetyCriticalEvidence::hardware_smoke(
        "phase-06-api-telemetry-smoke",
    ));

    // Act
    let snapshot = SafeTelemetrySnapshot::from_report(report);

    // Assert
    assert_eq!(
        snapshot.status,
        SafetyTelemetryStatus::Unavailable {
            reason: "legacy_telemetry_unstamped"
        }
    );
    assert_eq!(snapshot.evidence, report.evidence);
    assert_eq!(snapshot.power_watts, 0.0);
    assert_eq!(snapshot.voltage_volts, 0.0);
    assert_eq!(snapshot.current_amps, 0.0);
    assert_eq!(snapshot.chip_temp_celsius, 0.0);
    assert_eq!(snapshot.fan_rpm, 0);
    assert_eq!(
        snapshot.power_status.state,
        ObservationStateWire::Unavailable
    );
}

#[test]
fn safety_telemetry_projection_compatibility_zero_does_not_authenticate_truth() {
    // Arrange
    let unavailable = TelemetryObservations::default();
    let fresh_zero = TelemetryObservations {
        power_watts: fresh_f64_observation(0.0, 1),
        ..TelemetryObservations::default()
    };

    // Act
    let unavailable_projection = SafeTelemetrySnapshot::from_observations(&unavailable);
    let fresh_projection = SafeTelemetrySnapshot::from_observations(&fresh_zero);

    // Assert
    assert_eq!(unavailable_projection.power_watts, 0.0);
    assert_eq!(fresh_projection.power_watts, 0.0);
    assert_eq!(
        unavailable_projection.power_status.state,
        ObservationStateWire::Unavailable
    );
    assert_eq!(
        fresh_projection.power_status.state,
        ObservationStateWire::Fresh
    );
}

#[test]
fn safety_telemetry_projection_preserves_mixed_independent_states() {
    // Arrange
    let fresh_power = fresh_f64_observation(10.0, 1);
    let stale_voltage = fresh_f64_observation(5.0, 2)
        .mark_stale(StaleReason::PowerSampleStale)
        .expect("fresh voltage can become stale");
    let unavailable_current = Observation::unavailable(UnavailableReason::PowerSampleUnavailable);
    let fault_temperature =
        Observation::<f64>::unavailable(UnavailableReason::ThermalReadingUnavailable)
            .record_fault(FaultReason::ThermalReadingInvalid);
    let fresh_vr = fresh_f64_observation(42.0, 3);
    let fresh_fan = fresh_u16_observation(3_200, 4);
    let observations = TelemetryObservations {
        power_watts: fresh_power,
        bus_voltage_volts: stale_voltage,
        current_amps: unavailable_current,
        chip_temp_celsius: fault_temperature,
        vr_temp_celsius: fresh_vr,
        fan_rpm: fresh_fan,
    };

    // Act
    let projection = SafeTelemetrySnapshot::from_observations(&observations);

    // Assert
    assert_eq!(projection.power_status.state, ObservationStateWire::Fresh);
    assert_eq!(projection.voltage_status.state, ObservationStateWire::Stale);
    assert_eq!(
        projection.current_status.state,
        ObservationStateWire::Unavailable
    );
    assert_eq!(
        projection.chip_temp_status.state,
        ObservationStateWire::Fault
    );
    assert_eq!(projection.vr_temp_status.state, ObservationStateWire::Fresh);
    assert_eq!(projection.fan_rpm_status.state, ObservationStateWire::Fresh);
    assert_eq!(projection.power_watts, 10.0);
    assert_eq!(projection.voltage_volts, 0.0);
    assert_eq!(projection.vr_temp_celsius, 42.0);
    assert_eq!(projection.fan_rpm, 3_200);
}

#[test]
fn safety_telemetry_model_d17_stale_fault_unavailable_zero_numeric_projection() {
    // Arrange
    let stale = SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Stale {
            reason: "power_sample_stale",
        },
        ..fresh_report(SafetyCriticalEvidence::hardware_smoke(
            "phase-06-api-telemetry-smoke",
        ))
    };
    let fault = SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Fault {
            reason: "thermal_reading_invalid",
        },
        ..fresh_report(SafetyCriticalEvidence::hardware_smoke(
            "phase-06-api-telemetry-smoke",
        ))
    };
    let unavailable = SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Unavailable {
            reason: "safety_telemetry_unavailable",
        },
        ..fresh_report(SafetyCriticalEvidence::Missing)
    };

    // Act
    let projections = [
        SafeTelemetrySnapshot::from_report(stale),
        SafeTelemetrySnapshot::from_report(fault),
        SafeTelemetrySnapshot::from_report(unavailable),
    ];

    // Assert
    for projection in projections {
        assert_eq!(projection.power_watts, 0.0);
        assert_eq!(projection.voltage_volts, 0.0);
        assert_eq!(projection.current_amps, 0.0);
        assert_eq!(projection.fan_rpm, 0);
        assert_ne!(projection.status, SafetyTelemetryStatus::Fresh);
    }
}

#[test]
fn safety_telemetry_model_d18_fresh_unit_evidence_does_not_claim_hardware_values() {
    // Arrange
    let report = fresh_report(SafetyCriticalEvidence::implemented_not_verified("unit"));

    // Act
    let snapshot = SafeTelemetrySnapshot::from_report(report);

    // Assert
    assert_eq!(
        snapshot.status,
        SafetyTelemetryStatus::Unavailable {
            reason: "legacy_telemetry_unstamped"
        }
    );
    assert_eq!(snapshot.evidence, report.evidence);
    assert_eq!(snapshot.power_watts, 0.0);
    assert_eq!(snapshot.chip_temp_celsius, 0.0);
}

fn fresh_report(evidence: SafetyCriticalEvidence) -> SafetyTelemetryReport {
    SafetyTelemetryReport {
        status: SafetyTelemetryStatus::Fresh,
        evidence,
        power_watts: 11.5,
        voltage_volts: 5.1,
        current_amps: 2.25,
        chip_temp_celsius: 56.0,
        chip_temp2_celsius: 57.0,
        vr_temp_celsius: 45.0,
        core_voltage_actual_mv: 1_198.0,
        actual_frequency_mhz: 485.0,
        expected_hashrate_ghs: 525.0,
        fan_speed_percent: 70,
        fan_rpm: 3_200,
        fan2_rpm: 0,
        wifi_rssi_dbm: -50,
    }
}

fn fresh_f64_observation(value: f64, prior_sequence: u64) -> Observation<f64> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(prior_sequence),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance")
    .0
}

fn fresh_u16_observation(value: u16, prior_sequence: u64) -> Observation<u16> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(prior_sequence),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance")
    .0
}
