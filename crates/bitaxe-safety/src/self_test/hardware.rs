//! Pure hardware self-test contract: the shared stages and limits, and each board's profile,
//! hashrate and electrical windows (ADR-0041 item 14).
//!
//! Behavioral breadcrumb: `reference/esp-miner/main/self_test/self_test.c`.

use bitaxe_config::BoardCatalogEntry;
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
/// Upstream's `SELF_TEST_CORE_VOLTAGE_TOLERANCE`, in tenths: the core may read 10 % either side.
const CORE_VOLTAGE_TOLERANCE_DIVISOR: u16 = 10;
/// Upstream's `POWER_CONSUMPTION_MARGIN` over the board's power target.
const POWER_MARGIN_WATTS: f32 = 3.0;

/// The board-specific part of the self-test, derived from the built board's catalog entry as
/// upstream's `self_test.c` derives it from `DEVICE_CONFIG`: the default profile it runs, the
/// small cores its expected hashrate counts, the core voltage window (the default ±10 %) and the
/// power ceiling (the power target + 3 W).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardwareSelfTestContract {
    pub kind: &'static str,
    pub frequency_mhz: u16,
    pub core_voltage_mv: u16,
    pub core_voltage_min_mv: u16,
    pub core_voltage_max_mv: u16,
    pub power_max_watts: f32,
    pub small_core_count: u16,
    /// The regulator temperature ceiling on a board whose regulator reports one (the 601's
    /// TPS546); `None` on the 205, whose DS4432U does not.
    pub maybe_vr_temperature_max_c: Option<f32>,
    /// Whether the board's regulator reports power faults the evaluation must check (upstream's
    /// `test_vreg_faults`).
    pub checks_regulator_faults: bool,
}

impl HardwareSelfTestContract {
    /// The contract for `entry`, whose evidence records carry `kind`.
    #[must_use]
    pub fn for_board(
        kind: &'static str,
        entry: BoardCatalogEntry,
        maybe_vr_temperature_max_c: Option<f32>,
    ) -> Self {
        let asic = entry.asic();
        let core_voltage_mv = asic.default_voltage_mv();
        let tolerance_mv = core_voltage_mv / CORE_VOLTAGE_TOLERANCE_DIVISOR;
        Self {
            kind,
            frequency_mhz: asic.default_frequency_mhz(),
            core_voltage_mv,
            core_voltage_min_mv: core_voltage_mv - tolerance_mv,
            core_voltage_max_mv: core_voltage_mv + tolerance_mv,
            power_max_watts: f32::from(entry.power_consumption_target()) + POWER_MARGIN_WATTS,
            small_core_count: asic.small_core_count(),
            maybe_vr_temperature_max_c,
            checks_regulator_faults: entry.capabilities().tps546(),
        }
    }
}

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
    RegulatorFault,
    VrTemperatureExceeded,
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
            Self::RegulatorFault => "regulator_fault",
            Self::VrTemperatureExceeded => "vr_temperature_exceeded",
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
    /// The highest regulator temperature measured, on a board whose regulator reports one.
    pub maybe_maximum_vr_temperature_c: Option<f32>,
    /// Whether the regulator reported a power fault during the measurement.
    pub regulator_fault: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareSelfTestEvaluation {
    pub unreliable_domain_mask: u8,
}

#[must_use]
pub fn expected_hardware_self_test_hashrate_ghs(contract: &HardwareSelfTestContract) -> f32 {
    (f32::from(contract.frequency_mhz) * f32::from(contract.small_core_count) / 1_000.0)
        * HARDWARE_SELF_TEST_HASHRATE_PERCENT
}

#[must_use]
pub fn expected_hardware_self_test_domain_hashrate_ghs(contract: &HardwareSelfTestContract) -> f32 {
    expected_hardware_self_test_hashrate_ghs(contract) / HARDWARE_SELF_TEST_DOMAIN_COUNT as f32
}

