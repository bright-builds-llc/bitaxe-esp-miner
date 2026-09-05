//! Reserves the driver's mandatory internal memory before discretionary service allocations.
use super::*;

/// Constructed exactly once and consumed by the later blocking connection startup.
pub(crate) struct PreparedWifi {
    pub(super) wifi: FirmwareWifi,
    pub(super) sysloop: EspSystemEventLoop,
    pub(super) ap_mac: [u8; 6],
    pub(super) ap_configuration: AccessPointConfiguration,
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
    Ok(PreparedWifi {
        wifi,
        sysloop,
        ap_mac,
        ap_configuration,
    })
}
