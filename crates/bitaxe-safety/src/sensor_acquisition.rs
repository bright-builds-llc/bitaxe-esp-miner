//! Pure read-only INA260 and EMC2101 acquisition reduction.
//!
//! Firmware owns I2C, cadence, and the monotonic clock. This module accepts
//! completed acquisition outcomes and updates producer-owned observation truth.

use crate::{
    observation::{
        BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence,
        SequenceOverflow, StaleReason, UnavailableReason,
    },
    power::{Ina260RawSample, PowerObservation},
    thermal::{
        TachometerReading, ThermalObservation, ThermalReading, MAX_PLAUSIBLE_TEMP_C,
        MIN_PLAUSIBLE_TEMP_C,
    },
};

pub const MODULE_NAME: &str = "sensor-acquisition";
pub const INA260_CURRENT_MILLIAMPS_PER_BIT: f64 = 1.25;
pub const INA260_BUS_MILLIVOLTS_PER_BIT: f64 = 1.25;
pub const INA260_POWER_MILLIWATTS_PER_BIT: f64 = 10.0;
pub const EMC2101_TACHOMETER_NUMERATOR: u32 = 5_400_000;
pub const EMC2101_TACHOMETER_NO_SPIN_RPM: u32 = 82;
pub const EMC2101_TEMP_FAULT_OPEN_CIRCUIT: u16 = 0x03f8;
pub const EMC2101_TEMP_FAULT_SHORT: u16 = 0x03ff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorValidationError {
    OpenCircuit,
    ShortCircuit,
    TemperatureOutOfRange,
    TachometerOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcquisitionOutcome<T> {
    Success(T),
    ReadFailed,
    InvalidSample,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducerSequences {
    pub power: ObservationSequence,
    pub temperature: ObservationSequence,
    pub tachometer: ObservationSequence,
}

impl Default for ProducerSequences {
    fn default() -> Self {
        Self {
            power: ObservationSequence::ZERO,
            temperature: ObservationSequence::ZERO,
            tachometer: ObservationSequence::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorSweepOutcomes {
    pub power: AcquisitionOutcome<Ina260RawSample>,
    pub temperature_celsius: AcquisitionOutcome<f64>,
    pub tachometer_rpm: AcquisitionOutcome<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProducerSensorState {
    power: PowerObservation,
    thermal: ThermalObservation,
}

impl Default for ProducerSensorState {
    fn default() -> Self {
        Self {
            power: PowerObservation::unavailable(UnavailableReason::NotYetObserved),
            thermal: ThermalObservation::from_facts(
                Observation::unavailable(UnavailableReason::NotYetObserved),
                Observation::unavailable(UnavailableReason::NotYetObserved),
            ),
        }
    }
}

impl ProducerSensorState {
    #[must_use]
    pub const fn power(&self) -> &PowerObservation {
        &self.power
    }

    #[must_use]
    pub const fn thermal(&self) -> &ThermalObservation {
        &self.thermal
    }

    /// Applies producer-owned elapsed-time processing without reading a clock.
    #[must_use]
    pub fn mark_stale_at(self, now: MonotonicMillis, stale_after_ms: u64) -> Self {
        let power = if is_fresh_and_expired(self.power.truth(), now, stale_after_ms) {
            self.power
                .mark_stale(StaleReason::ProducerCadenceExpired)
                .unwrap_or(self.power)
        } else {
            self.power
        };
        let temperature = mark_observation_stale_if_expired(
            self.thermal.temperature_truth(),
            now,
            stale_after_ms,
            StaleReason::ProducerCadenceExpired,
        );
        let tachometer = mark_observation_stale_if_expired(
            self.thermal.tachometer_truth(),
            now,
            stale_after_ms,
            StaleReason::ProducerCadenceExpired,
        );

        Self {
            power,
            thermal: ThermalObservation::from_facts(temperature, tachometer),
        }
    }
}

pub fn decode_ina260(current: [u8; 2], bus_voltage: [u8; 2], power: [u8; 2]) -> Ina260RawSample {
    let current_ma = f64::from(i16::from_be_bytes(current)) * INA260_CURRENT_MILLIAMPS_PER_BIT;
    let bus_voltage_mv = f64::from(u16::from_be_bytes(bus_voltage)) * INA260_BUS_MILLIVOLTS_PER_BIT;
    let power_mw = f64::from(u16::from_be_bytes(power)) * INA260_POWER_MILLIWATTS_PER_BIT;

    Ina260RawSample {
        bus_voltage_volts: bus_voltage_mv / 1000.0,
        current_amps: current_ma / 1000.0,
        power_watts: power_mw / 1000.0,
        read_failed: false,
    }
}

pub fn decode_emc2101_external_temperature(bytes: [u8; 2]) -> Result<f64, SensorValidationError> {
    let raw = u16::from_be_bytes(bytes) >> 5;
    if raw == EMC2101_TEMP_FAULT_OPEN_CIRCUIT {
        return Err(SensorValidationError::OpenCircuit);
    }
    if raw == EMC2101_TEMP_FAULT_SHORT {
        return Err(SensorValidationError::ShortCircuit);
    }

    let signed = sign_extend_11_bit(raw);
    let temperature = f64::from(signed) / 8.0;
    if !(MIN_PLAUSIBLE_TEMP_C..=MAX_PLAUSIBLE_TEMP_C).contains(&temperature) {
        return Err(SensorValidationError::TemperatureOutOfRange);
    }

    Ok(temperature)
}

pub fn decode_emc2101_tachometer(bytes: [u8; 2]) -> Result<u16, SensorValidationError> {
    let raw = u16::from_le_bytes(bytes);
    if raw == 0 {
        return Ok(0);
    }

    let rpm = EMC2101_TACHOMETER_NUMERATOR / u32::from(raw);
    if rpm == EMC2101_TACHOMETER_NO_SPIN_RPM {
        return Ok(0);
    }

    u16::try_from(rpm).map_err(|_| SensorValidationError::TachometerOverflow)
}

pub fn reduce_sensor_sweep(
    prior: ProducerSensorState,
    sequences: ProducerSequences,
    outcomes: SensorSweepOutcomes,
    boot_session: BootSessionId,
    acquired_at: MonotonicMillis,
    board_power_target_watts: f64,
) -> Result<(ProducerSensorState, ProducerSequences), SequenceOverflow> {
    let (power, power_sequence) = reduce_power(
        prior.power,
        sequences.power,
        outcomes.power,
        boot_session,
        acquired_at,
        board_power_target_watts,
    )?;
    let (temperature, temperature_sequence) = reduce_temperature(
        prior.thermal.temperature_truth(),
        sequences.temperature,
        outcomes.temperature_celsius,
        boot_session,
        acquired_at,
    )?;
    let (tachometer, tachometer_sequence) = reduce_tachometer(
        prior.thermal.tachometer_truth(),
        sequences.tachometer,
        outcomes.tachometer_rpm,
        boot_session,
        acquired_at,
    )?;

    Ok((
        ProducerSensorState {
            power,
            thermal: ThermalObservation::from_facts(temperature, tachometer),
        },
        ProducerSequences {
            power: power_sequence,
            temperature: temperature_sequence,
            tachometer: tachometer_sequence,
        },
    ))
}

fn reduce_power(
    prior: PowerObservation,
    prior_sequence: ObservationSequence,
    outcome: AcquisitionOutcome<Ina260RawSample>,
    boot_session: BootSessionId,
    acquired_at: MonotonicMillis,
    board_power_target_watts: f64,
) -> Result<(PowerObservation, ObservationSequence), SequenceOverflow> {
    match outcome {
        AcquisitionOutcome::Success(sample) => prior.record_ina260_success(
            sample,
            board_power_target_watts,
            boot_session,
            prior_sequence,
            acquired_at,
        ),
        AcquisitionOutcome::ReadFailed => Ok((
            prior.record_fault(FaultReason::Ina260ReadFailed),
            prior_sequence,
        )),
        AcquisitionOutcome::InvalidSample => Ok((
            prior.record_fault(FaultReason::PowerReadingInvalid),
            prior_sequence,
        )),
    }
}

fn reduce_temperature(
    prior: &Observation<ThermalReading>,
    prior_sequence: ObservationSequence,
    outcome: AcquisitionOutcome<f64>,
    boot_session: BootSessionId,
    acquired_at: MonotonicMillis,
) -> Result<(Observation<ThermalReading>, ObservationSequence), SequenceOverflow> {
    match outcome {
        AcquisitionOutcome::Success(chip_temp_celsius) => Observation::record_success(
            ThermalReading {
                chip_temp_celsius,
                maybe_board_temp_celsius: None,
                maybe_vr_temp_celsius: None,
            },
            boot_session,
            prior_sequence,
            acquired_at,
        ),
        AcquisitionOutcome::ReadFailed => {
            Ok((prior.record_fault(FaultReason::ReadFailed), prior_sequence))
        }
        AcquisitionOutcome::InvalidSample => Ok((
            prior.record_fault(FaultReason::ThermalReadingInvalid),
            prior_sequence,
        )),
    }
}

fn reduce_tachometer(
    prior: &Observation<TachometerReading>,
    prior_sequence: ObservationSequence,
    outcome: AcquisitionOutcome<u16>,
    boot_session: BootSessionId,
    acquired_at: MonotonicMillis,
) -> Result<(Observation<TachometerReading>, ObservationSequence), SequenceOverflow> {
    match outcome {
        AcquisitionOutcome::Success(rpm) => Observation::record_success(
            TachometerReading::new(rpm),
            boot_session,
            prior_sequence,
            acquired_at,
        ),
        AcquisitionOutcome::ReadFailed => {
            Ok((prior.record_fault(FaultReason::ReadFailed), prior_sequence))
        }
        AcquisitionOutcome::InvalidSample => Ok((
            prior.record_fault(FaultReason::InvalidSample),
            prior_sequence,
        )),
    }
}

fn sign_extend_11_bit(raw: u16) -> i16 {
    if raw & 0x0400 == 0 {
        return raw as i16;
    }

    (raw | 0xf800) as i16
}

fn is_fresh_and_expired<T>(
    observation: &Observation<T>,
    now: MonotonicMillis,
    stale_after_ms: u64,
) -> bool {
    observation.is_fresh()
        && observation.maybe_last_good().is_some_and(|sample| {
            now.get().saturating_sub(sample.acquired_at().get()) > stale_after_ms
        })
}

fn mark_observation_stale_if_expired<T: Clone>(
    observation: &Observation<T>,
    now: MonotonicMillis,
    stale_after_ms: u64,
    reason: StaleReason,
) -> Observation<T> {
    if !is_fresh_and_expired(observation, now, stale_after_ms) {
        return observation.clone();
    }

    observation
        .mark_stale(reason)
        .unwrap_or_else(|_| observation.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: BootSessionId = BootSessionId::new(7);
    const ACQUIRED_AT: MonotonicMillis = MonotonicMillis::new(250);

    fn valid_power() -> Ina260RawSample {
        Ina260RawSample {
            bus_voltage_volts: 5.0,
            current_amps: 2.0,
            power_watts: 10.0,
            read_failed: false,
        }
    }

    fn successful_outcomes() -> SensorSweepOutcomes {
        SensorSweepOutcomes {
            power: AcquisitionOutcome::Success(valid_power()),
            temperature_celsius: AcquisitionOutcome::Success(60.0),
            tachometer_rpm: AcquisitionOutcome::Success(3_000),
        }
    }

    #[test]
    fn sensor_acquisition_ina260_current_is_signed() {
        // Arrange
        let negative_one_raw = [0xff, 0xff];

        // Act
        let sample = decode_ina260(negative_one_raw, [0x0f, 0xa0], [0x00, 0x01]);

        // Assert
        assert_eq!(sample.current_amps, -0.00125);
    }

    #[test]
    fn sensor_acquisition_emc2101_temperature_decodes_positive_and_negative_values() {
        // Arrange
        let positive = ((60_i16 * 8) as u16) << 5;
        let negative_11_bit = ((-10_i16 * 8) as u16) & 0x07ff;
        let negative = negative_11_bit << 5;

        // Act
        let positive = decode_emc2101_external_temperature(positive.to_be_bytes());
        let negative = decode_emc2101_external_temperature(negative.to_be_bytes());

        // Assert
        assert_eq!(positive, Ok(60.0));
        assert_eq!(negative, Ok(-10.0));
    }

    #[test]
    fn sensor_acquisition_emc2101_temperature_rejects_open_and_short_faults() {
        // Arrange
        let open = (EMC2101_TEMP_FAULT_OPEN_CIRCUIT << 5).to_be_bytes();
        let short = (EMC2101_TEMP_FAULT_SHORT << 5).to_be_bytes();

        // Act / Assert
        assert_eq!(
            decode_emc2101_external_temperature(open),
            Err(SensorValidationError::OpenCircuit)
        );
        assert_eq!(
            decode_emc2101_external_temperature(short),
            Err(SensorValidationError::ShortCircuit)
        );
    }

    #[test]
    fn sensor_acquisition_tachometer_handles_zero_sentinel_and_overflow() {
        // Arrange
        let sentinel_raw = u16::MAX;
        let overflow_raw = 1_u16;

        // Act / Assert
        assert_eq!(decode_emc2101_tachometer([0, 0]), Ok(0));
        assert_eq!(decode_emc2101_tachometer(sentinel_raw.to_le_bytes()), Ok(0));
        assert_eq!(
            decode_emc2101_tachometer(overflow_raw.to_le_bytes()),
            Err(SensorValidationError::TachometerOverflow)
        );
    }

    #[test]
    fn sensor_acquisition_success_advances_each_source_once() {
        // Arrange
        let prior = ProducerSensorState::default();
        let sequences = ProducerSequences::default();

        // Act
        let (state, sequences) = reduce_sensor_sweep(
            prior,
            sequences,
            successful_outcomes(),
            SESSION,
            ACQUIRED_AT,
            12.0,
        )
        .expect("fixture sequences should advance");

        // Assert
        assert_eq!(sequences.power, ObservationSequence::new(1));
        assert_eq!(sequences.temperature, ObservationSequence::new(1));
        assert_eq!(sequences.tachometer, ObservationSequence::new(1));
        assert!(state.power().truth().is_fresh());
        assert!(state.thermal().temperature_truth().is_fresh());
        assert!(state.thermal().tachometer_truth().is_fresh());
    }

    #[test]
    fn sensor_acquisition_failed_power_attempts_preserve_atomic_last_good_stamp() {
        // Arrange
        let (fresh, sequences) = reduce_sensor_sweep(
            ProducerSensorState::default(),
            ProducerSequences::default(),
            successful_outcomes(),
            SESSION,
            ACQUIRED_AT,
            12.0,
        )
        .expect("fixture sequences should advance");
        let expected = fresh
            .power()
            .truth()
            .maybe_last_good()
            .expect("fresh power owns a stamp")
            .to_owned();
        for power in [
            AcquisitionOutcome::ReadFailed,
            AcquisitionOutcome::InvalidSample,
        ] {
            // Act
            let (state, next_sequences) = reduce_sensor_sweep(
                fresh,
                sequences,
                SensorSweepOutcomes {
                    power,
                    ..successful_outcomes()
                },
                SESSION,
                MonotonicMillis::new(750),
                12.0,
            )
            .expect("unaffected sequences should advance");

            // Assert
            assert_eq!(next_sequences.power, sequences.power);
            assert_eq!(state.power().truth().maybe_last_good(), Some(&expected));
            assert_eq!(state.power().truth().state_label(), "fault");
        }
    }

    #[test]
    fn sensor_acquisition_temperature_and_tachometer_fail_independently() {
        // Arrange
        let outcomes = SensorSweepOutcomes {
            power: AcquisitionOutcome::Success(valid_power()),
            temperature_celsius: AcquisitionOutcome::ReadFailed,
            tachometer_rpm: AcquisitionOutcome::Success(3_000),
        };

        // Act
        let (state, sequences) = reduce_sensor_sweep(
            ProducerSensorState::default(),
            ProducerSequences::default(),
            outcomes,
            SESSION,
            ACQUIRED_AT,
            12.0,
        )
        .expect("successful sequences should advance");

        // Assert
        assert_eq!(state.thermal().temperature_truth().state_label(), "fault");
        assert!(state.thermal().tachometer_truth().is_fresh());
        assert_eq!(sequences.temperature, ObservationSequence::ZERO);
        assert_eq!(sequences.tachometer, ObservationSequence::new(1));
    }

    #[test]
    fn sensor_acquisition_tachometer_failure_does_not_discard_temperature() {
        // Arrange
        let outcomes = SensorSweepOutcomes {
            power: AcquisitionOutcome::Success(valid_power()),
            temperature_celsius: AcquisitionOutcome::Success(60.0),
            tachometer_rpm: AcquisitionOutcome::InvalidSample,
        };

        // Act
        let (state, sequences) = reduce_sensor_sweep(
            ProducerSensorState::default(),
            ProducerSequences::default(),
            outcomes,
            SESSION,
            ACQUIRED_AT,
            12.0,
        )
        .expect("successful sequences should advance");

        // Assert
        assert!(state.thermal().temperature_truth().is_fresh());
        assert_eq!(state.thermal().tachometer_truth().state_label(), "fault");
        assert_eq!(sequences.temperature, ObservationSequence::new(1));
        assert_eq!(sequences.tachometer, ObservationSequence::ZERO);
    }

    #[test]
    fn sensor_acquisition_stale_transition_uses_producer_supplied_time() {
        // Arrange
        let (fresh, _) = reduce_sensor_sweep(
            ProducerSensorState::default(),
            ProducerSequences::default(),
            successful_outcomes(),
            SESSION,
            ACQUIRED_AT,
            12.0,
        )
        .expect("fixture sequences should advance");

        // Act
        let retained = fresh.mark_stale_at(MonotonicMillis::new(1_250), 1_000);
        let stale = fresh.mark_stale_at(MonotonicMillis::new(1_251), 1_000);

        // Assert
        assert!(retained.power().truth().is_fresh());
        assert_eq!(stale.power().truth().state_label(), "stale");
        assert_eq!(stale.thermal().temperature_truth().state_label(), "stale");
        assert_eq!(stale.thermal().tachometer_truth().state_label(), "stale");
    }
}
