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
use crate::status::SafetyStatus;

pub const MODULE_NAME: &str = "thermal";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/thermal/thermal.c",
    "reference/esp-miner/main/thermal/PID.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];

pub const PID_KP: f64 = 5.0;
pub const PID_KI: f64 = 0.1;
pub const PID_KD: f64 = 2.0;
pub const PID_SAMPLE_TIME_MS: u32 = 100;
pub const PID_EMA_ALPHA: f64 = 0.2;
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
    pub board_temp_celsius: Option<f64>,
    pub vr_temp_celsius: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ThermalObservationStatus {
    Fresh,
    Fault { reason: &'static str },
    Unavailable { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThermalObservation {
    pub status: ThermalObservationStatus,
    pub chip_temp_celsius: f64,
    pub board_temp_celsius: Option<f64>,
    pub vr_temp_celsius: Option<f64>,
}

impl ThermalObservation {
    #[must_use]
    pub fn from_reading(maybe_reading: Option<ThermalReading>) -> Self {
        let Some(reading) = maybe_reading else {
            return Self::unavailable("thermal_reading_unavailable");
        };

        if reading.chip_temp_celsius == THERMAL_UNAVAILABLE_SENTINEL {
            return Self::unavailable("thermal_reading_unavailable");
        }

        if reading.chip_temp_celsius == THERMAL_DIODE_FAULT_SENTINEL
            || !plausible_temperature(reading.chip_temp_celsius)
            || reading
                .board_temp_celsius
                .is_some_and(|value| !plausible_temperature(value))
            || reading
                .vr_temp_celsius
                .is_some_and(|value| !plausible_temperature(value))
        {
            return Self::fault(reading, "thermal_reading_invalid");
        }

        Self {
            status: ThermalObservationStatus::Fresh,
            chip_temp_celsius: reading.chip_temp_celsius,
            board_temp_celsius: reading.board_temp_celsius,
            vr_temp_celsius: reading.vr_temp_celsius,
        }
    }

    #[must_use]
    pub const fn is_fresh_safe(self) -> bool {
        matches!(self.status, ThermalObservationStatus::Fresh)
            && self.chip_temp_celsius < ASIC_THROTTLE_TEMP_C
    }

    #[must_use]
    pub const fn reason(self) -> Option<&'static str> {
        match self.status {
            ThermalObservationStatus::Fresh => None,
            ThermalObservationStatus::Fault { reason }
            | ThermalObservationStatus::Unavailable { reason } => Some(reason),
        }
    }

    #[must_use]
    pub fn safety_plan(self) -> SafetyEffectPlan {
        let Some(reason) = self.reason() else {
            return SafetyEffectPlan::observe_only(
                SafetyStatus::Normal,
                SafetyCriticalEvidence::implemented_not_verified("unit"),
            );
        };

        SafetyEffectPlan::fail_closed(reason)
    }

    const fn unavailable(reason: &'static str) -> Self {
        Self {
            status: ThermalObservationStatus::Unavailable { reason },
            chip_temp_celsius: THERMAL_UNAVAILABLE_SENTINEL,
            board_temp_celsius: None,
            vr_temp_celsius: None,
        }
    }

    const fn fault(reading: ThermalReading, reason: &'static str) -> Self {
        Self {
            status: ThermalObservationStatus::Fault { reason },
            chip_temp_celsius: reading.chip_temp_celsius,
            board_temp_celsius: reading.board_temp_celsius,
            vr_temp_celsius: reading.vr_temp_celsius,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThermalEvidenceToken {
    pub chip_temp_celsius: f64,
    pub evidence: SafetyCriticalEvidence,
}

impl ThermalEvidenceToken {
    #[must_use]
    pub const fn from_observation(
        observation: ThermalObservation,
        evidence: SafetyCriticalEvidence,
    ) -> Option<Self> {
        if !observation.is_fresh_safe() || matches!(evidence, SafetyCriticalEvidence::Missing) {
            return None;
        }

        Some(Self {
            chip_temp_celsius: observation.chip_temp_celsius,
            evidence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PidState {
    pub integral: f64,
    pub previous_error: f64,
    pub ema_output: f64,
}

impl Default for PidState {
    fn default() -> Self {
        Self {
            integral: 0.0,
            previous_error: 0.0,
            ema_output: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PidController {
    pub state: PidState,
}

impl PidController {
    #[must_use]
    pub const fn new(state: PidState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn duty_percent(
        self,
        target_temp_celsius: f64,
        actual_temp_celsius: f64,
        min_fan_percent: u8,
    ) -> FanControlDecision {
        let error = actual_temp_celsius - target_temp_celsius;
        let integral = self.state.integral + error;
        let derivative = error - self.state.previous_error;
        let raw_output = PID_KP.mul_add(error, PID_KI * integral) + PID_KD * derivative;
        let ema_output =
            PID_EMA_ALPHA.mul_add(raw_output, (1.0 - PID_EMA_ALPHA) * self.state.ema_output);
        let clamped = ema_output
            .max(f64::from(min_fan_percent))
            .min(f64::from(OVERHEAT_FAN_DUTY_PERCENT))
            .round() as u8;

        FanControlDecision {
            duty_percent: clamped,
            status: SafetyStatus::Normal,
            plan: SafetyEffectPlan::with_effects(
                SafetyStatus::Normal,
                vec![SafetyEffect::SetFanDutyPercent { percent: clamped }],
                SafetyCriticalEvidence::implemented_not_verified("unit"),
            ),
            next_pid_state: Some(PidState {
                integral,
                previous_error: error,
                ema_output,
            }),
        }
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
    pub status: SafetyStatus,
    pub plan: SafetyEffectPlan,
    pub next_pid_state: Option<PidState>,
}

impl FanControlDecision {
    pub fn from_inputs(inputs: FanControlInputs) -> Result<Self, ConfigValidationError> {
        if inputs.observation.reason().is_some() {
            let plan = inputs.observation.safety_plan();
            return Ok(Self {
                duty_percent: 0,
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
                return Ok(PidController::new(pid_state).duty_percent(
                    target_temp_celsius,
                    inputs.observation.chip_temp_celsius,
                    min_fan,
                ));
            }
        };

        Ok(Self {
            duty_percent,
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
        if inputs.observation.reason().is_some() {
            return Self {
                state: OverheatState::SafeBlocked,
                plan: inputs.observation.safety_plan(),
            };
        }

        if inputs.observation.chip_temp_celsius >= ASIC_THROTTLE_TEMP_C {
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
            if inputs.observation.chip_temp_celsius <= SAFE_RESTART_TEMP_C {
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
mod tests {
    use serde_json::Value;

    use super::*;

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
    fn safety_thermal_evidence_token_requires_fresh_safe_observation() {
        // Arrange
        let fresh = fresh_observation(60.0);
        let overheat = fresh_observation(ASIC_THROTTLE_TEMP_C);
        let invalid = ThermalObservation::from_reading(Some(reading(THERMAL_DIODE_FAULT_SENTINEL)));
        let evidence = SafetyCriticalEvidence::implemented_not_verified("unit");

        // Act / Assert
        assert!(ThermalEvidenceToken::from_observation(fresh, evidence).is_some());
        assert!(ThermalEvidenceToken::from_observation(overheat, evidence).is_none());
        assert!(ThermalEvidenceToken::from_observation(invalid, evidence).is_none());
        assert!(
            ThermalEvidenceToken::from_observation(fresh, SafetyCriticalEvidence::Missing)
                .is_none()
        );
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
            serde_json::from_str(include_str!("../fixtures/safety/fan-pid-cases.json"))
                .expect("fan PID fixture should parse");
        let thermal_faults: Value =
            serde_json::from_str(include_str!("../fixtures/safety/thermal-fault-cases.json"))
                .expect("thermal fault fixture should parse");
        let overheat: Value =
            serde_json::from_str(include_str!("../fixtures/safety/overheat-state-cases.json"))
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
            board_temp_celsius: Some(40.0),
            vr_temp_celsius: Some(42.0),
        }
    }
}
