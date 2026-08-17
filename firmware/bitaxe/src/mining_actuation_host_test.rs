#![allow(dead_code)]

#[path = "mining_actuation.rs"]
mod mining_actuation;

use bitaxe_stratum::v1::production_session::{
    HardwareSafeStopPurpose, MiningCampaignDuration, MiningCampaignLease, MiningCampaignLeaseId,
    MiningCampaignState, MiningCampaignStopCondition, MiningHardwareProfile, MiningHardwareState,
    ProductionMiningSession, ProductionReadiness, ProductionSessionEffect, ProductionSessionEvent,
    ProductionSessionWakeup,
};
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use mining_actuation::{
    execute_preparation, execute_resumable_pause_shutdown, execute_safe_shutdown,
    execute_safe_stop, execute_safe_stop_with_progress, preparation_plan,
    resumable_pause_shutdown_plan, safe_shutdown_plan, MiningActuationBackend, PreparationStep,
    SafeShutdownStep,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InjectedFailure {
    Preparation(PreparationStep),
    SafeShutdown(SafeShutdownStep),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordedStep {
    Preparation(PreparationStep),
    SafeShutdown(SafeShutdownStep),
}

#[derive(Default)]
struct RecordingBackend {
    recorded: Vec<RecordedStep>,
    preparation_failures: Vec<PreparationStep>,
    safe_shutdown_failures: Vec<SafeShutdownStep>,
}

impl RecordingBackend {
    fn failing_preparation(step: PreparationStep) -> Self {
        Self {
            preparation_failures: vec![step],
            ..Self::default()
        }
    }

    fn failing_safe_shutdown(step: SafeShutdownStep) -> Self {
        Self {
            safe_shutdown_failures: vec![step],
            ..Self::default()
        }
    }
}

impl MiningActuationBackend for RecordingBackend {
    type Error = InjectedFailure;

    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        self.recorded.push(RecordedStep::Preparation(step));
        if self.preparation_failures.contains(&step) {
            return Err(InjectedFailure::Preparation(step));
        }

        Ok(())
    }

    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.recorded.push(RecordedStep::SafeShutdown(step));
        if self.safe_shutdown_failures.contains(&step) {
            return Err(InjectedFailure::SafeShutdown(step));
        }

        Ok(())
    }
}

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::ultra_205_bm1366(400, 1_100, 100)
        .expect("test hardware profile should be valid")
}

#[test]
fn preparation_plan_has_the_golden_safety_order() {
    // Arrange
    let profile = profile();

    // Act
    let plan = preparation_plan(profile);

    // Assert
    assert_eq!(
        plan,
        [
            PreparationStep::RequireFreshSafetyObservations,
            PreparationStep::SetFanDutyTo100Percent,
            PreparationStep::RequireFreshNonzeroFanRpm,
            PreparationStep::SetCoreVoltage(profile.core_voltage()),
            PreparationStep::WaitForCoreVoltageStabilization500Ms,
            PreparationStep::EnableAsic,
            PreparationStep::ResetAndDetectExactlyOneChip,
            PreparationStep::InitializeMiningReadyWithFrequencyRamp(profile.frequency()),
            PreparationStep::RetainProductionUart,
        ]
    );
}

#[test]
fn safe_shutdown_plan_has_the_golden_fail_closed_order() {
    // Arrange
    let expected = [
        SafeShutdownStep::StopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce,
        SafeShutdownStep::HoldResetLow,
        SafeShutdownStep::DisableCoreVoltage,
        SafeShutdownStep::DisableAsic,
        SafeShutdownStep::SetFanDutyTo100Percent,
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,
        SafeShutdownStep::SetFanDutyTo30Percent,
    ];

    // Act
    let plan = safe_shutdown_plan();

    // Assert
    assert_eq!(plan, expected);
}

#[test]
fn resumable_pause_plan_stops_hardware_promptly_and_retains_full_fan_duty() {
    // Arrange
    let expected = [
        SafeShutdownStep::StopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce,
        SafeShutdownStep::HoldResetLow,
        SafeShutdownStep::DisableCoreVoltage,
        SafeShutdownStep::DisableAsic,
        SafeShutdownStep::SetFanDutyTo100Percent,
    ];

    // Act
    let plan = resumable_pause_shutdown_plan();

    // Assert
    assert_eq!(plan, expected);
    assert!(!plan.contains(&SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C));
    assert!(!plan.contains(&SafeShutdownStep::SetFanDutyTo30Percent));
}

#[test]
fn successful_preparation_executes_the_golden_order() {
    // Arrange
    let profile = profile();
    let mut backend = RecordingBackend::default();
    let expected = preparation_plan(profile)
        .map(RecordedStep::Preparation)
        .to_vec();

    // Act
    let result = execute_preparation(&mut backend, profile);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(backend.recorded, expected);
}

