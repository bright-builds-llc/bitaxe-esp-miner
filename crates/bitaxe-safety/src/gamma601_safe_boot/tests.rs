use super::*;

#[test]
fn the_initial_record_reports_nothing_read() {
    // Act
    let marker = SafeBootReport::initial(true).usb_marker("disconnected");
    // Assert
    assert_eq!(
        marker,
        "usb_gamma601_safe_boot schema=v2 reset_held=true tps546=unread off=unread faults=unavailable cml=unavailable start_status=unavailable start_cml=unavailable vin_mv=unavailable vout_mv=unavailable iout_ma=unavailable vr_temp_c=unavailable board_temp_c=unavailable fan_duty_pct=100 fan_rpm=unavailable wifi=disconnected redacted=true"
    );
}

#[test]
fn a_confirmed_off_record_carries_its_readings() {
    // Arrange
    let report = SafeBootReport {
        controller: Controller::Found(DeviceModel::D24a),
        maybe_off: Some(OffState::Confirmed),
        maybe_faults: Some(0x0002),
        maybe_cml: Some(0x80),
        maybe_start_status: Some(0x0842),
        maybe_start_cml: Some(0x02),
        maybe_vin_volts: Some(5.02),
        maybe_vout_volts: Some(0.0),
        maybe_iout_amps: Some(0.0),
        maybe_vr_temperature_celsius: Some(31.4),
        maybe_board_temperature_celsius: Some(29.0),
        fan_duty_percent: FAN_DUTY_PERCENT,
        maybe_fan_rpm: Some(2400),
        ..SafeBootReport::initial(true)
    };
    // Act
    let marker = report.usb_marker("connected");
    // Assert
    assert!(report.off_confirmed());
    assert_eq!(
        marker,
        "usb_gamma601_safe_boot schema=v2 reset_held=true tps546=d24a off=confirmed faults=0x0002 cml=0x80 start_status=0x0842 start_cml=0x02 vin_mv=5020 vout_mv=0 iout_ma=0 vr_temp_c=31 board_temp_c=29 fan_duty_pct=30 fan_rpm=2400 wifi=connected redacted=true"
    );
}

#[test]
fn a_non_finite_reading_is_unavailable() {
    // Arrange
    let report = SafeBootReport {
        maybe_vout_volts: Some(f64::NAN),
        ..SafeBootReport::initial(true)
    };
    // Act
    let marker = report.usb_marker("disconnected");
    // Assert
    assert!(marker.contains(" vout_mv=unavailable "));
}

#[test]
fn a_wifi_status_that_is_not_a_closed_token_is_not_echoed() {
    // Arrange
    let report = SafeBootReport::initial(true);
    // Act
    let markers = [
        report.usb_marker("disconnected_auth_failed"),
        report.usb_marker("My Home Network"),
        report.usb_marker("192.168.1.20"),
        report.usb_marker(""),
    ];
    // Assert
    assert!(markers[0].contains(" wifi=disconnected_auth_failed "));
    assert!(markers[1..]
        .iter()
        .all(|marker| marker.contains(" wifi=other ")));
}

#[test]
fn an_unrecognised_controller_is_reported_as_such() {
    // Act
    let label = Controller::Unrecognised.label();
    // Assert
    assert_eq!(label, "unrecognised");
}

#[test]
fn the_fan_runs_full_until_the_output_is_confirmed_off() {
    // Arrange
    let unconfirmed = SafeBootReport {
        maybe_off: Some(OffState::VoltagePresent),
        ..SafeBootReport::initial(true)
    };
    let confirmed = SafeBootReport {
        maybe_off: Some(OffState::Confirmed),
        ..SafeBootReport::initial(true)
    };
    // Act
    let duties = [
        unconfirmed.fan_duty_for_state(),
        confirmed.fan_duty_for_state(),
    ];
    // Assert
    assert_eq!(duties, [100, 30]);
}
