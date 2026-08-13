use serde::Deserialize;
use serde_json::Value;

use super::*;

#[derive(Debug, Deserialize)]
struct FanPidFixture {
    pid_sequences: Vec<PidSequenceFixture>,
}

#[derive(Debug, Deserialize)]
struct PidSequenceFixture {
    name: String,
    steps: Vec<PidStepFixture>,
}

#[derive(Debug, Deserialize)]
struct PidStepFixture {
    target_temp_celsius: f64,
    raw_input_celsius: f64,
    min_fan_percent: i64,
    expected_filtered_input_celsius: f64,
    expected_raw_output_percent: f64,
    expected_applied_duty_percent: u8,
    expected_output_sum_percent: f64,
    expected_output_min_percent: f64,
    expected_output_max_percent: f64,
}

#[test]
fn safety_thermal_pid_constants_and_modes_match_expected_values() {
    // Arrange
    let observation = fresh_observation(65.0);
    let modes = [
        (FanControlMode::Overheat, OVERHEAT_FAN_DUTY_PERCENT),
        (FanControlMode::Startup, STARTUP_FAN_DUTY_PERCENT),
        (FanControlMode::PausedOrNoPool, PAUSED_FAN_DUTY_PERCENT),
        (FanControlMode::Manual { manual_percent: 42 }, 42),
    ];

    // Act / Assert
    assert_eq!(PID_KP, 5.0);
    assert_eq!(PID_KI, 0.1);
    assert_eq!(PID_KD, 2.0);
    assert_eq!(PID_SAMPLE_TIME_MS, 100);
    assert_eq!(PID_EMA_ALPHA, 0.2);
    for (mode, expected_duty) in modes {
        let decision = FanControlDecision::from_inputs(FanControlInputs { mode, observation })
            .expect("fan mode should parse");
        assert_eq!(decision.duty_percent, expected_duty);
    }
}

#[test]
fn safety_thermal_invalid_sentinels_fail_closed_before_fan_decisions() {
    // Arrange
    let invalid_readings = [
        None,
        Some(reading(THERMAL_UNAVAILABLE_SENTINEL)),
        Some(reading(THERMAL_DIODE_FAULT_SENTINEL)),
        Some(reading(f64::NAN)),
        Some(reading(MIN_PLAUSIBLE_TEMP_C - 1.0)),
        Some(reading(MAX_PLAUSIBLE_TEMP_C + 1.0)),
    ];

    // Act / Assert
    for maybe_reading in invalid_readings {
        let observation = ThermalObservation::from_reading(maybe_reading);
        let decision = FanControlDecision::from_inputs(FanControlInputs {
            mode: FanControlMode::Auto {
                target_temp_celsius: 60.0,
                min_percent: 25,
                pid_state: PidState::default(),
            },
            observation,
        })
        .expect("invalid thermal observation should produce safe fan decision");
        assert!(matches!(decision.status, SafetyStatus::SafeBlocked { .. }));
        assert!(decision
            .plan
            .effects
            .iter()
            .any(|effect| matches!(effect, SafetyEffect::BlockWorkSubmission { .. })));
    }
}

#[test]
fn safety_thermal_auto_pid_clamps_to_minimum_fan_floor() {
    // Arrange
    let observation = fresh_observation(55.0);

    // Act
    let decision = FanControlDecision::from_inputs(FanControlInputs {
        mode: FanControlMode::Auto {
            target_temp_celsius: 60.0,
            min_percent: 25,
            pid_state: PidState::default(),
        },
        observation,
    })
    .expect("auto fan decision should parse");

    // Assert
    assert_eq!(decision.duty_percent, 25);
    assert!(decision.next_pid_state.is_some());
}

