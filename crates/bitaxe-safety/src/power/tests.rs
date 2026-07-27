use serde_json::Value;

use bitaxe_config::catalog::{board_catalog, ultra_205_catalog_entry};

use super::*;

#[test]
fn safety_power_fresh_ina260_observation_produces_evidence_token() {
    // Arrange
    let sample = safe_sample();

    // Act
    let observation = PowerObservation::from_ina260_sample(
        Some(sample),
        PowerSampleAgeMs(100),
        f64::from(ultra_205_catalog_entry().power_consumption_target()),
    );
    let decision = PowerSafetyDecision::from_observation(observation);

    // Assert
    assert_eq!(observation.truth().state_label(), "fresh");
    assert!(decision.maybe_evidence.is_some());
    assert_eq!(decision.plan.status, SafetyStatus::Normal);
}

#[test]
fn safety_power_stale_missing_faulted_and_unsafe_observations_fail_closed() {
    // Arrange
    let cases = [
        (
            PowerObservation::from_ina260_sample(Some(safe_sample()), PowerSampleAgeMs(1001), 12.0),
            PowerFaultReason::Stale.as_str(),
        ),
        (
            PowerObservation::from_ina260_sample(None, PowerSampleAgeMs(0), 12.0),
            PowerFaultReason::Unavailable.as_str(),
        ),
        (
            PowerObservation::from_ina260_sample(
                Some(Ina260RawSample {
                    read_failed: true,
                    ..safe_sample()
                }),
                PowerSampleAgeMs(100),
                12.0,
            ),
            PowerFaultReason::Ina260ReadFailed.as_str(),
        ),
        (
            PowerObservation::from_ina260_sample(
                Some(Ina260RawSample {
                    bus_voltage_volts: 5.6,
                    ..safe_sample()
                }),
                PowerSampleAgeMs(100),
                12.0,
            ),
            PowerFaultReason::InputVoltageUnsafe.as_str(),
        ),
        (
            PowerObservation::from_ina260_sample(
                Some(Ina260RawSample {
                    power_watts: 16.0,
                    ..safe_sample()
                }),
                PowerSampleAgeMs(100),
                12.0,
            ),
            PowerFaultReason::PowerLimitExceeded.as_str(),
        ),
        (
            PowerObservation::from_ina260_sample(
                Some(Ina260RawSample {
                    current_amps: f64::NAN,
                    ..safe_sample()
                }),
                PowerSampleAgeMs(100),
                12.0,
            ),
            PowerFaultReason::PowerReadingInvalid.as_str(),
        ),
    ];

    // Act / Assert
    for (observation, expected_reason) in cases {
        let decision = PowerSafetyDecision::from_observation(observation);
        assert_eq!(
            decision.plan.status,
            SafetyStatus::SafeBlocked {
                reason: expected_reason
            }
        );
        assert!(decision.maybe_evidence.is_none());
        assert!(decision
            .plan
            .effects
            .contains(&SafetyEffect::BlockWorkSubmission {
                reason: expected_reason
            }));
    }
}

