//! Overheat recovery for local mining (ADR-0038 decision 2), as upstream's power
//! management task does it: an overheat stops mining and saves the fan as manual at
//! 100% with `overheat_mode`; after 30 s of cool readings mining restarts 100 MHz
//! and 100 mV lower, and `overheat_mode` clears once that run has prepared. The
//! pure state machine is `OverheatRecovery`; this shell saves and reads NVS.
//!
//! When the pair cannot be lowered within the catalog, the stop marker is saved
//! with the entry and mining stays stopped: the device pauses itself, and only the
//! operator's resume ends it. A lowered pair that could not be saved yet keeps the
//! ASIC down, since the next run would otherwise read the pair that overheated.
use super::overheat_store;
use super::*;
use bitaxe_config::{ConfigValidationError, NvsWrite};
use bitaxe_runtime::local_mining::persistence::{
    overheat_cleared_writes, overheat_entry_writes, recovery_pair_writes, stop_marker_write,
};
use bitaxe_runtime::local_mining::{reduced_profile, OverheatRecovery, OverheatStep};

#[derive(Default)]
pub(super) struct OverheatState {
    recovery: OverheatRecovery,
    boot_read: bool,
    /// `overheat_mode` is saved and clears once a run has prepared.
    clear_after_preparation: bool,
    /// The lowered pair recovery chose, until it is saved.
    maybe_unsaved_pair: Option<MiningHardwareProfile>,
    /// An entry that could not be saved yet, and whether it stays stopped.
    maybe_unsaved_entry: Option<bool>,
    /// The device has paused itself for this stop.
    stop_paused: bool,
}

impl OverheatState {
    /// Whether an overheat keeps the ASIC down.
    pub(super) fn holds(&self) -> bool {
        self.recovery.holds_power_down() || self.maybe_unsaved_pair.is_some()
    }

    /// The device paused itself for an overheat that stays stopped.
    pub(super) fn stopped_by_device(&self) -> bool {
        self.stop_paused
    }
}

impl OrdinaryEspProductionSessionAdapter {
    /// Once per boot: resume an overheat that was saved before the reboot.
    pub(super) fn read_overheat_at_boot(&mut self, now_ms: u64) {
        let overheat = &mut self.local.overheat;
        if overheat.boot_read {
            return;
        }
        overheat.boot_read = true;
        let saved = overheat_store::boot_state();
        // A set marker holds whatever `overheat_mode` says: clearing it from the
        // settings page is not the resume that ends a stop.
        if !saved.overheat_mode && !saved.stopped_marker {
            return;
        }
        overheat.clear_after_preparation = true;
        overheat
            .recovery
            .enter_after_boot(saved_profile().ok(), saved.stopped_marker, now_ms);
        super::retained(
            log::Level::Info,
            &format!(
                "local_mining=overheat_resumed_after_boot stopped={}",
                saved.stopped_marker
            ),
        );
    }

    /// The run ended in an overheat at `profile`.
    pub(super) fn enter_overheat(&mut self, profile: MiningHardwareProfile, now_ms: u64) {
        let overheat = &mut self.local.overheat;
        let entry = crate::board_identity::catalog_entry();
        overheat.recovery.enter(entry, profile, now_ms);
        overheat.clear_after_preparation = true;
        let stays_stopped = reduced_profile(entry, profile).is_none();
        super::retained(
            log::Level::Warn,
            &format!("local_mining=overheat stays_stopped={stays_stopped}"),
        );
        if !save("overheat_entry", overheat_entry_writes(stays_stopped)) {
            overheat.maybe_unsaved_entry = Some(stays_stopped);
        }
    }

    /// One idle tick of recovery: cool, save the lowered pair, or stay stopped.
    pub(super) fn service_overheat(&mut self, now_ms: u64) {
        if let Some(stays_stopped) = self.local.overheat.maybe_unsaved_entry {
            if !save("overheat_entry", overheat_entry_writes(stays_stopped)) {
                return;
            }
            self.local.overheat.maybe_unsaved_entry = None;
        }
        if let Some(pair) = self.local.overheat.maybe_unsaved_pair {
            if !save("overheat_pair", recovery_pair_writes(pair)) {
                return;
            }
            self.local.overheat.maybe_unsaved_pair = None;
        }
        match self
            .local
            .overheat
            .recovery
            .observe(cooling_sample(), now_ms)
        {
            OverheatStep::Recover(pair) => {
                super::retained(log::Level::Info, "local_mining=overheat_recovered");
                // After a reboot the pair is the saved one: saving it again could only
                // overwrite a pair the operator saved while it cooled.
                let already_saved = saved_profile() == Ok(pair);
                if !already_saved && !save("overheat_pair", recovery_pair_writes(pair)) {
                    self.local.overheat.maybe_unsaved_pair = Some(pair);
                }
            }
            OverheatStep::StayStopped if !self.local.overheat.stop_paused => {
                self.local.overheat.stop_paused = true;
                self.pause_by_device(DevicePauseReason::Overheat);
                super::retained(
                    log::Level::Warn,
                    "local_mining=overheat_stopped paused_by_device=true",
                );
            }
            OverheatStep::StayStopped | OverheatStep::Clear | OverheatStep::Cooling => {}
        }
    }

    /// A run has prepared: clear `overheat_mode`, retried each tick until saved.
    pub(super) fn clear_overheat_mode_after_preparation(&mut self) {
        if self.local.overheat.clear_after_preparation
            && save("overheat_clear", Ok(overheat_cleared_writes()))
        {
            self.local.overheat.clear_after_preparation = false;
        }
    }

    /// The operator resumed: a stop that stayed stopped ends, with its marker.
    pub(super) fn resume_overheat(&mut self) {
        let overheat = &mut self.local.overheat;
        if !overheat.stop_paused && !overheat.recovery.holds_power_down() {
            return;
        }
        overheat.recovery.resume();
        if overheat.recovery.holds_power_down() {
            return;
        }
        overheat.stop_paused = false;
        save("overheat_marker_clear", Ok(vec![stop_marker_write(false)]));
    }
}

/// Saves `writes`; a failure is logged, and the caller decides whether to retry.
fn save(what: &str, writes: Result<Vec<NvsWrite>, Vec<ConfigValidationError>>) -> bool {
    let result = writes
        .map_err(|errors| format!("{} invalid settings", errors.len()))
        .and_then(overheat_store::persist);
    if let Err(error) = &result {
        super::retained(
            log::Level::Error,
            &format!("local_mining=save_failed what={what} error={error}"),
        );
    }
    result.is_ok()
}

/// The board's latest restart-cooling sample: the chip reading on the 205, the regulator with
/// the chip unpowered on the 601 (ADR-0041 item 11).
fn cooling_sample() -> Option<bitaxe_api::CoolingSample> {
    crate::safety_adapter::observation_snapshot()
        .restart_cooling_sample(&crate::board_identity::MINING_SAFETY_LIMITS)
}