#[test]
fn every_preparation_stage_failure_runs_the_same_complete_safe_shutdown() {
    // Arrange
    let profile = profile();
    let preparation = preparation_plan(profile);
    let safe_shutdown = safe_shutdown_plan();

    for (failed_index, failed_step) in preparation.into_iter().enumerate() {
        let mut backend = RecordingBackend::failing_preparation(failed_step);
        let mut expected = preparation[..=failed_index]
            .iter()
            .copied()
            .map(RecordedStep::Preparation)
            .collect::<Vec<_>>();
        expected.extend(
            safe_shutdown
                .iter()
                .copied()
                .map(RecordedStep::SafeShutdown),
        );

        // Act
        let failure = execute_preparation(&mut backend, profile)
            .expect_err("injected preparation failure should fail");

        // Assert
        assert_eq!(failure.original().step(), failed_step);
        assert_eq!(
            failure.original().source(),
            &InjectedFailure::Preparation(failed_step)
        );
        assert_eq!(failure.maybe_safe_shutdown_failure(), None);
        assert_eq!(backend.recorded, expected);
    }
}

#[test]
fn rollback_failure_never_overwrites_the_original_preparation_failure() {
    // Arrange
    let original_step = PreparationStep::EnableAsic;
    let rollback_step = SafeShutdownStep::DisableCoreVoltage;
    let mut backend = RecordingBackend {
        preparation_failures: vec![original_step],
        safe_shutdown_failures: vec![rollback_step],
        ..RecordingBackend::default()
    };

    // Act
    let failure = execute_preparation(&mut backend, profile())
        .expect_err("preparation and rollback failures should be reported");

    // Assert
    assert_eq!(failure.original().step(), original_step);
    assert_eq!(
        failure.original().source(),
        &InjectedFailure::Preparation(original_step)
    );
    let rollback_failure = failure
        .maybe_safe_shutdown_failure()
        .expect("rollback failure should be retained separately");
    assert_eq!(rollback_failure.step(), rollback_step);
    assert_eq!(
        rollback_failure.source(),
        &InjectedFailure::SafeShutdown(rollback_step)
    );
}

#[test]
fn preparation_stops_at_the_earliest_injected_failure() {
    // Arrange
    let first_failure = PreparationStep::RequireFreshNonzeroFanRpm;
    let later_failure = PreparationStep::EnableAsic;
    let mut backend = RecordingBackend {
        preparation_failures: vec![first_failure, later_failure],
        ..RecordingBackend::default()
    };

    // Act
    let failure = execute_preparation(&mut backend, profile())
        .expect_err("the earliest preparation failure should stop preparation");

    // Assert
    assert_eq!(failure.original().step(), first_failure);
    assert!(!backend
        .recorded
        .contains(&RecordedStep::Preparation(later_failure)));
}