#[test]
fn safety_power_truth_preserves_last_good_across_stale_and_fault_states() {
    // Arrange
    let (fresh, sequence) = PowerObservation::from_stamped_ina260_sample(
        Some(safe_sample()),
        PowerSampleAgeMs(100),
        12.0,
        BootSessionId::new(7),
        ObservationSequence::new(9),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance");
    let expected = fresh
        .truth()
        .maybe_last_good()
        .expect("fresh power should own a sample")
        .to_owned();

    // Act
    let stale = fresh
        .mark_stale(StaleReason::PowerSampleStale)
        .expect("fresh power can become stale");
    let fault = stale.record_fault(FaultReason::ReadFailed);

    // Assert
    assert_eq!(sequence, ObservationSequence::new(10));
    assert_eq!(stale.truth().maybe_last_good(), Some(&expected));
    assert_eq!(fault.truth().maybe_last_good(), Some(&expected));
    assert_eq!(stale.truth().state_label(), "stale");
    assert_eq!(fault.truth().state_label(), "fault");
}

#[test]
fn safety_power_unavailable_and_invalid_attempts_publish_no_numeric_truth() {
    // Arrange
    let unavailable = PowerObservation::from_ina260_sample(None, PowerSampleAgeMs(0), 12.0);
    let invalid = PowerObservation::from_ina260_sample(
        Some(Ina260RawSample {
            current_amps: f64::NAN,
            ..safe_sample()
        }),
        PowerSampleAgeMs(100),
        12.0,
    );

    // Act
    let compatibility_values = [
        unavailable.bus_voltage_volts(),
        unavailable.current_amps(),
        unavailable.power_watts(),
        invalid.bus_voltage_volts(),
        invalid.current_amps(),
        invalid.power_watts(),
    ];

    // Assert
    assert_eq!(unavailable.truth().state_label(), "unavailable");
    assert_eq!(invalid.truth().state_label(), "fault");
    assert!(unavailable.truth().maybe_last_good().is_none());
    assert!(invalid.truth().maybe_last_good().is_none());
    assert_eq!(compatibility_values, [0.0; 6]);
}

#[test]
fn voltage_effect_observe_only_suppresses_write_without_hardware_evidence() {
    // Arrange
    let inputs = VoltageControllerInputs {
        requested_mv: 1200,
        board: ultra_205_catalog_entry(),
        observation: fresh_observation(),
        evidence: SafetyCriticalEvidence::implemented_not_verified("unit"),
        actuation_mode: VoltageActuationMode::ObserveOnly,
    };

    // Act
    let decision = inputs.plan();

    // Assert
    assert_eq!(
        decision.voltage_plan,
        VoltageEffectPlan::NoWrite {
            reason: "observe_only_hardware_evidence_missing"
        }
    );
    assert!(decision
        .safety_plan
        .effects
        .contains(&SafetyEffect::SuppressVoltageWrite));
}

#[test]
fn voltage_effect_invalid_setpoint_or_missing_capability_suppresses_write() {
    // Arrange
    let missing_ds4432u = board_catalog()
        .iter()
        .copied()
        .find(|board| !board.capabilities().ds4432u())
        .expect("fixture catalog should contain non-DS4432U boards");
    let cases = [
        (999, ultra_205_catalog_entry(), "invalid_voltage_setpoint"),
        (1200, missing_ds4432u, "ds4432u_capability_missing"),
    ];

    // Act / Assert
    for (requested_mv, board, expected_reason) in cases {
        let decision = VoltageControllerInputs {
            requested_mv,
            board,
            observation: fresh_observation(),
            evidence: SafetyCriticalEvidence::hardware_smoke("phase-06-ultra-205-safety"),
            actuation_mode: VoltageActuationMode::ArmedWithHardwareEvidence,
        }
        .plan();

        assert_eq!(
            decision.voltage_plan,
            VoltageEffectPlan::SuppressWrite {
                reason: expected_reason
            }
        );
        assert!(decision
            .safety_plan
            .effects
            .contains(&SafetyEffect::HoldResetLow));
    }
}

#[test]
fn voltage_effect_write_requires_supported_voltage_fresh_power_and_hardware_evidence() {
    // Arrange
    let hardware_evidence = SafetyCriticalEvidence::hardware_regression(
        "phase-06-ultra-205-safety-hardware-regression",
    );

    // Act
    let decision = VoltageControllerInputs {
        requested_mv: 1200,
        board: ultra_205_catalog_entry(),
        observation: fresh_observation(),
        evidence: hardware_evidence,
        actuation_mode: VoltageActuationMode::ArmedWithHardwareEvidence,
    }
    .plan();

    // Assert
    assert_eq!(
        decision.voltage_plan,
        VoltageEffectPlan::WriteDs4432u {
            i2c_address: DS4432U_I2C_ADDRESS,
            output_registers: [DS4432U_OUTPUT0_REGISTER, DS4432U_OUTPUT1_REGISTER],
            setpoint_mv: 1200,
        }
    );
    assert_eq!(decision.safety_plan.evidence, hardware_evidence);
}

#[test]
fn safety_power_fixtures_include_required_provenance() {
    // Arrange
    let power_fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/safety/power-telemetry-cases.json"
    ))
    .expect("power fixture should parse");
    let voltage_fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/safety/voltage-effect-cases.json"
    ))
    .expect("voltage fixture should parse");

    // Act
    let serialized = format!("{power_fixture}{voltage_fixture}");

    // Assert
    for expected in [
        "PWR-006",
        "PWR-003",
        "PWR-005",
        "SAFE-01",
        "SAFE-07",
        "SAFE-08",
        "c1915b0a63bfabebdb95a515cedfee05146c1d50",
        "hardware-smoke",
        "hardware-regression",
        "observe-only",
    ] {
        assert!(serialized.contains(expected), "missing {expected}");
    }
}

fn safe_sample() -> Ina260RawSample {
    Ina260RawSample {
        bus_voltage_volts: 5.0,
        current_amps: 2.0,
        power_watts: 10.0,
        read_failed: false,
    }
}

fn fresh_observation() -> PowerObservation {
    PowerObservation::from_ina260_sample(Some(safe_sample()), PowerSampleAgeMs(100), 12.0)
}
