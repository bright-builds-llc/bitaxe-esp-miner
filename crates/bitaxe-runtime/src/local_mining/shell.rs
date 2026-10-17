//! The local owner shell's per-tick decisions (ADR-0038), kept pure so they are
//! tested: whether the session adopted or ended the run's lease, the operator's
//! resume edge, and which hold the API names.

use bitaxe_stratum::v1::production_session::MiningCampaignLeaseId;
use bitaxe_stratum::v1::state::MiningOperatorIntent;

use super::policy::LocalHoldReason;

/// What one tick sees of the run's lease in the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunTick {
    /// The session holds the lease.
    Running,
    /// The session never adopted the lease: readiness changed between the owner's
    /// decision and the session's wake. Nothing was powered.
    NotAdopted,
    /// The session consumed the lease and its hardware is stopped.
    Ended,
}

/// Classifies one tick. A session clears its lease only as it marks the lease
/// consumed with the hardware stopped, so a powered run is never `NotAdopted`.
#[must_use]
pub fn run_tick(
    run: MiningCampaignLeaseId,
    maybe_held: Option<MiningCampaignLeaseId>,
    maybe_consumed: Option<MiningCampaignLeaseId>,
    hardware_stopped: bool,
) -> RunTick {
    if maybe_consumed == Some(run) && hardware_stopped {
        return RunTick::Ended;
    }
    if maybe_held == Some(run) || maybe_consumed == Some(run) {
        return RunTick::Running;
    }
    RunTick::NotAdopted
}

/// Watches the requested operator intent for the operator's resume.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResumeEdge {
    maybe_last: Option<MiningOperatorIntent>,
}

impl ResumeEdge {
    /// Returns whether `intent` is a resume: Run after Paused.
    pub fn observe(&mut self, intent: MiningOperatorIntent) -> bool {
        let resumed = self.maybe_last == Some(MiningOperatorIntent::Paused)
            && intent == MiningOperatorIntent::Run;
        self.maybe_last = Some(intent);
        resumed
    }

    /// The device paused itself: a resume before the next tick is still seen.
    pub fn note_device_pause(&mut self) {
        self.maybe_last = Some(MiningOperatorIntent::Paused);
    }
}

/// The hold the API names as `blockedReason`. The device's own pause names the
/// overheat or guard behind it; an operator pause, another owner or a Gate link
/// show their own state.
#[must_use]
pub const fn api_hold(
    maybe_hold: Option<LocalHoldReason>,
    paused_for_overheat: bool,
    paused_for_guard: bool,
) -> Option<LocalHoldReason> {
    match maybe_hold {
        Some(LocalHoldReason::Paused) if paused_for_overheat => Some(LocalHoldReason::Overheat),
        Some(LocalHoldReason::Paused) if paused_for_guard => {
            Some(LocalHoldReason::RestartGuardHeld)
        }
        Some(
            LocalHoldReason::Paused | LocalHoldReason::OtherBootOwner | LocalHoldReason::GateLink,
        )
        | None => None,
        Some(reason) => Some(reason),
    }
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod tests;
