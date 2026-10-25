//! The Gamma 601's sensor readings as the shared sweep outcomes: power, input voltage, current,
//! core voltage and regulator temperature from the TPS546, as upstream reports them.
//!
//! Reference: `reference/esp-miner/main/power/power.c` (`Power_get_output`: VOUT × IOUT plus
//! the family's power offset, with IOUT as the current; `Power_get_input_voltage` and
//! `Power_get_vreg_temp`) and `main/power/vcore.c` (the core voltage is VOUT over the voltage
//! domains, one on the Gamma).

use crate::observation::FaultReason;
use crate::power::Ina260RawSample;
use crate::sensor_acquisition::AcquisitionOutcome;
use crate::tps546::StatusWord;

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
/// - the current is IOUT. Only while the chip draws no hashing load (`unloaded`: the output
///   confirmed off, or the BM1370 powered but not yet initialized, so held in reset) is an
///   offset down to `IOUT_OFFSET_FLOOR_AMPS` reported as zero. An initialized chip's negative
///   reading stays invalid, so the floor can never hide power while mining. Phase F3.3's first
///   attempt found the regulator's offset (about -0.2 A on the 601) read negative in the first
///   sweep after the enable, which revoked the run.
///
/// Any other negative or non-finite IOUT is an invalid sample.
#[must_use]
pub fn power_outcome(
    readings: RegulatorReadings,
    unloaded: bool,
    power_offset_watts: f64,
) -> AcquisitionOutcome<Ina260RawSample> {
    let (Some(vin), Some(iout), Some(vout)) = (
        readings.maybe_vin_volts,
        readings.maybe_iout_amps,
        readings.maybe_vout_volts,
    ) else {
        return AcquisitionOutcome::ReadFailed;
    };
    let floor_amps = if unloaded {
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

/// The fault the regulator's status word puts on the power fact, as upstream's
/// `TPS546_check_status`: an unread status is a failed read.
#[must_use]
pub fn regulator_fault(
    maybe_status: Option<StatusWord>,
    output_expected_on: bool,
) -> Option<FaultReason> {
    match maybe_status {
        None => Some(FaultReason::Tps546ReadFailed),
        Some(status) if status.power_fault(output_expected_on) => {
            Some(FaultReason::Tps546PowerFault)
        }
        Some(_) => None,
    }
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
