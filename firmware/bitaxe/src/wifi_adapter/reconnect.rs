//! Nonblocking ESP-IDF bridge for the pure reconnect lifecycle policy.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use bitaxe_api::project_ipv6_address;
use bitaxe_core::wifi_reconnect::{
    WifiDisconnectReason, WifiReconnectAction, WifiReconnectEvent, WifiReconnectState,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::netif::IpEvent;
use esp_idf_svc::wifi::{Configuration, WifiEvent};

use super::{
    captive_dns, log_runtime_line, publish_connected_wifi, publish_ipv6_observation,
    wifi_snapshot_cell, WIFI_OWNER,
};

const PROBE_DISCONNECT_DELAY_MS: u64 = 2_000;
const PROBE_STABILITY_WINDOW_MS: u64 = 15_000;
static NETWORK_RECONNECT_PROBE_ARMED: AtomicBool = AtomicBool::new(false);

pub(super) fn start(
    sysloop: &EspSystemEventLoop,
    maybe_initial_reason: Option<WifiDisconnectReason>,
) -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::channel();
    let station_netif_address = {
        let owner = WIFI_OWNER
            .get()
            .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))?
            .lock()
            .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))?;
        owner.wifi.wifi().sta_netif().handle() as usize
    };

    let wifi_sender = sender.clone();
    let wifi_subscription = sysloop.subscribe::<WifiEvent, _>(move |event| {
        let maybe_event = match event {
            WifiEvent::StaDisconnected(disconnected) => {
                Some(WifiReconnectEvent::StationDisconnected(
                    WifiDisconnectReason::from_esp_reason(disconnected.reason()),
                ))
            }
            WifiEvent::ApStaConnected(_) => Some(WifiReconnectEvent::ProvisioningClientConnected),
            WifiEvent::ApStaDisconnected(_) => {
                Some(WifiReconnectEvent::ProvisioningClientDisconnected)
            }
            _ => None,
        };
        if let Some(event) = maybe_event {
            let _ = wifi_sender.send(event);
        }
    })?;

    let ip_sender = sender.clone();
    let ip_subscription = sysloop.subscribe::<IpEvent, _>(move |event| match event {
        IpEvent::DhcpIpAssigned(assignment)
            if assignment.netif_handle() as usize == station_netif_address =>
        {
            let _ = ip_sender.send(WifiReconnectEvent::Ipv4Assigned);
        }
        IpEvent::DhcpIp6Assigned(assignment)
            if assignment.netif_handle() as usize == station_netif_address =>
        {
            let interface_index = unsafe {
                esp_idf_svc::sys::esp_netif_get_netif_impl_index(assignment.netif_handle())
            };
            publish_ipv6_observation(project_ipv6_address(
                assignment.addr(),
                u32::try_from(interface_index).ok(),
            ));
        }
        _ => {}
    })?;

    {
        let mut owner = WIFI_OWNER
            .get()
            .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))?
            .lock()
            .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))?;
        owner._wifi_subscription = Some(wifi_subscription);
        owner._ip_subscription = Some(ip_subscription);
    }

    std::thread::Builder::new()
        .name("wifi-reconnect".to_owned())
        .stack_size(8_192)
        .spawn(move || run(receiver))?;
    request_ipv6_link_local(station_netif_address);
    if let Some(reason) = maybe_initial_reason {
        sender.send(WifiReconnectEvent::StationDisconnected(reason))?;
    }
    Ok(())
}

pub(super) fn start_probe() {
    let reconnect_available = WIFI_OWNER
        .get()
        .and_then(|owner| owner.lock().ok())
        .is_some_and(|owner| owner.maybe_client_configuration.is_some());
    if !reconnect_available {
        log::warn!("wifi_reconnect_probe=not_started category=station_configuration_unavailable");
        return;
    }
    if NETWORK_RECONNECT_PROBE_ARMED.swap(true, Ordering::SeqCst) {
        log::warn!("wifi_reconnect_probe=not_started category=already_armed");
        return;
    }
    log_runtime_line(&format!(
        "wifi_reconnect_probe=armed uptime_ms={}",
        crate::runtime_uptime::millis()
    ));
    if std::thread::Builder::new()
        .name("wifi-reconnect-probe".to_owned())
        .stack_size(4_096)
        .spawn(|| {
            std::thread::sleep(Duration::from_millis(PROBE_DISCONNECT_DELAY_MS));
            let result = WIFI_OWNER
                .get()
                .ok_or(())
                .and_then(|owner| owner.lock().map_err(|_| ()))
                .and_then(|mut owner| owner.wifi.wifi_mut().disconnect().map_err(|_| ()));
            if result.is_err() {
                NETWORK_RECONNECT_PROBE_ARMED.store(false, Ordering::SeqCst);
                log::warn!("wifi_reconnect_probe=failed category=disconnect_launch_failed");
            }
        })
        .is_err()
    {
        NETWORK_RECONNECT_PROBE_ARMED.store(false, Ordering::SeqCst);
        log::warn!("wifi_reconnect_probe=failed category=thread_spawn_failed");
    }
}

