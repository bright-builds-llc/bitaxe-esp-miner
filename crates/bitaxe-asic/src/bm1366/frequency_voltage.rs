//! Pure BM1366 frequency and voltage transition decisions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1366.c:BM1366_send_hash_frequency`
//! - `reference/esp-miner/components/asic/pll.c:pll_get_parameters`
//! - `reference/esp-miner/main/device_config.h`
//! - parity checklist rows `ASIC-006`, `ASIC-007`, and `ASIC-008`

use bitaxe_config::{AsicFrequencyMhz, ConfigValidationError, CoreVoltageMv};

use super::{
    command::{Bm1366Command, FrequencyPlan},
    registers::FREQUENCY_REGISTER,
};

const FREQ_MULT_MHZ: u16 = 25;
const BM1366_FB_DIVIDER_MIN: u16 = 144;
const BM1366_FB_DIVIDER_MAX: u16 = 235;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEffectEvidence {
    MissingHardwareEvidence,
    HardwareSmokeRecorded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityEffectStatus {
    ImplementedNotVerified,
    VerifiedWithHardwareEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366FrequencyPlan {
    frequency: AsicFrequencyMhz,
    command_plan: FrequencyPlan,
    hardware_effect_evidence: HardwareEffectEvidence,
    parity_status: ParityEffectStatus,
}

impl Bm1366FrequencyPlan {
    pub fn ultra_205_bm1366(value: i64) -> Result<Self, ConfigValidationError> {
        let frequency = AsicFrequencyMhz::ultra_205_bm1366(value)?;
        Ok(Self {
            frequency,
            command_plan: frequency_plan(frequency.mhz()),
            hardware_effect_evidence: HardwareEffectEvidence::MissingHardwareEvidence,
            parity_status: ParityEffectStatus::ImplementedNotVerified,
        })
    }

    #[must_use]
    pub const fn frequency_mhz(self) -> u16 {
        self.frequency.mhz()
    }

    #[must_use]
    pub const fn register(self) -> u8 {
        FREQUENCY_REGISTER
    }

    #[must_use]
    pub const fn command_plan(self) -> FrequencyPlan {
        self.command_plan
    }

    #[must_use]
    pub const fn command(self) -> Bm1366Command {
        Bm1366Command::SetFrequency(self.command_plan)
    }

    #[must_use]
    pub const fn hardware_effect_evidence(self) -> HardwareEffectEvidence {
        self.hardware_effect_evidence
    }

    #[must_use]
    pub const fn parity_status(self) -> ParityEffectStatus {
        self.parity_status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366VoltagePlan {
    voltage: CoreVoltageMv,
    hardware_effect_evidence: HardwareEffectEvidence,
    parity_status: ParityEffectStatus,
}

impl Bm1366VoltagePlan {
    pub fn ultra_205_bm1366(value: i64) -> Result<Self, ConfigValidationError> {
        let voltage = CoreVoltageMv::ultra_205_bm1366(value)?;
        Ok(Self {
            voltage,
            hardware_effect_evidence: HardwareEffectEvidence::MissingHardwareEvidence,
            parity_status: ParityEffectStatus::ImplementedNotVerified,
        })
    }

    #[must_use]
    pub const fn millivolts(self) -> u16 {
        self.voltage.millivolts()
    }

    #[must_use]
    pub const fn command(self) -> Option<Bm1366Command> {
        None
    }

    #[must_use]
    pub const fn hardware_effect_evidence(self) -> HardwareEffectEvidence {
        self.hardware_effect_evidence
    }

    #[must_use]
    pub const fn parity_status(self) -> ParityEffectStatus {
        self.parity_status
    }
}

fn frequency_plan(target_mhz: u16) -> FrequencyPlan {
    let pll = pll_parameters(target_mhz);
    let vdo_scale = if u16::from(pll.fb_divider) * FREQ_MULT_MHZ / u16::from(pll.refdiv) >= 2400 {
        0x50
    } else {
        0x40
    };
    let postdiv = (((pll.postdiv1 - 1) & 0x0f) << 4) | ((pll.postdiv2 - 1) & 0x0f);

    FrequencyPlan {
        vdo_scale,
        fb_divider: pll.fb_divider,
        refdiv: pll.refdiv,
        postdiv,
    }
}

fn pll_parameters(target_mhz: u16) -> PllParameters {
    let mut best = PllSearchCandidate {
        fb_divider: 0,
        refdiv: 0,
        postdiv1: 0,
        postdiv2: 0,
        diff_numerator: u32::MAX,
        divider: 1,
        vco_mhz: u16::MAX,
        postdiv_product: u16::MAX,
    };

    for refdiv in (1..=2).rev() {
        for postdiv1 in (1..=7).rev() {
            for postdiv2 in (1..=7).rev() {
                if postdiv1 <= postdiv2 {
                    continue;
                }

                let divider = refdiv * postdiv1 * postdiv2;
                let fb_divider = rounded_div(u32::from(target_mhz) * u32::from(divider));
                if !(BM1366_FB_DIVIDER_MIN..=BM1366_FB_DIVIDER_MAX).contains(&fb_divider) {
                    continue;
                }

                let candidate = PllSearchCandidate {
                    fb_divider: fb_divider as u8,
                    refdiv: refdiv as u8,
                    postdiv1: postdiv1 as u8,
                    postdiv2: postdiv2 as u8,
                    diff_numerator: actual_diff_numerator(target_mhz, fb_divider, divider),
                    divider,
                    vco_mhz: FREQ_MULT_MHZ * fb_divider / refdiv,
                    postdiv_product: postdiv1 * postdiv2,
                };

                if candidate.is_better_than(best) {
                    best = candidate;
                }
            }
        }
    }

    PllParameters {
        fb_divider: best.fb_divider,
        refdiv: best.refdiv,
        postdiv1: best.postdiv1,
        postdiv2: best.postdiv2,
    }
}

const fn rounded_div(numerator: u32) -> u16 {
    ((numerator + (FREQ_MULT_MHZ as u32 / 2)) / FREQ_MULT_MHZ as u32) as u16
}

const fn actual_diff_numerator(target_mhz: u16, fb_divider: u16, divider: u16) -> u32 {
    let actual_scaled = FREQ_MULT_MHZ as i32 * fb_divider as i32;
    let target_scaled = target_mhz as i32 * divider as i32;
    actual_scaled.abs_diff(target_scaled)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PllParameters {
    fb_divider: u8,
    refdiv: u8,
    postdiv1: u8,
    postdiv2: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PllSearchCandidate {
    fb_divider: u8,
    refdiv: u8,
    postdiv1: u8,
    postdiv2: u8,
    diff_numerator: u32,
    divider: u16,
    vco_mhz: u16,
    postdiv_product: u16,
}

impl PllSearchCandidate {
    fn is_better_than(self, other: Self) -> bool {
        let self_diff = u64::from(self.diff_numerator) * u64::from(other.divider);
        let other_diff = u64::from(other.diff_numerator) * u64::from(self.divider);

        self_diff < other_diff
            || (self_diff == other_diff && self.vco_mhz < other.vco_mhz)
            || (self_diff == other_diff
                && self.vco_mhz == other.vco_mhz
                && self.postdiv_product < other.postdiv_product)
    }
}

#[cfg(test)]
mod tests {
    use crate::bm1366::command::Bm1366Command;

    use super::{
        Bm1366FrequencyPlan, Bm1366VoltagePlan, HardwareEffectEvidence, ParityEffectStatus,
    };

    #[test]
    fn frequency_voltage_accept_ultra_205_bm1366_defaults_as_pure_decisions() {
        // Arrange
        let frequency_mhz = 485;
        let voltage_mv = 1200;

        // Act
        let frequency = Bm1366FrequencyPlan::ultra_205_bm1366(frequency_mhz);
        let voltage = Bm1366VoltagePlan::ultra_205_bm1366(voltage_mv);

        // Assert
        assert!(frequency.is_ok());
        assert!(voltage.is_ok());
        assert_eq!(
            frequency.map(|plan| plan.hardware_effect_evidence()),
            Ok(HardwareEffectEvidence::MissingHardwareEvidence)
        );
        assert_eq!(
            voltage.map(|plan| plan.parity_status()),
            Ok(ParityEffectStatus::ImplementedNotVerified)
        );
    }

    #[test]
    fn frequency_voltage_reject_unsupported_ultra_205_bm1366_values() {
        // Arrange
        let invalid_frequency_mhz = 390;
        let invalid_voltage_mv = 1400;

        // Act
        let frequency = Bm1366FrequencyPlan::ultra_205_bm1366(invalid_frequency_mhz);
        let voltage = Bm1366VoltagePlan::ultra_205_bm1366(invalid_voltage_mv);

        // Assert
        assert!(frequency.is_err());
        assert!(voltage.is_err());
    }

    #[test]
    fn frequency_decision_emits_register_0x08_command_but_remains_missing_hardware_evidence() {
        // Arrange
        let frequency = Bm1366FrequencyPlan::ultra_205_bm1366(485)
            .expect("485 MHz should be valid for Ultra 205 BM1366");

        // Act
        let command = frequency.command();

        // Assert
        assert_eq!(frequency.register(), 0x08);
        assert!(matches!(command, Bm1366Command::SetFrequency(_)));
        assert_eq!(
            frequency.hardware_effect_evidence(),
            HardwareEffectEvidence::MissingHardwareEvidence
        );
    }

    #[test]
    fn voltage_decision_is_pure_data_and_not_verified() {
        // Arrange
        let voltage = Bm1366VoltagePlan::ultra_205_bm1366(1200)
            .expect("1200 mV should be valid for Ultra 205 BM1366");

        // Act
        let maybe_command = voltage.command();

        // Assert
        assert_eq!(voltage.millivolts(), 1200);
        assert_eq!(maybe_command, None);
        assert_eq!(
            voltage.parity_status(),
            ParityEffectStatus::ImplementedNotVerified
        );
    }
}
