//! EMC2101 thermal and fan adapter.
//!
//! Reference: `reference/esp-miner/main/thermal/EMC2101.c`

use bitaxe_safety::sensor_acquisition::{
    decode_emc2101_external_temperature, decode_emc2101_internal_temperature,
    decode_emc2101_tachometer, AcquisitionOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Emc2101ReadRegister {
    InternalTemperature,
    ExternalTemperatureMsb,
    ExternalTemperatureLsb,
    TachometerLsb,
    TachometerMsb,
}

impl Emc2101ReadRegister {
    pub(crate) const fn address(self) -> u8 {
        match self {
            Self::InternalTemperature => 0x00,
            Self::ExternalTemperatureMsb => 0x01,
            Self::ExternalTemperatureLsb => 0x10,
            Self::TachometerLsb => 0x46,
            Self::TachometerMsb => 0x47,
        }
    }
}

pub(crate) trait Emc2101RegisterReader {
    type Error;

    fn read_emc2101(
        &mut self,
        register: Emc2101ReadRegister,
        output: &mut [u8; 1],
    ) -> Result<(), Self::Error>;
}

pub(crate) fn read_external_temperature_acquisition(
    bus: &mut impl Emc2101RegisterReader,
) -> AcquisitionOutcome<f64> {
    let mut msb = [0_u8; 1];
    let mut lsb = [0_u8; 1];

    if bus
        .read_emc2101(Emc2101ReadRegister::ExternalTemperatureMsb, &mut msb)
        .is_err()
        || bus
            .read_emc2101(Emc2101ReadRegister::ExternalTemperatureLsb, &mut lsb)
            .is_err()
    {
        return AcquisitionOutcome::ReadFailed;
    }

    match decode_emc2101_external_temperature([msb[0], lsb[0]]) {
        Ok(temperature) => AcquisitionOutcome::Success(temperature),
        Err(_) => AcquisitionOutcome::InvalidSample,
    }
}

pub(crate) fn read_internal_temperature_acquisition(
    bus: &mut impl Emc2101RegisterReader,
) -> AcquisitionOutcome<f64> {
    let mut temperature = [0_u8; 1];
    if bus
        .read_emc2101(Emc2101ReadRegister::InternalTemperature, &mut temperature)
        .is_err()
    {
        return AcquisitionOutcome::ReadFailed;
    }

    match decode_emc2101_internal_temperature(temperature[0]) {
        Ok(temperature) => AcquisitionOutcome::Success(temperature),
        Err(_) => AcquisitionOutcome::InvalidSample,
    }
}

pub(crate) fn read_tachometer_acquisition(
    bus: &mut impl Emc2101RegisterReader,
) -> AcquisitionOutcome<u16> {
    let mut lsb = [0_u8; 1];
    let mut msb = [0_u8; 1];

    if bus
        .read_emc2101(Emc2101ReadRegister::TachometerLsb, &mut lsb)
        .is_err()
        || bus
            .read_emc2101(Emc2101ReadRegister::TachometerMsb, &mut msb)
            .is_err()
    {
        return AcquisitionOutcome::ReadFailed;
    }

    match decode_emc2101_tachometer([lsb[0], msb[0]]) {
        Ok(rpm) => AcquisitionOutcome::Success(rpm),
        Err(_) => AcquisitionOutcome::InvalidSample,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug)]
    struct FakeReader {
        reads: VecDeque<(Emc2101ReadRegister, Result<u8, ()>)>,
    }

    impl Emc2101RegisterReader for FakeReader {
        type Error = ();

        fn read_emc2101(
            &mut self,
            register: Emc2101ReadRegister,
            output: &mut [u8; 1],
        ) -> Result<(), Self::Error> {
            let (expected_register, result) = self
                .reads
                .pop_front()
                .expect("unexpected extra register read");
            assert_eq!(register, expected_register);
            let value = result?;
            output[0] = value;
            Ok(())
        }
    }

    #[test]
    fn internal_temperature_acquisition_reads_the_vr_temperature_register() {
        // Arrange
        let mut reader = FakeReader {
            reads: VecDeque::from([(Emc2101ReadRegister::InternalTemperature, Ok(45))]),
        };

        // Act
        let outcome = read_internal_temperature_acquisition(&mut reader);

        // Assert
        assert_eq!(outcome, AcquisitionOutcome::Success(45.0));
        assert!(reader.reads.is_empty());
        assert_eq!(Emc2101ReadRegister::InternalTemperature.address(), 0x00);
    }

    #[test]
    fn internal_temperature_acquisition_fails_closed_on_read_or_decode_failure() {
        // Arrange
        let mut read_failure = FakeReader {
            reads: VecDeque::from([(Emc2101ReadRegister::InternalTemperature, Err(()))]),
        };
        let mut invalid_sample = FakeReader {
            reads: VecDeque::from([(
                Emc2101ReadRegister::InternalTemperature,
                Ok((-41_i8).to_ne_bytes()[0]),
            )]),
        };

        // Act
        let read_failure = read_internal_temperature_acquisition(&mut read_failure);
        let invalid_sample = read_internal_temperature_acquisition(&mut invalid_sample);

        // Assert
        assert_eq!(read_failure, AcquisitionOutcome::ReadFailed);
        assert_eq!(invalid_sample, AcquisitionOutcome::InvalidSample);
    }

    #[test]
    fn external_temperature_acquisition_reads_msb_then_lsb() {
        // Arrange
        let bytes = (((60_i16 * 8) as u16) << 5).to_be_bytes();
        let mut reader = FakeReader {
            reads: VecDeque::from([
                (Emc2101ReadRegister::ExternalTemperatureMsb, Ok(bytes[0])),
                (Emc2101ReadRegister::ExternalTemperatureLsb, Ok(bytes[1])),
            ]),
        };

        // Act
        let outcome = read_external_temperature_acquisition(&mut reader);

        // Assert
        assert_eq!(outcome, AcquisitionOutcome::Success(60.0));
        assert!(reader.reads.is_empty());
    }

    #[test]
    fn tachometer_acquisition_reads_lsb_then_msb() {
        // Arrange
        let bytes = 1_800_u16.to_le_bytes();
        let mut reader = FakeReader {
            reads: VecDeque::from([
                (Emc2101ReadRegister::TachometerLsb, Ok(bytes[0])),
                (Emc2101ReadRegister::TachometerMsb, Ok(bytes[1])),
            ]),
        };

        // Act
        let outcome = read_tachometer_acquisition(&mut reader);

        // Assert
        assert_eq!(outcome, AcquisitionOutcome::Success(3_000));
        assert!(reader.reads.is_empty());
    }
}