fn run(receiver: Receiver<WifiReconnectEvent>) {
    let mut state = WifiReconnectState::default();
    let mut maybe_retry_deadline: Option<Instant> = None;
    loop {
        let received = match maybe_retry_deadline {
            Some(deadline) => {
                receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            }
            None => match receiver.recv() {
                Ok(event) => Ok(event),
                Err(_) => break,
            },
        };
        let event = match received {
            Ok(event) => event,
            Err(RecvTimeoutError::Timeout) => WifiReconnectEvent::RetryDeadline,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if event == WifiReconnectEvent::RetryDeadline {
            maybe_retry_deadline = None;
        }
        let mut pending = state.apply(event);
        while let Some(action) = pending.first().copied() {
            pending.remove(0);
            match apply_action(action, &mut maybe_retry_deadline) {
                Ok(()) => {}
                Err(followup) => pending.extend(state.apply(followup)),
            }
        }
    }
    log::warn!("wifi_reconnect_lifecycle=stopped category=event_channel_closed");
}

fn apply_action(
    action: WifiReconnectAction,
    maybe_retry_deadline: &mut Option<Instant>,
) -> Result<(), WifiReconnectEvent> {
    match action {
        WifiReconnectAction::IgnoreRoaming => {
            log::info!("wifi_reconnect=ignored reason=roaming");
        }
        WifiReconnectAction::EnableConfigurationNetwork => {
            if enable_configuration_network().is_err() {
                log::warn!("wifi_reconnect=fallback_failed category=configuration_rejected");
            }
        }
        WifiReconnectAction::PublishDisconnected {
            reason,
            retry_ordinal,
        } => publish_disconnected(reason, retry_ordinal),
        WifiReconnectAction::ScheduleRetry {
            delay_ms,
            retry_ordinal,
        } => {
            *maybe_retry_deadline = Some(Instant::now() + Duration::from_millis(delay_ms));
            log::info!(
                "wifi_reconnect=scheduled retry_ordinal={retry_ordinal} retry_delay_ms={delay_ms}"
            );
        }
        WifiReconnectAction::StartReconnect { retry_ordinal } => {
            log_runtime_line(&format!(
                "wifi_reconnect=attempt_started retry_ordinal={retry_ordinal} uptime_ms={}",
                crate::runtime_uptime::millis()
            ));
            if launch_reconnect().is_err() {
                log::warn!(
                    "wifi_reconnect=launch_failed retry_ordinal={retry_ordinal} category=esp_rejected"
                );
                return Err(WifiReconnectEvent::ReconnectLaunchFailed);
            }
        }
        WifiReconnectAction::DisableConfigurationNetwork => {
            if disable_configuration_network().is_err() {
                log::warn!("wifi_reconnect=client_only_failed category=configuration_rejected");
            }
        }
        WifiReconnectAction::PublishConnected {
            completed_retry_ordinal,
        } => publish_connected(completed_retry_ordinal),
    }
    Ok(())
}

fn enable_configuration_network() -> anyhow::Result<()> {
    let (ap_ssid, ap_ipv4) = {
        let mut owner = WIFI_OWNER
            .get()
            .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))?
            .lock()
            .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))?;
        let client = owner
            .maybe_client_configuration
            .clone()
            .ok_or_else(|| anyhow::anyhow!("station configuration was unavailable"))?;
        let configuration = Configuration::Mixed(client, owner.ap_configuration.clone());
        owner.wifi.set_configuration(&configuration)?;
        let ap_ipv4 = owner.wifi.wifi().ap_netif().get_ip_info()?.ip;
        (owner.ap_ssid.clone(), ap_ipv4)
    };
    captive_dns::start_once(ap_ipv4)?;
    let mut snapshot = wifi_snapshot_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("Wi-Fi snapshot lock was poisoned"))?;
    snapshot.ap_enabled = true;
    snapshot.ap_ssid = ap_ssid;
    snapshot.ipv4 = "0.0.0.0".to_owned();
    Ok(())
}

