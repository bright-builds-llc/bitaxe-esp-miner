use super::*;

#[test]
fn the_off_sequence_turns_the_output_off_before_requiring_the_pin() {
    // Act
    let writes = OFF_SEQUENCE.map(OffWrite::bytes);
    // Assert
    assert_eq!(writes, [[0x01, 0x00], [0x02, 0x1f]]);
}

#[test]
fn no_off_write_targets_a_voltage_or_store_command() {
    // Arrange: VOUT_COMMAND 0x21, STORE_USER_ALL 0x15, STORE_DEFAULT_ALL 0x11.
    let forbidden = [0x21_u8, 0x15, 0x11];
    // Act
    let commands = OFF_SEQUENCE.map(|write| write.bytes()[0]);
    // Assert
    assert!(commands.iter().all(|command| !forbidden.contains(command)));
}

#[test]
fn no_off_write_turns_the_output_on() {
    // Act
    let operation = OffWrite::OperationOff.bytes();
    // Assert
    assert_eq!(operation[1] & 0x80, 0);
}

#[test]
fn both_gamma_controller_variants_are_recognised_after_the_length_byte() {
    // Arrange
    let d24a = [6, 0x54, 0x49, 0x54, 0x6d, 0x24, 0x41];
    let d24s = [6, 0x54, 0x49, 0x54, 0x6d, 0x24, 0x62];
    // Act
    let models = [classify_device_id(&d24a), classify_device_id(&d24s)];
    // Assert
    assert_eq!(models, [Some(DeviceModel::D24a), Some(DeviceModel::D24s)]);
}

#[test]
fn an_unknown_device_is_not_a_tps546() {
    // Act
    let model = classify_device_id(&[6, 0x54, 0x49, 0x54, 0x6d, 0x24, 0x00]);
    // Assert
    assert_eq!(model, None);
}

#[test]
fn linear11_decodes_a_negative_exponent() {
    // Arrange: exponent -2 (0b11110), mantissa 20 → 5.0.
    let raw = (0b11110 << 11) | 20;
    // Act
    let value = decode_linear11(raw);
    // Assert
    assert!((value - 5.0).abs() < f64::EPSILON);
}

#[test]
fn linear11_decodes_a_negative_mantissa() {
    // Arrange: exponent 0, mantissa -3.
    let raw = 0x07ff - 2;
    // Act
    let value = decode_linear11(raw);
    // Assert
    assert!((value + 3.0).abs() < f64::EPSILON);
}

#[test]
fn vout_scales_by_the_vout_mode_exponent() {
    // Arrange: VOUT_MODE exponent -9 (0x17), 0x0200 = 512 → 1.0 V.
    let (raw, mode) = (0x0200, 0x17);
    // Act
    let volts = decode_vout(raw, mode);
    // Assert
    assert_eq!(volts, Some(1.0));
}

#[test]
fn a_non_linear_vout_mode_is_not_decoded() {
    // Act
    let volts = decode_vout(0x0200, 0x40 | 0x17);
    // Assert
    assert_eq!(volts, None);
}

#[test]
fn the_relative_flag_does_not_block_decoding() {
    // Act
    let volts = decode_vout(0x0200, 0x80 | 0x17);
    // Assert
    assert_eq!(volts, Some(1.0));
}

#[test]
fn the_deliberate_off_bit_is_not_a_fault() {
    // Arrange: OFF and POWER_GOOD# set, nothing else.
    let status = StatusWord(0x0840);
    // Act
    let faults = status.faults();
    // Assert
    assert!(status.output_off());
    assert_eq!(faults, 0);
}

#[test]
fn an_overcurrent_is_a_fault() {
    // Act
    let faults = StatusWord(0x4050).faults();
    // Assert
    assert_eq!(faults, 0x4010);
}

#[test]
fn off_is_confirmed_only_with_the_bit_and_no_voltage() {
    // Act
    let states = [
        confirm_off(StatusWord(0x0040), 0.01),
        confirm_off(StatusWord(0x0000), 0.01),
        confirm_off(StatusWord(0x0040), 1.15),
        confirm_off(StatusWord(0x0040), f64::NAN),
    ];
    // Assert
    assert_eq!(
        states,
        [
            OffState::Confirmed,
            OffState::ReportsOn,
            OffState::VoltagePresent,
            OffState::VoltagePresent,
        ]
    );
}
