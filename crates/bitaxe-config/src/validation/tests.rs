use super::{
    validate_nvs_key_name, AsicFrequencyMhz, BoardVersion, BoolLike, CoreVoltageMv, FanDutyPercent,
    Hostname, MinFanDutyPercent, PortNumber, StratumProtocol, Sv2ChannelType, TemperatureCelsius,
    TlsMode, WifiPassword, WifiSsid,
};

#[test]
fn validation_accepts_ultra_205_frequency_and_voltage_options() {
    // Arrange
    let frequencies = [400, 425, 450, 475, 485, 500, 525, 550, 575];
    let voltages = [1100, 1150, 1200, 1250, 1300];

    // Act
    let parsed_frequencies = frequencies.map(AsicFrequencyMhz::ultra_205_bm1366);
    let parsed_voltages = voltages.map(CoreVoltageMv::ultra_205_bm1366);

    // Assert
    assert!(parsed_frequencies.iter().all(Result::is_ok));
    assert!(parsed_voltages.iter().all(Result::is_ok));
}

#[test]
fn validation_rejects_frequency_voltage_fan_temperature_bounds() {
    // Arrange
    let invalid_frequency = 0;
    let invalid_voltage = 0;
    let invalid_fan_duty = 101;
    let invalid_min_fan_duty = 100;
    let invalid_temperature = 67;
    let invalid_port = 65_536;

    // Act
    let frequency = AsicFrequencyMhz::parse(invalid_frequency);
    let voltage = CoreVoltageMv::parse(invalid_voltage);
    let fan_duty = FanDutyPercent::parse(invalid_fan_duty);
    let min_fan_duty = MinFanDutyPercent::parse(invalid_min_fan_duty);
    let temperature = TemperatureCelsius::parse(invalid_temperature);
    let port = PortNumber::parse(invalid_port);

    // Assert
    assert!(frequency.is_err());
    assert!(voltage.is_err());
    assert!(fan_duty.is_err());
    assert!(min_fan_duty.is_err());
    assert!(temperature.is_err());
    assert!(port.is_err());
}

#[test]
fn validation_rejects_invalid_text_and_protocol_values() {
    // Arrange
    let empty_hostname = "";
    let too_long_hostname = "123456789012345678901234567890123";
    let invalid_tls_mode = 4;
    let invalid_stratum_protocol = "SV3";
    let invalid_sv2_channel_type = "bad";

    // Act
    let empty_hostname = Hostname::parse(empty_hostname);
    let too_long_hostname = Hostname::parse(too_long_hostname);
    let tls_mode = TlsMode::parse(invalid_tls_mode);
    let stratum_protocol = StratumProtocol::parse(invalid_stratum_protocol);
    let sv2_channel_type = Sv2ChannelType::parse(invalid_sv2_channel_type);

    // Assert
    assert!(empty_hostname.is_err());
    assert!(too_long_hostname.is_err());
    assert!(tls_mode.is_err());
    assert!(stratum_protocol.is_err());
    assert!(sv2_channel_type.is_err());
}

#[test]
fn validation_accepts_wifi_station_credentials_at_bounds() {
    // Arrange
    let min_ssid = "a";
    let max_ssid_input = "s".repeat(32);
    let empty_password = "";
    let max_password_input = "p".repeat(63);

    // Act
    let min_ssid = WifiSsid::parse(min_ssid);
    let max_ssid = WifiSsid::parse(max_ssid_input.clone());
    let empty_password = WifiPassword::parse(empty_password);
    let max_password = WifiPassword::parse(max_password_input.clone());

    // Assert
    assert_eq!(min_ssid.expect("min ssid").as_str(), "a");
    assert_eq!(
        max_ssid.expect("max ssid").as_str(),
        max_ssid_input.as_str()
    );
    assert_eq!(empty_password.expect("empty password").as_str(), "");
    assert_eq!(
        max_password.expect("max password").as_str(),
        max_password_input.as_str()
    );
}

#[test]
fn validation_rejects_wifi_station_credentials_outside_bounds() {
    // Arrange
    let empty_ssid = "";
    let too_long_ssid = "s".repeat(33);
    let too_long_password = "p".repeat(64);

    // Act
    let empty_ssid = WifiSsid::parse(empty_ssid);
    let too_long_ssid = WifiSsid::parse(too_long_ssid);
    let too_long_password = WifiPassword::parse(too_long_password);

    // Assert
    assert!(empty_ssid.is_err());
    assert!(too_long_ssid.is_err());
    assert!(too_long_password.is_err());
}

#[test]
fn validation_rejects_invalid_bool_like_values() {
    // Arrange
    let false_number = 0;
    let true_number = 1;
    let invalid_number = 2;

    // Act
    let false_like = BoolLike::from_number(false_number, "autofanspeed");
    let true_like = BoolLike::from_number(true_number, "autofanspeed");
    let invalid_like = BoolLike::from_number(invalid_number, "autofanspeed");

    // Assert
    assert_eq!(false_like.map(BoolLike::as_bool), Ok(false));
    assert_eq!(true_like.map(BoolLike::as_bool), Ok(true));
    assert!(invalid_like.is_err());
}

#[test]
fn validation_rejects_invalid_nvs_key_names() {
    // Arrange
    let valid_frequency_key = "asicfrequency_f";
    let valid_fallback_sv2_key = "fbsv2authpubk";
    let too_long_key = "1234567890123456";

    // Act
    let frequency_key = validate_nvs_key_name(valid_frequency_key);
    let fallback_sv2_key = validate_nvs_key_name(valid_fallback_sv2_key);
    let too_long = validate_nvs_key_name(too_long_key);

    // Assert
    assert!(frequency_key.is_ok());
    assert!(fallback_sv2_key.is_ok());
    assert!(too_long.is_err());
}

#[test]
fn validation_rejects_non_205_active_board_scope() {
    // Arrange
    let gamma_601 = "601";

    // Act
    let active_scope = BoardVersion::active_hardware_verified(gamma_601);

    // Assert
    assert!(active_scope.is_err());
}
