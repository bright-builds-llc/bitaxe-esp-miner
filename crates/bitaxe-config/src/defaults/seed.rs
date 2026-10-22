//! The private NVS seed a factory install writes for a board (ADR-0041).
//!
//! Every board this project builds shares the Ultra 205's seed except its own profile fields
//! (`board_profile_defaults`), which come from its pinned upstream `config-<board>.cvs`. The
//! golden tests compare each seed with its upstream file.
//!
//! Reference: `reference/esp-miner/config-205.cvs`, `config-601.cvs`

use super::{board_profile_defaults, ultra_205_default_seed_values};
use crate::StoredValue;

/// The boards a factory install may seed.
const SEEDED_BOARDS: [&str; 2] = ["205", "601"];

/// Returns the private NVS seed for `board_version`, or `None` for a board this project does
/// not build. Callers must keep the pool identity fields out of logs and public evidence.
#[must_use]
pub fn board_default_seed_values(board_version: &str) -> Option<Vec<StoredValue>> {
    if !SEEDED_BOARDS.contains(&board_version) {
        return None;
    }
    let profile = board_profile_defaults()
        .iter()
        .find(|profile| profile.is_selectable() && profile.board_version() == board_version)?;
    let overrides = [
        StoredValue::u16("stratumport", profile.primary_pool_port()),
        StoredValue::u16("asicfrequency", profile.asic_frequency_mhz()),
        StoredValue::u16("asicvoltage", profile.asic_voltage_mv()),
        StoredValue::string("asicmodel", profile.asic_model()),
        StoredValue::string("devicemodel", profile.device_model()),
        StoredValue::string("boardversion", profile.board_version()),
        StoredValue::u16("rotation", profile.rotation()),
        StoredValue::u16("autofanspeed", u16::from(profile.auto_fan_speed())),
        StoredValue::u16("fanspeed", profile.manual_fan_speed()),
        StoredValue::u16("selftest", u16::from(profile.self_test())),
        StoredValue::u16("overheat_mode", u16::from(profile.overheat_mode())),
    ];
    let mut values = ultra_205_default_seed_values();
    for value in overrides {
        let slot = values.iter_mut().find(|seeded| seeded.key == value.key)?;
        *slot = value;
    }
    Some(values)
}

#[cfg(test)]
mod tests;
