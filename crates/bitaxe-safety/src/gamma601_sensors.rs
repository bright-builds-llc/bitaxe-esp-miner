//! The Gamma 601's sensor readings as the shared sweep outcomes: power, input voltage, current,
//! core voltage and regulator temperature from the TPS546, as upstream reports them.
//!
//! Reference: `reference/esp-miner/main/power/power.c` (`Power_get_output`: VOUT × IOUT plus
//! the family's power offset, with IOUT as the current; `Power_get_input_voltage` and
//! `Power_get_vreg_temp`) and `main/power/vcore.c` (the core voltage is VOUT over the voltage
//! domains, one on the Gamma).

use crate::power::Ina260RawSample;
use crate::sensor_acquisition::AcquisitionOutcome;

/// The most negative IOUT taken as the sensor's offset rather than a fault. With the output
/// off the 601 reads about -0.2 A (`docs/parity/evidence/20261022-gamma601-safe-boot.md`).
pub const IOUT_OFFSET_FLOOR_AMPS: f64 = -0.5;

/// One sweep's regulator telemetry; `None` where the read failed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RegulatorReadings {
    pub maybe_vin_volts: Option<f64>,
    pub maybe_iout_amps: Option<f64>,
    pub maybe_vout_volts: Option<f64>,
    pub maybe_temperature_celsius: Option<f64>,
}

/// The power facts in the shared reducer's sample shape:
/// - the input voltage is VIN;
/// - the power is VOUT × IOUT plus the family's offset, from the raw IOUT, as upstream;
/// - the current is IOUT. Only with the output confirmed off is an offset down to
///   `IOUT_OFFSET_FLOOR_AMPS` reported as zero, so it can never hide power while mining.
///
/// Any other negative or non-finite IOUT is an invalid sample.
#[must_use]
pub fn power_outcome(
    readings: RegulatorReadings,
    output_confirmed_off: bool,
    power_offset_watts: f64,
) -> AcquisitionOutcome<Ina260RawSample> {
    let (Some(vin), Some(iout), Some(vout)) = (
        readings.maybe_vin_volts,
        readings.maybe_iout_amps,
        readings.maybe_vout_volts,
    ) else {
        return AcquisitionOutcome::ReadFailed;
    };
    let floor_amps = if output_confirmed_off {
        IOUT_OFFSET_FLOOR_AMPS
    } else {
        0.0
    };
    if !iout.is_finite() || iout < floor_amps {
        return AcquisitionOutcome::InvalidSample;
    }
    AcquisitionOutcome::Success(Ina260RawSample {
        bus_voltage_volts: vin,
        current_amps: iout.max(0.0),
        power_watts: vout * iout + power_offset_watts,
        read_failed: false,
    })
}

/// The core voltage in millivolts: VOUT over the Gamma's one voltage domain.
#[must_use]
pub fn core_voltage_outcome(readings: RegulatorReadings) -> AcquisitionOutcome<u16> {
    let Some(vout) = readings.maybe_vout_volts else {
        return AcquisitionOutcome::ReadFailed;
    };
    let millivolts = (vout * 1000.0).round();
    if !(0.0..=f64::from(u16::MAX)).contains(&millivolts) {
        return AcquisitionOutcome::InvalidSample;
    }
    // The range check above keeps the cast exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    AcquisitionOutcome::Success(millivolts as u16)
}

/// The regulator temperature, as upstream's `Power_get_vreg_temp`.
#[must_use]
pub fn vr_temperature_outcome(readings: RegulatorReadings) -> AcquisitionOutcome<f64> {
    match readings.maybe_temperature_celsius {
        Some(celsius) if celsius.is_finite() => AcquisitionOutcome::Success(celsius),
        Some(_) => AcquisitionOutcome::InvalidSample,
        None => AcquisitionOutcome::ReadFailed,
    }
}

#[cfg(test)]
mod tests;
