//! Shared safety status contracts.
//!
//! Phase 6 boundary notes:
//! - D-03: safe-blocked and fault states must remain user-visible.
//! - D-17: unavailable telemetry and faulted status are explicit instead of zeroed.
//! - D-20: public reasons are stable contract strings for later adapters.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SafetyStatus {
    Normal,
    Unavailable { reason: &'static str },
    SafeBlocked { reason: &'static str },
    PowerFault { reason: &'static str },
    ThermalFault { reason: &'static str },
    FanFault { reason: &'static str },
    SelfTestRunning,
    SelfTestPassed,
    SelfTestFailed { reason: &'static str },
}

impl SafetyStatus {
    #[must_use]
    pub const fn public_reason(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Unavailable { reason }
            | Self::SafeBlocked { reason }
            | Self::PowerFault { reason }
            | Self::ThermalFault { reason }
            | Self::FanFault { reason }
            | Self::SelfTestFailed { reason } => reason,
            Self::SelfTestRunning => "self_test_running",
            Self::SelfTestPassed => "self_test_passed",
        }
    }
}

#[cfg(test)]
pub(crate) const MODULE_NAME: &str = "status";

#[cfg(test)]
mod tests {
    use super::SafetyStatus;

    #[test]
    fn public_reason_returns_visible_fault_reason() {
        // Arrange
        let status = SafetyStatus::PowerFault {
            reason: "power_fault",
        };

        // Act
        let reason = status.public_reason();

        // Assert
        assert_eq!(reason, "power_fault");
    }
}
