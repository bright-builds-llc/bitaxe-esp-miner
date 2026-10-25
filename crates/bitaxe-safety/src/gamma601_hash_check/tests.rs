use super::*;
use crate::tps546::StatusWord;

fn readings() -> HashReadings {
    HashReadings {
        vout_volts: 1.146,
        vin_volts: 5.40,
        iout_amps: 4.0,
        vr_temp_c: 40.0,
        maybe_diode_c: Some(45.0),
        maybe_fan_rpm: Some(6000),
        status: StatusWord(0),
    }
}

fn judge() -> HashWindowJudge {
    let mut judge = HashWindowJudge::new(1.15, 22.0, StatusWord(0));
    judge.require_diode(0);
    judge
}

fn sample(elapsed_ms: u32, readings: HashReadings) -> HashSample {
    HashSample {
        elapsed_ms,
        maybe_readings: Some(readings),
    }
}

fn verified(nonce: u32, meets_ticket: bool) -> VerifiedResult {
    VerifiedResult {
        nonce,
        version_bits: 0,
        difficulty: 300.0,
        meets_ticket,
        genesis: false,
    }
}

fn tally(verified_count: u32, invalid_count: u32) -> HashTally {
    let mut tally = HashTally::default();
    for nonce in 0..verified_count {
        tally.observe(verified(nonce, true));
    }
    for nonce in 0..invalid_count {
        tally.observe(verified(1_000 + nonce, false));
    }
    tally
}

#[test]
fn the_caps_allow_exactly_their_length() {
    // Act
    let late = [1_000, 1_001].map(|elapsed| late(2_000, 2_000 + elapsed, INIT_START_CAP_MS));
    // Assert
    assert_eq!(late, [false, true]);
}

#[test]
fn the_diode_gate_rejects_the_f31_reading() {
    // Act
    let gated = diode_gate(Some(127.75), Some(127.75));
    // Assert
    assert_eq!(gated, None);
}

#[test]
fn the_diode_gate_rejects_a_reading_below_the_range() {
    // Act
    let gated = diode_gate(Some(9.9), Some(10.2));
    // Assert
    assert_eq!(gated, None);
}

#[test]
fn the_diode_gate_rejects_a_failed_read() {
    // Act
    let gated = diode_gate(Some(30.0), None);
    // Assert
    assert_eq!(gated, None);
}

#[test]
fn the_diode_gate_rejects_readings_that_disagree() {
    // Act
    let gated = diode_gate(Some(30.0), Some(31.5));
    // Assert
    assert_eq!(gated, None);
}

#[test]
fn the_diode_gate_passes_the_higher_of_two_plausible_readings() {
    // Act
    let gated = diode_gate(Some(30.0), Some(30.5));
    // Assert
    assert_eq!(gated, Some(30.5));
}

#[test]
fn the_pass_rule_holds_at_its_boundaries() {
    // Act
    let passed = [(3, 0), (2, 0), (9, 2), (10, 1)].map(|(v, i)| tally(v, i).passed());
    // Assert
    assert_eq!(passed, [true, false, false, true]);
}

#[test]
fn a_repeated_result_counts_as_a_duplicate_only() {
    // Arrange
    let mut tally = HashTally::default();
    tally.observe(verified(7, true));
    // Act
    tally.observe(verified(7, true));
    // Assert
    assert_eq!((tally.verified, tally.duplicate), (1, 1));
}

#[test]
fn the_genesis_answer_is_recorded_when_it_verifies() {
    // Arrange
    let mut tally = HashTally::default();
    // Act
    tally.observe(VerifiedResult {
        genesis: true,
        ..verified(0x7c2b_ac1d, true)
    });
    // Assert
    assert!(tally.genesis_hit);
}

#[test]
fn a_healthy_sample_continues() {
    // Act
    let stop = judge().observe(sample(250, readings()));
    // Assert
    assert_eq!(stop, None);
}

