//! Bounded reset loop over injected effects; every stop decision here is pure.

use std::time::Duration;

use anyhow::Result;

use super::identity::{IdentityScanner, IdentityVerdict};
use super::model::{CycleRow, CycleRun, FirstFailure, ResetEnduranceConfig, ResetEnduranceStop};
use crate::{UsbCommandTermination, UsbProfile, UsbSessionError, UsbTerminalCategory};
use ResetEnduranceStop as Stop;

/// Profile facts for the selected node at the start of a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProfileFacts {
    pub(crate) profile: UsbProfile,
    pub(crate) physical_identity_matches: bool,
}

/// Supervised reset outcome; a failed child keeps its typed session error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResetFacts {
    pub(crate) termination: UsbCommandTermination,
    pub(crate) maybe_error: Option<UsbSessionError>,
}

/// Receive-only observation outcome; the scanner holds the line verdict.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ObserveFacts {
    pub(crate) interrupted: bool,
    pub(crate) maybe_identity_latency_ms: Option<u64>,
    pub(crate) serial: Vec<u8>,
}

/// Imperative boundary for one cycle; implementations own every device effect.
pub(crate) trait CycleEffects {
    fn monotonic_ms(&mut self) -> u64;
    fn inspect_profile(&mut self) -> Result<ProfileFacts, UsbSessionError>;
    fn reset_application(&mut self) -> ResetFacts;
    fn reacquire_transport(&mut self) -> Result<(UsbProfile, bool), UsbSessionError>;
    fn observe(
        &mut self,
        timeout: Duration,
        scanner: &mut IdentityScanner,
    ) -> Result<ObserveFacts, UsbSessionError>;
    fn prove_released(&mut self) -> Result<(), UsbSessionError>;
    fn record_cycle(&mut self, row: &CycleRow, serial: &[u8]) -> Result<()>;
}

struct CycleStop {
    category: ResetEnduranceStop,
    detail: String,
}

impl CycleStop {
    fn new(category: ResetEnduranceStop, detail: impl Into<String>) -> Self {
        Self {
            category,
            detail: detail.into(),
        }
    }

    fn from_error(category: ResetEnduranceStop, error: &UsbSessionError) -> Self {
        Self::new(category, error.to_string())
    }
}

/// Runs up to the configured cycles and stops at the first cycle that fails.
pub(crate) fn run_cycles(
    config: &ResetEnduranceConfig,
    effects: &mut impl CycleEffects,
) -> Result<CycleRun> {
    let mut run = CycleRun::default();
    let mut maybe_previous_ordinal = None;
    for cycle in 1..=config.cycles() {
        let (row, serial, maybe_stop) = run_cycle(cycle, maybe_previous_ordinal, config, effects);
        maybe_previous_ordinal = row.maybe_boot_ordinal;
        effects.record_cycle(&row, &serial)?;
        let bytes_observed = row.bytes_observed;
        run.rows.push(row);
        if let Some(stop) = maybe_stop {
            run.maybe_first_failure = Some(FirstFailure {
                cycle,
                category: stop.category,
                bytes_observed,
            });
            run.maybe_failure_detail = Some(stop.detail);
            break;
        }
    }
    Ok(run)
}

fn run_cycle(
    cycle: u16,
    maybe_previous_ordinal: Option<u64>,
    config: &ResetEnduranceConfig,
    effects: &mut impl CycleEffects,
) -> (CycleRow, Vec<u8>, Option<CycleStop>) {
    let started = effects.monotonic_ms();
    let mut row = CycleRow::new(cycle);
    let mut serial = Vec::new();
    let result = drive_cycle(
        config,
        maybe_previous_ordinal,
        effects,
        &mut row,
        &mut serial,
    );
    row.elapsed_ms = effects.monotonic_ms().saturating_sub(started);
    let maybe_stop = result.err();
    row.maybe_stop = maybe_stop.as_ref().map(|stop| stop.category);
    (row, serial, maybe_stop)
}

