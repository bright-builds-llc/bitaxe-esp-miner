//! The local mining owner's memory between ticks (ADR-0038). Pure: the firmware
//! shell reports each tick's facts and performs what this decides.
//!
//! With no run, [`LocalOwnerCore::decide_idle`] applies [`decide`] and remembers the
//! hold for status. During a run it accrues hashing time. When the run ends,
//! [`classify_run_ending`] tells the restart guard whether it failed, from the
//! generation's unsafe stop first and then the session's first stop cause: a
//! pause, a settings change or a lost readiness fact is not a failure, so a
//! network outage never spends the guard.

use bitaxe_stratum::v1::production_session::ProductionSessionBlocker;

use super::policy::{decide, LocalDecision, LocalHoldReason, LocalInputs, RestartGuard, RunEnding};

/// What the owner remembers between ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalOwnerCore {
    maybe_run: Option<RunProgress>,
    guard: RestartGuard,
    maybe_hold: Option<LocalHoldReason>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RunProgress {
    maybe_active_since_ms: Option<u64>,
    active_ms: u64,
    requested_stop: bool,
    handed_over: bool,
}

impl RunProgress {
    fn hashed_ms(&self, now_ms: u64) -> u64 {
        let ongoing = self
            .maybe_active_since_ms
            .map_or(0, |since| now_ms.saturating_sub(since));
        self.active_ms.saturating_add(ongoing)
    }
}

impl LocalOwnerCore {
    /// With no run, whether one may begin now; the hold is kept for status.
    pub fn decide_idle(&mut self, inputs: LocalInputs, now_ms: u64) -> LocalDecision {
        let decision = decide(inputs, &self.guard, now_ms);
        self.maybe_hold = match decision {
            LocalDecision::Hold(reason) => Some(reason),
            LocalDecision::Begin(_) => None,
        };
        decision
    }

    /// The shell began the run `decide_idle` allowed.
    pub fn run_began(&mut self) {
        self.maybe_run = Some(RunProgress::default());
    }

    /// Whether a run is in progress.
    #[must_use]
    pub const fn has_run(&self) -> bool {
        self.maybe_run.is_some()
    }

    /// One tick of a run: whether it is hashing (campaign `Active`).
    pub fn note_run_progress(&mut self, hashing: bool, now_ms: u64) {
        let Some(run) = self.maybe_run.as_mut() else {
            return;
        };
        match (hashing, run.maybe_active_since_ms) {
            (true, None) => run.maybe_active_since_ms = Some(now_ms),
            (false, Some(since)) => {
                run.active_ms = run.active_ms.saturating_add(now_ms.saturating_sub(since));
                run.maybe_active_since_ms = None;
            }
            _ => {}
        }
    }

    /// The run is stopping on request: the operator changed settings (the session
    /// ends the run so the next one reads them), or a Gate link takes over.
    pub fn note_requested_stop(&mut self) {
        if let Some(run) = self.maybe_run.as_mut() {
            run.requested_stop = true;
        }
    }

    /// Whether a Gate link that holds local mining off should take over the run:
    /// once per run.
    #[must_use]
    pub fn handover_due(&self, link_holds: bool) -> bool {
        link_holds && self.maybe_run.is_some_and(|run| !run.handed_over)
    }

    /// The shell revoked the run for a Gate link; `revoked` is whether that
    /// revocation took effect. When a device-local stop revoked it first, that
    /// stop's ending stands, so the run is not marked as requested.
    pub fn note_handover(&mut self, revoked: bool) {
        if let Some(run) = self.maybe_run.as_mut() {
            run.handed_over = true;
            run.requested_stop |= revoked;
        }
    }

    /// The run ended: `maybe_unsafe_stop` from the generation's unsafe detail, and
    /// the session's first stop cause for its lease. Returns how it ended, which
    /// the restart guard has recorded, or `None` when no run was in progress.
    pub fn run_ended(
        &mut self,
        maybe_unsafe_stop: Option<UnsafeStop>,
        maybe_stop_cause: Option<ProductionSessionBlocker>,
        now_ms: u64,
    ) -> Option<RunEnding> {
        let run = self.maybe_run.take()?;
        let ending = classify_run_ending(maybe_unsafe_stop, run.requested_stop, maybe_stop_cause);
        self.guard
            .note_run_ended(ending, run.hashed_ms(now_ms), now_ms);
        Some(ending)
    }

    /// The operator resumed: the restart guard starts over.
    pub fn resume(&mut self) {
        self.guard.resume();
    }

    /// Why no run may begin, as of the last idle decision.
    #[must_use]
    pub const fn maybe_hold(&self) -> Option<LocalHoldReason> {
        if self.maybe_run.is_some() {
            return None;
        }
        self.maybe_hold
    }
}

/// A device-local unsafe stop of the run's generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsafeStop {
    /// The chip reading taken with it was an overheat ([`super::is_overheat`]).
    Overheat,
    /// Any other unsafe reading, a stopped fan, or no safe sample.
    Other,
}

/// How a run ended, for the restart guard. An unsafe stop decides first: an
/// overheat is paced by `OverheatRecovery`, any other is a failure, whatever the
/// session blocked on afterwards. Then a requested stop (a settings change or a
/// Gate link), a pause or a lost readiness fact is requested or environmental;
/// anything else failed.
#[must_use]
pub fn classify_run_ending(
    maybe_unsafe_stop: Option<UnsafeStop>,
    requested_stop: bool,
    maybe_stop_cause: Option<ProductionSessionBlocker>,
) -> RunEnding {
    match maybe_unsafe_stop {
        Some(UnsafeStop::Overheat) => return RunEnding::Overheat,
        Some(UnsafeStop::Other) => return RunEnding::Failed,
        None => {}
    }
    if requested_stop {
        return RunEnding::Requested;
    }
    match maybe_stop_cause {
        Some(
            ProductionSessionBlocker::OperatorPaused
            | ProductionSessionBlocker::NetworkUnavailable
            | ProductionSessionBlocker::StratumV1Unsupported
            | ProductionSessionBlocker::SafetyPrerequisitesStale
            | ProductionSessionBlocker::PoolConfigurationUnavailable,
        ) => RunEnding::Requested,
        _ => RunEnding::Failed,
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;
