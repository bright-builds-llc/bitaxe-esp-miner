//! The terminal safe-stop with its cooling proof polled by the caller.
//!
//! The production owner must keep serving its loop while the board cools, so the
//! terminal plan's power-down runs at once and the proof is checked on each later
//! tick (ADR-0016). The steps, their order and the earliest-failure rule are those of
//! [`execute_safe_stop_with_progress`]; a resumable pause has no cooling wait and
//! still completes at once.

use super::{
    execute_safe_stop_with_progress, safe_shutdown_plan, MiningActuationBackend,
    SafeShutdownFailure, SafeShutdownStep,
};
use bitaxe_stratum::v1::production_session::HardwareSafeStopPurpose;

/// The cooling proof's bound, measured from the start of its step, as the blocking wait's.
pub const COOLING_PROOF_TIMEOUT_MS: u64 = 120_000;

/// The terminal plan's steps before the cooling proof: everything that removes power.
const POWER_DOWN_STEPS: usize = 6;

/// A backend whose cooling proof can be checked without blocking.
pub trait CoolingProofBackend: MiningActuationBackend {
    /// Opens the proof window (step 7); only readings taken after it may prove cooling.
    fn start_cooling_proof(&mut self) -> Result<(), Self::Error>;

    /// One non-blocking check: has a fresh reading at or below 45 C arrived in the window?
    fn cooling_proof_observed(&mut self) -> bool;

    /// The failure recorded when the proof's bound passes unproven.
    fn cooling_proof_timeout(&self) -> Self::Error;
}

/// A terminal stop whose power-down ran and whose cooling proof is still open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCoolingProof<E> {
    maybe_earliest_failure: Option<SafeShutdownFailure<E>>,
    deadline_ms: u64,
}

impl<E> PendingCoolingProof<E> {
    /// When the proof times out, on the caller's monotonic clock.
    pub const fn deadline_ms(&self) -> u64 {
        self.deadline_ms
    }
}

/// The state of a safe-stop after a begin or a poll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeStopProgress<E> {
    /// Every step ran; the earliest failure, if any.
    Complete(Result<(), SafeShutdownFailure<E>>),
    /// Power is off; the cooling proof and the paused fan duty remain.
    CoolingPending(PendingCoolingProof<E>),
}

/// Starts the stop for `purpose`. A terminal stop returns once power is off; its proof's
/// bound runs from the proof's start on `now_ms`, after the power-down, as the blocking wait's.
pub fn begin_safe_stop_with_progress<B>(
    backend: &mut B,
    purpose: HardwareSafeStopPurpose,
    now_ms: &dyn Fn() -> u64,
    progress: &mut dyn FnMut(SafeShutdownStep),
) -> SafeStopProgress<B::Error>
where
    B: CoolingProofBackend,
{
    if purpose == HardwareSafeStopPurpose::ResumablePause {
        return SafeStopProgress::Complete(execute_safe_stop_with_progress(
            backend, purpose, progress,
        ));
    }
    let mut maybe_earliest_failure = None;
    for step in safe_shutdown_plan().into_iter().take(POWER_DOWN_STEPS) {
        progress(step);
        if let Err(source) = backend.execute_safe_shutdown_step_with_progress(step, progress) {
            maybe_earliest_failure.get_or_insert(SafeShutdownFailure { step, source });
        }
    }
    progress(SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C);
    let started_at_ms = now_ms();
    if let Err(source) = backend.start_cooling_proof() {
        maybe_earliest_failure.get_or_insert(SafeShutdownFailure {
            step: SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,
            source,
        });
        return SafeStopProgress::Complete(finish(backend, maybe_earliest_failure, progress));
    }
    SafeStopProgress::CoolingPending(PendingCoolingProof {
        maybe_earliest_failure,
        deadline_ms: started_at_ms.saturating_add(COOLING_PROOF_TIMEOUT_MS),
    })
}

/// Checks the open proof once at `now_ms`, as one iteration of the blocking wait: a proof
/// first, then the bound. Either ends the stop with the paused fan duty.
pub fn poll_cooling_proof<B>(
    backend: &mut B,
    pending: PendingCoolingProof<B::Error>,
    now_ms: u64,
    progress: &mut dyn FnMut(SafeShutdownStep),
) -> SafeStopProgress<B::Error>
where
    B: CoolingProofBackend,
{
    let PendingCoolingProof {
        mut maybe_earliest_failure,
        deadline_ms,
    } = pending;
    if !backend.cooling_proof_observed() {
        if now_ms < deadline_ms {
            return SafeStopProgress::CoolingPending(PendingCoolingProof {
                maybe_earliest_failure,
                deadline_ms,
            });
        }
        maybe_earliest_failure.get_or_insert(SafeShutdownFailure {
            step: SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,
            source: backend.cooling_proof_timeout(),
        });
    }
    SafeStopProgress::Complete(finish(backend, maybe_earliest_failure, progress))
}

/// Runs the paused fan duty, which itself refuses without a proof, and keeps the earliest failure.
fn finish<B>(
    backend: &mut B,
    mut maybe_earliest_failure: Option<SafeShutdownFailure<B::Error>>,
    progress: &mut dyn FnMut(SafeShutdownStep),
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: CoolingProofBackend,
{
    let step = SafeShutdownStep::SetFanDutyTo30Percent;
    progress(step);
    if let Err(source) = backend.execute_safe_shutdown_step(step) {
        maybe_earliest_failure.get_or_insert(SafeShutdownFailure { step, source });
    }
    maybe_earliest_failure.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
