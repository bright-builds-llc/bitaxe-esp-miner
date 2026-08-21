//! Pure Ultra 205 hardware self-test contract.
//!
//! Behavioral breadcrumb: `reference/esp-miner/main/self_test/self_test.c`.

use core::fmt;

pub const HARDWARE_SELF_TEST_FREQUENCY_MHZ: u16 = 485;
pub const HARDWARE_SELF_TEST_CORE_VOLTAGE_MV: u16 = 1_200;
pub const HARDWARE_SELF_TEST_DIFFICULTY: u32 = 16;
pub const HARDWARE_SELF_TEST_WARMUP_C: f32 = 55.0;
pub const HARDWARE_SELF_TEST_TARGET_C: f32 = 65.0;
pub const HARDWARE_SELF_TEST_MAX_C: f32 = 70.0;
pub const HARDWARE_SELF_TEST_COOLING_C: f32 = 45.0;
pub const HARDWARE_SELF_TEST_WARMUP_TIMEOUT_MS: u64 = 180_000;
pub const HARDWARE_SELF_TEST_MEASUREMENT_MS: u64 = 30_000;
pub const HARDWARE_SELF_TEST_PLANNED_FAILURE_LOAD_MS: u64 = 5_000;
pub const HARDWARE_SELF_TEST_COOLING_TIMEOUT_MS: u64 = 120_000;
pub const HARDWARE_SELF_TEST_RESTART_DELAY_MS: u64 = 10_000;
pub const HARDWARE_SELF_TEST_FAN_MIN_PERCENT: u8 = 10;
pub const HARDWARE_SELF_TEST_FAN_MAX_PERCENT: u8 = 100;
pub const HARDWARE_SELF_TEST_FAN_SETTLE_PERCENT: u8 = 30;
pub const HARDWARE_SELF_TEST_FAN_RPM_MIN: u16 = 1_000;
pub const HARDWARE_SELF_TEST_POWER_MAX_WATTS: f32 = 15.0;
pub const HARDWARE_SELF_TEST_INPUT_VOLTAGE_MIN_VOLTS: f32 = 4.5;
pub const HARDWARE_SELF_TEST_INPUT_VOLTAGE_MAX_VOLTS: f32 = 5.5;
pub const HARDWARE_SELF_TEST_CORE_VOLTAGE_MIN_MV: u16 = 1_080;
pub const HARDWARE_SELF_TEST_CORE_VOLTAGE_MAX_MV: u16 = 1_320;
pub const HARDWARE_SELF_TEST_HASHRATE_PERCENT: f32 = 0.85;
pub const HARDWARE_SELF_TEST_DOMAIN_TOLERANCE: f32 = 0.33;
pub const HARDWARE_SELF_TEST_SMALL_CORE_COUNT: u16 = 894;
pub const HARDWARE_SELF_TEST_DOMAIN_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareSelfTestCase {
    PlannedFailure,
    Pass,
}

impl HardwareSelfTestCase {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::PlannedFailure => "planned_failure",
            Self::Pass => "pass",
        }
    }

    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "planned_failure" => Some(Self::PlannedFailure),
            "pass" => Some(Self::Pass),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareSelfTestStage {
    Admitted,
    Preflight,
    Preparing,
    Warming,
    Measuring,
    Evaluating,
    SafeStopping,
    AwaitingCancel,
    Restarting,
    Complete,
}

impl HardwareSelfTestStage {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Preflight => "preflight",
            Self::Preparing => "preparing",
            Self::Warming => "warming",
            Self::Measuring => "measuring",
            Self::Evaluating => "evaluating",
            Self::SafeStopping => "safe_stopping",
            Self::AwaitingCancel => "awaiting_cancel",
            Self::Restarting => "restarting",
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareSelfTestFailure {
    PsramMissing,
    SafetyUnavailable,
    PreparationFailed,
    WarmupTimedOut,
    TemperatureExceeded,
    MeasurementIncomplete,
    HashrateLow,
    DomainFailed,
    InputVoltageInvalid,
    CoreVoltageInvalid,
    PowerInvalid,
    FanInvalid,
    PlannedEvaluationFailure,
    SafeStopFailed,
    ClockRegressed,
    DeadlineOverflow,
}

impl HardwareSelfTestFailure {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::PsramMissing => "psram_missing",
            Self::SafetyUnavailable => "safety_unavailable",
            Self::PreparationFailed => "preparation_failed",
            Self::WarmupTimedOut => "warmup_timed_out",
            Self::TemperatureExceeded => "temperature_exceeded",
            Self::MeasurementIncomplete => "measurement_incomplete",
            Self::HashrateLow => "hashrate_low",
            Self::DomainFailed => "domain_failed",
            Self::InputVoltageInvalid => "input_voltage_invalid",
            Self::CoreVoltageInvalid => "core_voltage_invalid",
            Self::PowerInvalid => "power_invalid",
            Self::FanInvalid => "fan_invalid",
            Self::PlannedEvaluationFailure => "planned_evaluation_failure",
            Self::SafeStopFailed => "safe_stop_failed",
            Self::ClockRegressed => "clock_regressed",
            Self::DeadlineOverflow => "deadline_overflow",
        }
    }
}

