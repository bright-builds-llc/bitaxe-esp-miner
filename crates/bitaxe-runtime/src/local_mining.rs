//! Overheat recovery for local mining (ADR-0038 decision 2): the pure core the local
//! mining owner runs. It follows upstream's power management task
//! (`power_management_task.c`). An overheat stops mining and saves the fan as manual
//! at 100% with `overheat_mode`. After cooling, mining restarts 100 mV and 100 MHz
//! lower, with `overheat_mode` cleared once the restart succeeds.
//!
//! Two deliberate differences from upstream:
//! - The lowered pair must lie in the board catalog (ADR-0038 sub-decision 1).
//!   Otherwise mining stays stopped until the operator resumes.
//! - Cooling needs 30 s of continuous fresh chip readings at 45 C or less.
//!   Upstream's loop asks for the same, but on the Ultra 205 its chip reading is
//!   invalid while the ASIC is uninitialized, so in practice it waits a fixed 30 s.
//!   This firmware reads the EMC2101 die temperature while the ASIC is off, as the
//!   ordered safe-stop already relies on.

use bitaxe_safety::thermal::{ASIC_THROTTLE_TEMP_C, SAFE_RESTART_TEMP_C};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

use crate::revocation::RevocationReason;

#[path = "local_mining/hardware_fault.rs"]
pub mod hardware_fault;
#[path = "local_mining/link_presence.rs"]
pub mod link_presence;
#[path = "local_mining/owner.rs"]
pub mod owner;
#[path = "local_mining/persistence.rs"]
pub mod persistence;
#[path = "local_mining/policy.rs"]
pub mod policy;
#[path = "local_mining/shell.rs"]
pub mod shell;

/// The chip reading that ends a local run as an overheat: upstream's 75 C, the same
/// boundary as the shared mining-safety verdict. The test-only feature
/// `overheat-trip-test` lowers it so the recovery can be checked on hardware at normal
/// mining temperatures; that build is never packaged for release (ADR-0040).
#[cfg(not(feature = "overheat-trip-test"))]
pub const LOCAL_OVERHEAT_TRIP_C: f64 = ASIC_THROTTLE_TEMP_C;
/// See the production definition; this is the test-only lowered trip.
#[cfg(feature = "overheat-trip-test")]
pub const LOCAL_OVERHEAT_TRIP_C: f64 = 55.0;
// A trip at or below the restart temperature would never let cooling finish, and one
// above the shared verdict's limit would never fire.
const _: () = assert!(
    LOCAL_OVERHEAT_TRIP_C > SAFE_RESTART_TEMP_C && LOCAL_OVERHEAT_TRIP_C <= ASIC_THROTTLE_TEMP_C
);

/// How long readings must stay at or below the restart temperature (upstream's six
/// 5 s cooling cycles).
pub const OVERHEAT_COOLING_MS: u64 = 30_000;
/// Upstream's `ASIC_REDUCTION`: the step both frequency (MHz) and voltage (mV) drop.
pub const OVERHEAT_REDUCTION: u16 = 100;
/// A reading older than this is not fresh (the sensor sweep marks it stale at 1 s).
pub const CHIP_READING_FRESH_MS: u64 = 1_000;
/// Cool readings further apart than this are not continuous; the 30 s restarts.
pub const COOLING_MAX_GAP_MS: u64 = 2_000;

/// One chip temperature reading and when the sensor took it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipReading {
    pub celsius: f64,
    pub acquired_at_ms: u64,
}

/// Whether a revocation was an overheat, judged from the chip reading taken with it
/// rather than from the first failing fact, which may name another fact first.
#[must_use]
pub fn is_overheat(reason: RevocationReason, maybe_chip_temp_c: Option<f64>) -> bool {
    reason == RevocationReason::UnsafeObservation
        && maybe_chip_temp_c.is_some_and(|celsius| celsius >= LOCAL_OVERHEAT_TRIP_C)
}

/// The settings upstream saves on entering overheat mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverheatSettings {
    pub auto_fan_speed: bool,
    pub manual_fan_speed_percent: u8,
    pub overheat_mode: bool,
}

/// Saved when an overheat stops mining. Recovery clears only `overheat_mode`; the
/// fan stays manual at 100%, as upstream leaves it.
pub const OVERHEAT_ENTRY_SETTINGS: OverheatSettings = OverheatSettings {
    auto_fan_speed: false,
    manual_fan_speed_percent: 100,
    overheat_mode: true,
};

/// The lowered pair after cooling, or `None` when it leaves the board catalog.
#[must_use]
pub fn reduced_profile(profile: MiningHardwareProfile) -> Option<MiningHardwareProfile> {
    let frequency = profile.frequency().mhz().checked_sub(OVERHEAT_REDUCTION)?;
    let voltage = profile
        .core_voltage()
        .millivolts()
        .checked_sub(OVERHEAT_REDUCTION)?;
    MiningHardwareProfile::from_device_settings(Some(f32::from(frequency)), Some(voltage)).ok()
}

