//! ESP-IDF station and configuration-network owner.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::configuration_ap_ssid;
use bitaxe_config::{reload_snapshot, LoadedValue, WifiPassword, WifiSsid};
use bitaxe_core::input::{configuration_ap_toggle_mode, ConfigurationApMode};
use esp_idf_svc::eventloop::{EspSystemEventLoop, EspSystemSubscription};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration,
    EspWifi, WifiDeviceId,
};

use crate::{boot_evidence, log_buffer, network_stack, settings_adapter};

mod captive_dns;
mod reconnect;
mod scan;

pub use scan::{scan_visible_networks, WifiScanFailure};

static WIFI_RUNTIME_SNAPSHOT: OnceLock<Mutex<WifiRuntimeSnapshot>> = OnceLock::new();
static WIFI_OWNER: OnceLock<Mutex<WifiOwner>> = OnceLock::new();
const CONFIGURATION_AP_CHANNEL: u8 = 1;
const CONFIGURATION_AP_MAX_CONNECTIONS: u16 = 10;

type FirmwareWifi = BlockingWifi<EspWifi<'static>>;

struct WifiOwner {
    wifi: FirmwareWifi,
    ap_configuration: AccessPointConfiguration,
    maybe_client_configuration: Option<ClientConfiguration>,
    ap_ssid: String,
    _wifi_subscription: Option<EspSystemSubscription<'static>>,
    _ip_subscription: Option<EspSystemSubscription<'static>>,
}

/// Closed configuration-AP toggle failures without private network values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationApToggleError {
    OwnerUnavailable,
    OwnerLockUnavailable,
    SnapshotLockUnavailable,
    ApAddressUnavailable,
    CaptiveDnsUnavailable,
    ConfigurationRejected,
}

impl ConfigurationApToggleError {
    /// Stable redaction-safe log category.
    pub const fn category(self) -> &'static str {
        match self {
            Self::OwnerUnavailable => "owner_unavailable",
            Self::OwnerLockUnavailable => "owner_lock_unavailable",
            Self::SnapshotLockUnavailable => "snapshot_lock_unavailable",
            Self::ApAddressUnavailable => "ap_address_unavailable",
            Self::CaptiveDnsUnavailable => "captive_dns_unavailable",
            Self::ConfigurationRejected => "configuration_rejected",
        }
    }
}

impl std::fmt::Display for ConfigurationApToggleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.category())
    }
}

impl std::error::Error for ConfigurationApToggleError {}

/// API-visible Wi-Fi state collected by the firmware adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiRuntimeSnapshot {
    pub wifi_status: String,
    pub ssid: String,
    pub ap_ssid: String,
    pub ipv4: String,
    pub ipv6: String,
    pub mac_addr: String,
    pub ap_enabled: bool,
    pub maybe_rssi_dbm: Option<i16>,
}

impl Default for WifiRuntimeSnapshot {
    fn default() -> Self {
        Self {
            wifi_status: "disconnected".to_owned(),
            ssid: String::new(),
            ap_ssid: String::new(),
            ipv4: "0.0.0.0".to_owned(),
            ipv6: String::new(),
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
    let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop.clone())?;
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
                    ap_configuration,
                    Some(client_configuration),
                    ap_mac,
                    credentials.ssid.as_str().to_owned(),
                    ProvisioningReason::StationAdmissionFailed,
                    Some(&sysloop),
                );
            }

            wifi.set_configuration(&Configuration::Client(client_configuration.clone()))?;
            publish_connected_wifi(&wifi, credentials.ssid.as_str())?;
            install_wifi_owner(wifi, ap_configuration, Some(client_configuration), ap_mac)?;
            reconnect::start(&sysloop, None)
        }
    }
}