fn drive_cycle(
    config: &ResetEnduranceConfig,
    maybe_previous_ordinal: Option<u64>,
    effects: &mut impl CycleEffects,
    row: &mut CycleRow,
    serial: &mut Vec<u8>,
) -> Result<(), CycleStop> {
    let profile = effects.inspect_profile().map_err(|error| {
        CycleStop::from_error(drift_or(&error, Stop::RuntimeProfileUnexpected), &error)
    })?;
    profile_stop(profile)?;

    let reset = effects.reset_application();
    row.reset_termination = reset.termination;
    reset_stop(&reset)?;

    let (transport, reenumerated) = effects
        .reacquire_transport()
        .map_err(|error| CycleStop::from_error(reacquire_stop_category(&error), &error))?;
    row.reenumerated = reenumerated;
    if transport != UsbProfile::SerialJtagRuntime {
        return Err(CycleStop::new(
            Stop::RuntimeProfileUnexpected,
            "reacquired transport is not the Serial/JTAG runtime profile",
        ));
    }

    let mut scanner = IdentityScanner::new(config.expected_identity());
    let observed = effects.observe(config.observe_timeout(), &mut scanner);
    row.bytes_observed = scanner.bytes_observed();
    row.identity_observed = scanner.identity_observed();
    row.maybe_reset_reason = scanner.maybe_boot().map(|boot| boot.reset_reason().label());
    row.maybe_boot_ordinal = scanner.maybe_boot().map(|boot| boot.boot_ordinal());
    let facts = observed
        .map_err(|error| CycleStop::from_error(drift_or(&error, Stop::ObserverFailed), &error))?;
    row.maybe_identity_latency_ms = facts.maybe_identity_latency_ms;
    *serial = facts.serial;
    observation_stop(&scanner, maybe_previous_ordinal, facts.interrupted)?;

    effects
        .prove_released()
        .map_err(|error| CycleStop::from_error(Stop::CleanupUnproven, &error))
}

fn profile_stop(facts: ProfileFacts) -> Result<(), CycleStop> {
    if !facts.physical_identity_matches {
        return Err(CycleStop::new(
            Stop::PhysicalIdentityDrift,
            "selected node does not carry the expected physical identity",
        ));
    }
    if facts.profile != UsbProfile::SerialJtagRuntime {
        return Err(CycleStop::new(
            Stop::RuntimeProfileUnexpected,
            "selected node is not in the Serial/JTAG runtime profile",
        ));
    }
    Ok(())
}

fn reset_stop(reset: &ResetFacts) -> Result<(), CycleStop> {
    if let Some(error) = &reset.maybe_error {
        let category = match error.category {
            UsbTerminalCategory::CleanupFailed | UsbTerminalCategory::ForeignHolder => {
                Stop::CleanupUnproven
            }
            _ if reset.termination == UsbCommandTermination::Interrupted => Stop::Interrupted,
            _ => Stop::ResetFailed,
        };
        return Err(CycleStop::from_error(category, error));
    }
    match reset.termination {
        UsbCommandTermination::ExitedSuccess => Ok(()),
        UsbCommandTermination::Interrupted => {
            Err(CycleStop::new(Stop::Interrupted, "reset interrupted"))
        }
        _ => Err(CycleStop::new(
            Stop::ResetFailed,
            "reset did not exit successfully",
        )),
    }
}

fn reacquire_stop_category(error: &UsbSessionError) -> Stop {
    if error.category == UsbTerminalCategory::RuntimeProfileUnknown {
        return Stop::RuntimeProfileUnexpected;
    }
    drift_or(error, Stop::TransportNotReacquired)
}

/// A cycle passes only on the exact identity plus a boot ordinal newer than the last cycle's.
fn observation_stop(
    scanner: &IdentityScanner,
    maybe_previous_ordinal: Option<u64>,
    interrupted: bool,
) -> Result<(), CycleStop> {
    match scanner.verdict() {
        IdentityVerdict::Complete => ordinal_stop(scanner, maybe_previous_ordinal),
        IdentityVerdict::Mismatch => Err(CycleStop::new(
            Stop::IdentityMismatch,
            "a different application identity was observed",
        )),
        IdentityVerdict::OrdinalAmbiguous => Err(CycleStop::new(
            Stop::BootOrdinalAmbiguous,
            "more than one boot ordinal was observed in one window",
        )),
        IdentityVerdict::Pending if interrupted => {
            Err(CycleStop::new(Stop::Interrupted, "observation interrupted"))
        }
        IdentityVerdict::Pending if scanner.identity_observed() => Err(CycleStop::new(
            Stop::RebootNotProven,
            "identity was observed without a boot discriminator",
        )),
        IdentityVerdict::Pending => Err(CycleStop::new(
            Stop::ApplicationNotObserved,
            "expected identity was not observed within the bound",
        )),
    }
}

fn ordinal_stop(
    scanner: &IdentityScanner,
    maybe_previous_ordinal: Option<u64>,
) -> Result<(), CycleStop> {
    let Some(boot) = scanner.maybe_boot() else {
        return Err(CycleStop::new(
            Stop::RebootNotProven,
            "complete observation lacked a boot discriminator",
        ));
    };
    match maybe_previous_ordinal {
        Some(previous) if boot.boot_ordinal() <= previous => Err(CycleStop::new(
            Stop::RebootNotProven,
            "boot ordinal did not advance past the previous cycle",
        )),
        _ => Ok(()),
    }
}

fn drift_or(error: &UsbSessionError, fallback: Stop) -> Stop {
    match error.category {
        UsbTerminalCategory::PhysicalIdentityDrift | UsbTerminalCategory::IdentityDrift => {
            Stop::PhysicalIdentityDrift
        }
        _ => fallback,
    }
}
