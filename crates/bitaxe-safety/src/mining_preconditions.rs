//! Production mining prerequisite contract.
//!
//! Phase 22 boundary notes:
//! - D-05: require power, thermal, fan, voltage, and safety observations before mining.
//! - D-06: accept only fresh observations or bounded Ultra 205 evidence.
//! - D-07: parse shell/runtime observations into typed inputs before dispatch decisions.

#[cfg(test)]
mod tests {
    use crate::effects::{SafetyEffect, SafetyEffectPlan};
    use crate::power::{Ina260RawSample, PowerObservation, PowerSampleAgeMs};
    use crate::thermal::{ThermalObservation, ThermalReading};

    use super::*;

    #[test]
    fn production_mining_preconditions_are_ready_with_fresh_or_bounded_inputs() {
        // Arrange
        let all_fresh = ProductionMiningPreconditions {
            power: fresh_power_prerequisite(),
            thermal: fresh_thermal_prerequisite(),
            fan: ProductionMiningPrerequisite::Fresh,
            voltage: ProductionMiningPrerequisite::Fresh,
            safety: ProductionMiningPrerequisite::Fresh,
        };
        let all_bounded = ProductionMiningPreconditions {
            power: bounded("power"),
            thermal: bounded("thermal"),
            fan: bounded("fan"),
            voltage: bounded("voltage"),
            safety: bounded("safety"),
        };

        // Act
        let fresh_decision = all_fresh.decision();
        let bounded_decision = all_bounded.decision();

        // Assert
        assert_eq!(fresh_decision, ProductionMiningPreconditionDecision::Ready);
        assert_eq!(bounded_decision, ProductionMiningPreconditionDecision::Ready);
    }

    #[test]
    fn production_mining_preconditions_report_missing_prerequisite_reasons() {
        // Arrange
        let cases = [
            (
                ProductionMiningPreconditions {
                    power: ProductionMiningPrerequisite::from_power_observation(
                        PowerObservation::from_ina260_sample(None, PowerSampleAgeMs(0), 12.0),
                    ),
                    ..ready_preconditions()
                },
                "power_sample_unavailable",
            ),
            (
                ProductionMiningPreconditions {
                    thermal: ProductionMiningPrerequisite::from_thermal_observation(
                        ThermalObservation::from_reading(None),
                    ),
                    ..ready_preconditions()
                },
                "thermal_reading_unavailable",
            ),
            (
                ProductionMiningPreconditions {
                    fan: ProductionMiningPrerequisite::blocked(FAN_OBSERVATION_UNAVAILABLE),
                    ..ready_preconditions()
                },
                FAN_OBSERVATION_UNAVAILABLE,
            ),
            (
                ProductionMiningPreconditions {
                    voltage: ProductionMiningPrerequisite::blocked(VOLTAGE_OBSERVATION_UNAVAILABLE),
                    ..ready_preconditions()
                },
                VOLTAGE_OBSERVATION_UNAVAILABLE,
            ),
            (
                ProductionMiningPreconditions {
                    safety: ProductionMiningPrerequisite::blocked(SAFETY_PREFLIGHT_EVIDENCE_MISSING),
                    ..ready_preconditions()
                },
                SAFETY_PREFLIGHT_EVIDENCE_MISSING,
            ),
        ];

        // Act / Assert
        for (preconditions, expected_reason) in cases {
            assert_blocked_reason(preconditions.decision(), expected_reason);
        }
    }

    #[test]
    fn production_mining_preconditions_pass_through_existing_safety_reasons() {
        // Arrange
        let cases = [
            (
                ProductionMiningPrerequisite::from_power_observation(PowerObservation::from_ina260_sample(
                    Some(safe_power_sample()),
                    PowerSampleAgeMs(1001),
                    12.0,
                )),
                "power_sample_stale",
            ),
            (
                ProductionMiningPrerequisite::from_power_observation(PowerObservation::from_ina260_sample(
                    Some(Ina260RawSample {
                        bus_voltage_volts: 5.6,
                        ..safe_power_sample()
                    }),
                    PowerSampleAgeMs(100),
                    12.0,
                )),
                "input_voltage_unsafe",
            ),
            (
                ProductionMiningPrerequisite::from_thermal_observation(
                    ThermalObservation::from_reading(None),
                ),
                "thermal_reading_unavailable",
            ),
            (
                ProductionMiningPrerequisite::from_thermal_observation(
                    ThermalObservation::from_reading(Some(ThermalReading {
                        chip_temp_celsius: f64::NAN,
                        board_temp_celsius: Some(40.0),
                        vr_temp_celsius: Some(42.0),
                    })),
                ),
                "thermal_reading_invalid",
            ),
        ];

        // Act / Assert
        for (power_or_thermal, expected_reason) in cases {
            let preconditions = ProductionMiningPreconditions {
                power: power_or_thermal,
                ..ready_preconditions()
            };
            assert_blocked_reason(preconditions.decision(), expected_reason);
        }
    }

