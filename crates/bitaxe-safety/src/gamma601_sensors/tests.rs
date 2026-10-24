use super::*;
use crate::observation::{BootSessionId, FaultReason, MonotonicMillis, Observation};
use crate::sensor_acquisition::{
    reduce_tps546_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
};

const GAMMA_POWER_OFFSET_WATTS: f64 = 5.0;

fn readings(iout_amps: f64) -> RegulatorReadings {
    RegulatorReadings {
        maybe_vin_volts: Some(5.2),
        maybe_iout_amps: Some(iout_amps),
        maybe_vout_volts: Some(1.15),
        maybe_temperature_celsius: Some(48.0),
    }
}

fn sample(outcome: AcquisitionOutcome<Ina260RawSample>) -> Ina260RawSample {
    match outcome {
        AcquisitionOutcome::Success(sample) => sample,
        other => panic!("expected a sample, got {other:?}"),
    }
}

#[test]
fn power_is_vout_times_iout_plus_the_family_offset() {
    // Act
    let sample = sample(power_outcome(
        readings(10.0),
        false,
        GAMMA_POWER_OFFSET_WATTS,
    ));
    // Assert
    assert_eq!(
        (
            sample.bus_voltage_volts,
            sample.current_amps,
            sample.power_watts
        ),
        (5.2, 10.0, 1.15 * 10.0 + 5.0)
    );
}

#[test]
fn an_idle_negative_offset_reports_zero_current_and_the_raw_power() {
    // Act
    let sample = sample(power_outcome(
        readings(-0.2),
        true,
        GAMMA_POWER_OFFSET_WATTS,
    ));
    // Assert
    assert_eq!(
        (sample.current_amps, sample.power_watts),
        (0.0, 1.15 * -0.2 + 5.0)
    );
}

#[test]
fn the_offset_floor_admits_exactly_its_bound_with_the_output_off() {
    // Act
    let outcomes = [-0.5, -0.51].map(|iout| power_outcome(readings(iout), true, 5.0));
    // Assert
    assert!(matches!(outcomes[0], AcquisitionOutcome::Success(_)));
    assert_eq!(outcomes[1], AcquisitionOutcome::InvalidSample);
}

#[test]
fn a_negative_current_is_invalid_unless_the_output_is_confirmed_off() {
    // Act
    let outcome = power_outcome(readings(-0.2), false, GAMMA_POWER_OFFSET_WATTS);
    // Assert
    assert_eq!(outcome, AcquisitionOutcome::InvalidSample);
}

#[test]
fn a_non_finite_current_is_invalid_even_with_the_output_off() {
    // Act
    let outcomes = [f64::NAN, f64::INFINITY].map(|iout| power_outcome(readings(iout), true, 5.0));
    // Assert
    assert_eq!(outcomes, [AcquisitionOutcome::InvalidSample; 2]);
}

#[test]
fn a_missing_regulator_reading_is_a_read_failure() {
    // Arrange
    let missing_vin = RegulatorReadings {
        maybe_vin_volts: None,
        ..readings(1.0)
    };
    // Act
    let outcome = power_outcome(missing_vin, false, GAMMA_POWER_OFFSET_WATTS);
    // Assert
    assert_eq!(outcome, AcquisitionOutcome::ReadFailed);
}

#[test]
fn the_core_voltage_is_vout_in_millivolts() {
    // Act
    let outcomes = [
        core_voltage_outcome(readings(1.0)),
        core_voltage_outcome(RegulatorReadings {
            maybe_vout_volts: Some(-0.01),
            ..readings(1.0)
        }),
        core_voltage_outcome(RegulatorReadings::default()),
    ];
    // Assert
    assert_eq!(
        outcomes,
        [
            AcquisitionOutcome::Success(1150),
            AcquisitionOutcome::InvalidSample,
            AcquisitionOutcome::ReadFailed
        ]
    );
}

#[test]
fn the_regulator_temperature_passes_through() {
    // Act
    let outcomes = [
        vr_temperature_outcome(readings(1.0)),
        vr_temperature_outcome(RegulatorReadings {
            maybe_temperature_celsius: Some(f64::NAN),
            ..readings(1.0)
        }),
        vr_temperature_outcome(RegulatorReadings::default()),
    ];
    // Assert
    assert_eq!(
        outcomes,
        [
            AcquisitionOutcome::Success(48.0),
            AcquisitionOutcome::InvalidSample,
            AcquisitionOutcome::ReadFailed
        ]
    );
}

#[test]
fn a_failed_regulator_read_faults_power_as_a_tps546_read() {
    // Arrange
    let outcomes = SensorSweepOutcomes {
        power: power_outcome(RegulatorReadings::default(), true, GAMMA_POWER_OFFSET_WATTS),
        asic_temperature_celsius: AcquisitionOutcome::Success(50.0),
        vr_temperature_celsius: AcquisitionOutcome::Success(48.0),
        tachometer_rpm: AcquisitionOutcome::Success(3_000),
    };
    // Act
    let (state, _) = reduce_tps546_sensor_sweep(
        ProducerSensorState::default(),
        ProducerSequences::default(),
        outcomes,
        BootSessionId::new(7),
        MonotonicMillis::new(1_000),
        19.0,
    )
    .expect("sequences fit");
    // Assert
    assert!(matches!(
        state.power().truth(),
        Observation::Fault {
            reason: FaultReason::Tps546ReadFailed,
            ..
        }
    ));
}

#[test]
fn a_negative_power_estimate_faults_after_reduction() {
    // Arrange
    let outcomes = SensorSweepOutcomes {
        power: power_outcome(
            RegulatorReadings {
                maybe_vout_volts: Some(30.0),
                ..readings(-0.2)
            },
            true,
            GAMMA_POWER_OFFSET_WATTS,
        ),
        asic_temperature_celsius: AcquisitionOutcome::Success(50.0),
        vr_temperature_celsius: AcquisitionOutcome::Success(48.0),
        tachometer_rpm: AcquisitionOutcome::Success(3_000),
    };
    // Act
    let (state, _) = reduce_tps546_sensor_sweep(
        ProducerSensorState::default(),
        ProducerSequences::default(),
        outcomes,
        BootSessionId::new(7),
        MonotonicMillis::new(1_000),
        19.0,
    )
    .expect("sequences fit");
    // Assert
    assert!(matches!(
        state.power().truth(),
        Observation::Fault {
            reason: FaultReason::PowerReadingInvalid,
            ..
        }
    ));
}
