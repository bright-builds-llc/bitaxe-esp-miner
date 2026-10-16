//! DS4432U core-voltage adapter.
//!
//! Reference: `reference/esp-miner/main/power/DS4432U.c`

const BITAXE_IFS_AMPS: f64 = 0.000_098_921;
const BITAXE_RA_OHMS: f64 = 4_750.0;
const BITAXE_RB_OHMS: f64 = 3_320.0;
const BITAXE_NOMINAL_VOLTS: f64 = 1.451;
const TPS40305_FEEDBACK_VOLTS: f64 = 0.6;

/// A validated Ultra 205 DS4432U setpoint: any millivolt value in the BM1366
/// catalog range (ADR-0037).
///
/// There is deliberately no `Off` setpoint. Pinned upstream
/// `VCORE_set_voltage(0)` skips the DS4432U write and removes VCORE through
/// the separately owned GPIO10 ASIC-enable boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ultra205CoreVoltage(u16);

impl Ultra205CoreVoltage {
    /// The conservative mining profile's 1100 mV setpoint.
    #[cfg(test)]
    pub(crate) const CONSERVATIVE: Self = Self(1_100);
    /// The pinned upstream default profile's 1200 mV setpoint.
    #[cfg(test)]
    pub(crate) const UPSTREAM_DEFAULT: Self = Self(1_200);

    /// Accepts only a voltage inside the Ultra 205 BM1366 catalog range.
    pub(crate) fn ultra_205(millivolts: u16) -> Option<Self> {
        bitaxe_config::CoreVoltageMv::ultra_205_bm1366(i64::from(millivolts))
            .ok()
            .map(|voltage| Self(voltage.millivolts()))
    }

    const fn millivolts(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ds4432uWriteRegister {
    Output0,
}

impl Ds4432uWriteRegister {
    pub(crate) const fn address(self) -> u8 {
        match self {
            Self::Output0 => 0xf8,
        }
    }
}

pub(crate) trait Ds4432uRegisterWriter {
    type Error;

    fn write_ds4432u(
        &mut self,
        register: Ds4432uWriteRegister,
        value: u8,
    ) -> Result<(), Self::Error>;
}

pub(crate) fn write_core_voltage<Bus>(
    bus: &mut Bus,
    voltage: Ultra205CoreVoltage,
) -> Result<(), Bus::Error>
where
    Bus: Ds4432uRegisterWriter,
{
    let code = core_voltage_code(voltage.millivolts());
    bus.write_ds4432u(Ds4432uWriteRegister::Output0, code)
}

fn core_voltage_code(millivolts: u16) -> u8 {
    debug_assert!((1_100..=1_300).contains(&millivolts));

    let volts = f64::from(millivolts) / 1_000.0;
    let change = (((TPS40305_FEEDBACK_VOLTS / BITAXE_RB_OHMS
        - (volts - TPS40305_FEEDBACK_VOLTS) / BITAXE_RA_OHMS)
        / BITAXE_IFS_AMPS)
        * 127.0)
        .abs()
        .ceil();
    let mut code = change as u8;
    if volts < BITAXE_NOMINAL_VOLTS {
        code |= 0x80;
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct FakeWriter {
        writes: Vec<(Ds4432uWriteRegister, u8)>,
    }

    impl Ds4432uRegisterWriter for FakeWriter {
        type Error = ();

        fn write_ds4432u(
            &mut self,
            register: Ds4432uWriteRegister,
            value: u8,
        ) -> Result<(), Self::Error> {
            self.writes.push((register, value));
            Ok(())
        }
    }

    #[test]
    fn conservative_voltage_uses_upstream_output_zero_code() {
        // Arrange
        let mut writer = FakeWriter::default();

        // Act
        let result = write_core_voltage(&mut writer, Ultra205CoreVoltage::CONSERVATIVE);

        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(writer.writes, [(Ds4432uWriteRegister::Output0, 0xe1)]);
        assert_eq!(Ds4432uWriteRegister::Output0.address(), 0xf8);
    }

    #[test]
    fn upstream_default_voltage_uses_upstream_output_zero_code() {
        // Arrange
        let mut writer = FakeWriter::default();

        // Act
        let result = write_core_voltage(&mut writer, Ultra205CoreVoltage::UPSTREAM_DEFAULT);

        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(writer.writes, [(Ds4432uWriteRegister::Output0, 0xc6)]);
    }

    #[test]
    fn every_catalog_range_voltage_programs_a_below_nominal_code() {
        // Arrange: every millivolt from 1100 to 1300 is below the 1.451 V nominal.
        let voltages: Vec<Ultra205CoreVoltage> = (1_100..=1_300)
            .map(|mv| Ultra205CoreVoltage::ultra_205(mv).expect("catalog range"))
            .collect();

        // Act
        let codes: Vec<u8> = voltages
            .iter()
            .map(|voltage| core_voltage_code(voltage.millivolts()))
            .collect();

        // Assert: the reduce bit is set and the step size shrinks as voltage rises.
        assert!(codes.iter().all(|code| code & 0x80 != 0));
        assert!(codes
            .windows(2)
            .all(|pair| pair[0] & 0x7f >= pair[1] & 0x7f));
    }

    #[test]
    fn a_voltage_outside_the_catalog_range_is_not_a_setpoint() {
        // Arrange / Act
        let voltages = [1_099, 1_301].map(Ultra205CoreVoltage::ultra_205);

        // Assert
        assert_eq!(voltages, [None, None]);
    }

    #[test]
    fn interior_catalog_voltages_match_the_upstream_codes() {
        // Arrange: codes computed by the pinned upstream `DS4432U.c` formula.
        let voltages = [1_150, 1_250, 1_300];

        // Act
        let codes = voltages.map(core_voltage_code);

        // Assert
        assert_eq!(codes, [0xd4, 0xb9, 0xab]);
    }
}
