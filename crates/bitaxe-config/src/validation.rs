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
mod tests;
