//! Observe-only firmware safety adapter facade.

mod emc2101;
mod i2c_bus;
mod ina260;
mod observation_store;
mod thermal;
mod watchdog;

pub(crate) use i2c_bus::{BitaxeI2cBus, RuntimeI2cOwner};
pub(crate) use observation_store::{observation_snapshot, replace_observations_from_producer};
pub(crate) use watchdog::supervisor_checkpoint_history;

use bitaxe_safety::{power::Ina260RawSample, sensor_acquisition::AcquisitionOutcome};

pub(crate) fn read_power_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<Ina260RawSample> {
    ina260::read_acquisition(&mut owner.sensors())
}

pub(crate) fn read_temperature_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<f64> {
    emc2101::read_external_temperature_acquisition(&mut owner.sensors())
}

pub(crate) fn read_tachometer_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<u16> {
    emc2101::read_tachometer_acquisition(&mut owner.sensors())
}

pub fn start_safety_supervisor() {
    if let Err(error) = watchdog::start_safety_supervisor_thread() {
        log::warn!("safety_supervisor=unavailable reason=spawn_failed error={error}");
    }
}
