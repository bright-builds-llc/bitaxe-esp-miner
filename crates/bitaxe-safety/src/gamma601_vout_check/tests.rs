use super::*;

#[test]
fn the_pending_record_reports_nothing_read() {
    // Act
    let marker = VoutCheckRecord::pending(1150).usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_vout_check schema=v1 result=pending reason=none target_mv=1150 vout_mode=unread readback=unread config_faults=unread config_cml=unread faults_before=unread faults_after=0x0000 samples=0 vout_mv_min=0 vout_mv_max=0 iout_ma_max=0 vr_temp_c_max=0 board_temp_c_max=0 vin_mv_min=0 vin_mv_max=0 off=unread stack_free=unread redacted=true"
    );
}

#[test]
fn a_passed_record_carries_its_window() {
    // Arrange
    let mut record = VoutCheckRecord::pending(1150);
    record.maybe_vout_mode = Some(0x97);
    record.readback = Readback::Matched;
    record.maybe_config_faults = Some(0x0002);
    record.maybe_config_cml = Some(0x82);
    record.maybe_faults_before = Some(0x0000);
    record.summary = EnableSummary {
        samples: 240,
        vout_mv_min: 1146,
        vout_mv_max: 1152,
        iout_ma_max: 310,
        vr_temp_c_max: 41,
        board_temp_c_max: 33,
        vin_mv_min: 5410,
        vin_mv_max: 5470,
        faults_after: 0,
    };
    record.maybe_off = Some(OffState::Confirmed);
    record.maybe_stack_free_bytes = Some(5120);
    record.conclude(CheckResult::Passed, "none");
    // Act
    let marker = record.usb_marker();
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_vout_check schema=v1 result=passed reason=none target_mv=1150 vout_mode=0x97 readback=match config_faults=0x0002 config_cml=0x82 faults_before=0x0000 faults_after=0x0000 samples=240 vout_mv_min=1146 vout_mv_max=1152 iout_ma_max=310 vr_temp_c_max=41 board_temp_c_max=33 vin_mv_min=5410 vin_mv_max=5470 off=confirmed stack_free=5120 redacted=true"
    );
}

#[test]
fn a_readback_mismatch_names_the_register_and_both_values() {
    // Arrange
    let mut record = VoutCheckRecord::pending(1150);
    record.readback = Readback::Mismatch {
        command: 0x40,
        wrote: 0x0280,
        read: 0x0281,
    };
    record.conclude(CheckResult::Refused, "config_readback_mismatch");
    // Act
    let marker = record.usb_marker();
    // Assert
    assert!(marker.contains(
        "result=refused reason=config_readback_mismatch target_mv=1150 vout_mode=unread readback=mismatch:0x40:0x0280:0x0281 "
    ));
}
