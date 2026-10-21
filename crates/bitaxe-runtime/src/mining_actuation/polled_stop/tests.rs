use super::*;
use std::cell::Cell;
use std::rc::Rc;

/// Records each step, fails the named one, and proves cooling on the given poll.
#[derive(Default)]
struct Backend {
    steps: Vec<SafeShutdownStep>,
    maybe_failing: Option<SafeShutdownStep>,
    start_fails: bool,
    maybe_proven_on_poll: Option<usize>,
    polls: usize,
    proven: bool,
    clock_ms: Rc<Cell<u64>>,
}

/// The frequency ramp's duration, which the proof's bound must not include.
const RAMP_MS: u64 = 7_000;
const BEGIN_MS: u64 = 1_000;
/// When the proof starts: after the power-down, as in the blocking wait.
const PROOF_START_MS: u64 = BEGIN_MS + RAMP_MS;

impl MiningActuationBackend for Backend {
    type Error = &'static str;

    fn execute_preparation_step(
        &mut self,
        _step: super::super::PreparationStep,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.steps.push(step);
        if step == SafeShutdownStep::ReduceFrequencyAndResetNonce {
            self.clock_ms.set(self.clock_ms.get() + RAMP_MS);
        }
        if step == SafeShutdownStep::SetFanDutyTo30Percent && !self.proven {
            return Err("cooling_proof_required");
        }
        if self.maybe_failing == Some(step) {
            return Err("step_failed");
        }
        if step == SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C {
            self.proven = true;
        }
        Ok(())
    }
}

impl CoolingProofBackend for Backend {
    fn start_cooling_proof(&mut self) -> Result<(), Self::Error> {
        self.steps
            .push(SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C);
        if self.start_fails {
            Err("cooling_proof_required")
        } else {
            Ok(())
        }
    }

    fn cooling_proof_observed(&mut self) -> bool {
        self.polls += 1;
        self.proven = self
            .maybe_proven_on_poll
            .is_some_and(|poll| self.polls >= poll);
        self.proven
    }

    fn cooling_proof_timeout(&self) -> Self::Error {
        "cooling_proof_timed_out"
    }
}

fn begin(
    backend: &mut Backend,
    purpose: HardwareSafeStopPurpose,
) -> SafeStopProgress<&'static str> {
    backend.clock_ms.set(BEGIN_MS);
    let clock = Rc::clone(&backend.clock_ms);
    begin_safe_stop_with_progress(backend, purpose, &|| clock.get(), &mut |_| {})
}

fn pending(progress: SafeStopProgress<&'static str>) -> PendingCoolingProof<&'static str> {
    match progress {
        SafeStopProgress::CoolingPending(pending) => pending,
        SafeStopProgress::Complete(result) => panic!("expected a pending proof, got {result:?}"),
    }
}

fn complete(
    progress: SafeStopProgress<&'static str>,
) -> Result<(), SafeShutdownFailure<&'static str>> {
    match progress {
        SafeStopProgress::Complete(result) => result,
        SafeStopProgress::CoolingPending(_) => panic!("expected a complete stop"),
    }
}

/// Polls every 500 ms from the proof's start until the stop completes.
fn run_to_completion(
    backend: &mut Backend,
    mut pending: PendingCoolingProof<&'static str>,
) -> (u64, Result<(), SafeShutdownFailure<&'static str>>) {
    let mut now_ms = backend.clock_ms.get();
    loop {
        now_ms += 500;
        match poll_cooling_proof(backend, pending, now_ms, &mut |_| {}) {
            SafeStopProgress::Complete(result) => return (now_ms, result),
            SafeStopProgress::CoolingPending(next) => pending = next,
        }
    }
}

#[test]
fn a_terminal_begin_powers_down_and_leaves_the_proof_open() {
    // Arrange
    let mut backend = Backend::default();
    // Act
    let progress = begin(&mut backend, HardwareSafeStopPurpose::Terminal);
    // Assert
    assert_eq!(
        pending(progress).deadline_ms(),
        PROOF_START_MS + COOLING_PROOF_TIMEOUT_MS
    );
    assert_eq!(backend.steps, safe_shutdown_plan()[..7]);
}

