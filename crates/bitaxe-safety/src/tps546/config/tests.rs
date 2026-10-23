use super::*;
use crate::tps546::{confirm_off, StatusWord};

const FIXTURE: &str = include_str!("../../../fixtures/gamma-601-tps546-config.csv");

fn fixture_rows() -> Vec<ConfigWrite> {
    FIXTURE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .skip(1)
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            let hex =
                |text: &str| u16::from_str_radix(text.trim_start_matches("0x"), 16).expect("hex");
            ConfigWrite {
                command: u8::try_from(hex(fields[0])).expect("command byte"),
                word: match fields[1] {
                    "word" => true,
                    "byte" => false,
                    other => panic!("unknown width {other}"),
                },
                value: hex(fields[3]),
                encoding: match fields[2] {
                    "raw" => Encoding::Raw,
                    "linear11" => Encoding::Linear11,
                    "vout" => Encoding::Vout,
                    other => panic!("unknown encoding {other}"),
                },
            }
        })
        .collect()
}

fn off() -> crate::tps546::ConfirmedOff {
    confirm_off(StatusWord(0x0040), 0.01)
        .confirmed()
        .expect("confirmed off")
}

fn linear11_write(value: u16) -> ConfigWrite {
    ConfigWrite {
        command: 0x46,
        word: true,
        value,
        encoding: Encoding::Linear11,
    }
}

#[test]
fn the_plan_matches_upstream_s_gamma_writes() {
    // Act
    let plan = gamma_config_plan(off(), EXPECTED_VOUT_MODE, 1.15).expect("plan");
    // Assert
    assert_eq!(plan.writes(), fixture_rows());
}

#[test]
fn linear11_matches_upstream_s_float_encoder() {
    // Act
    let words =
        [4.8, 4.5, 6.5, 0.25, 25.0, 30.0].map(|value| encode_linear11(value).expect("encoded"));
    // Assert
    assert_eq!(words, [0xca66, 0xca40, 0xcb40, 0xaa00, 0xdb20, 0xdbc0]);
}

#[test]
fn linear11_round_trips_within_one_step() {
    // Arrange
    let values = [4.8, 0.25, 25.0, 1023.0, 2047.0];
    // Act / Assert
    for value in values {
        let encoded = encode_linear11(value).expect("encoded");
        let decoded = super::super::decode_linear11(encoded);
        assert!(
            (decoded - value).abs() <= linear11_step(encoded),
            "{value} -> {decoded}"
        );
    }
}

#[test]
fn a_non_positive_or_overflowing_value_has_no_linear11_encoding() {
    // Act
    let encoded = [0.0, -1.0, f64::NAN, 2048.0].map(encode_linear11);
    // Assert
    assert_eq!(encoded, [None; 4]);
}

#[test]
fn integers_take_the_smallest_exponent_that_fits() {
    // Act
    let words =
        [650, 105, 145, 3, 0, 2048].map(|value| encode_linear11_integer(value).expect("encoded"));
    // Assert
    assert_eq!(words, [0x028a, 0x0069, 0x0091, 0x0003, 0x0000, 0x1200]);
}

#[test]
fn vout_truncates_under_the_mode_exponent() {
    // Act
    let raw = encode_vout(1.15, EXPECTED_VOUT_MODE).expect("encoded");
    // Assert
    assert_eq!(raw, 0x024c);
}

#[test]
fn another_vout_mode_refuses_the_plan() {
    // Act
    let refusals =
        [0x17, 0x16, 0x00].map(|mode| gamma_config_plan(off(), mode, 1.15).expect_err("mode"));
    // Assert
    assert_eq!(refusals, [ConfigRefusal::VoutModeUnexpected; 3]);
}

#[test]
fn a_target_outside_the_clamp_refuses_the_plan() {
    // Act
    let refusals = [0.99, 1.21, 1.25, f64::NAN]
        .map(|target| gamma_config_plan(off(), EXPECTED_VOUT_MODE, target));
    // Assert
    assert!(refusals
        .iter()
        .all(|refusal| *refusal == Err(ConfigRefusal::TargetOutOfRange)));
}

#[test]
fn the_plan_never_targets_an_nvm_command() {
    // Act
    let plan = gamma_config_plan(off(), EXPECTED_VOUT_MODE, 1.20).expect("plan");
    // Assert
    assert!(plan
        .writes()
        .iter()
        .all(|write| !is_nvm_command(write.command)));
}

#[test]
fn the_store_restore_and_vendor_ranges_are_nvm_commands() {
    // Act
    let flagged = [0x10, 0x11, 0x15, 0x16, 0x18, 0x19, 0xee, 0xf0, 0xff].map(is_nvm_command);
    // Assert
    assert_eq!(
        flagged,
        [false, true, true, true, true, false, false, true, true]
    );
}

