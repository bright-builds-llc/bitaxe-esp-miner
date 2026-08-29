const MAIN_SOURCE: &str = include_str!("main.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const NVS_SOURCE: &str = include_str!("bwg_worker_nvs.rs");
const SESSION_SOURCE: &str = include_str!("bwg_worker_session.rs");
const USB_SOURCE: &str = include_str!("bwg_worker_usb.rs");
const USB_NATIVE_SOURCE: &str = include_str!("../bwg/native/bwg_usb.c");

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
fn tinyusb_descriptor_keeps_control_and_receive_only_evidence_disjoint() {
    // Arrange / Act
    let source = USB_NATIVE_SOURCE;

    // Assert
    for required in [
        "TUSB_CLASS_VENDOR_SPECIFIC, 0x42, 0x01",
        "0x01, TUSB_XFER_BULK",
        "0x81, TUSB_XFER_BULK",
        "TUD_CDC_DESCRIPTOR(BWG_INTERFACE_CDC",
        "0x82, 8, 0x03, 0x83, 64",
        "tud_cdc_n_read(interface, discarded",
    ] {
        assert!(source.contains(required), "missing descriptor contract: {required}");
    }
    assert!(!source.contains("bwg_worker_usb_vendor_received(discarded"));
}

#[test]
fn startup_owns_one_bwg_control_service_after_the_production_owner() {
    // Arrange
    let production = STARTUP_SOURCE
        .find("production_mining_session::start()")
        .expect("production owner startup should exist");
    let worker = STARTUP_SOURCE
        .find("bwg_worker_usb::start()")
        .expect("BWG worker startup should exist");

    // Act / Assert
    assert!(production < worker);
    assert_eq!(MAIN_SOURCE.matches("mod bwg_worker_usb;").count(), 1);
}
