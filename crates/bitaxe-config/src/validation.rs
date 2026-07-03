//! Pure boundary validation for config values.
//!
//! Reference: `reference/esp-miner/main/nvs_config.c`
//! Reference: `reference/esp-miner/main/http_server/http_server.c`
//!
//! This module returns inert domain values only. Hardware-sensitive values are
//! proved as configuration data here and remain effect-free.

use thiserror::Error;

use crate::{
    board_catalog, ultra_205_catalog_entry, NvsKeyName, NvsSchemaError, VerificationScope,
    NVS_KEY_NAME_MAX_BYTES,
};

/// Typed validation errors for raw config boundary values.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigValidationError {
    /// NVS key names must follow ESP-IDF key constraints.
    #[error("invalid NVS key name {value:?}; maximum length is {max_bytes} bytes")]
    InvalidNvsKeyName { value: String, max_bytes: usize },
    /// String-like values must fit upstream length bounds.
    #[error("{field} length {actual} is outside {min}..={max}")]
    InvalidLength {
        field: &'static str,
        min: usize,
        max: usize,
        actual: usize,
    },
    /// Numeric values must fit upstream range bounds.
    #[error("{field} value {actual} is outside {min}..={max}")]
    OutOfRange {
        field: &'static str,
        min: i64,
        max: i64,
        actual: i64,
    },
    /// Enumerated values must be one of the modeled upstream values.
    #[error("{field} has invalid enum value {value:?}")]
    InvalidEnum { field: &'static str, value: String },
    /// Only Ultra 205 is active hardware-verified scope in V1.
    #[error("board version {board_version:?} is not active hardware-verified scope")]
    InvalidBoardScope { board_version: String },
}

/// ASIC frequency in MHz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsicFrequencyMhz(u16);

impl AsicFrequencyMhz {
    /// Parses the schema-level frequency range.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u16_range("frequency", value, 1, 65_535).map(Self)
    }

    /// Parses the Ultra 205 BM1366 supported frequency options.
    pub fn ultra_205_bm1366(value: i64) -> Result<Self, ConfigValidationError> {
        let frequency = Self::parse(value)?;
        let options = ultra_205_catalog_entry().asic().frequency_options();

        if options.contains(&frequency.0) {
            return Ok(frequency);
        }

        Err(ConfigValidationError::InvalidEnum {
            field: "frequency",
            value: value.to_string(),
        })
    }

    /// Returns the frequency in MHz.
    #[must_use]
    pub const fn mhz(self) -> u16 {
        self.0
    }
}

/// ASIC core voltage in millivolts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreVoltageMv(u16);

impl CoreVoltageMv {
    /// Parses the schema-level voltage range.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u16_range("coreVoltage", value, 1, 65_535).map(Self)
    }

    /// Parses the Ultra 205 BM1366 supported voltage options.
    pub fn ultra_205_bm1366(value: i64) -> Result<Self, ConfigValidationError> {
        let voltage = Self::parse(value)?;
        let options = ultra_205_catalog_entry().asic().voltage_options();

        if options.contains(&voltage.0) {
            return Ok(voltage);
        }

        Err(ConfigValidationError::InvalidEnum {
            field: "coreVoltage",
            value: value.to_string(),
        })
    }

    /// Returns the voltage in millivolts.
    #[must_use]
    pub const fn millivolts(self) -> u16 {
        self.0
    }
}

/// Fan duty percentage for direct manual fan values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanDutyPercent(u8);

impl FanDutyPercent {
    /// Parses fan duty values from `0..=100`.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u8_range("manualFanSpeed", value, 0, 100).map(Self)
    }

    /// Returns the fan duty percentage.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.0
    }
}

/// Minimum fan duty percentage for automatic fan control floors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinFanDutyPercent(u8);

impl MinFanDutyPercent {
    /// Parses minimum fan duty values from `0..=99`.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u8_range("minFanSpeed", value, 0, 99).map(Self)
    }

    /// Returns the minimum fan duty percentage.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.0
    }
}

/// Temperature target in Celsius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemperatureCelsius(u8);

impl TemperatureCelsius {
    /// Parses `temptarget` values from `35..=66`.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u8_range("temptarget", value, 35, 66).map(Self)
    }

    /// Returns the temperature in Celsius.
    #[must_use]
    pub const fn celsius(self) -> u8 {
        self.0
    }
}

/// Network hostname.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hostname(String);

impl Hostname {
    /// Parses hostname values with upstream length bounds.
    pub fn parse(value: impl Into<String>) -> Result<Self, ConfigValidationError> {
        let value = value.into();
        validate_length("hostname", &value, 1, 32)?;
        Ok(Self(value))
    }

    /// Returns the hostname string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Wi-Fi station SSID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiSsid(String);

impl WifiSsid {
    /// Parses Wi-Fi SSIDs with upstream length bounds.
    pub fn parse(value: impl Into<String>) -> Result<Self, ConfigValidationError> {
        let value = value.into();
        validate_length("ssid", &value, 1, 32)?;
        Ok(Self(value))
    }

    /// Returns the SSID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Wi-Fi station password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiPassword(String);

impl WifiPassword {
    /// Parses Wi-Fi passwords with upstream length bounds.
    pub fn parse(value: impl Into<String>) -> Result<Self, ConfigValidationError> {
        let value = value.into();
        validate_length("wifiPass", &value, 0, 63)?;
        Ok(Self(value))
    }

    /// Returns the Wi-Fi password string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Network port number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortNumber(u16);

impl PortNumber {
    /// Parses port values from `0..=65535`.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u16_range("port", value, 0, 65_535).map(Self)
    }

