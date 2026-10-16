const ADAPTER_SOURCE: &str = include_str!("bap_adapter.rs");
const RUNTIME_SOURCE: &str = include_str!("bap_runtime.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const MAIN_SOURCE: &str = include_str!("main.rs");
const ACCESSORY_SOURCE: &str = include_str!("http_api/accessory_settings.rs");

#[test]
fn bap_owner_uses_one_pinned_uart2_shell_and_pure_runtime() {
    // Arrange
    // Act
    let uart_owner_count = ADAPTER_SOURCE.matches("UartDriver::new(").count();
    let owner_spawn_count = ADAPTER_SOURCE.matches(".name(\"bap-owner\"").count();

    // Assert
    assert_eq!(uart_owner_count, 1);
    assert_eq!(owner_spawn_count, 1);
    assert!(ADAPTER_SOURCE.contains("pub const BAP_UART_BAUD: u32 = 115_200"));
    assert!(ADAPTER_SOURCE.contains("pub const BAP_UART_TX_PIN: i32 = 39"));
    assert!(ADAPTER_SOURCE.contains("pub const BAP_UART_RX_PIN: i32 = 40"));
    assert!(ADAPTER_SOURCE.contains("config::DataBits::DataBits8"));
    assert!(ADAPTER_SOURCE.contains(".parity_none()"));
    assert!(ADAPTER_SOURCE.contains("config::StopBits::STOP1"));
    assert!(ADAPTER_SOURCE.contains("config::FlowControl::None"));
    assert!(STARTUP_SOURCE.contains(
        "bap_adapter::start(peripherals.uart2, pins.gpio39, pins.gpio40)"
    ));
    assert!(MAIN_SOURCE.contains("mod bap_adapter;"));
    assert!(MAIN_SOURCE.contains("mod bap_runtime;"));
    assert!(!ADAPTER_SOURCE.contains("uart1"));
    assert!(!RUNTIME_SOURCE.contains("uart1"));
}

#[test]
fn bap_diagnostics_do_not_render_frames_or_setting_values() {
    // Arrange
    let forbidden = [
        "input={",
        "frame={",
        "value={",
        "ssid={",
        "password={",
        "patch={",
        "{value",
        "{patch",
        "{setting:?}",
    ];

    // Act
    let maybe_forbidden = forbidden
        .iter()
        .find(|token| ADAPTER_SOURCE.contains(**token) || ACCESSORY_SOURCE.contains(**token));

    // Assert
    assert_eq!(maybe_forbidden, None);
    assert!(ADAPTER_SOURCE.contains("setting_category(&pending.setting)"));
    assert!(RUNTIME_SOURCE.contains("BapIngress"));
    assert!(RUNTIME_SOURCE.contains("plan_command"));
}

#[test]
fn bap_settings_run_on_the_http_task_through_its_settings_owners() {
    // Arrange / Act
    let queued = ACCESSORY_SOURCE.matches("sys::httpd_queue_work(").count();

    // Assert: the BAP owner never persists or actuates on its own stack.
    assert_eq!(queued, 1);
    assert!(ADAPTER_SOURCE.contains("crate::http_api::apply_accessory_setting("));
    assert!(ADAPTER_SOURCE.contains("crate::http_api::restart_for_accessory()"));
    assert!(!ADAPTER_SOURCE.contains("FirmwareSettingsAdapter"));
    assert!(ACCESSORY_SOURCE.contains("execute_settings_persistence_plan(&plan, &mut adapter)"));
    assert!(ACCESSORY_SOURCE.contains("apply_block_found_dismiss_command("));
}
