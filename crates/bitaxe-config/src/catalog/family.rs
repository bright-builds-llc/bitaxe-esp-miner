//! Facts upstream keeps per board family, and the core-voltage ceiling the firmware applies.
//!
//! Reference: `reference/esp-miner/main/device_config.h` (`FamilyConfig`, `FAMILY_*`).

use super::{const_str_eq, BoardCatalogEntry};

/// Upstream's per-family power and supply facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyFacts {
    /// The power the API reports as the family's maximum (`maxPower`), in watts.
    pub max_power_watts: u16,
    /// What upstream adds to a regulator's VOUT × IOUT to estimate board power, in watts.
    pub power_offset_watts: u16,
    /// The nominal supply voltage, in volts.
    pub nominal_voltage_volts: u16,
}

const FAMILIES: [(&str, FamilyFacts); 8] = [
    ("Max", facts(25, 5, 5)),
    ("Ultra", facts(25, 5, 5)),
    ("Hex", facts(90, 12, 12)),
    ("Supra", facts(40, 5, 5)),
    ("Gamma", facts(40, 5, 5)),
    ("GammaDuo", facts(40, 5, 5)),
    ("SupraHex", facts(120, 25, 12)),
    ("GammaTurbo", facts(60, 10, 12)),
];

/// The highest core voltage commanded on a board with a TPS546, in millivolts: the TPS546
/// configuration plan's 1.20 V clamp (ADR-0041 items 6 and 7). Upstream offers 1250 mV; the
/// firmware refuses it until a separate decision raises the clamp.
pub const TPS546_CORE_VOLTAGE_CEILING_MV: u16 = 1_200;

const fn facts(
    max_power_watts: u16,
    power_offset_watts: u16,
    nominal_voltage_volts: u16,
) -> FamilyFacts {
    FamilyFacts {
        max_power_watts,
        power_offset_watts,
        nominal_voltage_volts,
    }
}

impl BoardCatalogEntry {
    /// The family's power and supply facts. Every catalog family is listed, which a test
    /// checks; a family missing from the table panics here.
    #[must_use]
    pub const fn family_facts(&self) -> FamilyFacts {
        let mut index = 0;
        while index < FAMILIES.len() {
            if const_str_eq(FAMILIES[index].0, self.family()) {
                return FAMILIES[index].1;
            }
            index += 1;
        }
        panic!("board family missing from the family facts");
    }

    /// The highest core voltage the firmware commands on this board, when it is lower than the
    /// catalog's highest option.
    #[must_use]
    pub const fn maybe_core_voltage_ceiling_mv(&self) -> Option<u16> {
        if self.capabilities().tps546() {
            Some(TPS546_CORE_VOLTAGE_CEILING_MV)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
