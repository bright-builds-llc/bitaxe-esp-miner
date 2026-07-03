//! Minimal ESP-IDF Wi-Fi station adapter for operator-provided credentials.

use std::sync::{Mutex, OnceLock};

use bitaxe_config::{reload_snapshot, LoadedValue, WifiPassword, WifiSsid};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::wifi::{
    AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi, WifiDeviceId,
};

use crate::{log_buffer, network_stack, settings_adapter};

static WIFI_RUNTIME_SNAPSHOT: OnceLock<Mutex<WifiRuntimeSnapshot>> = OnceLock::new();

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

/// Starts STA Wi-Fi when NVS contains `wifissid`; otherwise leaves networking unavailable.
pub fn start_wifi_sta(modem: Modem<'static>) -> anyhow::Result<()> {
    let Some(credentials) = maybe_wifi_credentials() else {
        publish_wifi_state(WifiRuntimeSnapshot {
            wifi_status: "credentials_missing".to_owned(),
            ..WifiRuntimeSnapshot::default()
        });
        log_runtime_line("wifi_status=credentials_missing");
        return Ok(());
    };

    network_stack::initialize()?;

    let sysloop = EspSystemEventLoop::take()?;
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop)?;
    apply_sta_hostname(&credentials.hostname);

    let client_configuration =
        ClientConfiguration {
            ssid: credentials
                .ssid
                .as_str()
                .try_into()
                .map_err(|_| anyhow::anyhow!("validated Wi-Fi SSID did not fit ESP-IDF buffer"))?,
            password: credentials.password.as_str().try_into().map_err(|_| {
                anyhow::anyhow!("validated Wi-Fi password did not fit ESP-IDF buffer")
            })?,
            auth_method: wifi_auth_method(credentials.password.as_str()),
            ..Default::default()
        };
    wifi.set_configuration(&Configuration::Client(client_configuration))?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    let ipv4 = ip_info.ip.to_string();
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
        ssid: credentials.ssid.as_str().to_owned(),
        ipv4: ipv4.clone(),
        mac_addr,
        ap_enabled: false,
        maybe_rssi_dbm,
    });
    log_runtime_line(&format!(
        "wifi_status=connected ipv4={ipv4} device_url=http://{ipv4}"
    ));

    Box::leak(Box::new(wifi));
    Ok(())
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

fn maybe_wifi_credentials() -> Option<WifiCredentials> {
    let settings = settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&settings);
    let Some(LoadedValue::Str(ssid)) = loaded.loaded_value("wifissid") else {
        return None;
    };

    if ssid.is_empty() {
        return None;
    }

    let password = match loaded.loaded_value("wifipass") {
        Some(LoadedValue::Str(password)) => password.clone(),
        _ => String::new(),
    };
    let hostname = match loaded.loaded_value("hostname") {
        Some(LoadedValue::Str(hostname)) => hostname.clone(),
        _ => "bitaxe".to_owned(),
    };

    let ssid = match WifiSsid::parse(ssid.clone()) {
        Ok(ssid) => ssid,
        Err(error) => {
            publish_wifi_state(WifiRuntimeSnapshot {
                wifi_status: "credentials_invalid".to_owned(),
                ..WifiRuntimeSnapshot::default()
            });
            log::warn!("wifi_status=credentials_invalid field=ssid error={error}");
            return None;
        }
    };
    let password = match WifiPassword::parse(password) {
        Ok(password) => password,
        Err(error) => {
            publish_wifi_state(WifiRuntimeSnapshot {
                wifi_status: "credentials_invalid".to_owned(),
                ..WifiRuntimeSnapshot::default()
            });
            log::warn!("wifi_status=credentials_invalid field=wifiPass error={error}");
            return None;
        }
    };

    Some(WifiCredentials {
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