#[test]
fn safety_thermal_pid_matches_every_sequential_golden_vector() {
    // Arrange
    let fixture: FanPidFixture =
        serde_json::from_str(include_str!("../../fixtures/safety/fan-pid-cases.json"))
            .expect("fan PID fixture should parse");

    // Act / Assert
    for sequence in fixture.pid_sequences {
        let mut state = PidState::default();
        for (index, step) in sequence.steps.into_iter().enumerate() {
            let decision = FanControlDecision::from_inputs(FanControlInputs {
                mode: FanControlMode::Auto {
                    target_temp_celsius: step.target_temp_celsius,
                    min_percent: step.min_fan_percent,
                    pid_state: state,
                },
                observation: fresh_observation(step.raw_input_celsius),
            })
            .expect("golden PID input should be valid");
            let next_state = decision
                .next_pid_state
                .expect("automatic control should retain PID state");
            let raw_output = decision
                .maybe_raw_pid_output_percent
                .expect("automatic control should expose its raw PID output");
            let context = format!("{} step {index}", sequence.name);

            assert_close(
                f64::from(
                    next_state
                        .maybe_filtered_input_celsius
                        .expect("automatic control should retain the filtered input"),
                ),
                step.expected_filtered_input_celsius,
                &context,
            );
            assert_close(
                f64::from(raw_output),
                step.expected_raw_output_percent,
                &context,
            );
            assert_eq!(
                decision.duty_percent, step.expected_applied_duty_percent,
                "{context}"
            );
            assert_close(
                f64::from(next_state.output_sum_percent),
                step.expected_output_sum_percent,
                &context,
            );
            assert_close(
                f64::from(next_state.output_min_percent),
                step.expected_output_min_percent,
                &context,
            );
            assert_close(
                f64::from(next_state.output_max_percent),
                step.expected_output_max_percent,
                &context,
            );
            assert_close(
                f64::from(next_state.last_input_celsius),
                step.expected_filtered_input_celsius,
                &context,
            );
            assert!(next_state.automatic, "{context}");
            state = next_state;
        }
    }
}

#[test]
fn safety_thermal_evidence_token_requires_fresh_safe_observation() {
    // Arrange
    let fresh = fresh_observation(60.0);
    let overheat = fresh_observation(ASIC_THROTTLE_TEMP_C);
    let invalid = ThermalObservation::from_reading(Some(reading(THERMAL_DIODE_FAULT_SENTINEL)));
    let evidence = SafetyCriticalEvidence::implemented_not_verified("unit");

    // Act / Assert
    assert!(ThermalEvidenceToken::maybe_from_observation(fresh, evidence).is_some());
    assert!(ThermalEvidenceToken::maybe_from_observation(overheat, evidence).is_none());
    assert!(ThermalEvidenceToken::maybe_from_observation(invalid, evidence).is_none());
    assert!(
        ThermalEvidenceToken::maybe_from_observation(fresh, SafetyCriticalEvidence::Missing)
            .is_none()
    );
}

