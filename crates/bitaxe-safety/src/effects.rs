//! Shared safety effect contracts.
//!
//! Phase 6 boundary notes:
//! - D-03: fail closed with explicit reset, voltage, mining, and status effects.
//! - D-08: represent watchdog yielding as data for bounded firmware work.
//! - D-18: keep safety effects consumable by ASIC and mining gates.
//! - D-20: keep reference behavior at module boundaries without firmware effects.

use serde::Serialize;

use crate::evidence::SafetyCriticalEvidence;
use crate::status::SafetyStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SafetyEffect {
    HoldResetLow,
    DisableAsicEnable,
    SuppressVoltageWrite,
    SetFanDutyPercent { percent: u8 },
    BlockWorkSubmission { reason: &'static str },
    PublishStatus(SafetyStatus),
    YieldWatchdog { after_ms: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SafetyEffectPlan {
    pub status: SafetyStatus,
    pub effects: Vec<SafetyEffect>,
    pub evidence: SafetyCriticalEvidence,
}

impl SafetyEffectPlan {
    #[must_use]
    pub fn observe_only(status: SafetyStatus, evidence: SafetyCriticalEvidence) -> Self {
        Self {
            status,
            effects: vec![SafetyEffect::PublishStatus(status)],
            evidence,
        }
    }

    #[must_use]
    pub fn fail_closed(reason: &'static str) -> Self {
        let status = SafetyStatus::SafeBlocked { reason };
        Self {
            status,
            effects: vec![
                SafetyEffect::HoldResetLow,
                SafetyEffect::DisableAsicEnable,
                SafetyEffect::SuppressVoltageWrite,
                SafetyEffect::BlockWorkSubmission { reason },
                SafetyEffect::PublishStatus(status),
            ],
            evidence: SafetyCriticalEvidence::Missing,
        }
    }

    #[must_use]
    pub fn with_effects(
        status: SafetyStatus,
        effects: Vec<SafetyEffect>,
        evidence: SafetyCriticalEvidence,
    ) -> Self {
        Self {
            status,
            effects,
            evidence,
        }
    }
}

#[cfg(test)]
pub(crate) const MODULE_NAME: &str = "effects";

#[cfg(test)]
mod tests {
    use super::{SafetyEffect, SafetyEffectPlan};
    use crate::status::SafetyStatus;

    #[test]
    fn fail_closed_plan_includes_reset_voltage_work_and_visible_status() {
        // Arrange
        let reason = "power_fault";

        // Act
        let plan = SafetyEffectPlan::fail_closed(reason);

        // Assert
        assert_eq!(plan.status, SafetyStatus::SafeBlocked { reason });
        assert!(plan.effects.contains(&SafetyEffect::HoldResetLow));
        assert!(plan.effects.contains(&SafetyEffect::SuppressVoltageWrite));
        assert!(plan
            .effects
            .contains(&SafetyEffect::BlockWorkSubmission { reason }));
        assert!(plan
            .effects
            .contains(&SafetyEffect::PublishStatus(SafetyStatus::SafeBlocked {
                reason
            })));
    }
}
