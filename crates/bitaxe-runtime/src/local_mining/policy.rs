//! When the local mining owner may begin a run (ADR-0038). Pure: the shell gathers
//! the inputs each owner tick and acts on the decision.
//!
//! A run begins only when no other owner holds the device, mining is not paused,
//! no fault or overheat holds the ASIC down, the restart guard allows it, readiness
//! holds, and the saved profile passes the catalog check. The restart guard is the
//! owner's decision (2026-10-17), diverging from upstream, which retries forever:
//! runs that fail before hashing, or that end on a non-overheat unsafe stop, back
//! off exponentially, and after several in a row mining holds until resumed. A
//! stable run resets it. The shell forces the intent to Paused while the guard
//! holds, as for an overheat stop, so the operator has something to resume.
//!
//! A Gate link outranks a latched hardware fault in the reported reason: the fault
//! latch covers local mining only, and a link already keeps local mining off.

use bitaxe_stratum::v1::production_session::{DeviceProfileRefusal, MiningHardwareProfile};

/// The first backoff after a failed run; each further failure doubles it, so the
/// longest is 240 s before the guard holds.
pub const RESTART_BACKOFF_BASE_MS: u64 = 30_000;
/// Failed runs in a row after which mining holds until the operator resumes.
pub const RESTART_FAILURES_BEFORE_HOLD: u32 = 5;
/// A run that hashed this long is stable and resets the guard.
pub const STABLE_RUN_MS: u64 = 10 * 60_000;

/// What the shell observes this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalInputs {
    /// A campaign, diagnostic or stimulus owns this boot.
    pub other_boot_owner: bool,
    /// A Gate link exists; handing over between it and local mining is task 4.
    pub gate_link: bool,
    pub operator_paused: bool,
    /// A latched hardware fault, held until reboot.
    pub hardware_fault: bool,
    /// An overheat holds the ASIC down (`OverheatRecovery::holds_power_down`).
    pub overheat_holds: bool,
    pub network_ready: bool,
    pub stratum_v1_supported: bool,
    pub safety_fresh: bool,
    /// The safety actuation owner and the ASIC handle are available, so a run can
    /// prepare; without them a run would fail and spend the restart guard.
    pub actuation_available: bool,
    /// The saved settings' profile, checked against the board catalog.
    pub profile: Result<MiningHardwareProfile, DeviceProfileRefusal>,
}

/// Why no run may begin now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalHoldReason {
    OtherBootOwner,
    GateLink,
    HardwareFault,
    Overheat,
    Paused,
    RestartGuardHeld,
    RestartBackoff,
    NetworkUnavailable,
    StratumV1Unsupported,
    SafetyStale,
    ActuationUnavailable,
    ProfileRefused(DeviceProfileRefusal),
}

impl LocalHoldReason {
    /// A closed label for status reports.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OtherBootOwner => "local_other_boot_owner",
            Self::GateLink => "local_gate_link",
            Self::HardwareFault => "local_hardware_fault",
            Self::Overheat => "local_overheat",
            Self::Paused => "local_paused",
            Self::RestartGuardHeld => "local_restart_guard_held",
            Self::RestartBackoff => "local_restart_backoff",
            Self::NetworkUnavailable => "local_network_unavailable",
            Self::StratumV1Unsupported => "local_stratum_v1_unsupported",
            Self::SafetyStale => "local_safety_stale",
            Self::ActuationUnavailable => "local_actuation_unavailable",
            Self::ProfileRefused(_) => "local_profile_refused",
        }
    }
}

/// The owner's decision for this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalDecision {
    Begin(MiningHardwareProfile),
    Hold(LocalHoldReason),
}

/// How a run ended, for the restart guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEnding {
    /// Stopped on purpose or by the environment: a pause, a Gate link, a settings
    /// change, or a lost readiness fact such as the network.
    Requested,
    /// An overheat; `OverheatRecovery` paces the restart instead.
    Overheat,
    /// Any other ending: an unsafe stop, a failed preparation, or a fault.
    Failed,
}

/// The restart guard's memory of recent runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RestartGuard {
    consecutive_failures: u32,
    maybe_backoff_until_ms: Option<u64>,
}

impl RestartGuard {
    /// Records a run that ended at `now_ms` after hashing for `hashed_ms`.
    pub fn note_run_ended(&mut self, ending: RunEnding, hashed_ms: u64, now_ms: u64) {
        if hashed_ms >= STABLE_RUN_MS {
            *self = Self::default();
        }
        if ending != RunEnding::Failed {
            return;
        }
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let doublings = self.consecutive_failures.saturating_sub(1).min(16);
        let backoff_ms = RESTART_BACKOFF_BASE_MS.saturating_mul(1 << doublings);
        self.maybe_backoff_until_ms = Some(now_ms.saturating_add(backoff_ms));
    }

    /// The operator resumed: the guard starts over.
    pub fn resume(&mut self) {
        *self = Self::default();
    }

    fn maybe_hold(&self, now_ms: u64) -> Option<LocalHoldReason> {
        if self.consecutive_failures >= RESTART_FAILURES_BEFORE_HOLD {
            return Some(LocalHoldReason::RestartGuardHeld);
        }
        self.maybe_backoff_until_ms
            .is_some_and(|until| now_ms < until)
            .then_some(LocalHoldReason::RestartBackoff)
    }
}

/// Whether a run may begin now, and with which profile.
#[must_use]
pub fn decide(inputs: LocalInputs, guard: &RestartGuard, now_ms: u64) -> LocalDecision {
    let ordered_holds = [
        (inputs.other_boot_owner, LocalHoldReason::OtherBootOwner),
        (inputs.gate_link, LocalHoldReason::GateLink),
        (inputs.hardware_fault, LocalHoldReason::HardwareFault),
        (inputs.overheat_holds, LocalHoldReason::Overheat),
        (inputs.operator_paused, LocalHoldReason::Paused),
    ];
    if let Some((_, reason)) = ordered_holds.into_iter().find(|(holds, _)| *holds) {
        return LocalDecision::Hold(reason);
    }
    if let Some(reason) = guard.maybe_hold(now_ms) {
        return LocalDecision::Hold(reason);
    }
    let readiness_holds = [
        (!inputs.network_ready, LocalHoldReason::NetworkUnavailable),
        (
            !inputs.stratum_v1_supported,
            LocalHoldReason::StratumV1Unsupported,
        ),
        (!inputs.safety_fresh, LocalHoldReason::SafetyStale),
        (
            !inputs.actuation_available,
            LocalHoldReason::ActuationUnavailable,
        ),
    ];
    if let Some((_, reason)) = readiness_holds.into_iter().find(|(holds, _)| *holds) {
        return LocalDecision::Hold(reason);
    }
    match inputs.profile {
        Ok(profile) => LocalDecision::Begin(profile),
        Err(refusal) => LocalDecision::Hold(LocalHoldReason::ProfileRefused(refusal)),
    }
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
