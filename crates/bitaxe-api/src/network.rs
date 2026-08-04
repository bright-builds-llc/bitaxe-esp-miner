//! Pure Wi-Fi scan and IPv6 wire contracts.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/connect/connect.c:wifi_scan`
//! - `reference/esp-miner/components/connect/connect.c:IP_EVENT_GOT_IP6`
//! - `reference/esp-miner/main/http_server/http_server.c:GET_wifi_scan`

use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum number of visible networks returned by the upstream scan route.
pub const MAX_WIFI_SCAN_NETWORKS: usize = 20;

/// ESP-IDF-compatible authentication mode exposed by the AxeOS scan route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiScanAuthMode {
    Open,
    Wep,
    WpaPsk,
    Wpa2Psk,
    WpaWpa2Psk,
    Wpa2Enterprise,
    Wpa3Psk,
    Wpa2Wpa3Psk,
    WapiPsk,
    Unknown,
}

impl WifiScanAuthMode {
    /// Returns the numeric `wifi_auth_mode_t` value used by upstream JSON.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Open => 0,
            Self::Wep => 1,
            Self::WpaPsk => 2,
            Self::Wpa2Psk => 3,
            Self::WpaWpa2Psk => 4,
            Self::Wpa2Enterprise => 5,
            Self::Wpa3Psk => 6,
            Self::Wpa2Wpa3Psk => 7,
            Self::WapiPsk => 8,
            Self::Unknown => 9,
        }
    }
}

/// One visible network in the `/api/system/wifi/scan` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WifiNetworkWire {
    pub ssid: String,
    pub rssi: i8,
    pub authmode: u8,
}

impl WifiNetworkWire {
    /// Creates one bounded scan result without retaining BSSID or channel data.
    #[must_use]
    pub fn new(ssid: String, rssi: i8, auth_mode: WifiScanAuthMode) -> Self {
        Self {
            ssid,
            rssi,
            authmode: auth_mode.code(),
        }
    }
}

/// Successful AxeOS Wi-Fi scan response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WifiScanResponse {
    pub networks: Vec<WifiNetworkWire>,
}

impl WifiScanResponse {
    /// Accepts a scan only when it fits the upstream fixed 20-record buffer.
    pub fn try_new(networks: Vec<WifiNetworkWire>) -> Result<Self, WifiScanResponseError> {
        if networks.len() > MAX_WIFI_SCAN_NETWORKS {
            return Err(WifiScanResponseError::TooManyNetworks);
        }

        Ok(Self { networks })
    }
}

/// Closed construction failure for the public scan response.
#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum WifiScanResponseError {
    #[error("Wi-Fi scan exceeded the response limit")]
    TooManyNetworks,
}

/// Projects an IPv6 observation using upstream link-local zone behavior.
#[must_use]
pub fn project_ipv6_address(address: Ipv6Addr, maybe_interface_index: Option<u32>) -> String {
    if !is_link_local(address) {
        return address.to_string();
    }

    match maybe_interface_index {
        Some(interface_index) if interface_index > 0 => format!("{address}%{interface_index}"),
        _ => address.to_string(),
    }
}

const fn is_link_local(address: Ipv6Addr) -> bool {
    address.segments()[0] & 0xffc0 == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(index: usize) -> WifiNetworkWire {
        WifiNetworkWire::new(
            format!("test-network-{index}"),
            -42,
            WifiScanAuthMode::Wpa2Psk,
        )
    }

    #[test]
    fn scan_response_serializes_exact_upstream_shape_and_auth_codes() {
        // Arrange
        let response = WifiScanResponse::try_new(vec![
            WifiNetworkWire::new("open-test".to_owned(), -31, WifiScanAuthMode::Open),
            WifiNetworkWire::new("secure-test".to_owned(), -52, WifiScanAuthMode::Wpa3Psk),
        ])
        .expect("bounded response");

        // Act
        let value = serde_json::to_value(response).expect("serializable response");

        // Assert
        assert_eq!(
            value,
            serde_json::json!({
                "networks": [
                    {"ssid": "open-test", "rssi": -31, "authmode": 0},
                    {"ssid": "secure-test", "rssi": -52, "authmode": 6}
                ]
            })
        );
    }

    #[test]
    fn scan_response_accepts_twenty_and_rejects_twenty_one() {
        // Arrange
        let twenty = (0..MAX_WIFI_SCAN_NETWORKS).map(network).collect();
        let twenty_one = (0..=MAX_WIFI_SCAN_NETWORKS).map(network).collect();

        // Act
        let accepted = WifiScanResponse::try_new(twenty);
        let rejected = WifiScanResponse::try_new(twenty_one);

        // Assert
        assert_eq!(accepted.expect("twenty results").networks.len(), 20);
        assert_eq!(rejected, Err(WifiScanResponseError::TooManyNetworks));
    }

    #[test]
    fn auth_mode_codes_match_esp_idf_values() {
        // Arrange
        let modes = [
            WifiScanAuthMode::Open,
            WifiScanAuthMode::Wep,
            WifiScanAuthMode::WpaPsk,
            WifiScanAuthMode::Wpa2Psk,
            WifiScanAuthMode::WpaWpa2Psk,
            WifiScanAuthMode::Wpa2Enterprise,
            WifiScanAuthMode::Wpa3Psk,
            WifiScanAuthMode::Wpa2Wpa3Psk,
            WifiScanAuthMode::WapiPsk,
            WifiScanAuthMode::Unknown,
        ];

        // Act
        let codes = modes.map(WifiScanAuthMode::code);

        // Assert
        assert_eq!(codes, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn link_local_ipv6_uses_positive_interface_zone_only() {
        // Arrange
        let address = "fe80::1234".parse().expect("link-local IPv6");

        // Act
        let with_zone = project_ipv6_address(address, Some(7));
        let zero_zone = project_ipv6_address(address, Some(0));
        let missing_zone = project_ipv6_address(address, None);

        // Assert
        assert_eq!(with_zone, "fe80::1234%7");
        assert_eq!(zero_zone, "fe80::1234");
        assert_eq!(missing_zone, "fe80::1234");
    }

    #[test]
    fn global_and_ula_ipv6_ignore_interface_zone() {
        // Arrange
        let global = "2001:db8::5".parse().expect("global IPv6");
        let ula = "fd00::5".parse().expect("ULA IPv6");

        // Act
        let projected = [
            project_ipv6_address(global, Some(7)),
            project_ipv6_address(ula, Some(7)),
        ];

        // Assert
        assert_eq!(projected, ["2001:db8::5", "fd00::5"]);
    }
}