#[test]
fn safe_shutdown_preserves_the_earliest_failure_while_attempting_the_full_plan() {
    // Arrange
    let first_failure = SafeShutdownStep::HoldResetLow;
    let later_failure = SafeShutdownStep::DisableAsic;
    let mut backend = RecordingBackend {
        safe_shutdown_failures: vec![first_failure, later_failure],
        ..RecordingBackend::default()
    };

    // Act
    let failure = execute_safe_shutdown(&mut backend)
        .expect_err("the earliest safe-shutdown failure should be retained");

    // Assert
    assert_eq!(failure.step(), first_failure);
    assert_eq!(
        failure.source(),
        &InjectedFailure::SafeShutdown(first_failure)
    );
    assert_eq!(
        backend.recorded,
        safe_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
}

#[test]
fn resumable_pause_preserves_the_earliest_failure_while_attempting_its_full_plan() {
    // Arrange
    let first_failure = SafeShutdownStep::HoldResetLow;
    let later_failure = SafeShutdownStep::DisableAsic;
    let mut backend = RecordingBackend {
        safe_shutdown_failures: vec![first_failure, later_failure],
        ..RecordingBackend::default()
    };

    // Act
    let failure = execute_resumable_pause_shutdown(&mut backend)
        .expect_err("the earliest resumable-pause failure should be retained");

    // Assert
    assert_eq!(failure.step(), first_failure);
    assert_eq!(
        backend.recorded,
        resumable_pause_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
}

#[test]
fn typed_stop_purpose_selects_prompt_pause_and_complete_terminal_plans() {
    // Arrange
    let mut pause_backend = RecordingBackend::default();
    let mut terminal_backend = RecordingBackend::default();

    // Act
    execute_safe_stop(
        &mut pause_backend,
        HardwareSafeStopPurpose::ResumablePause,
    )
    .expect("resumable pause should succeed");
    execute_safe_stop(&mut terminal_backend, HardwareSafeStopPurpose::Terminal)
        .expect("terminal stop should succeed");

    // Assert
    assert_eq!(
        pause_backend.recorded,
        resumable_pause_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
    assert_eq!(
        terminal_backend.recorded,
        safe_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
}

#[test]
fn safe_stop_reports_progress_after_every_ordered_step() {
    // Arrange
    let mut backend = RecordingBackend::default();
    let mut progress_count = 0_usize;

    // Act
    execute_safe_stop_with_progress(
        &mut backend,
        HardwareSafeStopPurpose::ResumablePause,
        &mut || progress_count = progress_count.saturating_add(1),
    )
    .expect("resumable safe stop should succeed");

    // Assert
    assert_eq!(
        progress_count,
        resumable_pause_shutdown_plan().len()
    );
    assert_eq!(
        backend.recorded,
        resumable_pause_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
}

#[test]
fn production_resumable_pause_reaches_same_lease_confirmation_without_cooling_wait() {
    // Arrange
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(9).expect("lease id should be valid"),
        profile(),
        MiningCampaignStopCondition::ResumableActiveEpoch {
            activation_timeout: MiningCampaignDuration::new(1_000)
                .expect("activation timeout should be valid"),
            duration: MiningCampaignDuration::new(1_000).expect("duration should be valid"),
        },
    );
    let running = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: true,
        maybe_campaign_lease: Some(lease),
        actuation_qualified: true,
    };
    let mut session = ProductionMiningSession::new();
    let preparing = session
        .handle(ProductionSessionEvent::Wake {
            wakeup: None,
            readiness: running,
            now_ms: 0,
        })
        .expect("initial wake should be accepted");
    assert!(preparing.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::PrepareHardware { lease_id, .. } if *lease_id == lease.id()
    )));
    session
        .handle(ProductionSessionEvent::HardwarePrepared {
            lease_id: lease.id(),
            now_ms: 1,
        })
        .expect("hardware preparation should be accepted");
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };

    // Act
    let effects = session
        .handle(ProductionSessionEvent::Wake {
            wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
            readiness: paused,
            now_ms: 2,
        })
        .expect("operator pause should be accepted");
    let purpose = effects
        .iter()
        .find_map(|effect| match effect {
            ProductionSessionEffect::SafeStopHardware { lease_id, purpose }
                if *lease_id == lease.id() =>
            {
                Some(*purpose)
            }
            _ => None,
        })
        .expect("pause must select a same-lease hardware stop");
    let mut backend = RecordingBackend::default();
    execute_safe_stop(&mut backend, purpose).expect("prompt pause plan should succeed");
    session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id: lease.id(),
            now_ms: 3,
        })
        .expect("same-lease confirmation should be accepted");

    // Assert
    assert_eq!(purpose, HardwareSafeStopPurpose::ResumablePause);
    assert_eq!(
        backend.recorded,
        resumable_pause_shutdown_plan()
            .map(RecordedStep::SafeShutdown)
            .to_vec()
    );
    assert_eq!(session.snapshot().hardware_state, MiningHardwareState::Stopped);
    assert_eq!(session.snapshot().campaign_state, MiningCampaignState::Armed);
}

#[test]
fn repeated_safe_shutdown_uses_the_same_idempotent_plan() {
    // Arrange
    let mut backend = RecordingBackend::default();
    let expected_once = safe_shutdown_plan()
        .map(RecordedStep::SafeShutdown)
        .to_vec();

    // Act
    execute_safe_shutdown(&mut backend).expect("first safe shutdown should succeed");
    execute_safe_shutdown(&mut backend).expect("repeated safe shutdown should succeed");

    // Assert
    assert_eq!(backend.recorded[..expected_once.len()], expected_once);
    assert_eq!(backend.recorded[expected_once.len()..], expected_once);
}

#[test]
fn safe_shutdown_reports_each_possible_first_failure() {
    // Arrange
    let safe_shutdown = safe_shutdown_plan();

    for failed_step in safe_shutdown {
        let mut backend = RecordingBackend::failing_safe_shutdown(failed_step);
        let expected = safe_shutdown.map(RecordedStep::SafeShutdown).to_vec();

        // Act
        let failure = execute_safe_shutdown(&mut backend)
            .expect_err("injected safe-shutdown failure should fail");

        // Assert
        assert_eq!(failure.step(), failed_step);
        assert_eq!(
            failure.source(),
            &InjectedFailure::SafeShutdown(failed_step)
        );
        assert_eq!(backend.recorded, expected);
    }
}