#[test]
fn each_limit_stops_with_its_label() {
    // Arrange
    let cases: [(HashReadings, &str); 9] = [
        (
            HashReadings {
                status: StatusWord(0x0010),
                ..readings()
            },
            "new_fault",
        ),
        (
            HashReadings {
                vout_volts: 1.09,
                ..readings()
            },
            "vout_out_of_band",
        ),
        (
            HashReadings {
                vin_volts: 4.5,
                ..readings()
            },
            "vin_low",
        ),
        (
            HashReadings {
                iout_amps: 19.5,
                ..readings()
            },
            "over_power",
        ),
        (
            HashReadings {
                vout_volts: 1.10,
                iout_amps: 20.5,
                ..readings()
            },
            "over_power",
        ),
        (
            HashReadings {
                vr_temp_c: 91.0,
                ..readings()
            },
            "vr_over_temperature",
        ),
        (
            HashReadings {
                maybe_fan_rpm: Some(0),
                ..readings()
            },
            "fan_stopped",
        ),
        (
            HashReadings {
                maybe_diode_c: Some(65.0),
                ..readings()
            },
            "diode_over_temperature",
        ),
        (
            HashReadings {
                maybe_diode_c: Some(70.0),
                ..readings()
            },
            "diode_over_temperature",
        ),
    ];
    // Act
    let labels =
        cases.map(|(readings, _)| judge().observe(sample(250, readings)).map(HashStop::label));
    // Assert
    assert_eq!(labels, cases.map(|(_, label)| Some(label)));
}

#[test]
fn overcurrent_stops_below_the_power_limit() {
    // Arrange
    let mut judge = HashWindowJudge::new(1.15, 40.0, StatusWord(0));
    // Act
    let stop = judge.observe(sample(
        250,
        HashReadings {
            iout_amps: 20.5,
            ..readings()
        },
    ));
    // Assert
    assert_eq!(stop, Some(HashStop::Overcurrent));
}

#[test]
fn the_diode_counts_only_once_required() {
    // Arrange
    let mut judge = HashWindowJudge::new(1.15, 22.0, StatusWord(0));
    // Act
    let stop = judge.observe(sample(
        250,
        HashReadings {
            maybe_diode_c: Some(127.75),
            ..readings()
        },
    ));
    // Assert
    assert_eq!(stop, None);
}

#[test]
fn a_single_failed_read_is_skipped() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, readings()));
    // Act
    let stop = judge.observe(HashSample {
        elapsed_ms: 500,
        maybe_readings: None,
    });
    // Assert
    assert_eq!(stop, None);
}

#[test]
fn failed_reads_past_the_gap_stop() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, readings()));
    // Act
    let stop = judge.observe(HashSample {
        elapsed_ms: 1_751,
        maybe_readings: None,
    });
    // Assert
    assert_eq!(stop, Some(HashStop::MissedSample));
}

#[test]
fn a_missing_diode_past_the_gap_stops() {
    // Arrange
    let mut judge = judge();
    let no_diode = HashReadings {
        maybe_diode_c: None,
        ..readings()
    };
    judge.observe(sample(1_000, no_diode));
    // Act
    let stop = judge.observe(sample(1_501, no_diode));
    // Assert
    assert_eq!(stop, Some(HashStop::DiodeLost));
}

#[test]
fn a_missing_fan_reading_past_the_gap_stops() {
    // Arrange
    let mut judge = judge();
    let no_fan = HashReadings {
        maybe_fan_rpm: None,
        ..readings()
    };
    judge.observe(sample(1_000, no_fan));
    // Act
    let stop = judge.observe(sample(1_501, no_fan));
    // Assert
    assert_eq!(stop, Some(HashStop::FanStopped));
}

#[test]
fn a_clock_that_does_not_advance_stops() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, readings()));
    // Act
    let stop = judge.observe(sample(250, readings()));
    // Assert
    assert_eq!(stop, Some(HashStop::ClockNotAdvancing));
}