#[test]
fn a_poll_before_the_proof_keeps_it_open() {
    // Arrange
    let mut backend = Backend {
        maybe_proven_on_poll: Some(3),
        ..Backend::default()
    };
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Act
    let progress = poll_cooling_proof(&mut backend, open, PROOF_START_MS + 500, &mut |_| {});
    // Assert
    assert!(matches!(progress, SafeStopProgress::CoolingPending(_)));
    assert_eq!(backend.steps.len(), 7);
}

#[test]
fn a_proof_completes_the_stop_with_the_whole_plan() {
    // Arrange
    let mut backend = Backend {
        maybe_proven_on_poll: Some(3),
        ..Backend::default()
    };
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Act
    let (_, result) = run_to_completion(&mut backend, open);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(backend.steps, safe_shutdown_plan());
}

#[test]
fn the_bound_fails_the_proof_and_still_attempts_the_paused_duty() {
    // Arrange
    let mut backend = Backend::default();
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Act
    let (completed_at_ms, result) = run_to_completion(&mut backend, open);
    // Assert
    let failure = result.expect_err("timed out");
    assert_eq!(
        failure.step(),
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C
    );
    assert_eq!(*failure.source(), "cooling_proof_timed_out");
    assert_eq!(completed_at_ms, PROOF_START_MS + COOLING_PROOF_TIMEOUT_MS);
    assert_eq!(
        backend.steps.last(),
        Some(&SafeShutdownStep::SetFanDutyTo30Percent)
    );
}

#[test]
fn a_proof_on_the_last_poll_wins_over_the_bound() {
    // Arrange: the proof arrives on the poll at the deadline, as the blocking wait checks it first.
    let polls_to_deadline = usize::try_from(COOLING_PROOF_TIMEOUT_MS / 500).expect("small");
    let mut backend = Backend {
        maybe_proven_on_poll: Some(polls_to_deadline),
        ..Backend::default()
    };
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Act
    let (completed_at_ms, result) = run_to_completion(&mut backend, open);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(completed_at_ms, PROOF_START_MS + COOLING_PROOF_TIMEOUT_MS);
}

#[test]
fn an_earlier_failure_stays_earliest_after_a_proof() {
    // Arrange
    let mut backend = Backend {
        maybe_failing: Some(SafeShutdownStep::HoldResetLow),
        maybe_proven_on_poll: Some(1),
        ..Backend::default()
    };
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Act
    let (_, result) = run_to_completion(&mut backend, open);
    // Assert
    assert_eq!(
        result.expect_err("failed").step(),
        SafeShutdownStep::HoldResetLow
    );
    assert_eq!(backend.steps, safe_shutdown_plan());
}

#[test]
fn a_proof_that_cannot_start_completes_at_once() {
    // Arrange
    let mut backend = Backend {
        start_fails: true,
        ..Backend::default()
    };
    // Act
    let result = complete(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    // Assert
    assert_eq!(
        result.expect_err("failed").step(),
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C
    );
    assert_eq!(backend.steps, safe_shutdown_plan());
}

#[test]
fn a_resumable_pause_completes_at_once_without_a_proof() {
    // Arrange
    let mut backend = Backend::default();
    // Act
    let result = complete(begin(&mut backend, HardwareSafeStopPurpose::ResumablePause));
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(backend.steps, super::super::resumable_pause_shutdown_plan());
}

#[test]
fn the_paused_duty_reports_its_own_progress() {
    // Arrange
    let mut backend = Backend {
        maybe_proven_on_poll: Some(1),
        ..Backend::default()
    };
    let open = pending(begin(&mut backend, HardwareSafeStopPurpose::Terminal));
    let mut reported = Vec::new();
    // Act
    poll_cooling_proof(&mut backend, open, PROOF_START_MS + 500, &mut |step| {
        reported.push(step);
    });
    // Assert
    assert_eq!(reported, [SafeShutdownStep::SetFanDutyTo30Percent]);
}
