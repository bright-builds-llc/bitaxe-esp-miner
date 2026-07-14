//! Power, voltage, and current safety decisions.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/power/DS4432U.c` for Ultra 205 regulator behavior.
//! - `reference/esp-miner/main/power/INA260.c` for current, voltage, and power telemetry.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for stop, cool, and restart policy.
//!
//! This pure module plans safety effects only; firmware owns I2C and GPIO writes.

use serde::Serialize;

use bitaxe_config::catalog::BoardCatalogEntry;
use bitaxe_config::validation::CoreVoltageMv;

use crate::effects::{SafetyEffect, SafetyEffectPlan};
use crate::evidence::SafetyCriticalEvidence;
use crate::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence,
    SequenceOverflow, StaleReason, UnavailableReason,
};
use crate::status::SafetyStatus;

pub const MODULE_NAME: &str = "power";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/power/DS4432U.c",
    "reference/esp-miner/main/power/INA260.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];

pub const INA260_I2C_ADDRESS: u8 = 0x40;
pub const INA260_CURRENT_REGISTER: u8 = 0x01;
pub const INA260_BUS_VOLTAGE_REGISTER: u8 = 0x02;
pub const INA260_POWER_REGISTER: u8 = 0x03;
pub const DS4432U_I2C_ADDRESS: u8 = 0x48;
pub const DS4432U_OUTPUT0_REGISTER: u8 = 0xF8;
pub const DS4432U_OUTPUT1_REGISTER: u8 = 0xF9;
pub const POWER_SAMPLE_STALE_AFTER_MS: u32 = 1000;
pub const INPUT_VOLTAGE_NOMINAL_VOLTS: f64 = 5.0;
pub const INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;
pub const POWER_MARGIN_WATTS: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PowerSampleAgeMs(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Ina260RawSample {
    pub bus_voltage_volts: f64,
    pub current_amps: f64,
    pub power_watts: f64,
    pub read_failed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PowerReading {
    bus_voltage_volts: f64,
    current_amps: f64,
    power_watts: f64,
}

impl PowerReading {
    #[must_use]
    pub const fn bus_voltage_volts(self) -> f64 {
        self.bus_voltage_volts
    }

    #[must_use]
    pub const fn current_amps(self) -> f64 {
        self.current_amps
    }

    #[must_use]
    pub const fn power_watts(self) -> f64 {
        self.power_watts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PowerObservation {
    truth: Observation<PowerReading>,
}

impl PowerObservation {
    #[must_use]
    pub const fn unavailable(reason: UnavailableReason) -> Self {
        Self {
            truth: Observation::unavailable(reason),
        }
    }

    #[must_use]
    pub fn from_ina260_sample(
        maybe_sample: Option<Ina260RawSample>,
        age: PowerSampleAgeMs,
        board_power_target_watts: f64,
    ) -> Self {
        Self::from_stamped_ina260_sample(
            maybe_sample,
            age,
            board_power_target_watts,
            BootSessionId::new(0),
            ObservationSequence::ZERO,
            MonotonicMillis::new(0),
        )
        .expect("the zero compatibility sequence must advance")
        .0
    }

    pub fn from_stamped_ina260_sample(
        maybe_sample: Option<Ina260RawSample>,
        age: PowerSampleAgeMs,
        board_power_target_watts: f64,
        boot_session: BootSessionId,
        prior_sequence: ObservationSequence,
        acquired_at: MonotonicMillis,
    ) -> Result<(Self, ObservationSequence), SequenceOverflow> {
        let Some(sample) = maybe_sample else {
            return Ok((
                Self::unavailable(UnavailableReason::PowerSampleUnavailable),
                prior_sequence,
            ));
        };

        let previous = Self::unavailable(UnavailableReason::PowerSampleUnavailable);
        let (observation, sequence) = previous.record_ina260_success(
            sample,
            board_power_target_watts,
            boot_session,
            prior_sequence,
            acquired_at,
        )?;

        if !observation.is_fresh_safe() {
            return Ok((observation, sequence));
        }

        let fresh = observation.truth;
        let truth = if age.0 > POWER_SAMPLE_STALE_AFTER_MS {
            let Some(last_good) = fresh.maybe_last_good().copied() else {
                unreachable!("a fresh observation always owns a last-good sample");
            };
            Observation::Stale {
                last_good,
                reason: StaleReason::PowerSampleStale,
            }
        } else {
            fresh
        };

        Ok((Self { truth }, sequence))
    }

    /// Records one complete validated INA260 acquisition against the prior truth.
    ///
    /// Validation failures preserve the prior last-good sample and do not advance
    /// the source-local sequence.
    pub fn record_ina260_success(
        self,
        sample: Ina260RawSample,
        board_power_target_watts: f64,
        boot_session: BootSessionId,
        prior_sequence: ObservationSequence,
        acquired_at: MonotonicMillis,
    ) -> Result<(Self, ObservationSequence), SequenceOverflow> {
        if sample.read_failed {
            return Ok((
                self.record_fault(FaultReason::Ina260ReadFailed),
                prior_sequence,
            ));
        }

        let reading = match validated_power_reading(sample, board_power_target_watts) {
            Ok(reading) => reading,
            Err(reason) => return Ok((self.record_fault(reason), prior_sequence)),
        };
        let (truth, sequence) =
            Observation::record_success(reading, boot_session, prior_sequence, acquired_at)?;

        Ok((Self { truth }, sequence))
    }

    #[must_use]
    pub const fn truth(&self) -> &Observation<PowerReading> {
        &self.truth
    }

    #[must_use]
    pub const fn is_fresh_safe(self) -> bool {
        self.truth.is_fresh()
    }

    #[must_use]
    pub const fn reason(self) -> Option<&'static str> {
        self.truth.maybe_reason()
    }

    pub fn mark_stale(
        self,
        reason: StaleReason,
    ) -> Result<Self, crate::observation::MissingLastGood> {
        Ok(Self {
            truth: self.truth.mark_stale(reason)?,
        })
    }

    #[must_use]
    pub fn record_fault(self, reason: FaultReason) -> Self {
        Self {
            truth: self.truth.record_fault(reason),
        }
    }

    #[must_use]
    pub const fn maybe_reading(self) -> Option<PowerReading> {
        let Some(sample) = self.truth.maybe_last_good() else {
            return None;
        };

        Some(*sample.value())
    }

    #[must_use]
    pub const fn bus_voltage_volts(self) -> f64 {
        let Some(reading) = self.maybe_reading() else {
            return 0.0;
        };

        reading.bus_voltage_volts()
    }

    #[must_use]
    pub const fn current_amps(self) -> f64 {
        let Some(reading) = self.maybe_reading() else {
            return 0.0;
        };

        reading.current_amps()
    }

    #[must_use]
    pub const fn power_watts(self) -> f64 {
        let Some(reading) = self.maybe_reading() else {
            return 0.0;
        };

        reading.power_watts()
    }
}

fn validated_power_reading(
    sample: Ina260RawSample,
    board_power_target_watts: f64,
) -> Result<PowerReading, FaultReason> {
    if !sample.bus_voltage_volts.is_finite()
        || !sample.current_amps.is_finite()
        || !sample.power_watts.is_finite()
        || sample.current_amps < 0.0
        || sample.power_watts < 0.0
    {
        return Err(FaultReason::PowerReadingInvalid);
    }

    let min_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);
    let max_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 + INPUT_VOLTAGE_MARGIN_RATIO);
    if sample.bus_voltage_volts < min_voltage || sample.bus_voltage_volts > max_voltage {
        return Err(FaultReason::InputVoltageUnsafe);
    }

    if sample.power_watts > board_power_target_watts + POWER_MARGIN_WATTS {
        return Err(FaultReason::PowerLimitExceeded);
    }

    Ok(PowerReading {
        bus_voltage_volts: sample.bus_voltage_volts,
        current_amps: sample.current_amps,
        power_watts: sample.power_watts,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PowerFaultReason {
    Stale,
    Unavailable,
    Ina260ReadFailed,
    InputVoltageUnsafe,
    PowerLimitExceeded,
    PowerReadingInvalid,
}

impl PowerFaultReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stale => "power_sample_stale",
            Self::Unavailable => "power_sample_unavailable",
            Self::Ina260ReadFailed => "ina260_read_failed",
            Self::InputVoltageUnsafe => "input_voltage_unsafe",
            Self::PowerLimitExceeded => "power_limit_exceeded",
            Self::PowerReadingInvalid => "power_reading_invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PowerEvidenceToken {
    pub bus_voltage_volts: f64,
    pub current_amps: f64,
    pub power_watts: f64,
}

impl PowerEvidenceToken {
    #[must_use]
    pub const fn from_observation(observation: PowerObservation) -> Option<Self> {
        if !observation.is_fresh_safe() {
            return None;
        }

        Some(Self {
            bus_voltage_volts: observation.bus_voltage_volts(),
            current_amps: observation.current_amps(),
            power_watts: observation.power_watts(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PowerSafetyDecision {
    pub plan: SafetyEffectPlan,
    pub maybe_evidence: Option<PowerEvidenceToken>,
}

impl PowerSafetyDecision {
    #[must_use]
    pub fn from_observation(observation: PowerObservation) -> Self {
        let Some(reason) = observation.reason() else {
            return Self {
                plan: SafetyEffectPlan::observe_only(
                    SafetyStatus::Normal,
                    SafetyCriticalEvidence::implemented_not_verified("unit"),
                ),
                maybe_evidence: PowerEvidenceToken::from_observation(observation),
            };
        };

        Self {
            plan: SafetyEffectPlan::fail_closed(reason),
            maybe_evidence: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VoltageActuationMode {
    ObserveOnly,
    ArmedWithHardwareEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VoltageEffectPlan {
    NoWrite {
        reason: &'static str,
    },
    SuppressWrite {
        reason: &'static str,
    },
    WriteDs4432u {
        i2c_address: u8,
        output_registers: [u8; 2],
        setpoint_mv: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoltageControllerInputs {
    pub requested_mv: i64,
    pub board: BoardCatalogEntry,
    pub observation: PowerObservation,
    pub evidence: SafetyCriticalEvidence,
    pub actuation_mode: VoltageActuationMode,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VoltageControllerDecision {
    pub voltage_plan: VoltageEffectPlan,
    pub safety_plan: SafetyEffectPlan,
}

impl VoltageControllerInputs {
    #[must_use]
    pub fn plan(self) -> VoltageControllerDecision {
        let Ok(setpoint) = CoreVoltageMv::ultra_205_bm1366(self.requested_mv) else {
            return suppress_voltage("invalid_voltage_setpoint");
        };

        let capabilities = self.board.capabilities();
        if !capabilities.ds4432u() {
            return suppress_voltage("ds4432u_capability_missing");
        }
        if !capabilities.ina260() {
            return suppress_voltage("ina260_capability_missing");
        }
        if !capabilities.asic_enable() {
            return suppress_voltage("asic_enable_capability_missing");
        }

        if !self.observation.is_fresh_safe() {
            return suppress_voltage(self.observation.reason().unwrap_or("power_reading_invalid"));
        }

        if !self.evidence.is_hardware_verified()
            || self.actuation_mode != VoltageActuationMode::ArmedWithHardwareEvidence
        {
            return VoltageControllerDecision {
                voltage_plan: VoltageEffectPlan::NoWrite {
                    reason: "observe_only_hardware_evidence_missing",
                },
                safety_plan: SafetyEffectPlan::with_effects(
                    SafetyStatus::SafeBlocked {
                        reason: "observe_only_hardware_evidence_missing",
                    },
                    fail_closed_voltage_effects("observe_only_hardware_evidence_missing"),
                    self.evidence,
                ),
            };
        }

        VoltageControllerDecision {
            voltage_plan: VoltageEffectPlan::WriteDs4432u {
                i2c_address: DS4432U_I2C_ADDRESS,
                output_registers: [DS4432U_OUTPUT0_REGISTER, DS4432U_OUTPUT1_REGISTER],
                setpoint_mv: setpoint.millivolts(),
            },
            safety_plan: SafetyEffectPlan::observe_only(SafetyStatus::Normal, self.evidence),
        }
    }
}

fn suppress_voltage(reason: &'static str) -> VoltageControllerDecision {
    VoltageControllerDecision {
        voltage_plan: VoltageEffectPlan::SuppressWrite { reason },
        safety_plan: SafetyEffectPlan::with_effects(
            SafetyStatus::SafeBlocked { reason },
            fail_closed_voltage_effects(reason),
            SafetyCriticalEvidence::Missing,
        ),
    }
}

fn fail_closed_voltage_effects(reason: &'static str) -> Vec<SafetyEffect> {
    vec![
        SafetyEffect::SuppressVoltageWrite,
        SafetyEffect::HoldResetLow,
        SafetyEffect::DisableAsicEnable,
        SafetyEffect::BlockWorkSubmission { reason },
        SafetyEffect::PublishStatus(SafetyStatus::SafeBlocked { reason }),
    ]
}

#[cfg(test)]
mod tests {
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
                PowerObservation::from_ina260_sample(
                    Some(safe_sample()),
                    PowerSampleAgeMs(1001),
                    12.0,
                ),
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
            "../fixtures/safety/power-telemetry-cases.json"
        ))
        .expect("power fixture should parse");
        let voltage_fixture: Value =
            serde_json::from_str(include_str!("../fixtures/safety/voltage-effect-cases.json"))
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
}
