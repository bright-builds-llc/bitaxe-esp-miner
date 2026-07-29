//! DS4432U core-voltage adapter.
//!
//! Reference: `reference/esp-miner/main/power/DS4432U.c`

const BITAXE_IFS_AMPS: f64 = 0.000_098_921;
const BITAXE_RA_OHMS: f64 = 4_750.0;
const BITAXE_RB_OHMS: f64 = 3_320.0;
const BITAXE_NOMINAL_VOLTS: f64 = 1.451;
const TPS40305_FEEDBACK_VOLTS: f64 = 0.6;

/// Validated Ultra 205 DS4432U setpoints.
///
/// There is deliberately no `Off` setpoint. Pinned upstream
/// `VCORE_set_voltage(0)` skips the DS4432U write and removes VCORE through
/// the separately owned GPIO10 ASIC-enable boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ultra205CoreVoltage {
    /// Programs the conservative mining profile's 1100 mV setpoint.
    Conservative1100Millivolts,
    /// Programs the pinned upstream default profile's 1200 mV setpoint.
    UpstreamDefault1200Millivolts,
}

impl Ultra205CoreVoltage {
    const fn millivolts(self) -> u16 {
        match self {
            Self::Conservative1100Millivolts => 1_100,
            Self::UpstreamDefault1200Millivolts => 1_200,
        }
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
    debug_assert!(matches!(millivolts, 1_100 | 1_200));

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
        let result =
            write_core_voltage(&mut writer, Ultra205CoreVoltage::Conservative1100Millivolts);

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
        let result = write_core_voltage(
            &mut writer,
            Ultra205CoreVoltage::UpstreamDefault1200Millivolts,
        );

        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(writer.writes, [(Ds4432uWriteRegister::Output0, 0xc6)]);
    }
}
