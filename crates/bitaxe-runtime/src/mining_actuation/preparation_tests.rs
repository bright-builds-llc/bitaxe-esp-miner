use super::*;

/// Fails the named step; reports two heartbeats from within each step.
struct Backend {
    maybe_failing: Option<PreparationStep>,
}

impl MiningActuationBackend for Backend {
    type Error = &'static str;

    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        if self.maybe_failing == Some(step) {
            Err("step_failed")
        } else {
            Ok(())
        }
    }

    fn execute_preparation_step_with_progress(
        &mut self,
        step: PreparationStep,
        progress: &mut dyn FnMut(),
    ) -> Result<(), Self::Error> {
        progress();
        progress();
        self.execute_preparation_step(step)
    }

    fn execute_safe_shutdown_step(&mut self, _step: SafeShutdownStep) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::ultra_205_bm1366(485, 1_200, 100).expect("production profile")
}

fn run(maybe_failing: Option<PreparationStep>) -> (Vec<PreparationProgress>, bool) {
    let mut reported = Vec::new();
    let result = execute_preparation_with_progress(
        &mut Backend { maybe_failing },
        profile(),
        &mut |progress| reported.push(progress),
    );
    (reported, result.is_ok())
}

#[test]
fn a_preparation_reports_every_step_in_order() {
    // Arrange / Act
    let (reported, prepared) = run(None);
    // Assert
    let steps: Vec<_> = reported
        .iter()
        .filter_map(|progress| match progress {
            PreparationProgress::Step(step) => Some(*step),
            PreparationProgress::Rollback(_) => None,
        })
        .collect();
    let mut expected = Vec::new();
    for step in preparation_plan(profile()) {
        expected.extend([step, step, step]);
    }
    assert!(prepared);
    assert_eq!(steps, expected);
}

#[test]
fn a_heartbeat_within_a_step_reports_that_step() {
    // Arrange / Act
    let (reported, _) = run(None);
    // Assert: the step's own report, then its two heartbeats, before the next step.
    let first = preparation_plan(profile())[0];
    assert_eq!(reported[..3], [PreparationProgress::Step(first); 3]);
}

#[test]
fn a_failed_step_reports_every_rollback_step() {
    // Arrange
    let failing = PreparationStep::EnableAsic;
    // Act
    let (reported, prepared) = run(Some(failing));
    // Assert
    let rollback: Vec<_> = reported
        .iter()
        .filter_map(|progress| match progress {
            PreparationProgress::Rollback(step) => Some(*step),
            PreparationProgress::Step(_) => None,
        })
        .collect();
    assert!(!prepared);
    assert_eq!(rollback, safe_shutdown_plan());
}

#[test]
fn the_original_failure_stays_primary_with_progress() {
    // Arrange
    let failing = PreparationStep::RequireFreshNonzeroFanRpm;
    let mut backend = Backend {
        maybe_failing: Some(failing),
    };
    // Act
    let failure = execute_preparation_with_progress(&mut backend, profile(), &mut |_| {})
        .expect_err("preparation fails");
    // Assert
    assert_eq!(failure.original().step(), failing);
    assert_eq!(*failure.original().source(), "step_failed");
    assert!(failure.maybe_safe_shutdown_failure().is_none());
}
