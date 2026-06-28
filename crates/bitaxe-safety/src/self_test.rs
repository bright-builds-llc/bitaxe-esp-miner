//! Self-test lifecycle module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/self_test/self_test.c` for factory and manual self-test lifecycle behavior.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for safe blocked interaction during thermal or power faults.
//!
//! This pure module models self-test state and effects without firmware tasks, sleeps, or hardware I/O.

use serde::Serialize;

use crate::evidence::SafetyCriticalEvidence;
use crate::status::SafetyStatus;
use crate::watchdog::{StepProgress, StepSupervisor, WatchdogDecision};

pub const MODULE_NAME: &str = "self_test";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/self_test/self_test.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];

pub const SELF_TEST_MIN_FAN_RPM: u16 = 1000;
pub const SELF_TEST_MIN_FAN_DUTY_PERCENT: u8 = 10;
pub const SELF_TEST_MAX_FAN_DUTY_PERCENT: u8 = 100;
pub const SELF_TEST_DOMAIN_HASHRATE_TOLERANCE: f64 = 0.33;
pub const SELF_TEST_REJECTED_WARN_RATIO: f64 = 0.25;
pub const SELF_TEST_CORE_VOLTAGE_TOLERANCE: f64 = 0.10;
pub const SELF_TEST_POWER_MARGIN_WATTS: f64 = 3.0;
pub const SELF_TEST_INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;
pub const SELF_TEST_DIFFICULTY: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SelfTestOrigin {
    FactoryFlag,
    BootButton,
    ManualApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SelfTestStep {
    Started,
    FanCheck,
    PowerCheck,
    DiagnosticWork,
    ReportResult,
}

impl SelfTestStep {
    const fn next(self) -> Self {
        match self {
            Self::Started => Self::FanCheck,
            Self::FanCheck => Self::PowerCheck,
            Self::PowerCheck => Self::DiagnosticWork,
            Self::DiagnosticWork | Self::ReportResult => Self::ReportResult,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SelfTestCommand {
    StartFactory,
    StartBootButton,
    StartManual,
    Step { progress: StepProgress },
    Pass,
    Fail { reason: &'static str },
    Restart,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SelfTestState {
    Idle,
    Running {
        origin: SelfTestOrigin,
        step: SelfTestStep,
    },
    Passed,
    Failed {
        reason: &'static str,
    },
    Canceled,
    RestartRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SelfTestEffect {
    SetFactoryFlag { enabled: bool },
    ClearFactoryFlag,
    BlockProductionMining,
    UseDiagnosticWorkOnly,
    RecordResult { passed: bool },
    PublishStatus(SafetyStatus),
    RestartSelfTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SelfTestEvidence {
    pub safety_evidence: SafetyCriticalEvidence,
    pub power_evidence_present: bool,
    pub thermal_evidence_present: bool,
    pub asic_evidence_present: bool,
    pub hardware_evidence_ack: bool,
}

impl SelfTestEvidence {
    #[must_use]
    pub const fn missing() -> Self {
        Self {
            safety_evidence: SafetyCriticalEvidence::Missing,
            power_evidence_present: false,
            thermal_evidence_present: false,
            asic_evidence_present: false,
            hardware_evidence_ack: false,
        }
    }

    #[must_use]
    pub const fn diagnostic_hardware_acknowledged(evidence_id: &'static str) -> Self {
        Self {
            safety_evidence: SafetyCriticalEvidence::hardware_smoke(evidence_id),
            power_evidence_present: true,
            thermal_evidence_present: true,
            asic_evidence_present: true,
            hardware_evidence_ack: true,
        }
    }

    #[must_use]
    pub const fn allows_diagnostic_hardware(self) -> bool {
        self.safety_evidence.is_hardware_verified()
            && self.power_evidence_present
            && self.thermal_evidence_present
            && self.asic_evidence_present
            && self.hardware_evidence_ack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfTestDecision {
    pub state: SelfTestState,
    pub effects: Vec<SelfTestEffect>,
    pub watchdog_decision: Option<WatchdogDecision>,
}

pub struct SelfTestLifecycle;

impl SelfTestLifecycle {
    #[must_use]
    pub fn apply(
        state: SelfTestState,
        command: SelfTestCommand,
        evidence: SelfTestEvidence,
    ) -> SelfTestDecision {
        match command {
            SelfTestCommand::StartFactory => Self::start(SelfTestOrigin::FactoryFlag, evidence),
            SelfTestCommand::StartBootButton => Self::start(SelfTestOrigin::BootButton, evidence),
            SelfTestCommand::StartManual => Self::start(SelfTestOrigin::ManualApi, evidence),
            SelfTestCommand::Step { progress } => Self::step(state, progress, evidence),
            SelfTestCommand::Pass => Self::pass(state),
            SelfTestCommand::Fail { reason } => Self::fail(reason),
            SelfTestCommand::Restart => Self::restart(),
            SelfTestCommand::Cancel => Self::cancel(state),
        }
    }

    fn start(origin: SelfTestOrigin, evidence: SelfTestEvidence) -> SelfTestDecision {
        let state = SelfTestState::Running {
            origin,
            step: SelfTestStep::Started,
        };

        if !evidence.allows_diagnostic_hardware() {
            return SelfTestDecision {
                state,
                effects: missing_evidence_effects(),
                watchdog_decision: None,
            };
        }

        let mut effects = vec![
            SelfTestEffect::BlockProductionMining,
            SelfTestEffect::UseDiagnosticWorkOnly,
            SelfTestEffect::PublishStatus(SafetyStatus::SelfTestRunning),
        ];
        if origin == SelfTestOrigin::FactoryFlag {
            effects.push(SelfTestEffect::SetFactoryFlag { enabled: true });
        }

        SelfTestDecision {
            state,
            effects,
            watchdog_decision: None,
        }
    }

    fn step(
        state: SelfTestState,
        progress: StepProgress,
        evidence: SelfTestEvidence,
    ) -> SelfTestDecision {
        let SelfTestState::Running { origin, step } = state else {
            return SelfTestDecision {
                state,
                effects: vec![SelfTestEffect::BlockProductionMining],
                watchdog_decision: Some(StepSupervisor::decision(progress)),
            };
        };

        let next_state = SelfTestState::Running {
            origin,
            step: step.next(),
        };
        let watchdog_decision = StepSupervisor::decision(progress);

        if !evidence.allows_diagnostic_hardware() {
            return SelfTestDecision {
                state: next_state,
                effects: missing_evidence_effects(),
                watchdog_decision: Some(watchdog_decision),
            };
        }

        SelfTestDecision {
            state: next_state,
            effects: vec![
                SelfTestEffect::BlockProductionMining,
                SelfTestEffect::UseDiagnosticWorkOnly,
                SelfTestEffect::PublishStatus(SafetyStatus::SelfTestRunning),
            ],
            watchdog_decision: Some(watchdog_decision),
        }
    }

    fn pass(state: SelfTestState) -> SelfTestDecision {
        let mut effects = vec![
            SelfTestEffect::BlockProductionMining,
            SelfTestEffect::RecordResult { passed: true },
            SelfTestEffect::PublishStatus(SafetyStatus::SelfTestPassed),
        ];
        if factory_origin(state) {
            effects.push(SelfTestEffect::ClearFactoryFlag);
        }

        SelfTestDecision {
            state: SelfTestState::Passed,
            effects,
            watchdog_decision: None,
        }
    }

    fn fail(reason: &'static str) -> SelfTestDecision {
        SelfTestDecision {
            state: SelfTestState::Failed { reason },
            effects: vec![
                SelfTestEffect::BlockProductionMining,
                SelfTestEffect::RecordResult { passed: false },
                SelfTestEffect::PublishStatus(SafetyStatus::SelfTestFailed { reason }),
            ],
            watchdog_decision: None,
        }
    }

    fn restart() -> SelfTestDecision {
        let reason = "self_test_restart_requested";
        SelfTestDecision {
            state: SelfTestState::RestartRequested,
            effects: vec![
                SelfTestEffect::BlockProductionMining,
                SelfTestEffect::RestartSelfTest,
                SelfTestEffect::PublishStatus(SafetyStatus::SafeBlocked { reason }),
            ],
            watchdog_decision: None,
        }
    }

    fn cancel(state: SelfTestState) -> SelfTestDecision {
        let reason = "self_test_canceled";
        let mut effects = vec![
            SelfTestEffect::BlockProductionMining,
            SelfTestEffect::RecordResult { passed: false },
            SelfTestEffect::PublishStatus(SafetyStatus::SafeBlocked { reason }),
        ];
        if factory_origin(state) {
            effects.push(SelfTestEffect::ClearFactoryFlag);
        }

        SelfTestDecision {
            state: SelfTestState::Canceled,
            effects,
            watchdog_decision: None,
        }
    }
}

fn missing_evidence_effects() -> Vec<SelfTestEffect> {
    let reason = "self_test_hardware_evidence_missing";
    vec![
        SelfTestEffect::BlockProductionMining,
        SelfTestEffect::PublishStatus(SafetyStatus::SafeBlocked { reason }),
    ]
}

fn factory_origin(state: SelfTestState) -> bool {
    matches!(
        state,
        SelfTestState::Running {
            origin: SelfTestOrigin::FactoryFlag,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::watchdog::StepKind;

    #[test]
    fn self_test_factory_and_boot_button_start_running_with_mining_blocked() {
        // Arrange
        let evidence = SelfTestEvidence::diagnostic_hardware_acknowledged(
            "phase-06-self-test-diagnostic-smoke",
        );

        // Act
        let factory =
            SelfTestLifecycle::apply(SelfTestState::Idle, SelfTestCommand::StartFactory, evidence);
        let boot = SelfTestLifecycle::apply(
            SelfTestState::Idle,
            SelfTestCommand::StartBootButton,
            evidence,
        );

        // Assert
        assert!(matches!(
            factory.state,
            SelfTestState::Running {
                origin: SelfTestOrigin::FactoryFlag,
                step: SelfTestStep::Started
            }
        ));
        assert!(factory
            .effects
            .contains(&SelfTestEffect::BlockProductionMining));
        assert!(factory
            .effects
            .contains(&SelfTestEffect::SetFactoryFlag { enabled: true }));
        assert!(matches!(
            boot.state,
            SelfTestState::Running {
                origin: SelfTestOrigin::BootButton,
                step: SelfTestStep::Started
            }
        ));
        assert!(boot
            .effects
            .contains(&SelfTestEffect::BlockProductionMining));
    }

    #[test]
    fn self_test_pass_cancel_and_fail_report_results_and_factory_flag_policy() {
        // Arrange
        let factory_running = SelfTestState::Running {
            origin: SelfTestOrigin::FactoryFlag,
            step: SelfTestStep::ReportResult,
        };
        let manual_running = SelfTestState::Running {
            origin: SelfTestOrigin::ManualApi,
            step: SelfTestStep::ReportResult,
        };

        // Act
        let passed = SelfTestLifecycle::apply(
            factory_running,
            SelfTestCommand::Pass,
            SelfTestEvidence::missing(),
        );
        let canceled = SelfTestLifecycle::apply(
            factory_running,
            SelfTestCommand::Cancel,
            SelfTestEvidence::missing(),
        );
        let failed = SelfTestLifecycle::apply(
            manual_running,
            SelfTestCommand::Fail {
                reason: "hashrate_fail",
            },
            SelfTestEvidence::missing(),
        );

        // Assert
        assert_eq!(passed.state, SelfTestState::Passed);
        assert!(passed.effects.contains(&SelfTestEffect::ClearFactoryFlag));
        assert!(passed
            .effects
            .contains(&SelfTestEffect::RecordResult { passed: true }));
        assert_eq!(canceled.state, SelfTestState::Canceled);
        assert!(canceled.effects.contains(&SelfTestEffect::ClearFactoryFlag));
        assert_eq!(
            failed.state,
            SelfTestState::Failed {
                reason: "hashrate_fail"
            }
        );
        assert!(failed
            .effects
            .contains(&SelfTestEffect::BlockProductionMining));
        assert!(!failed.effects.contains(&SelfTestEffect::ClearFactoryFlag));
    }

    #[test]
    fn self_test_restart_cancel_and_missing_evidence_never_enable_production_work() {
        // Arrange
        let running = SelfTestState::Running {
            origin: SelfTestOrigin::ManualApi,
            step: SelfTestStep::Started,
        };

        // Act
        let missing = SelfTestLifecycle::apply(
            SelfTestState::Idle,
            SelfTestCommand::StartManual,
            SelfTestEvidence::missing(),
        );
        let restart = SelfTestLifecycle::apply(
            running,
            SelfTestCommand::Restart,
            SelfTestEvidence::missing(),
        );
        let cancel = SelfTestLifecycle::apply(
            running,
            SelfTestCommand::Cancel,
            SelfTestEvidence::missing(),
        );

        // Assert
        assert_eq!(
            missing.effects,
            vec![
                SelfTestEffect::BlockProductionMining,
                SelfTestEffect::PublishStatus(SafetyStatus::SafeBlocked {
                    reason: "self_test_hardware_evidence_missing"
                }),
            ]
        );
        assert!(!missing
            .effects
            .contains(&SelfTestEffect::UseDiagnosticWorkOnly));
        assert!(restart.effects.contains(&SelfTestEffect::RestartSelfTest));
        assert!(cancel
            .effects
            .contains(&SelfTestEffect::BlockProductionMining));
        for decision in [missing, restart, cancel] {
            assert!(!decision.effects.iter().any(|effect| matches!(
                effect,
                SelfTestEffect::PublishStatus(SafetyStatus::Normal)
            )));
        }
    }

    #[test]
    fn self_test_step_uses_bounded_watchdog_supervision() {
        // Arrange
        let running = SelfTestState::Running {
            origin: SelfTestOrigin::ManualApi,
            step: SelfTestStep::Started,
        };
        let progress = StepProgress {
            kind: StepKind::SelfTest,
            elapsed_ms: 26,
            consecutive_steps: 1,
        };

        // Act
        let decision = SelfTestLifecycle::apply(
            running,
            SelfTestCommand::Step { progress },
            SelfTestEvidence::diagnostic_hardware_acknowledged(
                "phase-06-self-test-diagnostic-smoke",
            ),
        );

        // Assert
        assert_eq!(
            decision.watchdog_decision,
            Some(WatchdogDecision::ResetOrFeedWatchdog {
                reason: "step_budget_exceeded"
            })
        );
        assert!(matches!(
            decision.state,
            SelfTestState::Running {
                step: SelfTestStep::FanCheck,
                ..
            }
        ));
    }

    #[test]
    fn self_test_fixtures_include_required_provenance() {
        // Arrange
        let fixture: Value = serde_json::from_str(include_str!(
            "../fixtures/safety/self-test-lifecycle-cases.json"
        ))
        .expect("self-test fixture should parse");

        // Act
        let serialized = fixture.to_string();

        // Assert
        for expected in [
            "SELF-001",
            "SAFE-05",
            "SAFE-08",
            "SAFE-09",
            "fake nonce",
            "mock Stratum",
            "c1915b0a63bfabebdb95a515cedfee05146c1d50",
        ] {
            assert!(serialized.contains(expected), "missing {expected}");
        }
    }
}
