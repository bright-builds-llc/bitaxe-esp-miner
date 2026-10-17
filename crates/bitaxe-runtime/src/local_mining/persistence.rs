//! What local mining saves across an overheat (ADR-0038), as NVS write sets the
//! firmware commits in one verified settings transaction. Pure.
//!
//! The fan and the lowered pair go through the settings validator, so they are
//! stored, with their legacy keys, exactly as an operator's PATCH stores them.
//! `overheat_mode` is set directly: the API only accepts clearing it.
//!
//! The stop marker is project-owned and device-internal: it is not in the REST
//! schema or the update-preservation fingerprint. It records that an overheat
//! stopped for good (`StayStopped`), so a reboot keeps the stop until resumed.

use bitaxe_config::{
    apply_settings_patch, ConfigValidationError, NvsSnapshot, NvsWrite, RawSettingValue,
    SettingsPatch, SettingsUpdateDecision,
};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

pub use bitaxe_api::OVERHEAT_STOP_MARKER_KEY;

use super::OVERHEAT_ENTRY_SETTINGS;

/// Saved when an overheat stops mining: the stop marker (set when recovery cannot
/// lower the pair), the fan as manual at 100%, then `overheat_mode`. NVS writes
/// each key as it is set, so the marker goes first: a power cut part-way never
/// leaves `overheat_mode` beside a stale marker from an earlier overheat.
///
/// # Errors
/// The settings validator refused a value; nothing should be written.
pub fn overheat_entry_writes(stopped: bool) -> Result<Vec<NvsWrite>, Vec<ConfigValidationError>> {
    let mut writes = vec![stop_marker_write(stopped)];
    writes.extend(validated([
        (
            "autofanspeed",
            RawSettingValue::Bool(OVERHEAT_ENTRY_SETTINGS.auto_fan_speed),
        ),
        (
            "manualFanSpeed",
            RawSettingValue::Number(i64::from(OVERHEAT_ENTRY_SETTINGS.manual_fan_speed_percent)),
        ),
    ])?);
    writes.push(NvsWrite::u16(
        "overheat_mode",
        u16::from(OVERHEAT_ENTRY_SETTINGS.overheat_mode),
    ));
    Ok(writes)
}

/// Saved before recovery powers up again: the lowered pair.
///
/// # Errors
/// The settings validator refused the pair; nothing should be written.
pub fn recovery_pair_writes(
    profile: MiningHardwareProfile,
) -> Result<Vec<NvsWrite>, Vec<ConfigValidationError>> {
    validated([
        (
            "frequency",
            RawSettingValue::Number(i64::from(profile.frequency().mhz())),
        ),
        (
            "coreVoltage",
            RawSettingValue::Number(i64::from(profile.core_voltage().millivolts())),
        ),
    ])
}

/// Saved once the restart's preparation succeeds, as upstream clears it after it
/// detects the ASIC. The marker is cleared too, so none outlives its overheat.
#[must_use]
pub fn overheat_cleared_writes() -> Vec<NvsWrite> {
    vec![NvsWrite::u16("overheat_mode", 0), stop_marker_write(false)]
}

/// Saved when an overheat stops for good, and cleared by a resume.
#[must_use]
pub fn stop_marker_write(stopped: bool) -> NvsWrite {
    NvsWrite::u16(OVERHEAT_STOP_MARKER_KEY, u16::from(stopped))
}

/// What boot reads to resume an overheat (`OverheatRecovery::enter_after_boot`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OverheatBootState {
    pub overheat_mode: bool,
    pub stopped_marker: bool,
}

impl OverheatBootState {
    /// Reads `overheat_mode` from the confirmed settings and the marker's raw value;
    /// a missing marker reads as clear.
    #[must_use]
    pub fn read(settings: &NvsSnapshot, maybe_stop_marker: Option<u16>) -> Self {
        let overheat_mode = matches!(
            bitaxe_config::reload_snapshot(settings).maybe_loaded_value("overheat_mode"),
            Some(bitaxe_config::LoadedValue::Bool(true))
        );
        Self {
            overheat_mode,
            stopped_marker: maybe_stop_marker.is_some_and(|value| value != 0),
        }
    }
}

/// Builds writes through the settings validator, as a PATCH would.
fn validated(
    pairs: [(&str, RawSettingValue); 2],
) -> Result<Vec<NvsWrite>, Vec<ConfigValidationError>> {
    match apply_settings_patch(&SettingsPatch::from_pairs(pairs)) {
        SettingsUpdateDecision::Accepted { writes } => Ok(writes),
        SettingsUpdateDecision::Rejected { errors } => Err(errors),
    }
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;
