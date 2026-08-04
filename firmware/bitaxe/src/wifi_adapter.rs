//! ESP-IDF station and configuration-network owner.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::configuration_ap_ssid;
use bitaxe_config::{reload_snapshot, LoadedValue, WifiPassword, WifiSsid};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration,
    EspWifi, WifiDeviceId,
};

use crate::{boot_evidence, log_buffer, network_stack, settings_adapter};

mod captive_dns;

static WIFI_RUNTIME_SNAPSHOT: OnceLock<Mutex<WifiRuntimeSnapshot>> = OnceLock::new();
const CONFIGURATION_AP_CHANNEL: u8 = 1;
const CONFIGURATION_AP_MAX_CONNECTIONS: u16 = 10;

/// API-visible Wi-Fi state collected by the firmware adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiRuntimeSnapshot {
    pub wifi_status: String,
    pub ssid: String,
    pub ipv4: String,
    pub mac_addr: String,
    pub ap_enabled: bool,
    pub maybe_rssi_dbm: Option<i16>,
}

impl Default for WifiRuntimeSnapshot {
    fn default() -> Self {
        Self {
            wifi_status: "disconnected".to_owned(),
            ssid: String::new(),
            ipv4: "0.0.0.0".to_owned(),
            mac_addr: "00:00:00:00:00:00".to_owned(),
            ap_enabled: false,
            maybe_rssi_dbm: None,
        }
    }
}

struct WifiCredentials {
    ssid: WifiSsid,
    password: WifiPassword,
    hostname: String,
}

enum WifiCredentialState {
    Missing,
    Invalid,
    Valid(WifiCredentials),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProvisioningReason {
    CredentialsMissing,
    CredentialsInvalid,
    StationAdmissionFailed,
}

impl ProvisioningReason {
    const fn wifi_status(self) -> &'static str {
        match self {
            Self::CredentialsMissing => "credentials_missing",
            Self::CredentialsInvalid => "credentials_invalid",
            Self::StationAdmissionFailed => "connection_failed",
        }
    }
}

/// Starts the sole Wi-Fi owner in AP-only or mixed AP+STA mode.
pub fn start_wifi(modem: Modem<'static>) -> anyhow::Result<()> {
    let credential_state = wifi_credential_state();
    network_stack::initialize()?;

    let sysloop = EspSystemEventLoop::take()?;
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop)?;
    let ap_mac = wifi.wifi().get_mac(WifiDeviceId::Ap)?;
    let ap_configuration = configuration_ap(ap_mac)?;

    match credential_state {
        WifiCredentialState::Missing => start_provisioning(
            wifi,
            ap_configuration,
            ap_mac,
            String::new(),
            ProvisioningReason::CredentialsMissing,
        ),
        WifiCredentialState::Invalid => start_provisioning(
            wifi,
            ap_configuration,
            ap_mac,
            String::new(),
            ProvisioningReason::CredentialsInvalid,
        ),
        WifiCredentialState::Valid(credentials) => {
            apply_sta_hostname(&credentials.hostname);
            let client_configuration = configuration_client(&credentials)?;
            wifi.set_configuration(&Configuration::Mixed(
                client_configuration.clone(),
                ap_configuration.clone(),
            ))?;
            wifi.start()?;

            if wifi.connect().and_then(|()| wifi.wait_netif_up()).is_err() {
                log::warn!("wifi_status=connection_failed fallback=configuration_ap");
                return retain_provisioning(
                    wifi,
                    ap_mac,
                    credentials.ssid.as_str().to_owned(),
                    ProvisioningReason::StationAdmissionFailed,
                );
            }

            wifi.set_configuration(&Configuration::Client(client_configuration))?;
            publish_connected_wifi(&wifi, credentials.ssid.as_str())?;
            Box::leak(Box::new(wifi));
            Ok(())
        }
    }
}

/// Returns the current API-visible Wi-Fi snapshot.
#[must_use]
pub fn current_wifi_snapshot() -> WifiRuntimeSnapshot {
    let snapshot = wifi_snapshot_cell();
    let Ok(snapshot) = snapshot.lock() else {
        log::warn!("wifi_status=unavailable reason=mutex_poisoned");
        return WifiRuntimeSnapshot::default();
    };

    snapshot.clone()
}

fn configuration_ap(ap_mac: [u8; 6]) -> anyhow::Result<AccessPointConfiguration> {
    let ssid = configuration_ap_ssid(ap_mac);
    Ok(AccessPointConfiguration {
        ssid: ssid
            .as_str()
            .try_into()
            .map_err(|_| anyhow::anyhow!("configuration AP SSID did not fit ESP-IDF buffer"))?,
        ssid_hidden: false,
        channel: CONFIGURATION_AP_CHANNEL,
        auth_method: AuthMethod::None,
        password: Default::default(),
        max_connections: CONFIGURATION_AP_MAX_CONNECTIONS,
        ..Default::default()
    })
}

fn configuration_client(credentials: &WifiCredentials) -> anyhow::Result<ClientConfiguration> {
    Ok(ClientConfiguration {
        ssid: credentials
            .ssid
            .as_str()
            .try_into()
            .map_err(|_| anyhow::anyhow!("validated Wi-Fi SSID did not fit ESP-IDF buffer"))?,
        password: credentials
            .password
            .as_str()
            .try_into()
            .map_err(|_| anyhow::anyhow!("validated Wi-Fi password did not fit ESP-IDF buffer"))?,
        auth_method: wifi_auth_method(credentials.password.as_str()),
        ..Default::default()
    })
}

