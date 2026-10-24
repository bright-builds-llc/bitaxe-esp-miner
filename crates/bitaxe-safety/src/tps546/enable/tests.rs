use super::*;

const TARGET: f64 = 1.15;
/// `STATUS_WORD` with only the `OFF` bit: the baseline before the enable.
const OFF: u16 = 0x0040;

fn nominal() -> Readings {
    Readings {
        vout_volts: 1.15,
        iout_amps: 0.4,
        vin_volts: 5.2,
        vr_temp_c: 40.0,
        board_temp_c: 35.0,
        status: StatusWord(0x0000),
    }
}

fn judge() -> EnableJudge {
    EnableJudge::new(TARGET, StatusWord(OFF)).expect("judge")
}

fn sample(elapsed_ms: u32, readings: Readings) -> Sample {
    Sample {
        elapsed_ms,
        maybe_readings: Some(readings),
    }
}

/// Feeds nominal samples every 250 ms up to, but not including, `until_ms`.
fn run_nominal(judge: &mut EnableJudge, until_ms: u32) {
    for elapsed_ms in (250..until_ms).step_by(250) {
        assert_eq!(judge.observe(sample(elapsed_ms, nominal())), Step::Continue);
    }
}

#[test]
fn a_nominal_window_completes_after_sixty_seconds() {
    // Arrange
    let mut judge = judge();
    run_nominal(&mut judge, WINDOW_MS);
    // Act
    let step = judge.observe(sample(WINDOW_MS, nominal()));
    // Assert
    assert_eq!(step, Step::Done);
}

#[test]
fn a_nominal_window_counts_every_sample() {
    // Arrange
    let mut judge = judge();
    run_nominal(&mut judge, WINDOW_MS);
    // Act
    judge.observe(sample(WINDOW_MS, nominal()));
    // Assert
    assert_eq!(judge.summary().samples, 240);
}

#[test]
fn a_failed_read_is_a_missed_sample() {
    // Act
    let step = judge().observe(Sample {
        elapsed_ms: 250,
        maybe_readings: None,
    });
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::MissedSample));
}

#[test]
fn a_late_sample_is_a_missed_sample() {
    // Act
    let step = judge().observe(sample(MAX_SAMPLE_GAP_MS + 1, nominal()));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::MissedSample));
}

#[test]
fn a_stalled_clock_aborts() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, nominal()));
    // Act
    let step = judge.observe(sample(250, nominal()));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::ClockNotAdvancing));
}

#[test]
fn a_clock_that_runs_backwards_aborts() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(500, nominal()));
    // Act
    let step = judge.observe(sample(250, nominal()));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::ClockNotAdvancing));
}

#[test]
fn a_slow_clock_cannot_stretch_the_window_past_its_sample_cap() {
    // Arrange: the clock advances 1 ms per sample, so it never reaches the window's end.
    let mut judge = judge();
    let mut maybe_last = None;
    // Act
    for elapsed_ms in 1..=MAX_SAMPLES {
        maybe_last = Some(judge.observe(sample(elapsed_ms, nominal())));
    }
    // Assert
    assert_eq!(maybe_last, Some(Step::Abort(EnableAbort::WindowOverrun)));
}

#[test]
fn an_abort_is_latched() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(
        250,
        Readings {
            vout_volts: 1.26,
            ..nominal()
        },
    ));
    // Act
    let step = judge.observe(sample(500, nominal()));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VoutOverCeiling));
}

#[test]
fn a_non_finite_reading_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            iout_amps: f64::NAN,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::NonFiniteReading));
}

#[test]
fn vout_above_the_ceiling_aborts_even_before_settling() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vout_volts: 1.26,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VoutOverCeiling));
}

#[test]
fn a_rising_vout_is_allowed_before_settling() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vout_volts: 0.6,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Continue);
}

#[test]
fn vout_out_of_band_after_settling_aborts() {
    // Arrange
    let mut judge = judge();
    run_nominal(&mut judge, SETTLE_MS);
    // Act
    let step = judge.observe(sample(
        SETTLE_MS,
        Readings {
            vout_volts: 1.09,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VoutOutOfBand));
}

#[test]
fn overcurrent_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            iout_amps: 5.1,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::Overcurrent));
}

#[test]
fn a_large_negative_current_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            iout_amps: -20.0,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::Overcurrent));
}

#[test]
fn a_hot_regulator_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vr_temp_c: 81.0,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VrOverTemperature));
}

#[test]
fn a_hot_board_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            board_temp_c: 61.0,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::BoardOverTemperature));
}

#[test]
fn a_temperature_below_zero_is_implausible() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vr_temp_c: -40.0,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::ImplausibleReading));
}

#[test]
fn a_low_vin_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vin_volts: 4.7,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VinOutOfRange));
}

#[test]
fn a_high_vin_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            vin_volts: 5.8,
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::VinOutOfRange));
}