#[test]
fn a_stop_is_final() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(
        250,
        HashReadings {
            vin_volts: 4.0,
            ..readings()
        },
    ));
    // Act
    let stop = judge.observe(sample(500, readings()));
    // Assert
    assert_eq!(stop, Some(HashStop::VinLow));
}

#[test]
fn the_extremes_fold_every_accepted_sample() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, readings()));
    // Act
    judge.observe(sample(
        500,
        HashReadings {
            vin_volts: 5.2,
            iout_amps: 5.0,
            ..readings()
        },
    ));
    // Assert
    let extremes = judge.extremes();
    assert_eq!(
        (
            extremes.maybe_vin_mv_min,
            extremes.iout_ma_max,
            extremes.power_mw_max
        ),
        (Some(5200), 5000, 5730)
    );
}

#[test]
fn a_pending_record_reads_unread_everywhere() {
    // Act
    let marker = HashCheckRecord::pending().usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_hash_check schema=v1 result=pending reason=none target_mv=1150 freq_mhz=300 chips=unread blocker=none diode_before_c=unread diode_gate_c=unread diode_max_c=unread vout_mv=unread vin_mv_min=unread iout_ma_max=0 power_mw_max=0 vr_temp_c_max=0 verified=0 invalid=0 duplicate=0 discarded=0 best_difficulty=0 genesis_hit=false first_result_ms=unread window_ms=unread on_ms=unread off=unread stack_free=unread redacted=true"
    );
}

#[test]
fn a_passed_record_carries_its_readings() {
    // Arrange
    let mut record = HashCheckRecord::pending();
    record.power.chips = ChipCount::Counted(1);
    record.power.maybe_diode_before_c = Some(127.75);
    record.power.maybe_vout_mv = Some(1146);
    record.power.observe_on(Some(1.6), Some(33.0));
    record.power.maybe_on_ms = Some(24_310);
    record.power.maybe_off = Some(OffState::Confirmed);
    record.power.maybe_stack_free_bytes = Some(4096);
    record.power.conclude(CheckResult::Passed, "none");
    record.maybe_diode_gate_c = Some(41.25);
    record.extremes = HashExtremes {
        maybe_vin_mv_min: Some(5380),
        iout_ma_max: 4200,
        power_mw_max: 4813,
        vr_temp_c_max: 38,
        maybe_diode_max_c: Some(47.5),
    };
    record.tally = tally(8, 0);
    record.tally.best_difficulty = 1234.9;
    record.maybe_first_result_ms = Some(1840);
    record.maybe_window_ms = Some(14_250);
    // Act
    let marker = record.usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_hash_check schema=v1 result=passed reason=none target_mv=1150 freq_mhz=300 chips=1 blocker=none diode_before_c=127.8 diode_gate_c=41.2 diode_max_c=47.5 vout_mv=1146 vin_mv_min=5380 iout_ma_max=4200 power_mw_max=4813 vr_temp_c_max=38 verified=8 invalid=0 duplicate=0 discarded=0 best_difficulty=1234 genesis_hit=false first_result_ms=1840 window_ms=14250 on_ms=24310 off=confirmed stack_free=4096 redacted=true"
    );
}

#[test]
fn a_new_communication_bit_alone_continues() {
    // Act
    let stop = judge().observe(sample(
        250,
        HashReadings {
            status: StatusWord(0x0002),
            ..readings()
        },
    ));
    // Assert
    assert_eq!(stop, None);
}

#[test]
fn a_gap_longer_than_allowed_stops_even_with_a_good_reading() {
    // Arrange
    let mut judge = judge();
    judge.observe(sample(250, readings()));
    // Act
    let stop = judge.observe(sample(1_751, readings()));
    // Assert
    assert_eq!(stop, Some(HashStop::MissedSample));
}
