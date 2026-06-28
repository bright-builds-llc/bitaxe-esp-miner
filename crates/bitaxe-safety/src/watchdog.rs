//! Watchdog supervision module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/tasks/power_management_task.c` for bounded power-management loop behavior.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for bounded fan-control loop behavior.
//! - `reference/esp-miner/main/self_test/self_test.c` for self-test work that must remain supervised.
//!
//! This pure module models bounded work steps without firmware timing or task effects.

use serde::Serialize;

pub const MODULE_NAME: &str = "watchdog";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/tasks/power_management_task.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/self_test/self_test.c",
];

pub const SAFETY_STEP_BUDGET_MS: u32 = 25;
pub const WATCHDOG_YIELD_INTERVAL_MS: u32 = 100;
pub const MAX_CONSECUTIVE_STEPS_BEFORE_YIELD: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StepKind {
    Power,
    Thermal,
    Fan,
    SelfTest,
    Telemetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StepProgress {
    pub kind: StepKind,
    pub elapsed_ms: u32,
    pub consecutive_steps: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WatchdogDecision {
    Continue,
    YieldNow { reason: &'static str },
    ResetOrFeedWatchdog { reason: &'static str },
}

pub struct StepSupervisor;

impl StepSupervisor {
    #[must_use]
    pub fn decision(progress: StepProgress) -> WatchdogDecision {
        if progress.elapsed_ms > SAFETY_STEP_BUDGET_MS {
            return WatchdogDecision::ResetOrFeedWatchdog {
                reason: "step_budget_exceeded",
            };
        }

        let accumulated_elapsed_ms = progress
            .elapsed_ms
            .saturating_mul(u32::from(progress.consecutive_steps));
        if accumulated_elapsed_ms >= WATCHDOG_YIELD_INTERVAL_MS {
            return WatchdogDecision::YieldNow {
                reason: "yield_interval_reached",
            };
        }

        if progress.consecutive_steps >= MAX_CONSECUTIVE_STEPS_BEFORE_YIELD {
            return WatchdogDecision::YieldNow {
                reason: "consecutive_step_limit_reached",
            };
        }

        WatchdogDecision::Continue
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn watchdog_allows_step_at_or_below_budget_without_yield() {
        // Arrange
        let progress = StepProgress {
            kind: StepKind::Power,
            elapsed_ms: SAFETY_STEP_BUDGET_MS,
            consecutive_steps: 1,
        };

        // Act
        let decision = StepSupervisor::decision(progress);

        // Assert
        assert_eq!(decision, WatchdogDecision::Continue);
    }

    #[test]
    fn watchdog_requires_reset_or_feed_when_step_budget_is_exceeded() {
        // Arrange
        let progress = StepProgress {
            kind: StepKind::Thermal,
            elapsed_ms: SAFETY_STEP_BUDGET_MS + 1,
            consecutive_steps: 1,
        };

        // Act
        let decision = StepSupervisor::decision(progress);

        // Assert
        assert_eq!(
            decision,
            WatchdogDecision::ResetOrFeedWatchdog {
                reason: "step_budget_exceeded"
            }
        );
    }

    #[test]
    fn watchdog_yields_when_interval_or_consecutive_limit_is_reached() {
        // Arrange
        let interval = StepProgress {
            kind: StepKind::Fan,
            elapsed_ms: 25,
            consecutive_steps: 4,
        };
        let consecutive = StepProgress {
            kind: StepKind::Telemetry,
            elapsed_ms: 1,
            consecutive_steps: MAX_CONSECUTIVE_STEPS_BEFORE_YIELD,
        };

        // Act
        let interval_decision = StepSupervisor::decision(interval);
        let consecutive_decision = StepSupervisor::decision(consecutive);

        // Assert
        assert_eq!(
            interval_decision,
            WatchdogDecision::YieldNow {
                reason: "yield_interval_reached"
            }
        );
        assert_eq!(
            consecutive_decision,
            WatchdogDecision::YieldNow {
                reason: "consecutive_step_limit_reached"
            }
        );
    }

    #[test]
    fn watchdog_fixtures_include_required_provenance() {
        // Arrange
        let fixture: Value =
            serde_json::from_str(include_str!("../fixtures/safety/watchdog-step-cases.json"))
                .expect("watchdog fixture should parse");

        // Act
        let serialized = fixture.to_string();

        // Assert
        for expected in [
            "SAFE-09",
            "Power",
            "Thermal",
            "Fan",
            "SelfTest",
            "Telemetry",
            "100 ms",
            "c1915b0a63bfabebdb95a515cedfee05146c1d50",
        ] {
            assert!(serialized.contains(expected), "missing {expected}");
        }
    }
}