fn start_provisioning(
    mut wifi: BlockingWifi<EspWifi<'static>>,
    ap_configuration: AccessPointConfiguration,
    ap_mac: [u8; 6],
    station_ssid: String,
    reason: ProvisioningReason,
) -> anyhow::Result<()> {
    wifi.set_configuration(&Configuration::AccessPoint(ap_configuration))?;
    wifi.start()?;
    wifi.wait_netif_up()?;
    retain_provisioning(wifi, ap_mac, station_ssid, reason)
}

fn retain_provisioning(
    wifi: BlockingWifi<EspWifi<'static>>,
    ap_mac: [u8; 6],
    station_ssid: String,
    reason: ProvisioningReason,
) -> anyhow::Result<()> {
    let ap_ipv4 = wifi.wifi().ap_netif().get_ip_info()?.ip;
    captive_dns::start(ap_ipv4)?;
    publish_wifi_state(WifiRuntimeSnapshot {
        wifi_status: reason.wifi_status().to_owned(),
        ssid: station_ssid,
        ipv4: ap_ipv4.to_string(),
        mac_addr: format_mac_addr(ap_mac),
        ap_enabled: true,
        maybe_rssi_dbm: None,
    });
    log_runtime_line(&format!(
        "wifi_status={} ap_enabled=true captive_dns=started",
        reason.wifi_status()
    ));
    Box::leak(Box::new(wifi));
    Ok(())
}

fn publish_connected_wifi(
    wifi: &BlockingWifi<EspWifi<'static>>,
    station_ssid: &str,
) -> anyhow::Result<()> {
    let ipv4 = wifi.wifi().sta_netif().get_ip_info()?.ip.to_string();
    let mac_addr = wifi
        .wifi()
        .get_mac(WifiDeviceId::Sta)
        .map(format_mac_addr)
        .unwrap_or_else(|error| {
            log::warn!("wifi_mac_status=unavailable error={error}");
            WifiRuntimeSnapshot::default().mac_addr
        });
    let maybe_rssi_dbm = wifi
        .wifi()
        .get_rssi()
        .ok()
        .and_then(|rssi| i16::try_from(rssi).ok());

    publish_wifi_state(WifiRuntimeSnapshot {
        wifi_status: "connected".to_owned(),
        ssid: station_ssid.to_owned(),
        ipv4: ipv4.clone(),
        mac_addr,
        ap_enabled: false,
        maybe_rssi_dbm,
    });
    log_runtime_line(&format!(
        "wifi_status=connected ipv4={ipv4} device_url=http://{ipv4}"
    ));
    boot_evidence::publish_connected_origin(format!("http://{ipv4}"));
    Ok(())
}

fn wifi_credential_state() -> WifiCredentialState {
    let settings = settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&settings);
    let Some(LoadedValue::Str(ssid)) = loaded.maybe_loaded_value("wifissid") else {
        return WifiCredentialState::Missing;
    };
    if ssid.is_empty() {
        return WifiCredentialState::Missing;
    }

    let password = match loaded.maybe_loaded_value("wifipass") {
        Some(LoadedValue::Str(password)) => password.clone(),
        _ => String::new(),
    };
    let hostname = match loaded.maybe_loaded_value("hostname") {
        Some(LoadedValue::Str(hostname)) => hostname.clone(),
        _ => "bitaxe".to_owned(),
    };

    let ssid = match WifiSsid::parse(ssid.clone()) {
        Ok(ssid) => ssid,
        Err(error) => {
            log::warn!("wifi_status=credentials_invalid field=ssid error={error}");
            return WifiCredentialState::Invalid;
        }
    };
    let password = match WifiPassword::parse(password) {
        Ok(password) => password,
        Err(error) => {
            log::warn!("wifi_status=credentials_invalid field=wifiPass error={error}");
            return WifiCredentialState::Invalid;
        }
    };

    WifiCredentialState::Valid(WifiCredentials {
        ssid,
        password,
        hostname,
    })
}

fn wifi_auth_method(password: &str) -> AuthMethod {
    if password.is_empty() {
        return AuthMethod::None;
    }

    AuthMethod::WPA2Personal
}

fn apply_sta_hostname(hostname: &str) {
    let Ok(hostname_cstr) = std::ffi::CString::new(hostname) else {
        log::warn!("wifi_hostname_status=skipped reason=interior_nul");
        return;
    };

    let netif = unsafe {
        esp_idf_svc::sys::esp_netif_get_handle_from_ifkey(b"WIFI_STA_DEF\0".as_ptr().cast())
    };
    if netif.is_null() {
        log::warn!("wifi_hostname_status=skipped reason=netif_unavailable");
        return;
    }

    let result = unsafe { esp_idf_svc::sys::esp_netif_set_hostname(netif, hostname_cstr.as_ptr()) };
    if result == esp_idf_svc::sys::ESP_OK {
        log::info!("wifi_hostname_status=applied");
        return;
    }

    log::warn!("wifi_hostname_status=failed esp_err={result}");
}

fn publish_wifi_state(snapshot: WifiRuntimeSnapshot) {
    let cell = wifi_snapshot_cell();
    let Ok(mut current) = cell.lock() else {
        log::warn!("wifi_status=unavailable reason=mutex_poisoned");
        return;
    };

    *current = snapshot;
}

fn wifi_snapshot_cell() -> &'static Mutex<WifiRuntimeSnapshot> {
    WIFI_RUNTIME_SNAPSHOT.get_or_init(|| Mutex::new(WifiRuntimeSnapshot::default()))
}

fn log_runtime_line(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}

fn format_mac_addr(mac: [u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}