    #[test]
    fn bounded_observation_evidence_rejects_undocumented_ambiguous_or_wrong_board_inputs() {
        // Arrange
        let cases = [
            (
                BoundedObservationEvidence {
                    source: "",
                    ..valid_bounded_evidence("power")
                },
                BOUNDED_OBSERVATION_UNDOCUMENTED,
            ),
            (
                BoundedObservationEvidence {
                    evidence_id: "",
                    ..valid_bounded_evidence("power")
                },
                BOUNDED_OBSERVATION_UNDOCUMENTED,
            ),
            (
                BoundedObservationEvidence {
                    reason: "",
                    ..valid_bounded_evidence("power")
                },
                BOUNDED_OBSERVATION_UNDOCUMENTED,
            ),
            (
                BoundedObservationEvidence {
                    validity_window_ms: 0,
                    ..valid_bounded_evidence("power")
                },
                BOUNDED_OBSERVATION_AMBIGUOUS,
            ),
            (
                BoundedObservationEvidence {
                    board: "204",
                    ..valid_bounded_evidence("power")
                },
                BOUNDED_OBSERVATION_BOARD_MISMATCH,
            ),
        ];

        // Act / Assert
        for (evidence, expected_reason) in cases {
            let preconditions = ProductionMiningPreconditions {
                power: ProductionMiningPrerequisite::Bounded(evidence),
                ..ready_preconditions()
            };
            assert_blocked_reason(preconditions.decision(), expected_reason);
        }
    }

    #[test]
    fn blocked_precondition_decision_contains_fail_closed_effect_plan() {
        // Arrange
        let preconditions = ProductionMiningPreconditions {
            fan: ProductionMiningPrerequisite::blocked(FAN_OBSERVATION_STALE),
            ..ready_preconditions()
        };

        // Act
        let decision = preconditions.decision();

        // Assert
        let ProductionMiningPreconditionDecision::Blocked { reason, plan } = decision else {
            panic!("stale fan observation should block production mining");
        };
        assert_eq!(reason, FAN_OBSERVATION_STALE);
        assert_eq!(plan, SafetyEffectPlan::fail_closed(FAN_OBSERVATION_STALE));
        assert!(plan.effects.contains(&SafetyEffect::BlockWorkSubmission {
            reason: FAN_OBSERVATION_STALE,
        }));
        assert!(plan.effects.contains(&SafetyEffect::SuppressVoltageWrite));
        assert!(plan.effects.contains(&SafetyEffect::DisableAsicEnable));
        assert!(plan.effects.contains(&SafetyEffect::HoldResetLow));
    }

    fn ready_preconditions() -> ProductionMiningPreconditions {
        ProductionMiningPreconditions {
            power: fresh_power_prerequisite(),
            thermal: fresh_thermal_prerequisite(),
            fan: ProductionMiningPrerequisite::Fresh,
            voltage: ProductionMiningPrerequisite::Fresh,
            safety: ProductionMiningPrerequisite::Fresh,
        }
    }

    fn fresh_power_prerequisite() -> ProductionMiningPrerequisite {
        ProductionMiningPrerequisite::from_power_observation(PowerObservation::from_ina260_sample(
            Some(safe_power_sample()),
            PowerSampleAgeMs(100),
            12.0,
        ))
    }

    fn fresh_thermal_prerequisite() -> ProductionMiningPrerequisite {
        ProductionMiningPrerequisite::from_thermal_observation(ThermalObservation::from_reading(
            Some(ThermalReading {
                chip_temp_celsius: 55.0,
                board_temp_celsius: Some(40.0),
                vr_temp_celsius: Some(42.0),
            }),
        ))
    }

    fn bounded(source: &'static str) -> ProductionMiningPrerequisite {
        ProductionMiningPrerequisite::Bounded(valid_bounded_evidence(source))
    }

    fn valid_bounded_evidence(source: &'static str) -> BoundedObservationEvidence {
        BoundedObservationEvidence {
            source,
            board: "205",
            evidence_id: "phase-22-prerequisite-proof",
            validity_window_ms: 60_000,
            reason: "bounded_observation_accepted",
        }
    }

    fn safe_power_sample() -> Ina260RawSample {
        Ina260RawSample {
            bus_voltage_volts: 5.0,
            current_amps: 2.0,
            power_watts: 10.0,
            read_failed: false,
        }
    }

    fn assert_blocked_reason(
        decision: ProductionMiningPreconditionDecision,
        expected_reason: &'static str,
    ) {
        let ProductionMiningPreconditionDecision::Blocked { reason, plan } = decision else {
            panic!("expected blocked production mining precondition");
        };
        assert_eq!(reason, expected_reason);
        assert_eq!(plan, SafetyEffectPlan::fail_closed(expected_reason));
    }
}