pub fn evaluate_hardware_self_test_metrics(
    contract: &HardwareSelfTestContract,
    metrics: HardwareSelfTestMetrics,
) -> Result<HardwareSelfTestEvaluation, HardwareSelfTestFailure> {
    if metrics.maximum_temperature_c > HARDWARE_SELF_TEST_MAX_C
        || !metrics.maximum_temperature_c.is_finite()
    {
        return Err(HardwareSelfTestFailure::TemperatureExceeded);
    }
    if contract.checks_regulator_faults && metrics.regulator_fault {
        return Err(HardwareSelfTestFailure::RegulatorFault);
    }
    if let Some(maximum_c) = contract.maybe_vr_temperature_max_c {
        let within = metrics
            .maybe_maximum_vr_temperature_c
            .is_some_and(|temperature| temperature.is_finite() && temperature <= maximum_c);
        if !within {
            return Err(HardwareSelfTestFailure::VrTemperatureExceeded);
        }
    }
    if metrics.measured_ms < HARDWARE_SELF_TEST_MEASUREMENT_MS {
        return Err(HardwareSelfTestFailure::MeasurementIncomplete);
    }
    if !metrics.total_hashrate_ghs.is_finite()
        || metrics.total_hashrate_ghs < expected_hardware_self_test_hashrate_ghs(contract)
    {
        return Err(HardwareSelfTestFailure::HashrateLow);
    }
    let expected_domain = expected_hardware_self_test_domain_hashrate_ghs(contract);
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
    if !(contract.core_voltage_min_mv..=contract.core_voltage_max_mv)
        .contains(&metrics.core_voltage_mv)
    {
        return Err(HardwareSelfTestFailure::CoreVoltageInvalid);
    }
    if !metrics.power_watts.is_finite()
        || !(0.0..=contract.power_max_watts).contains(&metrics.power_watts)
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
    use bitaxe_config::{gamma_601_catalog_entry, ultra_205_catalog_entry};

    fn ultra_205() -> HardwareSelfTestContract {
        HardwareSelfTestContract::for_board("ultra205_full_v1", ultra_205_catalog_entry(), None)
    }

    fn gamma_601() -> HardwareSelfTestContract {
        HardwareSelfTestContract::for_board(
            "gamma601_full_v1",
            gamma_601_catalog_entry(),
            Some(105.0),
        )
    }

    fn valid_metrics(contract: &HardwareSelfTestContract) -> HardwareSelfTestMetrics {
        HardwareSelfTestMetrics {
            measured_ms: HARDWARE_SELF_TEST_MEASUREMENT_MS,
            total_hashrate_ghs: expected_hardware_self_test_hashrate_ghs(contract),
            domain_hashrate_ghs: [expected_hardware_self_test_domain_hashrate_ghs(contract);
                HARDWARE_SELF_TEST_DOMAIN_COUNT],
            domain_sample_counts: [30; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            domain_rejected_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            input_voltage_volts: 5.0,
            core_voltage_mv: contract.core_voltage_mv,
            power_watts: 12.0,
            fan_rpm: HARDWARE_SELF_TEST_FAN_RPM_MIN + 1,
            maximum_temperature_c: HARDWARE_SELF_TEST_TARGET_C,
            maybe_maximum_vr_temperature_c: contract.maybe_vr_temperature_max_c.map(|_| 80.0),
            regulator_fault: false,
        }
    }

    #[test]
    fn the_205_contract_keeps_its_verified_constants() {
        // Act
        let contract = ultra_205();
        // Assert
        assert_eq!(
            (
                contract.frequency_mhz,
                contract.core_voltage_mv,
                contract.core_voltage_min_mv,
                contract.core_voltage_max_mv,
                contract.small_core_count,
            ),
            (
                HARDWARE_SELF_TEST_FREQUENCY_MHZ,
                HARDWARE_SELF_TEST_CORE_VOLTAGE_MV,
                HARDWARE_SELF_TEST_CORE_VOLTAGE_MIN_MV,
                HARDWARE_SELF_TEST_CORE_VOLTAGE_MAX_MV,
                HARDWARE_SELF_TEST_SMALL_CORE_COUNT,
            )
        );
        assert_eq!(contract.power_max_watts, HARDWARE_SELF_TEST_POWER_MAX_WATTS);
        assert_eq!(
            (
                contract.maybe_vr_temperature_max_c,
                contract.checks_regulator_faults
            ),
            (None, false)
        );
    }

    #[test]
    fn the_601_contract_runs_its_default_with_its_own_windows() {
        // Act
        let contract = gamma_601();
        // Assert
        assert_eq!(
            (
                contract.frequency_mhz,
                contract.core_voltage_mv,
                contract.core_voltage_min_mv,
                contract.core_voltage_max_mv,
                contract.small_core_count,
            ),
            (525, 1_150, 1_035, 1_265, 2_040)
        );
        assert_eq!(contract.power_max_watts, 22.0);
        assert!(contract.checks_regulator_faults);
        assert!((expected_hardware_self_test_hashrate_ghs(&contract) - 910.35).abs() < 0.01);
    }

    #[test]
    fn exact_profile_metrics_pass_and_one_unreliable_domain_is_retained() {
        // Arrange
        let contract = ultra_205();
        let mut metrics = valid_metrics(&contract);
        metrics.domain_sample_counts[2] = 0;
        metrics.domain_rejected_counts[2] = 30;

        // Act
        let evaluation =
            evaluate_hardware_self_test_metrics(&contract, metrics).expect("profile should pass");

        // Assert
        assert_eq!(evaluation.unreliable_domain_mask, 0b0100);
    }

    #[test]
    fn each_safety_and_performance_boundary_fails_closed() {
        // Arrange
        let contract = ultra_205();
        let cases = [
            (
                HardwareSelfTestMetrics {
                    maximum_temperature_c: HARDWARE_SELF_TEST_MAX_C + 0.1,
                    ..valid_metrics(&contract)
                },
                HardwareSelfTestFailure::TemperatureExceeded,
            ),
            (
                HardwareSelfTestMetrics {
                    total_hashrate_ghs: expected_hardware_self_test_hashrate_ghs(&contract) - 0.1,
                    ..valid_metrics(&contract)
                },
                HardwareSelfTestFailure::HashrateLow,
            ),
            (
                HardwareSelfTestMetrics {
                    power_watts: HARDWARE_SELF_TEST_POWER_MAX_WATTS + 0.1,
                    ..valid_metrics(&contract)
                },
                HardwareSelfTestFailure::PowerInvalid,
            ),
            (
                HardwareSelfTestMetrics {
                    fan_rpm: HARDWARE_SELF_TEST_FAN_RPM_MIN,
                    ..valid_metrics(&contract)
                },
                HardwareSelfTestFailure::FanInvalid,
            ),
        ];

        // Act / Assert
        for (metrics, expected) in cases {
            assert_eq!(
                evaluate_hardware_self_test_metrics(&contract, metrics),
                Err(expected)
            );
        }
    }

    #[test]
    fn the_205_ignores_regulator_facts_it_does_not_have() {
        // Arrange
        let contract = ultra_205();
        let metrics = HardwareSelfTestMetrics {
            regulator_fault: true,
            maybe_maximum_vr_temperature_c: None,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluation = evaluate_hardware_self_test_metrics(&contract, metrics);
        // Assert
        assert!(evaluation.is_ok());
    }

    #[test]
    fn a_601_passes_at_its_default_profile() {
        // Arrange
        let contract = gamma_601();
        let metrics = HardwareSelfTestMetrics {
            power_watts: 20.4,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluation = evaluate_hardware_self_test_metrics(&contract, metrics);
        // Assert
        assert!(evaluation.is_ok());
    }

    #[test]
    fn a_601_regulator_fault_fails() {
        // Arrange
        let contract = gamma_601();
        let metrics = HardwareSelfTestMetrics {
            regulator_fault: true,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluation = evaluate_hardware_self_test_metrics(&contract, metrics);
        // Assert
        assert_eq!(evaluation, Err(HardwareSelfTestFailure::RegulatorFault));
    }

    #[test]
    fn a_601_regulator_above_its_ceiling_or_unread_fails() {
        // Arrange
        let contract = gamma_601();
        let hot = HardwareSelfTestMetrics {
            maybe_maximum_vr_temperature_c: Some(105.1),
            ..valid_metrics(&contract)
        };
        let unread = HardwareSelfTestMetrics {
            maybe_maximum_vr_temperature_c: None,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluations =
            [hot, unread].map(|metrics| evaluate_hardware_self_test_metrics(&contract, metrics));
        // Assert
        assert_eq!(
            evaluations,
            [Err(HardwareSelfTestFailure::VrTemperatureExceeded); 2]
        );
    }

    #[test]
    fn a_601_above_22_w_fails_on_power() {
        // Arrange
        let contract = gamma_601();
        let metrics = HardwareSelfTestMetrics {
            power_watts: 22.1,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluation = evaluate_hardware_self_test_metrics(&contract, metrics);
        // Assert
        assert_eq!(evaluation, Err(HardwareSelfTestFailure::PowerInvalid));
    }

    #[test]
    fn a_601_core_below_1035_mv_fails_on_core_voltage() {
        // Arrange
        let contract = gamma_601();
        let metrics = HardwareSelfTestMetrics {
            core_voltage_mv: 1_034,
            ..valid_metrics(&contract)
        };
        // Act
        let evaluation = evaluate_hardware_self_test_metrics(&contract, metrics);
        // Assert
        assert_eq!(evaluation, Err(HardwareSelfTestFailure::CoreVoltageInvalid));
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
