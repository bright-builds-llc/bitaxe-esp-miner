//! EMC2101 thermal and fan adapter.
//!
//! Reference: `reference/esp-miner/main/thermal/EMC2101.c`

use bitaxe_safety::sensor_acquisition::{
    decode_emc2101_external_temperature, decode_emc2101_tachometer, AcquisitionOutcome,
};

use super::i2c_bus::{Emc2101ReadRegister, ReadOnlySensorBus};

pub fn read_external_temperature_acquisition(
    bus: &mut ReadOnlySensorBus<'_, '_>,
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

pub fn read_tachometer_acquisition(bus: &mut ReadOnlySensorBus<'_, '_>) -> AcquisitionOutcome<u16> {
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
