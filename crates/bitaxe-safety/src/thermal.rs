//! Thermal, fan, PID, and overheat safety decisions.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/thermal/thermal.c` for sensor abstraction and sentinel values.
//! - `reference/esp-miner/main/thermal/PID.c` for controller constants and output limits.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for fan modes and visible fan faults.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for overheat stop and cool behavior.
//!
//! This pure module plans fan and thermal safety effects without firmware PWM or sensor I/O.

use serde::Serialize;

use bitaxe_config::validation::{ConfigValidationError, FanDutyPercent, MinFanDutyPercent};

use crate::effects::{SafetyEffect, SafetyEffectPlan};
use crate::evidence::SafetyCriticalEvidence;
use crate::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence,
    SequenceOverflow, UnavailableReason,
};
use crate::status::SafetyStatus;

mod pid;

pub use pid::{PidController, PidState, PidStep};

pub const MODULE_NAME: &str = "thermal";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/thermal/thermal.c",
    "reference/esp-miner/main/thermal/PID.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];

pub use pid::{PID_EMA_ALPHA, PID_KD, PID_KI, PID_KP, PID_SAMPLE_TIME_MS};
pub const STARTUP_FAN_DUTY_PERCENT: u8 = 70;
pub const PAUSED_FAN_DUTY_PERCENT: u8 = 30;
pub const OVERHEAT_FAN_DUTY_PERCENT: u8 = 100;
pub const THERMAL_UNAVAILABLE_SENTINEL: f64 = -1.0;
pub const THERMAL_DIODE_FAULT_SENTINEL: f64 = 127.0;
pub const MIN_PLAUSIBLE_TEMP_C: f64 = -40.0;
pub const MAX_PLAUSIBLE_TEMP_C: f64 = 150.0;
pub const ASIC_THROTTLE_TEMP_C: f64 = 75.0;
pub const SAFE_RESTART_TEMP_C: f64 = 45.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThermalReading {
    pub chip_temp_celsius: f64,
    pub maybe_board_temp_celsius: Option<f64>,
    pub maybe_vr_temp_celsius: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TachometerReading {
    rpm: u16,
}

impl TachometerReading {
    #[must_use]
    pub const fn new(rpm: u16) -> Self {
        Self { rpm }
    }

    #[must_use]
    pub const fn rpm(self) -> u16 {
        self.rpm
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThermalObservation {
    temperature: Observation<ThermalReading>,
    tachometer: Observation<TachometerReading>,
}

impl ThermalObservation {
    #[must_use]
    pub fn from_reading(maybe_reading: Option<ThermalReading>) -> Self {
        Self::from_stamped_reading(
            maybe_reading,
            BootSessionId::new(0),
            ObservationSequence::ZERO,
            MonotonicMillis::new(0),
        )
        .expect("the zero compatibility sequence must advance or remain unchanged")
        .0
    }

    pub fn from_stamped_reading(
        maybe_reading: Option<ThermalReading>,
        boot_session: BootSessionId,
        prior_sequence: ObservationSequence,
        acquired_at: MonotonicMillis,
    ) -> Result<(Self, ObservationSequence), SequenceOverflow> {
        let tachometer = Observation::unavailable(UnavailableReason::TachometerUnavailable);
        let Some(reading) = maybe_reading else {
            return Ok((
                Self {
                    temperature: Observation::unavailable(
                        UnavailableReason::ThermalReadingUnavailable,
                    ),
                    tachometer,
                },
                prior_sequence,
            ));
        };

        if reading.chip_temp_celsius == THERMAL_UNAVAILABLE_SENTINEL {
            return Ok((
                Self {
                    temperature: Observation::unavailable(
                        UnavailableReason::ThermalReadingUnavailable,
                    ),
                    tachometer,
                },
                prior_sequence,
            ));
        }

        if !valid_thermal_reading(reading) {
            return Ok((
                Self {
                    temperature: Observation::Fault {
                        reason: FaultReason::ThermalReadingInvalid,
                        maybe_last_good: None,
                    },
                    tachometer,
                },
                prior_sequence,
            ));
        }

        let (temperature, sequence) =
            Observation::record_success(reading, boot_session, prior_sequence, acquired_at)?;
        Ok((
            Self {
                temperature,
                tachometer,
            },
            sequence,
        ))
    }

    #[must_use]
    pub const fn from_facts(
        temperature: Observation<ThermalReading>,
        tachometer: Observation<TachometerReading>,
    ) -> Self {
        Self {
            temperature,
            tachometer,
        }
    }

    #[must_use]
    pub const fn temperature_truth(&self) -> &Observation<ThermalReading> {
        &self.temperature
    }

    #[must_use]
    pub const fn tachometer_truth(&self) -> &Observation<TachometerReading> {
        &self.tachometer
    }

    #[must_use]
    pub const fn with_tachometer(self, tachometer: Observation<TachometerReading>) -> Self {
        Self {
            temperature: self.temperature,
            tachometer,
        }
    }

    #[must_use]
    pub const fn is_fresh_safe(self) -> bool {
        self.temperature.is_fresh() && self.chip_temp_celsius() < ASIC_THROTTLE_TEMP_C
    }

    #[must_use]
    pub const fn maybe_reason(self) -> Option<&'static str> {
        self.temperature.maybe_reason()
    }

    #[must_use]
    pub const fn chip_temp_celsius(self) -> f64 {
        let Some(sample) = self.temperature.maybe_last_good() else {
            return THERMAL_UNAVAILABLE_SENTINEL;
        };

        sample.value().chip_temp_celsius
    }

    #[must_use]
    pub const fn maybe_board_temp_celsius(self) -> Option<f64> {
        let Some(sample) = self.temperature.maybe_last_good() else {
            return None;
        };

        sample.value().maybe_board_temp_celsius
    }

    #[must_use]
    pub const fn maybe_vr_temp_celsius(self) -> Option<f64> {
        let Some(sample) = self.temperature.maybe_last_good() else {
            return None;
        };

        sample.value().maybe_vr_temp_celsius
    }

    #[must_use]
    pub fn safety_plan(self) -> SafetyEffectPlan {
        let Some(reason) = self.maybe_reason() else {
            return SafetyEffectPlan::observe_only(
                SafetyStatus::Normal,
                SafetyCriticalEvidence::implemented_not_verified("unit"),
            );
        };

        SafetyEffectPlan::fail_closed(reason)
    }
}

fn valid_thermal_reading(reading: ThermalReading) -> bool {
    reading.chip_temp_celsius != THERMAL_DIODE_FAULT_SENTINEL
        && plausible_temperature(reading.chip_temp_celsius)
        && reading
            .maybe_board_temp_celsius
            .is_none_or(plausible_temperature)
        && reading
            .maybe_vr_temp_celsius
            .is_none_or(plausible_temperature)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThermalEvidenceToken {
    pub chip_temp_celsius: f64,
    pub evidence: SafetyCriticalEvidence,
}

impl ThermalEvidenceToken {
    #[must_use]
    pub const fn maybe_from_observation(
        observation: ThermalObservation,
        evidence: SafetyCriticalEvidence,
    ) -> Option<Self> {
        if !observation.is_fresh_safe() || matches!(evidence, SafetyCriticalEvidence::Missing) {
            return None;
        }

        Some(Self {
            chip_temp_celsius: observation.chip_temp_celsius(),
            evidence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum FanControlMode {
    Overheat,
    Startup,
    PausedOrNoPool,
    Manual {
        manual_percent: i64,
    },
    Auto {
        target_temp_celsius: f64,
        min_percent: i64,
        pid_state: PidState,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FanControlInputs {
    pub mode: FanControlMode,
    pub observation: ThermalObservation,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FanControlDecision {
    pub duty_percent: u8,
    pub maybe_raw_pid_output_percent: Option<f64>,
    pub status: SafetyStatus,
    pub plan: SafetyEffectPlan,
    pub next_pid_state: Option<PidState>,
}

impl FanControlDecision {
    pub fn from_inputs(inputs: FanControlInputs) -> Result<Self, ConfigValidationError> {
        if inputs.observation.maybe_reason().is_some() {
            let plan = inputs.observation.safety_plan();
            return Ok(Self {
                duty_percent: 0,
                maybe_raw_pid_output_percent: None,
                status: plan.status,
                plan,
                next_pid_state: None,
            });
        }

        let duty_percent = match inputs.mode {
            FanControlMode::Overheat => OVERHEAT_FAN_DUTY_PERCENT,
            FanControlMode::Startup => STARTUP_FAN_DUTY_PERCENT,
            FanControlMode::PausedOrNoPool => PAUSED_FAN_DUTY_PERCENT,
            FanControlMode::Manual { manual_percent } => {
                FanDutyPercent::parse(manual_percent)?.percent()
            }
            FanControlMode::Auto {
                target_temp_celsius,
                min_percent,
                pid_state,
            } => {
                let min_fan = MinFanDutyPercent::parse(min_percent)?.percent();
                let step = PidController::new(pid_state).step(
                    target_temp_celsius,
                    inputs.observation.chip_temp_celsius(),
                    min_fan,
                );
                let duty_percent = step
                    .output_percent
                    .clamp(0.0, f64::from(OVERHEAT_FAN_DUTY_PERCENT))
                    .round() as u8;
                return Ok(Self {
                    duty_percent,
                    maybe_raw_pid_output_percent: Some(step.output_percent),
                    status: SafetyStatus::Normal,
                    plan: SafetyEffectPlan::with_effects(
                        SafetyStatus::Normal,
                        vec![SafetyEffect::SetFanDutyPercent {
                            percent: duty_percent,
                        }],
                        SafetyCriticalEvidence::implemented_not_verified("unit"),
                    ),
                    next_pid_state: Some(step.next_state),
                });
            }
        };

        Ok(Self {
            duty_percent,
            maybe_raw_pid_output_percent: None,
            status: SafetyStatus::Normal,
            plan: SafetyEffectPlan::with_effects(
                SafetyStatus::Normal,
                vec![SafetyEffect::SetFanDutyPercent {
                    percent: duty_percent,
                }],
                SafetyCriticalEvidence::implemented_not_verified("unit"),
            ),
            next_pid_state: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OverheatState {
    Normal,
    SafeStopped,
    Cooling,
    RestartCandidate,
    SafeBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct OverheatInputs {
    pub prior_state: OverheatState,
    pub observation: ThermalObservation,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverheatDecision {
    pub state: OverheatState,
    pub plan: SafetyEffectPlan,
}

impl OverheatDecision {
    #[must_use]
    pub fn from_inputs(inputs: OverheatInputs) -> Self {
        if inputs.observation.maybe_reason().is_some() {
            return Self {
                state: OverheatState::SafeBlocked,
                plan: inputs.observation.safety_plan(),
            };
        }

        if inputs.observation.chip_temp_celsius() >= ASIC_THROTTLE_TEMP_C {
            let reason = "overheat_safe_stop";
            return Self {
                state: OverheatState::SafeStopped,
                plan: SafetyEffectPlan::with_effects(
                    SafetyStatus::ThermalFault { reason },
                    vec![
                        SafetyEffect::HoldResetLow,
                        SafetyEffect::SuppressVoltageWrite,
                        SafetyEffect::BlockWorkSubmission { reason },
                        SafetyEffect::SetFanDutyPercent {
                            percent: OVERHEAT_FAN_DUTY_PERCENT,
                        },
                        SafetyEffect::PublishStatus(SafetyStatus::ThermalFault { reason }),
                    ],
                    SafetyCriticalEvidence::Missing,
                ),
            };
        }

        if matches!(
            inputs.prior_state,
            OverheatState::SafeStopped | OverheatState::Cooling
        ) {
            if inputs.observation.chip_temp_celsius() <= SAFE_RESTART_TEMP_C {
                let reason = "restart_requires_hardware_gates";
                return Self {
                    state: OverheatState::RestartCandidate,
                    plan: SafetyEffectPlan::with_effects(
                        SafetyStatus::SafeBlocked { reason },
                        vec![
                            SafetyEffect::BlockWorkSubmission { reason },
                            SafetyEffect::PublishStatus(SafetyStatus::SafeBlocked { reason }),
                        ],
                        SafetyCriticalEvidence::implemented_not_verified("unit"),
                    ),
                };
            }

            return Self {
                state: OverheatState::Cooling,
                plan: SafetyEffectPlan::with_effects(
                    SafetyStatus::ThermalFault {
                        reason: "cooling_after_overheat",
                    },
                    vec![SafetyEffect::SetFanDutyPercent {
                        percent: OVERHEAT_FAN_DUTY_PERCENT,
                    }],
                    SafetyCriticalEvidence::implemented_not_verified("unit"),
                ),
            };
        }

        Self {
            state: OverheatState::Normal,
            plan: SafetyEffectPlan::observe_only(
                SafetyStatus::Normal,
                SafetyCriticalEvidence::implemented_not_verified("unit"),
            ),
        }
    }
}

fn plausible_temperature(value: f64) -> bool {
    value.is_finite() && (MIN_PLAUSIBLE_TEMP_C..=MAX_PLAUSIBLE_TEMP_C).contains(&value)
}

#[cfg(test)]
mod tests;
