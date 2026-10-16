use super::*;
use crate::{system_info_from_snapshot, ApiSnapshot};

fn info() -> SystemInfoWire {
    system_info_from_snapshot(&ApiSnapshot::safe_ultra_205())
}

fn values(parameter: BapParameter, info: &SystemInfoWire) -> Vec<(&'static str, String)> {
    bap_subscription_values(parameter, info)
}

#[test]
fn device_model_is_the_board_family_not_the_board_version() {
    // Arrange
    let mut info = info();
    info.board_version = "205".to_owned();
    // Act
    let snapshot = bap_request_snapshot(info);
    // Assert: reference `DEVICE_CONFIG.family.name` for board 205.
    assert_eq!(snapshot.device_model, "Ultra");
}

#[test]
fn an_unknown_board_has_no_family() {
    // Arrange / Act
    let family = board_family("999");
    // Assert
    assert_eq!(family, "");
}

#[test]
fn suffix_string_matches_the_reference_cases() {
    // Arrange: integer division to three significant thousands, then `%.2f`.
    let cases = [
        (0, "0"),
        (999, "999"),
        (1_000, "1.00k"),
        (1_999, "2.00k"),
        (1_234_567, "1.23M"),
        (4_294_967_295, "4.29G"),
        (12_345_678_901_234, "12.35T"),
        (u64::MAX, "18.45E"),
    ];
    // Act
    let rendered: Vec<String> = cases
        .iter()
        .map(|(value, _)| suffix_string(*value))
        .collect();
    // Assert
    let expected: Vec<&str> = cases.iter().map(|(_, text)| *text).collect();
    assert_eq!(rendered, expected);
}

#[test]
fn power_module_values_use_reference_units_and_formats() {
    // Arrange
    let mut info = info();
    info.temp = 53.25;
    info.vr_temp = 47.5;
    info.power = 11.7;
    info.voltage_millivolts = 5_266.25;
    info.current_milliamps = 2_221.5;
    // Act
    let rendered = [
        values(BapParameter::Temperature, &info),
        values(BapParameter::Power, &info),
        values(BapParameter::Voltage, &info),
        values(BapParameter::Current, &info),
    ]
    .concat();
    // Assert: `%f` temperatures, `%.2f` power in W, voltage in mV, current in mA.
    assert_eq!(
        rendered,
        vec![
            ("chipTemp", "53.250000".to_owned()),
            ("vrTemp", "47.500000".to_owned()),
            ("power", "11.70".to_owned()),
            ("voltage", "5266.25".to_owned()),
            ("current", "2221.50".to_owned()),
        ]
    );
}

#[test]
fn fan_speed_is_rpm_and_best_difficulty_is_suffixed() {
    // Arrange
    let mut info = info();
    info.fan_rpm = 7_307;
    info.fan_speed = 100;
    info.best_diff = 1_234_567.9;
    // Act
    let rendered = [
        values(BapParameter::FanSpeed, &info),
        values(BapParameter::BestDifficulty, &info),
    ]
    .concat();
    // Assert
    assert_eq!(
        rendered,
        vec![
            ("fan_speed", "7307".to_owned()),
            ("best_difficulty", "1.23M".to_owned()),
        ]
    );
}

#[test]
fn hashrate_keeps_two_decimals() {
    // Arrange
    let mut info = info();
    info.hash_rate = 550.6449;
    // Act
    let rendered = values(BapParameter::Hashrate, &info);
    // Assert
    assert_eq!(rendered, vec![("hashrate", "550.64".to_owned())]);
}

#[test]
fn the_wifi_subscription_never_sends_the_password() {
    // Arrange / Act
    let tokens: Vec<&str> = values(BapParameter::Wifi, &info())
        .into_iter()
        .map(|(token, _)| token)
        .collect();
    // Assert: owner-accepted divergence from the reference `wifi_password` frame.
    assert_eq!(tokens, vec!["wifi_ssid", "wifi_rssi", "wifi_ip"]);
}

#[test]
fn the_wifi_subscription_reports_the_rssi_while_connected() {
    // Arrange
    let mut info = info();
    info.wifi_status = "connected".to_owned();
    info.wifi_rssi = -61;
    // Act
    let rendered = values(BapParameter::Wifi, &info);
    // Assert
    assert_eq!(rendered[1], ("wifi_rssi", "-61".to_owned()));
}

#[test]
fn the_wifi_subscription_reports_minus_128_while_disconnected() {
    // Arrange: system info keeps its -90 default; the reference BAP sends -128.
    let mut info = info();
    info.wifi_status = "disconnected_beacon_timeout".to_owned();
    info.wifi_rssi = -90;
    // Act
    let rendered = values(BapParameter::Wifi, &info);
    // Assert
    assert_eq!(rendered[1], ("wifi_rssi", "-128".to_owned()));
}

#[test]
fn best_difficulty_publishes_the_all_time_nonce_best() {
    // Arrange: upstream `best_diff_string` is the all-time best, not the session's.
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.best_difficulty = crate::BestDifficultyState {
        all_time: 1_234_567,
        session: 999,
    };
    let info = system_info_from_snapshot(&snapshot);
    // Act
    let rendered = values(BapParameter::BestDifficulty, &info);
    // Assert
    assert_eq!(rendered, vec![("best_difficulty", "1.23M".to_owned())]);
}