#[test]
fn the_highest_target_is_commanded_as_encoded() {
    // Act
    let plan = gamma_config_plan(off(), EXPECTED_VOUT_MODE, TARGET_MAX_VOLTS).expect("plan");
    // Assert
    let command = plan
        .writes()
        .iter()
        .find(|write| write.command == 0x21)
        .expect("VOUT_COMMAND");
    assert_eq!(command.value, 0x0266);
}

#[test]
fn vout_max_holds_the_check_s_ceiling() {
    // Act
    let plan = gamma_config_plan(off(), EXPECTED_VOUT_MODE, 1.15).expect("plan");
    // Assert
    let maximum = plan
        .writes()
        .iter()
        .find(|write| write.command == 0x24)
        .expect("VOUT_MAX");
    assert_eq!(maximum.value, 0x0280);
}

#[test]
fn a_raw_byte_reads_back_exactly() {
    // Arrange
    let write = ConfigWrite {
        command: 0x02,
        word: false,
        value: 0x1b,
        encoding: Encoding::Raw,
    };
    // Act
    let matches = [0x1b, 0x1f].map(|read| readback_matches(&write, read, EXPECTED_VOUT_MODE));
    // Assert
    assert_eq!(matches, [true, false]);
}

#[test]
fn a_re_encoded_linear11_value_reads_back_as_a_match() {
    // Arrange: 25 A written as 800 x 2^-5; the controller may hold it as 400 x 2^-4.
    let write = linear11_write(0xdb20);
    let re_encoded = (0b11100 << 11) | 400;
    // Act
    let matches = readback_matches(&write, re_encoded, EXPECTED_VOUT_MODE);
    // Assert
    assert!(matches);
}

#[test]
fn a_materially_different_linear11_value_is_a_mismatch() {
    // Arrange: 30 A against 25 A, 64 A (0x3001), zero (0x7800) and an all-zero bus read.
    let write = linear11_write(encode_linear11(30.0).expect("30"));
    let reads = [0xdb20, 0x3001, 0x7800, 0x0000];
    // Act
    let matches = reads.map(|read| readback_matches(&write, read, EXPECTED_VOUT_MODE));
    // Assert
    assert_eq!(matches, [false; 4]);
}

#[test]
fn a_coarse_exponent_cannot_widen_the_tolerance() {
    // Arrange: 145 C, 6.5 V and 0.25 against reads whose own exponents would cover them.
    let cases = [
        (encode_linear11_integer(145).expect("145"), 0x4001),
        (encode_linear11(6.5).expect("6.5"), 0x7801),
        (encode_linear11(0.25).expect("0.25"), 0x0000),
    ];
    // Act
    let matches = cases
        .map(|(value, read)| readback_matches(&linear11_write(value), read, EXPECTED_VOUT_MODE));
    // Assert
    assert_eq!(matches, [false; 3]);
}

#[test]
fn a_written_zero_reads_back_only_as_zero() {
    // Arrange: TON_DELAY 0 against an all-ones bus read (-0.5) and a small positive value.
    let write = linear11_write(0x0000);
    // Act
    let matches =
        [0x0000, 0xffff, 0xf801].map(|read| readback_matches(&write, read, EXPECTED_VOUT_MODE));
    // Assert
    assert_eq!(matches, [true, false, false]);
}

#[test]
fn a_vout_value_reads_back_within_one_count() {
    // Arrange
    let write = ConfigWrite {
        command: 0x21,
        word: true,
        value: 0x024c,
        encoding: Encoding::Vout,
    };
    // Act
    let matches = [0x024d, 0x0250].map(|read| readback_matches(&write, read, EXPECTED_VOUT_MODE));
    // Assert
    assert_eq!(matches, [true, false]);
}

#[test]
fn the_plan_writes_exactly_the_allowed_commands_in_order() {
    // Act
    let plan = gamma_config_plan(off(), EXPECTED_VOUT_MODE, 1.15).expect("plan");
    // Assert
    let commands: Vec<u8> = plan.writes().iter().map(|write| write.command).collect();
    assert_eq!(commands, PLAN_COMMANDS);
}

#[test]
fn no_allowed_command_turns_the_output_on_or_touches_nvm() {
    // Act
    let forbidden = PLAN_COMMANDS
        .iter()
        .filter(|command| **command == 0x01 || **command == 0x03 || is_nvm_command(**command))
        .count();
    // Assert
    assert_eq!(forbidden, 0);
}

#[test]
fn the_catalog_core_voltage_ceiling_is_the_plan_s_clamp() {
    // Act
    let ceiling_mv = bitaxe_config::catalog::TPS546_CORE_VOLTAGE_CEILING_MV;
    // Assert
    assert_eq!(f64::from(ceiling_mv), TARGET_MAX_VOLTS * 1000.0);
}