/// One tick's outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverheatStep {
    /// No overheat is in progress.
    Clear,
    /// Not cool for long enough yet.
    Cooling,
    /// Save this pair and power up again; clear `overheat_mode` once that succeeds.
    Recover(MiningHardwareProfile),
    /// The lowered pair leaves the catalog: stay stopped until the operator resumes.
    StayStopped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Phase {
    #[default]
    Clear,
    Cooling {
        profile: MiningHardwareProfile,
        /// Whether recovery lowers the pair; not after a reboot, which may follow a
        /// recovery that already lowered it.
        lower: bool,
        entered_at_ms: u64,
        maybe_cool_since_ms: Option<u64>,
        maybe_last_cool_ms: Option<u64>,
    },
    Stopped,
}

/// The overheat state machine: clear, cooling after an overheat, or stopped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverheatRecovery {
    phase: Phase,
}

impl OverheatRecovery {
    /// An overheat stopped mining at `profile`. The shell saves
    /// [`OVERHEAT_ENTRY_SETTINGS`] and runs the ordered safe-stop.
    pub fn enter(&mut self, profile: MiningHardwareProfile, now_ms: u64) {
        self.phase = Phase::Cooling {
            profile,
            lower: true,
            entered_at_ms: now_ms,
            maybe_cool_since_ms: None,
            maybe_last_cool_ms: None,
        };
    }

    /// The device booted with `overheat_mode` saved. A saved stop marker means the
    /// last recovery stayed stopped, which only an operator resume ends, so the
    /// stop holds. Otherwise the reboot interrupted a recovery (owner decision
    /// 2026-10-17): cool for 30 s, then restart at the saved pair without lowering
    /// it again, since it may already be lowered. A saved pair outside the catalog
    /// (`None`) leaves nothing to cool for; the owner then holds on the profile.
    pub fn enter_after_boot(
        &mut self,
        maybe_saved: Option<MiningHardwareProfile>,
        stopped_marker: bool,
        now_ms: u64,
    ) {
        if stopped_marker {
            self.phase = Phase::Stopped;
            return;
        }
        let Some(saved) = maybe_saved else {
            self.phase = Phase::Clear;
            return;
        };
        self.phase = Phase::Cooling {
            profile: saved,
            lower: false,
            entered_at_ms: now_ms,
            maybe_cool_since_ms: None,
            maybe_last_cool_ms: None,
        };
    }

    /// One cooling tick. Only a fresh reading taken after the overheat counts. A
    /// hot, stale or missing reading, or a gap between cool readings, restarts the
    /// 30 s.
    pub fn observe(&mut self, maybe_reading: Option<ChipReading>, now_ms: u64) -> OverheatStep {
        let Phase::Cooling {
            profile,
            lower,
            entered_at_ms,
            maybe_cool_since_ms,
            maybe_last_cool_ms,
        } = self.phase
        else {
            return match self.phase {
                Phase::Stopped => OverheatStep::StayStopped,
                _ => OverheatStep::Clear,
            };
        };
        let maybe_cool_at_ms = maybe_reading
            .filter(|reading| {
                reading.acquired_at_ms >= entered_at_ms
                    && now_ms.saturating_sub(reading.acquired_at_ms) <= CHIP_READING_FRESH_MS
                    && reading.celsius <= SAFE_RESTART_TEMP_C
            })
            .map(|reading| reading.acquired_at_ms);
        let Some(cool_at_ms) = maybe_cool_at_ms else {
            self.phase = Phase::Cooling {
                profile,
                lower,
                entered_at_ms,
                maybe_cool_since_ms: None,
                maybe_last_cool_ms: None,
            };
            return OverheatStep::Cooling;
        };
        let continuous = maybe_last_cool_ms
            .is_some_and(|last| cool_at_ms.saturating_sub(last) <= COOLING_MAX_GAP_MS);
        let cool_since_ms = match maybe_cool_since_ms {
            Some(since) if continuous => since,
            _ => cool_at_ms,
        };
        self.phase = Phase::Cooling {
            profile,
            lower,
            entered_at_ms,
            maybe_cool_since_ms: Some(cool_since_ms),
            maybe_last_cool_ms: Some(cool_at_ms),
        };
        if cool_at_ms.saturating_sub(cool_since_ms) < OVERHEAT_COOLING_MS {
            return OverheatStep::Cooling;
        }
        let maybe_restart = if lower {
            reduced_profile(profile)
        } else {
            Some(profile)
        };
        match maybe_restart {
            Some(reduced) => {
                self.phase = Phase::Clear;
                OverheatStep::Recover(reduced)
            }
            None => {
                self.phase = Phase::Stopped;
                OverheatStep::StayStopped
            }
        }
    }

    /// The operator resumed: a stop after an overheat ends here. A resume never
    /// cancels an overheat that is still cooling.
    pub fn resume(&mut self) {
        if self.phase == Phase::Stopped {
            self.phase = Phase::Clear;
        }
    }

    /// Whether an overheat still holds the ASIC down.
    #[must_use]
    pub fn holds_power_down(&self) -> bool {
        self.phase != Phase::Clear
    }
}

#[cfg(test)]
#[path = "local_mining/tests.rs"]
mod tests;
