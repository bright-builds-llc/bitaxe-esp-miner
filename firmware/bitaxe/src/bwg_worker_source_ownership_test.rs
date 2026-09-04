const MAIN_SOURCE: &str = include_str!("main.rs");
const ASIC_SOURCE: &str = include_str!("asic_adapter.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const NVS_SOURCE: &str = include_str!("bwg_worker_nvs.rs");
const SESSION_SOURCE: &str = include_str!("bwg_worker_session.rs");
const USB_SOURCE: &str = include_str!("bwg_worker_usb.rs");
const USB_RUNTIME_SOURCE: &str = include_str!("usb_runtime.rs");
const USB_CALLBACK_SOURCE: &str = include_str!("usb_runtime/callbacks.rs");
const USB_TINYUSB_SOURCE: &str = include_str!("usb_runtime/tinyusb.rs");
const USB_PHY_SOURCE: &str = include_str!("../bwg/native/usb_phy_handoff.c");

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
    for forbidden in ["mining_actuation", "asic_adapter", "safety_adapter", "EspNvs"] {
        assert!(!source.contains(forbidden));
    }
}

#[test]
fn rust_owns_tinyusb_while_c_is_only_the_phy_handoff_adapter() {
    // Arrange / Act / Assert
    let rust_source = format!("{USB_RUNTIME_SOURCE}{USB_CALLBACK_SOURCE}{USB_TINYUSB_SOURCE}");
    for required in [
        "WORKER_DEVICE_DESCRIPTOR",
        "WORKER_CONFIGURATION_DESCRIPTOR",
        "tinyusb_driver_install",
        "tud_mount_cb",
        "tud_umount_cb",
        "tud_vendor_rx_cb",
        "tud_cdc_rx_cb",
        "tud_cdc_line_coding_cb",
        "tud_cdc_line_state_cb",
        "tud_cdc_n_read",
        "bytes.is_null()",
        "coding.is_null()",
        "read_unaligned()",
    ] {
        assert!(
            rust_source.contains(required),
            "missing Rust USB ownership: {required}"
        );
    }

    for forbidden in [
        "tinyusb_driver_install",
        "tud_mount_cb",
        "tud_umount_cb",
        "tud_vendor_rx_cb",
        "tud_cdc_rx_cb",
        "TUD_CDC_DESCRIPTOR",
        "BWG_DEVICE_DESCRIPTOR",
    ] {
        assert!(
            !USB_PHY_SOURCE.contains(forbidden),
            "C retained Rust-owned behavior: {forbidden}"
        );
    }
    assert_eq!(USB_PHY_SOURCE.matches("bitaxe_usb_restart_bootloader").count(), 1);
    assert!(!USB_SOURCE.contains("extern \"C\""));
    assert!(!USB_SOURCE.contains("#[no_mangle]"));
}

#[test]
fn worker_evidence_requires_mount_but_not_dtr() {
    // Arrange / Act / Assert
    assert!(USB_TINYUSB_SOURCE.contains("if !unsafe { sys::tud_mounted() }"));
    assert!(!USB_TINYUSB_SOURCE.contains("tud_cdc_n_connected"));
    assert!(USB_SOURCE.contains("crate::boot_evidence::worker_usb_boot_marker()"));
    assert!(USB_SOURCE.contains("crate::boot_evidence::worker_rust_panic_marker()"));
    assert!(USB_SOURCE.contains("crate::boot_evidence::worker_allocation_failure_marker()"));
}

#[test]
fn phy_handoff_drives_disconnect_and_requires_bounded_bus_reset() {
    // Arrange
    let disconnect = USB_PHY_SOURCE
        .find("CLEAR_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_USB_PAD_ENABLE)")
        .expect("Serial/JTAG pad disconnect");
    let drive_low = USB_PHY_SOURCE
        .find("gpio_set_level(USBPHY_DM_NUM, 0)")
        .expect("physical D-/D+ disconnect");
    let drive_dp_low = USB_PHY_SOURCE
        .find("gpio_set_level(USBPHY_DP_NUM, 0)")
        .expect("physical D+ disconnect");
    let arm_reset = USB_PHY_SOURCE
        .find("usb_serial_jtag_ll_ena_intr_mask(USB_SERIAL_JTAG_INTR_BUS_RESET)")
        .expect("BUS_RESET observation arm");
    let reconnect = USB_PHY_SOURCE
        .find("SET_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_USB_PAD_ENABLE)")
        .expect("Serial/JTAG pad reconnect");
    let observed_reset = USB_PHY_SOURCE
        .find("usb_serial_jtag_ll_get_intraw_mask() & USB_SERIAL_JTAG_INTR_BUS_RESET")
        .expect("BUS_RESET observation");
    let timeout = USB_PHY_SOURCE
        .find("BUS_RESET_TIMEOUT_US")
        .expect("bounded BUS_RESET timeout");

    // Act / Assert
    assert!(disconnect < drive_low);
    assert!(drive_low < drive_dp_low);
    assert!(drive_dp_low < arm_reset);
    assert!(arm_reset < reconnect);
    assert!(reconnect < observed_reset);
    assert!(observed_reset < timeout || timeout < disconnect);
    let switch_result = USB_PHY_SOURCE
        .find("result = bitaxe_usb_switch_to_serial_jtag()")
        .expect("checked PHY switch result");
    let restart = USB_PHY_SOURCE.find("esp_restart()").expect("ESP restart");
    assert!(switch_result < restart);
}

#[test]
fn maintenance_commit_receipt_precedes_the_phy_restart() {
    // Arrange
    let receipt = USB_SOURCE
        .find("usb_maintenance={\\\"status\\\":\\\"committed\\\"}")
        .expect("committed maintenance receipt");
    let restart = USB_SOURCE
        .find("restart_into_rom_downloader")
        .expect("ROM restart call");

    // Act / Assert
    assert!(receipt < restart);
    let commit_failure = USB_SOURCE
        .find("usb_maintenance=failed category=commit_receipt")
        .expect("commit-receipt failure");
    let failure_return = USB_SOURCE[commit_failure..]
        .find("return;")
        .map(|offset| commit_failure + offset)
        .expect("commit-receipt failure return");
    assert!(commit_failure < failure_return);
    assert!(failure_return < restart);
}

#[test]
fn startup_recovers_before_optional_owners_and_starts_control_after_wifi() {
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
    let worker = run
        .find("start_deferred_usb_runtime(runtime_services.deferred_usb_runtime)")
        .expect("deferred BWG worker startup should exist");

    // Act / Assert
    assert!(baseline < recovery);
    assert!(recovery < production);
    assert!(network < worker);
    assert!(USB_SOURCE.contains("const OWNER_STACK_BYTES: usize = 16 * 1024;"));
    assert!(STARTUP_SOURCE.contains(
        "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL == 65_536"
    ));
    assert!(STARTUP_SOURCE.contains("usb_memory_checkpoint stage=worker_start"));
    assert!(STARTUP_SOURCE.contains("bwg_worker_start_failure category=startup_failed"));
    assert!(STARTUP_SOURCE.contains("bwg_worker_start_failure_detail(&error)"));
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