fn disable_configuration_network() -> anyhow::Result<()> {
    let mut owner = WIFI_OWNER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))?
        .lock()
        .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))?;
    let client = owner
        .maybe_client_configuration
        .clone()
        .ok_or_else(|| anyhow::anyhow!("station configuration was unavailable"))?;
    owner
        .wifi
        .set_configuration(&Configuration::Client(client))?;
    Ok(())
}

fn launch_reconnect() -> anyhow::Result<()> {
    let mut owner = WIFI_OWNER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))?
        .lock()
        .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))?;
    owner.wifi.wifi_mut().connect()?;
    Ok(())
}

fn publish_disconnected(reason: WifiDisconnectReason, retry_ordinal: u32) {
    if let Ok(mut snapshot) = wifi_snapshot_cell().lock() {
        snapshot.wifi_status = format!("disconnected_{}", reason.category());
        snapshot.maybe_rssi_dbm = None;
    }
    log_runtime_line(&format!(
        "wifi_reconnect=disconnected reason={} retry_ordinal={retry_ordinal} fallback=true retry_delay_ms=5000 uptime_ms={}",
        reason.category(),
        crate::runtime_uptime::millis()
    ));
    notify_network_changed();
}

fn publish_connected(completed_retry_ordinal: u32) {
    let station_ssid = wifi_snapshot_cell()
        .lock()
        .map(|snapshot| snapshot.ssid.clone())
        .unwrap_or_default();
    let result = WIFI_OWNER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Wi-Fi owner was unavailable"))
        .and_then(|owner| {
            owner
                .lock()
                .map_err(|_| anyhow::anyhow!("Wi-Fi owner lock was poisoned"))
        })
        .and_then(|owner| publish_connected_wifi(&owner.wifi, &station_ssid));
    if result.is_err() {
        log::warn!("wifi_reconnect=publication_failed category=netif_unavailable");
        return;
    }
    log_runtime_line(&format!(
        "wifi_reconnect=connected completed_retry_ordinal={completed_retry_ordinal} retry_ordinal=0 fallback=false uptime_ms={}",
        crate::runtime_uptime::millis()
    ));
    request_current_ipv6_link_local();
    notify_network_changed();
    if NETWORK_RECONNECT_PROBE_ARMED.swap(false, Ordering::SeqCst) {
        spawn_probe_stability_check(completed_retry_ordinal);
    }
}

fn spawn_probe_stability_check(completed_retry_ordinal: u32) {
    log_runtime_line(&format!(
        "wifi_reconnect_probe=recovered completed_retry_ordinal={completed_retry_ordinal} uptime_ms={}",
        crate::runtime_uptime::millis()
    ));
    if std::thread::Builder::new()
        .name("wifi-reconnect-stability".to_owned())
        .stack_size(4_096)
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(PROBE_STABILITY_WINDOW_MS));
            let stable = wifi_snapshot_cell()
                .lock()
                .is_ok_and(|snapshot| snapshot.wifi_status == "connected" && !snapshot.ap_enabled);
            if stable {
                log_runtime_line(&format!(
                    "wifi_reconnect_probe=stable completed_retry_ordinal={completed_retry_ordinal} stability_ms={PROBE_STABILITY_WINDOW_MS} uptime_ms={}",
                    crate::runtime_uptime::millis()
                ));
            } else {
                log::warn!("wifi_reconnect_probe=unstable category=post_recovery_state_changed");
            }
        })
        .is_err()
    {
        log::warn!("wifi_reconnect_probe=unstable category=thread_spawn_failed");
    }
}

fn request_current_ipv6_link_local() {
    let maybe_address = WIFI_OWNER
        .get()
        .and_then(|owner| owner.lock().ok())
        .map(|owner| owner.wifi.wifi().sta_netif().handle() as usize);
    if let Some(address) = maybe_address {
        request_ipv6_link_local(address);
    }
}

fn request_ipv6_link_local(station_netif_address: usize) {
    let result = unsafe {
        esp_idf_svc::sys::esp_netif_create_ip6_linklocal(
            station_netif_address as *mut esp_idf_svc::sys::esp_netif_t,
        )
    };
    if result == esp_idf_svc::sys::ESP_OK {
        log::info!("wifi_ipv6_status=link_local_requested");
    } else {
        log::warn!("wifi_ipv6_status=link_local_request_failed esp_err={result}");
    }
}

fn notify_network_changed() {
    if matches!(
        crate::production_mining_session::notify(
            bitaxe_stratum::v1::production_session::ProductionSessionWakeup::NetworkChanged,
        ),
        bitaxe_stratum::v1::production_session::ProductionSessionNotificationOutcome::OwnerUnavailable
    ) {
        log::warn!("wifi_reconnect=applied network_notification=owner_unavailable");
    }
}
