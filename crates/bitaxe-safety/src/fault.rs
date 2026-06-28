//! Safety fault-policy decisions.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/tasks/power_management_task.c` for overheat and power safe-stop policy.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for fan set failure and visible fault behavior.
//! - `reference/esp-miner/main/thermal/thermal.c` for unavailable or invalid temperature observations.
//!
//! This pure module classifies sustained faults into visible fail-closed safety plans.

use serde::Serialize;

use crate::effects::{SafetyEffect, SafetyEffectPlan};
use crate::evidence::SafetyCriticalEvidence;
use crate::status::SafetyStatus;

pub const MODULE_NAME: &str = "fault";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/tasks/power_management_task.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/thermal/thermal.c",
];

pub const ZERO_RPM_FAULT_MIN_DUTY_PERCENT: u8 = 30;
pub const ZERO_RPM_FAULT_SAMPLE_COUNT: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SafetyFault {
    Overheat,
    FanZeroRpm,
    FanSetFailed,
    ThermalSensorFault,
    PowerFault,
    AsicFault,
}

impl SafetyFault {
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Overheat => "overheat_safe_stop",
            Self::FanZeroRpm => "fan_zero_rpm",
            Self::FanSetFailed => "fan_set_failed",
            Self::ThermalSensorFault => "thermal_sensor_fault",
            Self::PowerFault => "power_fault",
            Self::AsicFault => "asic_fault",
        }
    }

    #[must_use]
    pub const fn status(self) -> SafetyStatus {
        let reason = self.reason();
        match self {
            Self::Overheat | Self::ThermalSensorFault => SafetyStatus::ThermalFault { reason },
            Self::FanZeroRpm | Self::FanSetFailed => SafetyStatus::FanFault { reason },
            Self::PowerFault => SafetyStatus::PowerFault { reason },
            Self::AsicFault => SafetyStatus::SafeBlocked { reason },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaultDecision {
    pub fault: SafetyFault,
    pub plan: SafetyEffectPlan,
}

impl FaultDecision {
    #[must_use]
    pub fn from_fault(fault: SafetyFault) -> Self {
        let reason = fault.reason();
        Self {
            fault,
            plan: SafetyEffectPlan::with_effects(
                fault.status(),
                vec![
                    SafetyEffect::HoldResetLow,
                    SafetyEffect::SuppressVoltageWrite,
                    SafetyEffect::BlockWorkSubmission { reason },
                    SafetyEffect::PublishStatus(fault.status()),
                ],
                SafetyCriticalEvidence::Missing,
            ),
        }
    }

    #[must_use]
    pub fn from_fan_feedback(
        target_duty_percent: u8,
        rpm: u16,
        consecutive_zero_rpm_samples: u8,
        fan_set_succeeded: bool,
    ) -> Option<Self> {
        if !fan_set_succeeded {
            return Some(Self::from_fault(SafetyFault::FanSetFailed));
        }

        if target_duty_percent >= ZERO_RPM_FAULT_MIN_DUTY_PERCENT
            && rpm == 0
            && consecutive_zero_rpm_samples >= ZERO_RPM_FAULT_SAMPLE_COUNT
        {
            return Some(Self::from_fault(SafetyFault::FanZeroRpm));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_fault_zero_rpm_and_fan_set_failure_publish_visible_faults() {
        // Arrange / Act
        let zero_rpm = FaultDecision::from_fan_feedback(30, 0, 3, true)
            .expect("zero RPM should become a fault");
        let set_failed = FaultDecision::from_fan_feedback(100, 0, 1, false)
            .expect("fan set failure should become a fault");

        // Assert
        assert_eq!(zero_rpm.fault, SafetyFault::FanZeroRpm);
        assert_eq!(
            zero_rpm.plan.status,
            SafetyStatus::FanFault {
                reason: "fan_zero_rpm"
            }
        );
        assert_eq!(set_failed.fault, SafetyFault::FanSetFailed);
        assert!(set_failed
            .plan
            .effects
            .contains(&SafetyEffect::BlockWorkSubmission {
                reason: "fan_set_failed"
            }));
    }

    #[test]
    fn safety_fault_power_thermal_and_asic_faults_fail_closed() {
        // Arrange
        let faults = [
            SafetyFault::Overheat,
            SafetyFault::ThermalSensorFault,
            SafetyFault::PowerFault,
            SafetyFault::AsicFault,
        ];

        // Act / Assert
        for fault in faults {
            let decision = FaultDecision::from_fault(fault);
            assert!(decision.plan.effects.contains(&SafetyEffect::HoldResetLow));
            assert!(decision
                .plan
                .effects
                .contains(&SafetyEffect::SuppressVoltageWrite));
            assert!(matches!(
                decision.plan.status,
                SafetyStatus::ThermalFault { .. }
                    | SafetyStatus::PowerFault { .. }
                    | SafetyStatus::SafeBlocked { .. }
            ));
        }
    }
}
