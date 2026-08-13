//! Pure production fan-controller orchestration.

use bitaxe_config::validation::{
    ConfigValidationError, FanDutyPercent, MinFanDutyPercent, TemperatureCelsius,
};
use bitaxe_safety::{
    effects::SafetyEffect,
    thermal::{
        FanControlDecision, FanControlInputs, FanControlMode, PidState, ThermalObservation,
        PID_SAMPLE_TIME_MS,
    },
};

pub(crate) const FAN_CONTROLLER_CADENCE_MS: u64 = PID_SAMPLE_TIME_MS as u64;
pub(crate) const FAN_CONTROLLER_RETRY_MS: u64 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FanControllerSettings {
    auto_fan_speed: bool,
    manual_fan_speed: FanDutyPercent,
    min_fan_speed: MinFanDutyPercent,
    temp_target: TemperatureCelsius,
    overheat_mode: bool,
}

impl FanControllerSettings {
    pub(crate) fn parse(
        auto_fan_speed: bool,
        manual_fan_speed: i64,
        min_fan_speed: i64,
        temp_target: i64,
        overheat_mode: bool,
    ) -> Result<Self, ConfigValidationError> {
        Ok(Self {
            auto_fan_speed,
            manual_fan_speed: FanDutyPercent::parse(manual_fan_speed)?,
            min_fan_speed: MinFanDutyPercent::parse(min_fan_speed)?,
            temp_target: TemperatureCelsius::parse(temp_target)?,
            overheat_mode,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FanRuntimeStatus {
    pub(crate) hardware_control_qualified: bool,
    pub(crate) operator_paused: bool,
    pub(crate) pools_unavailable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FanControllerModeLabel {
    Overheat,
    Paused,
    NoPool,
    Auto,
    Manual,
}

impl FanControllerModeLabel {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Overheat => "overheat",
            Self::Paused => "paused",
            Self::NoPool => "no_pool",
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FanControllerPlan {
    Apply {
        percent: u8,
        mode: FanControllerModeLabel,
    },
    Unchanged,
    Deferred {
        reason: &'static str,
    },
    RetryDeferred,
    SafeBlocked {
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FanControllerPlanError {
    InvalidSettings(ConfigValidationError),
    AmbiguousFanEffects,
}

impl From<ConfigValidationError> for FanControllerPlanError {
    fn from(error: ConfigValidationError) -> Self {
        Self::InvalidSettings(error)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FanControllerState {
    pid_state: PidState,
    maybe_applied_duty_percent: Option<u8>,
    maybe_retry_not_before_ms: Option<u64>,
}

impl FanControllerState {
    pub(crate) fn plan(
        &mut self,
        settings: FanControllerSettings,
        runtime: FanRuntimeStatus,
        observation: ThermalObservation,
        now_ms: u64,
    ) -> Result<FanControllerPlan, FanControllerPlanError> {
        if !runtime.hardware_control_qualified {
            self.invalidate_applied_duty();
            return Ok(FanControllerPlan::Deferred {
                reason: "hardware_control_not_qualified",
            });
        }

        let (mode, mode_label) = select_mode(settings, runtime, self.pid_state);
        let decision = FanControlDecision::from_inputs(FanControlInputs { mode, observation })?;
        if let Some(next_pid_state) = decision.next_pid_state {
            self.pid_state = next_pid_state;
        }
        let Some(percent) = fan_duty_effect(&decision)? else {
            self.invalidate_applied_duty();
            return Ok(FanControllerPlan::SafeBlocked {
                reason: decision.status.public_reason(),
            });
        };
        if self.maybe_applied_duty_percent == Some(percent) {
            return Ok(FanControllerPlan::Unchanged);
        }
        if self
            .maybe_retry_not_before_ms
            .is_some_and(|retry_at| now_ms < retry_at)
        {
            return Ok(FanControllerPlan::RetryDeferred);
        }

        Ok(FanControllerPlan::Apply {
            percent,
            mode: mode_label,
        })
    }

    pub(crate) fn record_applied(&mut self, percent: u8) {
        self.maybe_applied_duty_percent = Some(percent);
        self.maybe_retry_not_before_ms = None;
    }

    pub(crate) fn record_apply_failure(&mut self, now_ms: u64) {
        self.maybe_retry_not_before_ms = Some(now_ms.saturating_add(FAN_CONTROLLER_RETRY_MS));
    }

    pub(crate) fn invalidate_applied_duty(&mut self) {
        self.maybe_applied_duty_percent = None;
    }
}

fn select_mode(
    settings: FanControllerSettings,
    runtime: FanRuntimeStatus,
    pid_state: PidState,
) -> (FanControlMode, FanControllerModeLabel) {
    if settings.overheat_mode {
        return (FanControlMode::Overheat, FanControllerModeLabel::Overheat);
    }
    if runtime.operator_paused {
        return (
            FanControlMode::PausedOrNoPool,
            FanControllerModeLabel::Paused,
        );
    }
    if runtime.pools_unavailable {
        return (
            FanControlMode::PausedOrNoPool,
            FanControllerModeLabel::NoPool,
        );
    }
    if settings.auto_fan_speed {
        return (
            FanControlMode::Auto {
                target_temp_celsius: f64::from(settings.temp_target.celsius()),
                min_percent: i64::from(settings.min_fan_speed.percent()),
                pid_state,
            },
            FanControllerModeLabel::Auto,
        );
    }

    (
        FanControlMode::Manual {
            manual_percent: i64::from(settings.manual_fan_speed.percent()),
        },
        FanControllerModeLabel::Manual,
    )
}

fn fan_duty_effect(decision: &FanControlDecision) -> Result<Option<u8>, FanControllerPlanError> {
    let mut maybe_percent = None;
    for effect in &decision.plan.effects {
        let SafetyEffect::SetFanDutyPercent { percent } = effect else {
            continue;
        };
        if maybe_percent.replace(*percent).is_some() {
            return Err(FanControllerPlanError::AmbiguousFanEffects);
        }
    }
    Ok(maybe_percent)
}

#[cfg(test)]
mod tests {
    use bitaxe_safety::thermal::{ThermalObservation, ThermalReading};

    use super::*;

    fn settings(
        auto_fan_speed: bool,
        manual_fan_speed: i64,
        overheat_mode: bool,
    ) -> FanControllerSettings {
        FanControllerSettings::parse(auto_fan_speed, manual_fan_speed, 25, 60, overheat_mode)
            .expect("fixture settings are valid")
    }

    fn observation(temperature: f64) -> ThermalObservation {
        ThermalObservation::from_reading(Some(ThermalReading {
            chip_temp_celsius: temperature,
            maybe_board_temp_celsius: None,
            maybe_vr_temp_celsius: None,
        }))
    }

    fn active() -> FanRuntimeStatus {
        FanRuntimeStatus {
            hardware_control_qualified: true,
            operator_paused: false,
            pools_unavailable: false,
        }
    }

    #[test]
    fn control_is_deferred_until_hardware_is_qualified_by_active_mining() {
        // Arrange
        let mut state = FanControllerState::default();
        let runtime = FanRuntimeStatus {
            hardware_control_qualified: false,
            ..active()
        };

        // Act
        let plan = state
            .plan(settings(false, 42, false), runtime, observation(60.0), 0)
            .expect("deferred planning succeeds");

        // Assert
        assert_eq!(
            plan,
            FanControllerPlan::Deferred {
                reason: "hardware_control_not_qualified"
            }
        );
    }

    #[test]
    fn cadence_and_mode_labels_are_stable() {
        // Arrange / Act / Assert
        assert_eq!(FAN_CONTROLLER_CADENCE_MS, 100);
        assert_eq!(FanControllerModeLabel::Auto.as_str(), "auto");
    }

    #[test]
    fn fixed_mode_priority_matches_overheat_pause_and_no_pool_order() {
        // Arrange
        let paused = FanRuntimeStatus {
            operator_paused: true,
            pools_unavailable: true,
            ..active()
        };
        let no_pool = FanRuntimeStatus {
            pools_unavailable: true,
            ..active()
        };

        // Act
        let overheat = FanControllerState::default()
            .plan(settings(true, 42, true), paused, observation(60.0), 0)
            .expect("overheat planning succeeds");
        let paused = FanControllerState::default()
            .plan(settings(true, 42, false), paused, observation(60.0), 0)
            .expect("paused planning succeeds");
        let no_pool = FanControllerState::default()
            .plan(settings(true, 42, false), no_pool, observation(60.0), 0)
            .expect("no-pool planning succeeds");

        // Assert
        assert_eq!(
            overheat,
            FanControllerPlan::Apply {
                percent: 100,
                mode: FanControllerModeLabel::Overheat
            }
        );
        assert_eq!(
            paused,
            FanControllerPlan::Apply {
                percent: 30,
                mode: FanControllerModeLabel::Paused
            }
        );
        assert_eq!(
            no_pool,
            FanControllerPlan::Apply {
                percent: 30,
                mode: FanControllerModeLabel::NoPool
            }
        );
    }

    #[test]
    fn manual_mode_produces_the_validated_duty() {
        // Arrange
        let mut manual_state = FanControllerState::default();

        // Act
        let manual = manual_state
            .plan(settings(false, 42, false), active(), observation(60.0), 0)
            .expect("manual planning succeeds");

        // Assert
        assert_eq!(
            manual,
            FanControllerPlan::Apply {
                percent: 42,
                mode: FanControllerModeLabel::Manual
            }
        );
    }

    #[test]
    fn auto_mode_produces_the_pid_floor() {
        // Arrange
        let mut state = FanControllerState::default();

        // Act
        let auto = state
            .plan(settings(true, 42, false), active(), observation(55.0), 0)
            .expect("auto planning succeeds");

        // Assert
        assert_eq!(
            auto,
            FanControllerPlan::Apply {
                percent: 25,
                mode: FanControllerModeLabel::Auto
            }
        );
    }

    #[test]
    fn auto_mode_retains_pid_state_between_iterations() {
        // Arrange
        let mut state = FanControllerState::default();
        let settings = settings(true, 42, false);
        let observation = observation(74.0);

        // Act
        let first = state
            .plan(settings, active(), observation, 0)
            .expect("first auto plan succeeds");
        let FanControllerPlan::Apply {
            percent: first_percent,
            ..
        } = first
        else {
            panic!("first auto plan must apply");
        };
        state.record_applied(first_percent);
        let second = state
            .plan(settings, active(), observation, 100)
            .expect("second auto plan succeeds");

        // Assert
        assert_eq!(first_percent, 25);
        assert!(matches!(
            second,
            FanControllerPlan::Apply { percent: 30, .. }
        ));
    }

    #[test]
    fn applied_duty_is_suppressed_until_control_becomes_unqualified() {
        // Arrange
        let mut state = FanControllerState::default();
        let settings = settings(false, 42, false);
        let observation = observation(60.0);
        state.record_applied(42);

        // Act
        let unchanged = state
            .plan(settings, active(), observation, 0)
            .expect("unchanged planning succeeds");
        let deferred = state
            .plan(
                settings,
                FanRuntimeStatus {
                    hardware_control_qualified: false,
                    ..active()
                },
                observation,
                0,
            )
            .expect("deferred planning succeeds");
        let reasserted = state
            .plan(settings, active(), observation, 0)
            .expect("reassertion planning succeeds");

        // Assert
        assert_eq!(unchanged, FanControllerPlan::Unchanged);
        assert!(matches!(deferred, FanControllerPlan::Deferred { .. }));
        assert!(matches!(
            reasserted,
            FanControllerPlan::Apply { percent: 42, .. }
        ));
    }

    #[test]
    fn failed_write_uses_a_bounded_retry_delay() {
        // Arrange
        let mut state = FanControllerState::default();
        let settings = settings(false, 42, false);
        let observation = observation(60.0);
        state.record_apply_failure(1_000);

        // Act
        let early = state
            .plan(settings, active(), observation, 2_999)
            .expect("early retry planning succeeds");
        let due = state
            .plan(settings, active(), observation, 3_000)
            .expect("due retry planning succeeds");

        // Assert
        assert_eq!(early, FanControllerPlan::RetryDeferred);
        assert!(matches!(due, FanControllerPlan::Apply { percent: 42, .. }));
    }

    #[test]
    fn invalid_settings_are_rejected() {
        // Arrange / Act
        let result = FanControllerSettings::parse(false, 101, 25, 60, false);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn invalid_thermal_truth_fails_closed() {
        // Arrange
        let mut state = FanControllerState::default();

        // Act
        let invalid_observation = state
            .plan(
                settings(true, 42, false),
                active(),
                ThermalObservation::from_reading(None),
                0,
            )
            .expect("invalid thermal truth yields a closed plan");

        // Assert
        assert!(matches!(
            invalid_observation,
            FanControllerPlan::SafeBlocked { .. }
        ));
    }
}
