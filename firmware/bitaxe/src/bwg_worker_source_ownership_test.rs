const MAIN_SOURCE: &str = include_str!("main.rs");
const ASIC_SOURCE: &str = include_str!("asic_adapter.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const NVS_SOURCE: &str = include_str!("bwg_worker_nvs.rs");
const SESSION_SOURCE: &str = include_str!("bwg_worker_session.rs");
const USB_SOURCE: &str = include_str!("bwg_worker_usb.rs");
const USB_RUNTIME_SOURCE: &str = include_str!("usb_runtime.rs");

#[test]
fn bwg_private_state_has_one_dedicated_nvs_owner() {
    // Arrange
    let private_names = ["device_seed", "lease_seq"];

    // Act / Assert
    for name in private_names {
        assert_eq!(NVS_SOURCE.matches(name).count(), 1);
        assert!(!MAIN_SOURCE.contains(name));
        assert!(!STARTUP_SOURCE.contains(name));
        assert!(!SESSION_SOURCE.contains(name));
        assert!(!USB_SOURCE.contains(name));
    }
    assert_eq!(NVS_SOURCE.matches("\"effect_pending\"").count(), 1);
    for source in [MAIN_SOURCE, STARTUP_SOURCE, SESSION_SOURCE, USB_SOURCE] {
        assert!(!source.contains("effect_pending"));
    }
    assert!(NVS_SOURCE.contains("esp_fill_random"));
    assert!(!NVS_SOURCE.contains("state.retain("));
    assert!(NVS_SOURCE.contains("state.len() > 8"));
    assert!(!NVS_SOURCE.contains("log::"));
}

#[test]
fn bwg_mining_effects_route_only_through_the_production_owner() {
    // Arrange / Act
    let source = SESSION_SOURCE;

    // Assert
    assert!(source.contains("production_mining_session::bwg_start"));
    assert!(source.contains("production_mining_session::bwg_renew"));
    assert!(source.contains("production_mining_session::bwg_safe_stop"));
    for forbidden in [
        "mining_actuation",
        "asic_adapter",
        "safety_adapter",
        "EspNvs",
    ] {
        assert!(!source.contains(forbidden));
    }
}

#[test]
fn fixed_serial_driver_has_one_writer_and_no_phy_switching() {
    // Arrange
    let writer = include_str!("bwg_worker_usb/writer.rs");
    let link = include_str!("bwg_worker_usb/link.rs");

    // Act / Assert
    assert!(USB_RUNTIME_SOURCE.contains("usb_serial_jtag_driver_install"));
    assert!(USB_RUNTIME_SOURCE.contains("usb_serial_jtag_read_bytes"));
    assert!(USB_RUNTIME_SOURCE.contains("usb_serial_jtag_write_bytes"));
    for source in [USB_RUNTIME_SOURCE, USB_SOURCE, writer, link] {
        assert!(!source.contains("tinyusb_driver_install"));
        assert!(!source.contains("restart_into_rom_downloader"));
        assert!(!source.contains("LineCoding"));
    }
    assert!(!link.contains("recv_timeout"));
    assert!(!link.contains("send_control"));
    assert!(link.contains("revocation::check_deadline(now)"));
    assert!(link.contains("revocation::revoke_reason_at("));
    assert!(link.contains("RevocationReason::LinkClosed"));
    assert!(link.contains("link.liveness.poll(now)"));
    assert!(writer.contains("MAXIMUM") || writer.contains("DIAGNOSTIC_BYTES"));
}

#[test]
fn startup_prepares_worker_before_wifi_and_installs_usb_after_wifi() {
    // Arrange
    let baseline = STARTUP_SOURCE
        .find("startup_diagnostics?;")
        .expect("boot-safe baseline should confirm");
    let recovery = STARTUP_SOURCE
        .find("bwg_worker_usb::recover_interrupted_effect")
        .expect("interrupted BWG effect should recover");
    let production = STARTUP_SOURCE
        .find("production_mining_session::start()")
        .expect("production owner startup should exist");
    let run_start = STARTUP_SOURCE
        .find("pub(crate) fn run()")
        .expect("startup entrypoint should exist");
    let run_end = STARTUP_SOURCE[run_start..]
        .find("fn initialize_boot_identity_and_settings(")
        .map(|offset| run_start + offset)
        .expect("startup entrypoint boundary should exist");
    let run = &STARTUP_SOURCE[run_start..run_end];
    let network = run
        .find("start_network_services(maybe_modem)")
        .expect("Wi-Fi startup should exist");
    let worker_install = run
        .find("start_deferred_usb_runtime(runtime_services.deferred_usb_runtime)")
        .expect("deferred Worker USB installation should exist");
    let statistics = run
        .find("start_statistics_runtime()")
        .expect("statistics startup should exist");

    // Act / Assert
    assert!(baseline < recovery);
    assert!(recovery < production);
    let prepare = STARTUP_SOURCE
        .find("bwg_worker_usb::prepare(bwg_recovery)")
        .expect("Worker owner preparation should exist");
    let runtime_services = run
        .find("start_runtime_services(startup_diagnostics)")
        .expect("runtime services should exist");
    assert!(recovery < prepare);
    assert!(runtime_services < network);
    assert!(network < worker_install);
    assert!(worker_install < statistics);
    assert!(USB_SOURCE.contains("const OWNER_STACK_BYTES: usize = 16 * 1024;"));
    assert!(STARTUP_SOURCE.contains("CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL == 98_304"));
    assert!(STARTUP_SOURCE.contains("retain_usb_memory_checkpoint(\"worker_owner_prepare\")"));
    assert!(STARTUP_SOURCE.contains("retain_usb_memory_checkpoint(\"usb_install\")"));
    assert!(STARTUP_SOURCE.contains("bwg_worker_start_failure category=startup_failed"));
    assert!(STARTUP_SOURCE.contains("bwg_worker_start_failure_detail(error)"));
    assert!(USB_SOURCE.contains("pub(crate) fn prepare("));
    assert!(USB_SOURCE.contains("pub(crate) fn install("));
    assert_eq!(MAIN_SOURCE.matches("mod bwg_worker_usb;").count(), 1);
}

#[test]
fn only_confirmed_boot_hardware_actions_can_advance_bwg_recovery() {
    // Arrange / Act / Assert
    for reason in [
        "reason=enable_unavailable",
        "reason=enable_disable_failed",
        "reason=reset_unavailable",
        "reason=reset_hold_failed",
        "reason=uart_unavailable",
    ] {
        let start = ASIC_SOURCE
            .find(reason)
            .unwrap_or_else(|| panic!("missing boot failure branch {reason}"));
        let boundary = &ASIC_SOURCE[start..start.saturating_add(450).min(ASIC_SOURCE.len())];
        assert!(
            boundary.contains("BootMiningBaseline::Unconfirmed"),
            "boot failure branch could mint a confirmed baseline: {reason}"
        );
    }
    let without_peripherals = ASIC_SOURCE
        .split("pub fn run_boot_gate_without_peripherals")
        .nth(1)
        .expect("without-peripherals boot gate should exist");
    assert_eq!(
        without_peripherals
            .matches("BootMiningBaseline::Unconfirmed")
            .count(),
        3
    );
    assert!(ASIC_SOURCE.contains(
        "production::store_production_peripherals(uart, reset, enable, false);\n    Ok(BootMiningBaseline::Confirmed)"
    ));
}
