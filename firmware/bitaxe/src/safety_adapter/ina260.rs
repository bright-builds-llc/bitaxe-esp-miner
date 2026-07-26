//! INA260 power telemetry adapter.
//!
//! Reference: `reference/esp-miner/main/power/INA260.c`

use bitaxe_safety::{
    power::Ina260RawSample,
    sensor_acquisition::{decode_ina260, AcquisitionOutcome},
};

use super::i2c_bus::{Ina260ReadRegister, ReadOnlySensorBus};

/// Reads one complete INA260 triple through the closed read-only capability.
pub fn read_acquisition(
    bus: &mut ReadOnlySensorBus<'_, '_>,
) -> AcquisitionOutcome<Ina260RawSample> {
    let mut current = [0_u8; 2];
    let mut bus_voltage = [0_u8; 2];
    let mut power = [0_u8; 2];

    let current_result = bus.read_ina260(Ina260ReadRegister::Current, &mut current);
    let bus_voltage_result = bus.read_ina260(Ina260ReadRegister::BusVoltage, &mut bus_voltage);
    let power_result = bus.read_ina260(Ina260ReadRegister::Power, &mut power);
    if current_result.is_err() || bus_voltage_result.is_err() || power_result.is_err() {
        return AcquisitionOutcome::ReadFailed;
    }

    AcquisitionOutcome::Success(decode_ina260(current, bus_voltage, power))
}