impl fmt::Display for HardwareSelfTestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.token())
    }
}

impl std::error::Error for HardwareSelfTestFailure {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardwareSelfTestMetrics {
    pub measured_ms: u64,
    pub total_hashrate_ghs: f32,
    pub domain_hashrate_ghs: [f32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    pub domain_sample_counts: [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    pub domain_rejected_counts: [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    pub input_voltage_volts: f32,
    pub core_voltage_mv: u16,
    pub power_watts: f32,
    pub fan_rpm: u16,
    pub maximum_temperature_c: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareSelfTestEvaluation {
    pub unreliable_domain_mask: u8,
}

#[must_use]
pub const fn expected_hardware_self_test_hashrate_ghs() -> f32 {
    (HARDWARE_SELF_TEST_FREQUENCY_MHZ as f32 * HARDWARE_SELF_TEST_SMALL_CORE_COUNT as f32 / 1_000.0)
        * HARDWARE_SELF_TEST_HASHRATE_PERCENT
}

#[must_use]
pub const fn expected_hardware_self_test_domain_hashrate_ghs() -> f32 {
    expected_hardware_self_test_hashrate_ghs() / HARDWARE_SELF_TEST_DOMAIN_COUNT as f32
}

pub fn evaluate_hardware_self_test_metrics(
    metrics: HardwareSelfTestMetrics,
) -> Result<HardwareSelfTestEvaluation, HardwareSelfTestFailure> {
    if metrics.maximum_temperature_c > HARDWARE_SELF_TEST_MAX_C
        || !metrics.maximum_temperature_c.is_finite()
    {
        return Err(HardwareSelfTestFailure::TemperatureExceeded);
    }
    if metrics.measured_ms < HARDWARE_SELF_TEST_MEASUREMENT_MS {
        return Err(HardwareSelfTestFailure::MeasurementIncomplete);
    }
    if !metrics.total_hashrate_ghs.is_finite()
        || metrics.total_hashrate_ghs < expected_hardware_self_test_hashrate_ghs()
    {
        return Err(HardwareSelfTestFailure::HashrateLow);
    }
    let expected_domain = expected_hardware_self_test_domain_hashrate_ghs();
    let domain_minimum = expected_domain * (1.0 - HARDWARE_SELF_TEST_DOMAIN_TOLERANCE);
    let domain_maximum = expected_domain * (1.0 + HARDWARE_SELF_TEST_DOMAIN_TOLERANCE);
    let mut unreliable_domain_mask = 0_u8;
    for index in 0..HARDWARE_SELF_TEST_DOMAIN_COUNT {
        let accepted = metrics.domain_sample_counts[index];
        let rejected = metrics.domain_rejected_counts[index];
        let total = accepted.saturating_add(rejected);
        let unreliable =
            (accepted == 0 && rejected > 0) || (total > 0 && rejected.saturating_mul(4) >= total);
        if unreliable {
            unreliable_domain_mask |= 1 << index;
            continue;
        }
        let hashrate = metrics.domain_hashrate_ghs[index];
        if accepted == 0
            || !hashrate.is_finite()
            || !(domain_minimum..=domain_maximum).contains(&hashrate)
        {
            return Err(HardwareSelfTestFailure::DomainFailed);
        }
    }
    if !metrics.input_voltage_volts.is_finite()
        || !(HARDWARE_SELF_TEST_INPUT_VOLTAGE_MIN_VOLTS
            ..=HARDWARE_SELF_TEST_INPUT_VOLTAGE_MAX_VOLTS)
            .contains(&metrics.input_voltage_volts)
    {
        return Err(HardwareSelfTestFailure::InputVoltageInvalid);
    }
    if !(HARDWARE_SELF_TEST_CORE_VOLTAGE_MIN_MV..=HARDWARE_SELF_TEST_CORE_VOLTAGE_MAX_MV)
        .contains(&metrics.core_voltage_mv)
    {
        return Err(HardwareSelfTestFailure::CoreVoltageInvalid);
    }
    if !metrics.power_watts.is_finite()
        || !(0.0..=HARDWARE_SELF_TEST_POWER_MAX_WATTS).contains(&metrics.power_watts)
    {
        return Err(HardwareSelfTestFailure::PowerInvalid);
    }
    if metrics.fan_rpm <= HARDWARE_SELF_TEST_FAN_RPM_MIN {
        return Err(HardwareSelfTestFailure::FanInvalid);
    }
    Ok(HardwareSelfTestEvaluation {
        unreliable_domain_mask,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareSelfTestSchedule {
    stage: HardwareSelfTestStage,
    entered_at_ms: u64,
    maybe_deadline_ms: Option<u64>,
}

impl HardwareSelfTestSchedule {
    #[must_use]
    pub const fn admitted(now_ms: u64) -> Self {
        Self {
            stage: HardwareSelfTestStage::Admitted,
            entered_at_ms: now_ms,
            maybe_deadline_ms: None,
        }
    }

    pub fn enter(
        &mut self,
        stage: HardwareSelfTestStage,
        now_ms: u64,
        maybe_timeout_ms: Option<u64>,
    ) -> Result<(), HardwareSelfTestFailure> {
        if now_ms < self.entered_at_ms {
            return Err(HardwareSelfTestFailure::ClockRegressed);
        }
        let maybe_deadline_ms = match maybe_timeout_ms {
            Some(timeout) => Some(
                now_ms
                    .checked_add(timeout)
                    .ok_or(HardwareSelfTestFailure::DeadlineOverflow)?,
            ),
            None => None,
        };
        self.stage = stage;
        self.entered_at_ms = now_ms;
        self.maybe_deadline_ms = maybe_deadline_ms;
        Ok(())
    }

    #[must_use]
    pub const fn stage(self) -> HardwareSelfTestStage {
        self.stage
    }

    pub fn deadline_expired(self, now_ms: u64) -> Result<bool, HardwareSelfTestFailure> {
        if now_ms < self.entered_at_ms {
            return Err(HardwareSelfTestFailure::ClockRegressed);
        }
        Ok(self
            .maybe_deadline_ms
            .is_some_and(|deadline| now_ms >= deadline))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_metrics() -> HardwareSelfTestMetrics {
        HardwareSelfTestMetrics {
            measured_ms: HARDWARE_SELF_TEST_MEASUREMENT_MS,
            total_hashrate_ghs: expected_hardware_self_test_hashrate_ghs(),
            domain_hashrate_ghs: [expected_hardware_self_test_domain_hashrate_ghs();
                HARDWARE_SELF_TEST_DOMAIN_COUNT],
            domain_sample_counts: [30; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            domain_rejected_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            input_voltage_volts: 5.0,
            core_voltage_mv: HARDWARE_SELF_TEST_CORE_VOLTAGE_MV,
            power_watts: 12.0,
            fan_rpm: HARDWARE_SELF_TEST_FAN_RPM_MIN + 1,
            maximum_temperature_c: HARDWARE_SELF_TEST_TARGET_C,
        }
    }

    #[test]
    fn exact_profile_metrics_pass_and_one_unreliable_domain_is_retained() {
        // Arrange
        let mut metrics = valid_metrics();
        metrics.domain_sample_counts[2] = 0;
        metrics.domain_rejected_counts[2] = 30;

        // Act
        let evaluation = evaluate_hardware_self_test_metrics(metrics).expect("profile should pass");

        // Assert
        assert_eq!(evaluation.unreliable_domain_mask, 0b0100);
    }

    #[test]
    fn each_safety_and_performance_boundary_fails_closed() {
        // Arrange
        let cases = [
            (
                HardwareSelfTestMetrics {
                    maximum_temperature_c: HARDWARE_SELF_TEST_MAX_C + 0.1,
                    ..valid_metrics()
                },
                HardwareSelfTestFailure::TemperatureExceeded,
            ),
            (
                HardwareSelfTestMetrics {
                    total_hashrate_ghs: expected_hardware_self_test_hashrate_ghs() - 0.1,
                    ..valid_metrics()
                },
                HardwareSelfTestFailure::HashrateLow,
            ),
            (
                HardwareSelfTestMetrics {
                    power_watts: HARDWARE_SELF_TEST_POWER_MAX_WATTS + 0.1,
                    ..valid_metrics()
                },
                HardwareSelfTestFailure::PowerInvalid,
            ),
            (
                HardwareSelfTestMetrics {
                    fan_rpm: HARDWARE_SELF_TEST_FAN_RPM_MIN,
                    ..valid_metrics()
                },
                HardwareSelfTestFailure::FanInvalid,
            ),
        ];

        // Act / Assert
        for (metrics, expected) in cases {
            assert_eq!(evaluate_hardware_self_test_metrics(metrics), Err(expected));
        }
    }

    #[test]
    fn schedule_rejects_regression_and_overflow_and_uses_exact_deadline() {
        // Arrange
        let mut schedule = HardwareSelfTestSchedule::admitted(10);

        // Act
        schedule
            .enter(HardwareSelfTestStage::Warming, 20, Some(100))
            .expect("deadline should fit");

        // Assert
        assert_eq!(schedule.deadline_expired(119), Ok(false));
        assert_eq!(schedule.deadline_expired(120), Ok(true));
        assert_eq!(
            schedule.enter(HardwareSelfTestStage::Measuring, 19, None),
            Err(HardwareSelfTestFailure::ClockRegressed)
        );
        assert_eq!(
            schedule.enter(HardwareSelfTestStage::Measuring, u64::MAX, Some(1)),
            Err(HardwareSelfTestFailure::DeadlineOverflow)
        );
    }
}