/// Starts the private one-shot live reconnect probe only after HTTP readiness.
pub(crate) fn maybe_start_network_reconnect_probe(route_shell_ready: bool) {
    if !route_shell_ready {
        return;
    }
    match settings_adapter::consume_network_reconnect_probe() {
        Ok(true) => reconnect::start_probe(),
        Ok(false) => {}
        Err(error) => {
            log::warn!(
                "wifi_reconnect_probe=not_started category=marker_consume_failed error={error}"
            )
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

/// Toggles only the configuration AP mode retained by the sole Wi-Fi owner.
pub fn toggle_configuration_ap() -> Result<bool, ConfigurationApToggleError> {
    let owner = WIFI_OWNER
        .get()
        .ok_or(ConfigurationApToggleError::OwnerUnavailable)?;
    let mut owner = owner
        .lock()
        .map_err(|_| ConfigurationApToggleError::OwnerLockUnavailable)?;
    let mut snapshot = wifi_snapshot_cell()
        .lock()
        .map_err(|_| ConfigurationApToggleError::SnapshotLockUnavailable)?;
    let next_mode = configuration_ap_toggle_mode(
        snapshot.ap_enabled,
        owner.maybe_client_configuration.is_some(),
    );
    let previous_mode = retained_configuration_mode(
        snapshot.ap_enabled,
        owner.maybe_client_configuration.is_some(),
    );
    let enabling_ap = matches!(
        next_mode,
        ConfigurationApMode::AccessPointOnly | ConfigurationApMode::StationAndAccessPoint
    );
    let configuration = configuration_for_mode(&owner, next_mode)?;
    owner
        .wifi
        .set_configuration(&configuration)
        .map_err(|_| ConfigurationApToggleError::ConfigurationRejected)?;
    if enabling_ap {
        let ap_ipv4 = match owner.wifi.wifi().ap_netif().get_ip_info() {
            Ok(info) => info.ip,
            Err(_) => {
                restore_wifi_mode(&mut owner, previous_mode);
                return Err(ConfigurationApToggleError::ApAddressUnavailable);
            }
        };
        if captive_dns::start_once(ap_ipv4).is_err() {
            restore_wifi_mode(&mut owner, previous_mode);
            return Err(ConfigurationApToggleError::CaptiveDnsUnavailable);
        }
    }

    snapshot.ap_enabled = enabling_ap;
    snapshot.ap_ssid = if enabling_ap {
        owner.ap_ssid.clone()
    } else {
        String::new()
    };
    drop(snapshot);
    drop(owner);
    if matches!(
        crate::production_mining_session::notify(
            bitaxe_stratum::v1::production_session::ProductionSessionWakeup::NetworkChanged,
        ),
        bitaxe_stratum::v1::production_session::ProductionSessionNotificationOutcome::OwnerUnavailable
    ) {
        log::warn!("wifi_ap_toggle=applied network_notification=owner_unavailable");
    }
    Ok(enabling_ap)
}

fn configuration_for_mode(
    owner: &WifiOwner,
    mode: ConfigurationApMode,
) -> Result<Configuration, ConfigurationApToggleError> {
    Ok(match mode {
        ConfigurationApMode::None => Configuration::None,
        ConfigurationApMode::StationOnly => Configuration::Client(
            owner
                .maybe_client_configuration
                .clone()
                .ok_or(ConfigurationApToggleError::ConfigurationRejected)?,
        ),
        ConfigurationApMode::AccessPointOnly => {
            Configuration::AccessPoint(owner.ap_configuration.clone())
        }
        ConfigurationApMode::StationAndAccessPoint => Configuration::Mixed(
            owner
                .maybe_client_configuration
                .clone()
                .ok_or(ConfigurationApToggleError::ConfigurationRejected)?,
            owner.ap_configuration.clone(),
        ),
    })
}

const fn retained_configuration_mode(
    ap_enabled: bool,
    station_configuration_available: bool,
) -> ConfigurationApMode {
    match (ap_enabled, station_configuration_available) {
        (true, true) => ConfigurationApMode::StationAndAccessPoint,
        (true, false) => ConfigurationApMode::AccessPointOnly,
        (false, true) => ConfigurationApMode::StationOnly,
        (false, false) => ConfigurationApMode::None,
    }
}

fn restore_wifi_mode(owner: &mut WifiOwner, mode: ConfigurationApMode) {
    let Ok(configuration) = configuration_for_mode(owner, mode) else {
        log::warn!("wifi_ap_toggle=recovery_failed category=configuration_unavailable");
        return;
    };
    if owner.wifi.set_configuration(&configuration).is_err() {
        log::warn!("wifi_ap_toggle=recovery_failed category=configuration_rejected");
    }
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
    wifi.set_configuration(&Configuration::AccessPoint(ap_configuration.clone()))?;
    wifi.start()?;
    wifi.wait_netif_up()?;
    retain_provisioning(
        wifi,
        ap_configuration,
        None,
        ap_mac,
        station_ssid,
        reason,
        None,
    )
}

fn retain_provisioning(
    wifi: FirmwareWifi,
    ap_configuration: AccessPointConfiguration,
    maybe_client_configuration: Option<ClientConfiguration>,
    ap_mac: [u8; 6],
    station_ssid: String,
    reason: ProvisioningReason,
    maybe_sysloop: Option<&EspSystemEventLoop>,
) -> anyhow::Result<()> {
    let ap_ipv4 = wifi.wifi().ap_netif().get_ip_info()?.ip;
    captive_dns::start_once(ap_ipv4)?;
    publish_wifi_state(WifiRuntimeSnapshot {
        wifi_status: reason.wifi_status().to_owned(),
        ssid: station_ssid,
        ap_ssid: configuration_ap_ssid(ap_mac).as_str().to_owned(),
        ipv4: ap_ipv4.to_string(),
        ipv6: String::new(),
        mac_addr: format_mac_addr(ap_mac),
        ap_enabled: true,
        maybe_rssi_dbm: None,
    });
    log_runtime_line(&format!(
        "wifi_status={} ap_enabled=true captive_dns=started",
        reason.wifi_status()
    ));
    let reconnect_available = maybe_client_configuration.is_some();
    install_wifi_owner(wifi, ap_configuration, maybe_client_configuration, ap_mac)?;
    if reconnect_available {
        let sysloop = maybe_sysloop
            .ok_or_else(|| anyhow::anyhow!("Wi-Fi reconnect event loop was unavailable"))?;
        reconnect::start(
            sysloop,
            Some(bitaxe_core::wifi_reconnect::WifiDisconnectReason::Other),
        )?;
    }
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
        ap_ssid: String::new(),
        ipv4: ipv4.clone(),
        ipv6: String::new(),
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

fn install_wifi_owner(
    wifi: FirmwareWifi,
    ap_configuration: AccessPointConfiguration,
    maybe_client_configuration: Option<ClientConfiguration>,
    ap_mac: [u8; 6],
) -> anyhow::Result<()> {
    WIFI_OWNER
        .set(Mutex::new(WifiOwner {
            wifi,
            ap_configuration,
            maybe_client_configuration,
            ap_ssid: configuration_ap_ssid(ap_mac).as_str().to_owned(),
            _wifi_subscription: None,
            _ip_subscription: None,
        }))
        .map_err(|_| anyhow::anyhow!("Wi-Fi owner was already installed"))
}

fn publish_ipv6_observation(ipv6: String) {
    let snapshot = wifi_snapshot_cell();
    let Ok(mut snapshot) = snapshot.lock() else {
        log::warn!("wifi_ipv6_status=publication_failed reason=mutex_poisoned");
        return;
    };

    snapshot.ipv6 = ipv6;
    log::info!("wifi_ipv6_status=published");
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
