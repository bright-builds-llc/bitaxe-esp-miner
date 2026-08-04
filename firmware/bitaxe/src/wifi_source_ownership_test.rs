const WIFI_ADAPTER_SOURCE: &str = include_str!("wifi_adapter.rs");
const CAPTIVE_DNS_SOURCE: &str = include_str!("wifi_adapter/captive_dns.rs");
const WIFI_SCAN_SOURCE: &str = include_str!("wifi_adapter/scan.rs");
const HTTP_API_SOURCE: &str = include_str!("http_api.rs");
const HTTP_HANDLERS_SOURCE: &str = include_str!("http_api/handlers.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn startup_has_one_wifi_owner_before_network_notification() {
    // Arrange
    let wifi_start = STARTUP_SOURCE
        .find("wifi_adapter::start_wifi(modem)")
        .expect("startup must invoke the Wi-Fi owner");
    let network_notification = STARTUP_SOURCE
        .find("ProductionSessionWakeup::NetworkChanged")
        .expect("startup must notify production readiness");

    // Act / Assert
    assert_eq!(STARTUP_SOURCE.matches("wifi_adapter::start_wifi(modem)").count(), 1);
    assert!(wifi_start < network_notification);
}

#[test]
fn configuration_network_has_exact_ap_shape_and_one_dns_owner() {
    // Arrange / Act / Assert
    assert!(WIFI_ADAPTER_SOURCE.contains("CONFIGURATION_AP_CHANNEL: u8 = 1"));
    assert!(WIFI_ADAPTER_SOURCE.contains("CONFIGURATION_AP_MAX_CONNECTIONS: u16 = 10"));
    assert!(WIFI_ADAPTER_SOURCE.contains("ssid_hidden: false"));
    assert!(WIFI_ADAPTER_SOURCE.contains("auth_method: AuthMethod::None"));
    assert!(WIFI_ADAPTER_SOURCE.contains("Configuration::AccessPoint(ap_configuration.clone())"));
    assert!(WIFI_ADAPTER_SOURCE.contains("Configuration::Mixed("));
    assert_eq!(WIFI_ADAPTER_SOURCE.matches("captive_dns::start_once(").count(), 2);
    assert_eq!(CAPTIVE_DNS_SOURCE.matches("pub(super) fn start_once(").count(), 1);
    assert_eq!(CAPTIVE_DNS_SOURCE.matches("UdpSocket::bind(").count(), 1);
    assert_eq!(CAPTIVE_DNS_SOURCE.matches(".spawn(move || run").count(), 1);
}

#[test]
fn station_success_disables_ap_and_failures_retain_provisioning() {
    // Arrange
    let mixed_mode = WIFI_ADAPTER_SOURCE
        .find("Configuration::Mixed(")
        .expect("valid credentials must begin in mixed mode");
    let station_admission = WIFI_ADAPTER_SOURCE
        .find(".connect()")
        .expect("mixed mode must attempt station admission");
    let fallback = WIFI_ADAPTER_SOURCE
        .find("ProvisioningReason::StationAdmissionFailed")
        .expect("station failure must retain provisioning");
    let client_only = WIFI_ADAPTER_SOURCE
        .find("Configuration::Client(client_configuration.clone())")
        .expect("station success must disable the AP");

    // Act / Assert
    assert!(mixed_mode < station_admission);
    assert!(station_admission < fallback);
    assert!(fallback < client_only);
    assert!(WIFI_ADAPTER_SOURCE.contains("ap_enabled: true"));
    assert!(WIFI_ADAPTER_SOURCE.contains("ap_enabled: false"));
}

#[test]
fn long_press_toggle_retains_private_configuration_and_publishes_after_apply() {
    // Arrange
    let function = "pub fn toggle_configuration_ap()";
    let start = WIFI_ADAPTER_SOURCE
        .find(function)
        .expect("typed configuration AP toggle");
    let end = WIFI_ADAPTER_SOURCE[start..]
        .find("fn configuration_ap(")
        .map(|offset| start + offset)
        .expect("toggle boundary");
    let source = &WIFI_ADAPTER_SOURCE[start..end];

    // Act
    let apply = source
        .find(".set_configuration(&configuration)")
        .expect("ESP-IDF configuration apply");
    let publish = source
        .find("snapshot.ap_enabled = enabling_ap")
        .expect("runtime publication");

    // Assert
    assert!(source.contains("configuration_ap_toggle_mode("));
    assert!(source.contains("maybe_client_configuration"));
    assert!(source.contains("ap_configuration.clone()"));
    assert!(source.contains("ConfigurationApToggleError::SnapshotLockUnavailable"));
    assert!(apply < publish);
    assert!(!source.contains("ssid={"));
    assert!(!source.contains("ipv4={"));
}

#[test]
fn captive_dns_logs_only_closed_categories() {
    // Arrange / Act / Assert
    assert!(!CAPTIVE_DNS_SOURCE.contains("peer={"));
    assert!(!CAPTIVE_DNS_SOURCE.contains("request={"));
    assert!(!CAPTIVE_DNS_SOURCE.contains("response={"));
    assert!(CAPTIVE_DNS_SOURCE.contains("captive_dns=request_rejected category={error}"));
}

#[test]
fn scan_route_uses_one_exclusive_wifi_owner_and_restores_ap_mode() {
    // Arrange / Act / Assert
    assert_eq!(WIFI_ADAPTER_SOURCE.matches("static WIFI_OWNER:").count(), 1);
    assert!(WIFI_SCAN_SOURCE.contains("owner.try_lock()"));
    assert!(WIFI_SCAN_SOURCE.contains("scan_n::<MAX_WIFI_SCAN_NETWORKS>()"));
    assert!(WIFI_SCAN_SOURCE.contains("Configuration::AccessPoint(ap_configuration)"));
    assert_eq!(
        HTTP_API_SOURCE
            .matches("\"/api/system/wifi/scan\"")
            .count(),
        1
    );
    assert!(HTTP_HANDLERS_SOURCE.contains("wifi_adapter::scan_visible_networks()"));
    assert!(HTTP_HANDLERS_SOURCE.contains("body: \"WiFi scan failed\""));
}

#[test]
fn ipv6_reporting_is_station_bound_and_logs_categories_only() {
    // Arrange / Act / Assert
    assert!(WIFI_ADAPTER_SOURCE.contains("subscribe::<IpEvent"));
    assert!(WIFI_ADAPTER_SOURCE.contains("IpEvent::DhcpIp6Assigned"));
    assert!(WIFI_ADAPTER_SOURCE.contains("assignment.netif_handle() as usize"));
    assert!(WIFI_ADAPTER_SOURCE.contains("esp_netif_create_ip6_linklocal(station_netif)"));
    assert!(WIFI_ADAPTER_SOURCE.contains("wifi_ipv6_status=published"));
    assert!(!WIFI_ADAPTER_SOURCE.contains("wifi_ipv6_address="));
    assert!(!WIFI_ADAPTER_SOURCE.contains("wifi_scan_ssid="));
    assert!(!WIFI_SCAN_SOURCE.contains("log::"));
}
