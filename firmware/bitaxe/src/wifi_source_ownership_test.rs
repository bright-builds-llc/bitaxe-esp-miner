const WIFI_ADAPTER_SOURCE: &str = include_str!("wifi_adapter.rs");
const CAPTIVE_DNS_SOURCE: &str = include_str!("wifi_adapter/captive_dns.rs");
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
    assert!(WIFI_ADAPTER_SOURCE.contains("Configuration::AccessPoint(ap_configuration)"));
    assert!(WIFI_ADAPTER_SOURCE.contains("Configuration::Mixed("));
    assert_eq!(WIFI_ADAPTER_SOURCE.matches("captive_dns::start(").count(), 1);
    assert_eq!(CAPTIVE_DNS_SOURCE.matches("pub(super) fn start(").count(), 1);
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
        .find("Configuration::Client(client_configuration)")
        .expect("station success must disable the AP");

    // Act / Assert
    assert!(mixed_mode < station_admission);
    assert!(station_admission < fallback);
    assert!(fallback < client_only);
    assert!(WIFI_ADAPTER_SOURCE.contains("ap_enabled: true"));
    assert!(WIFI_ADAPTER_SOURCE.contains("ap_enabled: false"));
}

#[test]
fn captive_dns_logs_only_closed_categories() {
    // Arrange / Act / Assert
    assert!(!CAPTIVE_DNS_SOURCE.contains("peer={"));
    assert!(!CAPTIVE_DNS_SOURCE.contains("request={"));
    assert!(!CAPTIVE_DNS_SOURCE.contains("response={"));
    assert!(CAPTIVE_DNS_SOURCE.contains("captive_dns=request_rejected category={error}"));
}
