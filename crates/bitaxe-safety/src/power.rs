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
    pub const fn maybe_reason(self) -> Option<&'static str> {
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
    pub const fn maybe_from_observation(observation: PowerObservation) -> Option<Self> {
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
        let Some(reason) = observation.maybe_reason() else {
            return Self {
                plan: SafetyEffectPlan::observe_only(
                    SafetyStatus::Normal,
                    SafetyCriticalEvidence::implemented_not_verified("unit"),
                ),
                maybe_evidence: PowerEvidenceToken::maybe_from_observation(observation),
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
            return suppress_voltage(
                self.observation
                    .maybe_reason()
                    .unwrap_or("power_reading_invalid"),
            );
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
mod tests;
