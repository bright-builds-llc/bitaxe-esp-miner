const BOOT_EVIDENCE_SOURCE: &str = include_str!("boot_evidence.rs");
const USB_PROFILE_SOURCE: &str = include_str!("boot_evidence/usb_profile.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn boot_identity_replay_keeps_the_ten_second_contract() {
    // Arrange
    let observer_start = BOOT_EVIDENCE_SOURCE
        .find("fn observe_boot_lifetime()")
        .expect("boot-lifetime observer must exist");
    let observer_end = BOOT_EVIDENCE_SOURCE[observer_start..]
        .find("fn runtime_attestation(")
        .map(|offset| observer_start + offset)
        .expect("observer boundary must exist");

    // Act
    let observer = &BOOT_EVIDENCE_SOURCE[observer_start..observer_end];

    // Assert
    assert!(observer.contains(
        "let mut identity_deadline_ms = started_at_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);",
    ));
    assert!(observer.contains(
        "identity_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);",
    ));
    assert_eq!(observer.matches("emit_boot_identity(").count(), 1);
}

#[test]
fn platform_readiness_avoids_the_full_operator_snapshot_stack() {
    // Arrange
    let function = "fn publish_platform_readiness(";
    let start = STARTUP_SOURCE
        .find(function)
        .expect("platform-readiness publisher must exist");
    let end = STARTUP_SOURCE[start..]
        .find("const fn readiness_label(")
        .map(|offset| start + offset)
        .expect("platform-readiness publisher boundary must exist");

    // Act
    let publisher = &STARTUP_SOURCE[start..end];

    // Assert
    assert!(publisher.contains("runtime_snapshot::collect_platform_readiness_snapshot()"));
    assert!(!publisher.contains("collect_api_snapshot()"));
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains(
        "collect_platform_snapshot(PlatformSnapshot::safe_ultra_205(), &platform_identity)",
    ));
}

#[test]
fn platform_readiness_precedes_blocking_wifi_admission() {
    // Arrange
    let run_start = STARTUP_SOURCE
        .find("pub(crate) fn run()")
        .expect("startup entrypoint must exist");
    let run_end = STARTUP_SOURCE[run_start..]
        .find("fn initialize_boot_identity_and_settings(")
        .map(|offset| run_start + offset)
        .expect("startup entrypoint boundary must exist");
    let run = &STARTUP_SOURCE[run_start..run_end];

    // Act
    let readiness = run
        .find("publish_platform_readiness(")
        .expect("platform readiness must be published");
    let network = run
        .find("start_network_services(")
        .expect("network admission must be visible at the startup boundary");
    let network_start = STARTUP_SOURCE
        .find("fn start_network_services(")
        .expect("network service boundary must exist");
    let network_end = STARTUP_SOURCE[network_start..]
        .find("fn start_storage_and_http(")
        .map(|offset| network_start + offset)
        .expect("network service boundary must end");
    let network_service = &STARTUP_SOURCE[network_start..network_end];

    // Assert
    assert!(readiness < network);
    assert!(network_service.contains("wifi_adapter::start_wifi(modem)"));
}

#[test]
fn usb_boot_profile_is_selected_once_and_replayed_by_the_boot_lifetime_owner() {
    // Arrange
    let observer_start = BOOT_EVIDENCE_SOURCE
        .find("fn observe_boot_lifetime()")
        .expect("boot-lifetime observer must exist");
    let observer_end = BOOT_EVIDENCE_SOURCE[observer_start..]
        .find("fn runtime_attestation(")
        .map(|offset| observer_start + offset)
        .expect("observer boundary must exist");
    let observer = &BOOT_EVIDENCE_SOURCE[observer_start..observer_end];

    // Act / Assert
    assert!(BOOT_EVIDENCE_SOURCE.contains("pub fn publish_usb_boot_profile("));
    assert!(observer.contains("usb_profile::emit_due(now_ms);"));
    assert!(USB_PROFILE_SOURCE.contains("UsbBootProfileReplay::new"));
    assert!(USB_PROFILE_SOURCE.contains("replay.maybe_take_due(now_ms)"));
    assert!(STARTUP_SOURCE.contains("UsbBootProfileReason::DiagnosticOwner"));
    assert!(STARTUP_SOURCE.contains("UsbBootProfileReason::WorkerStarted"));
    assert!(STARTUP_SOURCE.contains("UsbBootProfileReason::BootBaselineUnconfirmed"));
    assert_eq!(STARTUP_SOURCE.matches("publish_usb_boot_profile(").count(), 3);
}

#[test]
fn worker_mount_replays_the_closed_reboot_discriminator() {
    // Act / Assert
    assert!(BOOT_EVIDENCE_SOURCE.contains("pub fn worker_usb_boot_marker()"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("WorkerUsbBootMarker::new("));
}
