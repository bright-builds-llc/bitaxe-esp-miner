use super::*;

#[test]
fn a_pending_record_reads_unread_everywhere() {
    // Act
    let marker = ChipCheckRecord::pending(1150).usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_chip_check schema=v1 result=pending reason=none target_mv=1150 chips=unread blocker=none diode_before_c=unread diode_after_c=unread vout_mv=unread iout_ma_max=0 vr_temp_c_max=0 on_ms=unread off=unread stack_free=unread redacted=true"
    );
}

#[test]
fn a_passed_record_carries_its_readings() {
    // Arrange
    let mut record = ChipCheckRecord::pending(1150);
    record.chips = ChipCount::Counted(1);
    record.maybe_diode_before_c = Some(24.25);
    record.maybe_diode_after_c = Some(27.0);
    record.maybe_vout_mv = Some(1146);
    record.observe_on(Some(0.71), Some(36.4));
    record.observe_on(Some(2.5), Some(35.0));
    record.maybe_on_ms = Some(2380);
    record.maybe_off = Some(OffState::Confirmed);
    record.maybe_stack_free_bytes = Some(5120);
    record.conclude(CheckResult::Passed, "none");
    // Act
    let marker = record.usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_chip_check schema=v1 result=passed reason=none target_mv=1150 chips=1 blocker=none diode_before_c=24.2 diode_after_c=27.0 vout_mv=1146 iout_ma_max=2500 vr_temp_c_max=36 on_ms=2380 off=confirmed stack_free=5120 redacted=true"
    );
}

#[test]
fn readings_that_are_negative_or_not_finite_never_raise_the_extremes() {
    // Arrange
    let mut record = ChipCheckRecord::pending(1150);
    // Act
    record.observe_on(Some(-0.2), Some(f64::NAN));
    record.observe_on(Some(f64::INFINITY), None);
    // Assert
    assert_eq!((record.iout_ma_max, record.vr_temp_c_max), (0, 0));
}

#[test]
fn an_incomplete_count_reads_as_such() {
    // Arrange
    let mut record = ChipCheckRecord::pending(1150);
    record.chips = ChipCount::Incomplete;
    record.maybe_blocker = Some("uart_failed");
    record.conclude(CheckResult::Aborted, "chip_count_incomplete");
    // Act
    let marker = record.usb_marker();
    // Assert
    assert!(marker.contains("result=aborted reason=chip_count_incomplete"));
    assert!(marker.contains("chips=incomplete blocker=uart_failed"));
}

#[test]
fn the_count_may_start_up_to_five_seconds_after_the_power_on_sequence() {
    // Act
    let late = [5_000, 5_001, 0].map(|elapsed| detect_start_late(7_000, 7_000 + elapsed));
    // Assert
    assert_eq!(late, [false, true, false]);
}