#[test]
fn a_fault_bit_absent_from_the_baseline_aborts() {
    // Act
    let step = judge().observe(sample(
        250,
        Readings {
            status: StatusWord(0x8000),
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Abort(EnableAbort::NewFault));
}

#[test]
fn a_cml_bit_latched_before_the_enable_does_not_abort() {
    // Arrange
    let mut judge = EnableJudge::new(TARGET, StatusWord(OFF | 0x0002)).expect("judge");
    // Act
    let step = judge.observe(sample(
        250,
        Readings {
            status: StatusWord(0x0002),
            ..nominal()
        },
    ));
    // Assert
    assert_eq!(step, Step::Continue);
}

#[test]
fn a_baseline_with_another_fault_bit_refuses_the_enable() {
    // Act
    let refusals = [0x2000, 0x0004, 0x8002]
        .map(|status| EnableJudge::new(TARGET, StatusWord(OFF | status)).err());
    // Assert
    assert_eq!(refusals, [Some(EnableRefusal::BaselineFaults); 3]);
}

#[test]
fn a_target_outside_the_clamp_refuses_the_enable() {
    // Act
    let refusals =
        [0.99, 1.21, f64::NAN].map(|target| EnableJudge::new(target, StatusWord(OFF)).err());
    // Assert
    assert_eq!(refusals, [Some(EnableRefusal::TargetOutOfRange); 3]);
}

#[test]
fn the_summary_keeps_the_extremes() {
    // Arrange
    let mut judge = judge();
    // Act
    judge.observe(sample(
        250,
        Readings {
            vout_volts: 1.14,
            iout_amps: 0.2,
            ..nominal()
        },
    ));
    judge.observe(sample(
        500,
        Readings {
            vout_volts: 1.16,
            iout_amps: 0.9,
            vr_temp_c: 45.0,
            ..nominal()
        },
    ));
    // Assert
    let summary = judge.summary();
    assert_eq!(
        (
            summary.vout_mv_min,
            summary.vout_mv_max,
            summary.iout_ma_max,
            summary.vr_temp_c_max
        ),
        (1140, 1160, 900, 45)
    );
}

#[test]
fn a_baseline_without_the_output_off_refuses_the_enable() {
    // Act
    let refusal = EnableJudge::new(TARGET, StatusWord(0x0000)).err();
    // Assert
    assert_eq!(refusal, Some(EnableRefusal::BaselineOutputOn));
}

fn confirmed_off() -> ConfirmedOff {
    crate::tps546::confirm_off(StatusWord(OFF), 0.01)
        .confirmed()
        .expect("confirmed off")
}

#[test]
fn an_enable_is_armed_only_when_every_precondition_holds() {
    // Act
    let armed = arm(&judge(), Readback::Matched, confirmed_off(), true);
    // Assert
    assert_eq!(armed, Ok(EnableArmed(())));
}

#[test]
fn an_unmatched_readback_cannot_arm_the_enable() {
    // Act
    let armed = arm(&judge(), Readback::Unread, confirmed_off(), true);
    // Assert
    assert_eq!(armed, Err(EnableRefusal::ReadbackUnmatched));
}

#[test]
fn a_released_asic_reset_cannot_arm_the_enable() {
    // Act
    let armed = arm(&judge(), Readback::Matched, confirmed_off(), false);
    // Assert
    assert_eq!(armed, Err(EnableRefusal::ResetNotHeld));
}

#[test]
fn operation_on_is_the_only_output_on_write() {
    // Assert
    assert_eq!((CLEAR_FAULTS, OPERATION_ON), ([0x03], [0x01, 0x80]));
}

#[test]
fn a_production_enable_arms_from_a_clean_off_baseline() {
    // Act
    let armed = arm_production(
        1.15,
        StatusWord(OFF),
        Readback::Matched,
        confirmed_off(),
        true,
    );
    // Assert
    assert!(armed.is_ok(), "{armed:?}");
}

#[test]
fn a_production_enable_refuses_what_the_check_refuses() {
    // Act
    let refusals = [
        arm_production(
            1.25,
            StatusWord(OFF),
            Readback::Matched,
            confirmed_off(),
            true,
        ),
        arm_production(
            1.15,
            StatusWord(0),
            Readback::Matched,
            confirmed_off(),
            true,
        ),
        arm_production(
            1.15,
            StatusWord(OFF | 0x0010),
            Readback::Matched,
            confirmed_off(),
            true,
        ),
        arm_production(
            1.15,
            StatusWord(OFF),
            Readback::Unread,
            confirmed_off(),
            true,
        ),
        arm_production(
            1.15,
            StatusWord(OFF),
            Readback::Matched,
            confirmed_off(),
            false,
        ),
    ]
    .map(|armed| armed.err());
    // Assert
    assert_eq!(
        refusals,
        [
            Some(EnableRefusal::TargetOutOfRange),
            Some(EnableRefusal::BaselineOutputOn),
            Some(EnableRefusal::BaselineFaults),
            Some(EnableRefusal::ReadbackUnmatched),
            Some(EnableRefusal::ResetNotHeld),
        ]
    );
}

#[test]
fn a_settled_production_enable_reads_on_near_its_target_without_new_faults() {
    // Act
    let settled = judge_settled(
        1.15,
        StatusWord(OFF | 0x0002),
        0x80,
        StatusWord(0x0002),
        1.146,
    );
    // Assert
    assert_eq!(settled, Ok(()));
}

#[test]
fn an_unsettled_production_enable_names_why() {
    // Arrange
    let baseline = StatusWord(OFF);
    // Act
    let failures = [
        judge_settled(1.15, baseline, 0x00, StatusWord(0), 1.15),
        judge_settled(1.15, baseline, 0x80, StatusWord(OFF), 1.15),
        judge_settled(1.15, baseline, 0x80, StatusWord(0), f64::NAN),
        judge_settled(1.15, baseline, 0x80, StatusWord(0), 1.21),
        judge_settled(1.15, baseline, 0x80, StatusWord(0x0020), 1.15),
    ]
    .map(Result::err);
    // Assert
    assert_eq!(
        failures,
        [
            Some(SettleFailure::OperationNotOn),
            Some(SettleFailure::OutputOff),
            Some(SettleFailure::NonFiniteVout),
            Some(SettleFailure::VoutOutOfBand),
            Some(SettleFailure::NewFault),
        ]
    );
}
