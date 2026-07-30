use super::*;

const SESSION: BootSessionId = BootSessionId::new(7);
const ACQUIRED_AT: MonotonicMillis = MonotonicMillis::new(250);

fn successful_outcomes() -> SensorSweepOutcomes {
    SensorSweepOutcomes {
        power: AcquisitionOutcome::Success(Ina260RawSample {
            bus_voltage_volts: 5.0,
            current_amps: 2.0,
            power_watts: 10.0,
            read_failed: false,
        }),
        asic_temperature_celsius: AcquisitionOutcome::Success(60.0),
        vr_temperature_celsius: AcquisitionOutcome::Success(45.0),
        tachometer_rpm: AcquisitionOutcome::Success(3_000),
    }
}

#[test]
fn ina260_current_is_signed() {
    // Arrange
    let negative_one_raw = [0xff, 0xff];

    // Act
    let sample = decode_ina260(negative_one_raw, [0x0f, 0xa0], [0x00, 0x01]);

    // Assert
    assert_eq!(sample.current_amps, -0.00125);
}

#[test]
fn emc2101_internal_temperature_decodes_signed_values_and_rejects_range() {
    // Arrange
    let positive = 45_u8;
    let negative = (-10_i8).to_ne_bytes()[0];
    let below_plausible_range = (-41_i8).to_ne_bytes()[0];

    // Act
    let positive = decode_emc2101_internal_temperature(positive);
    let negative = decode_emc2101_internal_temperature(negative);
    let below_plausible_range = decode_emc2101_internal_temperature(below_plausible_range);

    // Assert
    assert_eq!(positive, Ok(45.0));
    assert_eq!(negative, Ok(-10.0));
    assert_eq!(
        below_plausible_range,
        Err(SensorValidationError::TemperatureOutOfRange)
    );
}

#[test]
fn vr_failure_preserves_asic_temperature_and_last_good_vr_sample() {
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
    let expected_vr = fresh
        .vr_temperature()
        .maybe_last_good()
        .copied()
        .expect("fresh VR temperature owns a stamp");

    for vr_temperature_celsius in [
        AcquisitionOutcome::ReadFailed,
        AcquisitionOutcome::InvalidSample,
    ] {
        // Act
        let (faulted, next_sequences) = reduce_sensor_sweep(
            fresh,
            sequences,
            SensorSweepOutcomes {
                vr_temperature_celsius,
                ..successful_outcomes()
            },
            SESSION,
            MonotonicMillis::new(750),
            12.0,
        )
        .expect("unaffected sequences should advance");

        // Assert
        assert!(faulted.thermal().temperature_truth().is_fresh());
        assert_eq!(faulted.vr_temperature().state_label(), "fault");
        assert_eq!(
            faulted.vr_temperature().maybe_last_good(),
            Some(&expected_vr)
        );
        assert_eq!(next_sequences.vr_temperature, sequences.vr_temperature);
        assert_eq!(
            next_sequences.asic_temperature,
            sequences
                .asic_temperature
                .advance()
                .expect("fixture sequence should advance")
        );
    }
}

#[test]
fn unsupported_ultra205_vr_source_remains_unavailable_without_a_stamp() {
    // Arrange
    let outcomes = SensorSweepOutcomes {
        vr_temperature_celsius: AcquisitionOutcome::Unavailable(
            UnavailableReason::UnsupportedOnBoard,
        ),
        ..successful_outcomes()
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
    .expect("supported fixture sequences should advance");

    // Assert
    assert!(state.thermal().temperature_truth().is_fresh());
    assert_eq!(state.vr_temperature().state_label(), "unavailable");
    assert_eq!(
        state.vr_temperature().maybe_reason(),
        Some("unsupported_on_board")
    );
    assert!(state.vr_temperature().maybe_last_good().is_none());
    assert_eq!(sequences.vr_temperature, ObservationSequence::ZERO);
}

#[test]
fn sustained_failures_age_all_retained_facts_to_stale() {
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
    let expected_power = fresh.power().truth().maybe_last_good().cloned();
    let expected_temperature = fresh
        .thermal()
        .temperature_truth()
        .maybe_last_good()
        .cloned();
    let expected_vr_temperature = fresh.vr_temperature().maybe_last_good().copied();
    let expected_tachometer = fresh
        .thermal()
        .tachometer_truth()
        .maybe_last_good()
        .cloned();
    let failed_outcomes = SensorSweepOutcomes {
        power: AcquisitionOutcome::ReadFailed,
        asic_temperature_celsius: AcquisitionOutcome::ReadFailed,
        vr_temperature_celsius: AcquisitionOutcome::ReadFailed,
        tachometer_rpm: AcquisitionOutcome::ReadFailed,
    };

    // Act
    let (faulted, next_sequences) = reduce_sensor_sweep(
        fresh,
        sequences,
        failed_outcomes,
        SESSION,
        MonotonicMillis::new(750),
        12.0,
    )
    .expect("failed attempts preserve sequences");
    let retained = faulted.mark_stale_at(MonotonicMillis::new(1_250), 1_000);
    let stale = faulted.mark_stale_at(MonotonicMillis::new(1_251), 1_000);

    // Assert
    assert_eq!(next_sequences, sequences);
    assert_eq!(retained.power().truth().state_label(), "fault");
    assert_eq!(
        retained.thermal().temperature_truth().state_label(),
        "fault"
    );
    assert_eq!(retained.vr_temperature().state_label(), "fault");
    assert_eq!(retained.thermal().tachometer_truth().state_label(), "fault");
    assert_eq!(stale.power().truth().state_label(), "stale");
    assert_eq!(
        stale.power().truth().maybe_last_good(),
        expected_power.as_ref()
    );
    assert_eq!(stale.thermal().temperature_truth().state_label(), "stale");
    assert_eq!(
        stale.thermal().temperature_truth().maybe_last_good(),
        expected_temperature.as_ref()
    );
    assert_eq!(stale.vr_temperature().state_label(), "stale");
    assert_eq!(
        stale.vr_temperature().maybe_last_good(),
        expected_vr_temperature.as_ref()
    );
    assert_eq!(stale.thermal().tachometer_truth().state_label(), "stale");
    assert_eq!(
        stale.thermal().tachometer_truth().maybe_last_good(),
        expected_tachometer.as_ref()
    );
}
