//! Exclusive bounded scan transaction for the retained Wi-Fi owner.

use std::sync::TryLockError;

use bitaxe_api::{WifiNetworkWire, WifiScanAuthMode, WifiScanResponse, MAX_WIFI_SCAN_NETWORKS};
use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use super::{FirmwareWifi, WIFI_OWNER};

/// Closed failure categories for one exclusive Wi-Fi scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiScanFailure {
    OwnerUnavailable,
    Busy,
    ConfigurationUnavailable,
    Driver,
    Restore,
    ResponseInvalid,
}

impl WifiScanFailure {
    /// Returns a redaction-safe diagnostic category.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::OwnerUnavailable => "owner_unavailable",
            Self::Busy => "busy",
            Self::ConfigurationUnavailable => "configuration_unavailable",
            Self::Driver => "driver",
            Self::Restore => "restore",
            Self::ResponseInvalid => "response_invalid",
        }
    }
}

/// Runs one bounded scan through the sole retained ESP-IDF Wi-Fi owner.
pub fn scan_visible_networks() -> Result<WifiScanResponse, WifiScanFailure> {
    let owner = WIFI_OWNER.get().ok_or(WifiScanFailure::OwnerUnavailable)?;
    let mut owner = match owner.try_lock() {
        Ok(owner) => owner,
        Err(TryLockError::WouldBlock) => return Err(WifiScanFailure::Busy),
        Err(TryLockError::Poisoned(_)) => return Err(WifiScanFailure::OwnerUnavailable),
    };

    let access_points = scan_with_configuration_restoration(&mut owner.wifi)?;
    let networks = access_points
        .into_iter()
        .map(|access_point| {
            WifiNetworkWire::new(
                access_point.ssid.as_str().to_owned(),
                access_point.signal_strength,
                scan_auth_mode(access_point.auth_method),
            )
        })
        .collect();

    WifiScanResponse::try_new(networks).map_err(|_| WifiScanFailure::ResponseInvalid)
}

fn scan_with_configuration_restoration(
    wifi: &mut FirmwareWifi,
) -> Result<Vec<esp_idf_svc::wifi::AccessPointInfo>, WifiScanFailure> {
    let configuration = wifi
        .get_configuration()
        .map_err(|_| WifiScanFailure::ConfigurationUnavailable)?;

    let Configuration::AccessPoint(ap_configuration) = configuration else {
        if matches!(configuration, Configuration::None) {
            return Err(WifiScanFailure::ConfigurationUnavailable);
        }

        return wifi
            .scan_n::<MAX_WIFI_SCAN_NETWORKS>()
            .map(|(access_points, _)| access_points.into_iter().collect())
            .map_err(|_| WifiScanFailure::Driver);
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration::default(),
        ap_configuration.clone(),
    ))
    .map_err(|_| WifiScanFailure::Driver)?;

    let scan_result = wifi
        .scan_n::<MAX_WIFI_SCAN_NETWORKS>()
        .map(|(access_points, _)| access_points.into_iter().collect())
        .map_err(|_| WifiScanFailure::Driver);
    let restore_result = wifi
        .set_configuration(&Configuration::AccessPoint(ap_configuration))
        .map_err(|_| WifiScanFailure::Restore);

    match (scan_result, restore_result) {
        (Err(primary), _) => Err(primary),
        (Ok(_), Err(restore)) => Err(restore),
        (Ok(access_points), Ok(())) => Ok(access_points),
    }
}

fn scan_auth_mode(maybe_auth_method: Option<AuthMethod>) -> WifiScanAuthMode {
    match maybe_auth_method {
        Some(AuthMethod::None) => WifiScanAuthMode::Open,
        Some(AuthMethod::WEP) => WifiScanAuthMode::Wep,
        Some(AuthMethod::WPA) => WifiScanAuthMode::WpaPsk,
        Some(AuthMethod::WPA2Personal) => WifiScanAuthMode::Wpa2Psk,
        Some(AuthMethod::WPAWPA2Personal) => WifiScanAuthMode::WpaWpa2Psk,
        Some(AuthMethod::WPA2Enterprise) => WifiScanAuthMode::Wpa2Enterprise,
        Some(AuthMethod::WPA3Personal) => WifiScanAuthMode::Wpa3Psk,
        Some(AuthMethod::WPA2WPA3Personal) => WifiScanAuthMode::Wpa2Wpa3Psk,
        Some(AuthMethod::WAPIPersonal) => WifiScanAuthMode::WapiPsk,
        None => WifiScanAuthMode::Unknown,
    }
}