#[test]
fn safety_thermal_temperature_and_tachometer_truth_are_independent() {
    // Arrange
    let fresh_temperature = fresh_observation(60.0);
    let invalid_temperature = ThermalObservation::from_reading(Some(reading(f64::NAN)));
    let (fresh_tachometer, _) = Observation::record_success(
        TachometerReading::new(3_000),
        BootSessionId::new(7),
        ObservationSequence::ZERO,
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance");

    // Act
    let temperature_without_tachometer = fresh_temperature;
    let tachometer_without_temperature = invalid_temperature.with_tachometer(fresh_tachometer);

    // Assert
    assert!(temperature_without_tachometer
        .temperature_truth()
        .is_fresh());
    assert_eq!(
        temperature_without_tachometer
            .tachometer_truth()
            .state_label(),
        "unavailable"
    );
    assert_eq!(
        tachometer_without_temperature
            .temperature_truth()
            .state_label(),
        "fault"
    );
    assert_eq!(
        tachometer_without_temperature
            .tachometer_truth()
            .maybe_last_good()
            .map(|sample| sample.value().rpm()),
        Some(3_000)
    );
}

#[test]
fn safety_thermal_stale_and_fault_states_retain_the_exact_temperature_stamp() {
    // Arrange
    let fresh = fresh_observation(60.0);
    let expected = fresh
        .temperature_truth()
        .maybe_last_good()
        .expect("fresh temperature should own a sample")
        .to_owned();
    let stale_temperature = fresh
        .temperature_truth()
        .mark_stale(crate::observation::StaleReason::ThermalSampleStale)
        .expect("fresh temperature can become stale");
    let stale = ThermalObservation::from_facts(stale_temperature, *fresh.tachometer_truth());

    // Act
    let fault_temperature = stale
        .temperature_truth()
        .record_fault(FaultReason::ReadFailed);
    let fault = ThermalObservation::from_facts(fault_temperature, *stale.tachometer_truth());

    // Assert
    assert_eq!(stale.temperature_truth().maybe_last_good(), Some(&expected));
    assert_eq!(fault.temperature_truth().maybe_last_good(), Some(&expected));
    assert!(matches!(
        FanControlDecision::from_inputs(FanControlInputs {
            mode: FanControlMode::Startup,
            observation: stale,
        })
        .expect("stale temperature should produce a safe fan decision")
        .status,
        SafetyStatus::SafeBlocked { .. }
    ));
}

#[test]
fn safety_fault_overheat_stop_and_restart_candidate_are_fail_closed() {
    // Arrange
    let hot = fresh_observation(75.0);
    let cool = fresh_observation(45.0);

    // Act
    let stop = OverheatDecision::from_inputs(OverheatInputs {
        prior_state: OverheatState::Normal,
        observation: hot,
    });
    let restart = OverheatDecision::from_inputs(OverheatInputs {
        prior_state: OverheatState::SafeStopped,
        observation: cool,
    });

    // Assert
    assert_eq!(stop.state, OverheatState::SafeStopped);
    assert_eq!(
        stop.plan.status,
        SafetyStatus::ThermalFault {
            reason: "overheat_safe_stop"
        }
    );
    assert!(stop
        .plan
        .effects
        .contains(&SafetyEffect::SetFanDutyPercent { percent: 100 }));
    assert_eq!(restart.state, OverheatState::RestartCandidate);
    assert_eq!(
        restart.plan.status,
        SafetyStatus::SafeBlocked {
            reason: "restart_requires_hardware_gates"
        }
    );
    assert!(!restart
        .plan
        .effects
        .contains(&SafetyEffect::PublishStatus(SafetyStatus::Normal)));
}

#[test]
fn safety_thermal_fixtures_include_required_provenance() {
    // Arrange
    let fan_pid: Value =
        serde_json::from_str(include_str!("../../fixtures/safety/fan-pid-cases.json"))
            .expect("fan PID fixture should parse");
    let thermal_faults: Value = serde_json::from_str(include_str!(
        "../../fixtures/safety/thermal-fault-cases.json"
    ))
    .expect("thermal fault fixture should parse");
    let overheat: Value = serde_json::from_str(include_str!(
        "../../fixtures/safety/overheat-state-cases.json"
    ))
    .expect("overheat fixture should parse");

    // Act
    let serialized = format!("{fan_pid}{thermal_faults}{overheat}");

    // Assert
    for expected in [
        "THR-001",
        "THR-002",
        "THR-003",
        "PWR-001",
        "PWR-002",
        "SAFE-02",
        "SAFE-03",
        "SAFE-04",
        "SAFE-07",
        "SAFE-08",
        "not verified",
        "c1915b0a63bfabebdb95a515cedfee05146c1d50",
    ] {
        assert!(serialized.contains(expected), "missing {expected}");
    }
}

fn fresh_observation(chip_temp_celsius: f64) -> ThermalObservation {
    ThermalObservation::from_reading(Some(reading(chip_temp_celsius)))
}

fn reading(chip_temp_celsius: f64) -> ThermalReading {
    ThermalReading {
        chip_temp_celsius,
        maybe_board_temp_celsius: Some(40.0),
        maybe_vr_temp_celsius: Some(42.0),
    }
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{context}: expected {expected}, got {actual}"
    );
}
