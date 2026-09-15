//! Reserves the driver's mandatory internal memory before discretionary service allocations.
use super::*;

/// Constructed exactly once and consumed by the later blocking connection startup.
pub(crate) struct PreparedWifi {
    pub(super) wifi: FirmwareWifi,
    pub(super) sysloop: EspSystemEventLoop,
    pub(super) ap_mac: [u8; 6],
    pub(super) ap_configuration: AccessPointConfiguration,
    pub(super) credential_state: reconnect::PreparedCredentials<WifiCredentials>,
}

/// Allocates the driver and netifs without starting RF, association, or DHCP waits.
pub(crate) fn prepare_wifi(modem: Modem<'static>) -> anyhow::Result<PreparedWifi> {
    observe(Phase::Netif, network_stack::initialize())?;
    let sysloop = observe(Phase::EventLoop, EspSystemEventLoop::take())?;
    let esp_wifi = observe(Phase::Driver, EspWifi::new(modem, sysloop.clone(), None))?;
    let wifi = BlockingWifi::wrap(esp_wifi, sysloop.clone())?;
    let ap_mac = observe(
        Phase::ApConfiguration,
        wifi.wifi().get_mac(WifiDeviceId::Ap),
    )?;
    let ap_configuration = observe(Phase::ApConfiguration, configuration_ap(ap_mac))?;
    // Capture one boot configuration before reserving only the station-owned worker.
    let credential_state = wifi_credential_state();
    let credential_state = observe(
        Phase::ReconnectSpawn,
        reconnect::prepare_credentials(credential_state),
    )?;
    Ok(PreparedWifi {
        wifi,
        sysloop,
        ap_mac,
        ap_configuration,
        credential_state,
    })
}

pub(super) fn apply_sta_hostname(hostname: &str) {
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