    /// Returns the port as a `u16`.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Stratum TLS mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlsMode(u8);

impl TlsMode {
    /// Parses TLS mode values from `0..=3`.
    pub fn parse(value: i64) -> Result<Self, ConfigValidationError> {
        parse_u8_range("stratumTLS", value, 0, 3).map(Self)
    }

    /// Returns the TLS mode as a `u8`.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Boolean-like setting value accepted by upstream settings PATCH behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolLike(bool);

impl BoolLike {
    /// Parses a boolean value.
    #[must_use]
    pub const fn from_bool(value: bool) -> Self {
        Self(value)
    }

    /// Parses numeric `0` and `1` as boolean-like values.
    pub fn from_number(value: i64, field: &'static str) -> Result<Self, ConfigValidationError> {
        match value {
            0 => Ok(Self(false)),
            1 => Ok(Self(true)),
            actual => Err(ConfigValidationError::OutOfRange {
                field,
                min: 0,
                max: 1,
                actual,
            }),
        }
    }

    /// Returns the boolean value.
    #[must_use]
    pub const fn as_bool(self) -> bool {
        self.0
    }

    /// Returns the value in upstream NVS bool-as-u16 form.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        if self.0 {
            return 1;
        }

        0
    }
}

/// Stratum protocol setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StratumProtocol {
    /// Stratum v1.
    Sv1,
    /// Stratum v2.
    Sv2,
}

impl StratumProtocol {
    /// Parses upstream protocol names.
    pub fn parse(value: &str) -> Result<Self, ConfigValidationError> {
        match value {
            "SV1" => Ok(Self::Sv1),
            "SV2" => Ok(Self::Sv2),
            value => Err(ConfigValidationError::InvalidEnum {
                field: "stratumProtocol",
                value: value.to_owned(),
            }),
        }
    }

    /// Returns the upstream protocol string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sv1 => "SV1",
            Self::Sv2 => "SV2",
        }
    }
}

/// Stratum v2 channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sv2ChannelType {
    /// Standard SV2 channel.
    Standard,
    /// Extended SV2 channel.
    Extended,
}

impl Sv2ChannelType {
    /// Parses upstream SV2 channel type names.
    pub fn parse(value: &str) -> Result<Self, ConfigValidationError> {
        match value {
            "standard" => Ok(Self::Standard),
            "extended" => Ok(Self::Extended),
            value => Err(ConfigValidationError::InvalidEnum {
                field: "stratumV2ChannelType",
                value: value.to_owned(),
            }),
        }
    }

    /// Returns the upstream channel type string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Extended => "extended",
        }
    }
}

/// Board version present in the config catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardVersion(String);

impl BoardVersion {
    /// Parses a board version present in the catalog.
    pub fn parse(value: impl Into<String>) -> Result<Self, ConfigValidationError> {
        let value = value.into();

        if board_catalog()
            .iter()
            .any(|entry| entry.board_version() == value)
        {
            return Ok(Self(value));
        }

        Err(ConfigValidationError::InvalidEnum {
            field: "boardVersion",
            value,
        })
    }

    /// Parses a board version and requires active Ultra 205 evidence scope.
    pub fn active_hardware_verified(
        value: impl Into<String>,
    ) -> Result<Self, ConfigValidationError> {
        let board_version = Self::parse(value)?;
        let maybe_entry = board_catalog()
            .iter()
            .find(|entry| entry.board_version() == board_version.0);

        let Some(entry) = maybe_entry else {
            return Err(ConfigValidationError::InvalidEnum {
                field: "boardVersion",
                value: board_version.0,
            });
        };

        if entry.verification_scope() == VerificationScope::ActiveUltra205 {
            return Ok(board_version);
        }

        Err(ConfigValidationError::InvalidBoardScope {
            board_version: board_version.0,
        })
    }

    /// Returns the board version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validates an ESP-IDF NVS key name and returns the existing schema key type.
pub fn validate_nvs_key_name(value: &str) -> Result<NvsKeyName, ConfigValidationError> {
    NvsKeyName::parse(value).map_err(|err| match err {
        NvsSchemaError::EmptyKeyName
        | NvsSchemaError::NonAsciiKeyName { .. }
        | NvsSchemaError::KeyNameTooLong { .. } => ConfigValidationError::InvalidNvsKeyName {
            value: value.to_owned(),
            max_bytes: NVS_KEY_NAME_MAX_BYTES,
        },
        NvsSchemaError::EmptyRestFieldName => ConfigValidationError::InvalidNvsKeyName {
            value: value.to_owned(),
            max_bytes: NVS_KEY_NAME_MAX_BYTES,
        },
    })
}

fn parse_u16_range(
    field: &'static str,
    value: i64,
    min: i64,
    max: i64,
) -> Result<u16, ConfigValidationError> {
    if value < min || value > max {
        return Err(ConfigValidationError::OutOfRange {
            field,
            min,
            max,
            actual: value,
        });
    }

    Ok(value as u16)
}

fn parse_u8_range(
    field: &'static str,
    value: i64,
    min: i64,
    max: i64,
) -> Result<u8, ConfigValidationError> {
    if value < min || value > max {
        return Err(ConfigValidationError::OutOfRange {
            field,
            min,
            max,
            actual: value,
        });
    }

    Ok(value as u8)
}

fn validate_length(
    field: &'static str,
    value: &str,
    min: usize,
    max: usize,
) -> Result<(), ConfigValidationError> {
    let actual = value.len();
    if actual < min || actual > max {
        return Err(ConfigValidationError::InvalidLength {
            field,
            min,
            max,
            actual,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_nvs_key_name, AsicFrequencyMhz, BoardVersion, BoolLike, CoreVoltageMv,
        FanDutyPercent, Hostname, MinFanDutyPercent, PortNumber, StratumProtocol, Sv2ChannelType,
        TemperatureCelsius, TlsMode, WifiPassword, WifiSsid,
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
}
