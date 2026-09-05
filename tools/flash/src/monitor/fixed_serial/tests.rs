use super::*;
fn identity() -> ExpectedRuntimeAttestationIdentity {
    ExpectedRuntimeAttestationIdentity {
        firmware_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        app_elf_sha256: "c".repeat(64),
    }
}
fn sample(ordinal: u64, uptime: u64, failure: &str) -> String {
    let id = identity();
    let profile = serde_json::json!({"schema_version":1,"transport":"serial_jtag_runtime","reason":"worker_started","baseline":"confirmed","firmware_commit":id.firmware_commit,"app_elf_sha256":id.app_elf_sha256,"boot_ordinal":ordinal});
    format!("usb_reboot_discriminator schema=v1 boot_ordinal={ordinal} reset_reason=other uptime_ms={uptime} redacted=true\nusb_runtime_identity schema=v1 firmware_commit={} app_elf_sha256={} redacted=true\nusb_boot_profile={profile}\nusb_startup schema=v1 stage=runtime_ready state=complete first_failure={failure} uptime_ms={uptime} redacted=true\n", id.firmware_commit, id.app_elf_sha256)
}
fn healthy() -> String {
    sample(7, 1000, "none") + &sample(7, 3000, "none")
}

#[test]
fn healthy_fixed_records_qualify_without_classic_transcript() {
    let assessment = assess(&healthy(), Some(&identity()));
    assert!(assessment.qualified(), "{assessment:?}");
}
#[test]
fn boot_noise_and_incomplete_tail_do_not_erase_complete_evidence() {
    let log = format!(
        "esp_image: partial ROM FIFO\n{}usb_startup schema=v1 stage=net",
        healthy()
    );
    assert!(assess(&log, Some(&identity())).qualified());
}
#[test]
fn prior_revision_receipts_do_not_become_current_failures() {
    let log = healthy() + "allocation_failure_context schema=v1 requested_bytes=852 capabilities=0000080c source_hash=af7d136df3c6596c stage=network redacted=true\nrust_panic_receipt schema=v1 file_hash=12345678 line=99 redacted=true\n";
    let assessment = assess(&log, Some(&identity()));
    assert!(assessment.qualified());
    assert!(assessment.retained_failure_history);
}
#[test]
fn attempt004_execution_is_present_but_reboot_and_network_failure_are_unqualified() {
    // Arrange
    let log = sample(2, 1000, "none")
        + &sample(3, 1000, "network").replace("reset_reason=other", "reset_reason=panic")
        + &sample(3, 27000, "network").replace("reset_reason=other", "reset_reason=panic")
        + "wifi_startup_failure schema=v1 phase=driver error=no_memory redacted=true\n";
    // Act
    let assessment = assess(&log, Some(&identity()));
    // Assert
    assert!(assessment.execution_present);
    assert!(assessment.startup_failed);
    assert!(!assessment.qualified());
    assert!(assessment
        .issues
        .contains(&FixedSerialIssue::RebootObserved));
    assert!(assessment
        .conclusion()
        .contains("execution present; startup failed"));
}
#[test]
fn wrong_or_abbreviated_firmware_never_qualifies() {
    for source in ["a".repeat(12), "d".repeat(40)] {
        let log = healthy().replace(&"a".repeat(40), &source);
        assert!(!assess(&log, Some(&identity())).qualified());
    }
}
#[test]
fn wrong_elf_never_qualifies() {
    assert!(!assess(
        &healthy().replace(&"c".repeat(64), &"d".repeat(64)),
        Some(&identity())
    )
    .qualified());
}
#[test]
fn mixed_old_and_current_identity_cannot_be_hidden_by_later_exact_records() {
    let log = sample(7, 1, "none").replace(&"a".repeat(40), &"d".repeat(40)) + &healthy();
    let assessment = assess(&log, Some(&identity()));
    assert!(assessment.execution_present);
    assert!(assessment.issues.contains(&FixedSerialIssue::MixedIdentity));
    assert!(!assessment.qualified());
}
#[test]
fn unconfirmed_baseline_cannot_be_promoted_by_complete_startup() {
    let log = healthy().replace("\"baseline\":\"confirmed\"", "\"baseline\":\"unconfirmed\"");
    assert!(!assess(&log, Some(&identity())).qualified());
}
#[test]
fn truncated_only_evidence_never_proves_execution() {
    let assessment = assess(
        "usb_runtime_identity schema=v1 firmware_commit=aaa",
        Some(&identity()),
    );
    assert!(!assessment.execution_present);
    assert!(!assessment.qualified());
}
#[test]
fn malformed_complete_record_rejects_otherwise_healthy_capture() {
    let log = healthy() + "usb_runtime_identity schema=v1 firmware_commit=broken\n";
    assert!(assess(&log, Some(&identity()))
        .issues
        .contains(&FixedSerialIssue::MalformedRecord));
}
#[test]
fn duplicate_time_samples_are_not_fresh_progress() {
    assert!(!assess(&sample(7, 1000, "none").repeat(2), Some(&identity())).qualified());
}
#[test]
fn backward_uptime_is_rejected() {
    let log = sample(7, 3000, "none") + &sample(7, 1000, "none");
    assert!(assess(&log, Some(&identity()))
        .issues
        .contains(&FixedSerialIssue::NonMonotonicUptime));
}
#[test]
fn startup_errors_are_not_erased_by_later_ready_samples() {
    let log = sample(7, 1, "network") + &healthy();
    assert!(assess(&log, Some(&identity())).startup_failed);
}
#[test]
fn absent_or_single_startup_completion_is_unqualified() {
    let no_startup = healthy()
        .lines()
        .filter(|line| !line.starts_with("usb_startup"))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    assert!(!assess(&no_startup, Some(&identity())).qualified());
    assert!(!assess(&sample(7, 1000, "none"), Some(&identity())).qualified());
}
#[test]
fn current_transmit_failure_remains_a_qualification_failure() {
    let log = healthy() + "usb_tx_failure schema=v1 stage=flush_timeout elapsed_ms=2000 queued_bytes=65000 record_bytes=65000 redacted=true\n";
    assert!(!assess(&log, Some(&identity())).qualified());
}
#[test]
fn no_package_identity_cannot_be_substituted_by_self_reported_markers() {
    assert!(!assess(&healthy(), None).qualified());
}

#[test]
fn completed_startup_cannot_regress_then_hide_behind_later_completion() {
    let log = healthy() + "usb_startup schema=v1 stage=hardware state=entered first_failure=none uptime_ms=4000 redacted=true\n" + &sample(7,5000,"none");
    assert!(!assess(&log, Some(&identity())).qualified());
}
#[test]
fn malformed_complete_heap_record_remains_unqualified() {
    let log = healthy() + "usb_memory_checkpoint stage=usb_install free_bytes=unknown largest_block_bytes=0 reserve_bytes=98304 redacted=true\n";
    assert!(assess(&log, Some(&identity()))
        .issues
        .contains(&FixedSerialIssue::MalformedRecord));
}

#[test]
fn explicit_storage_or_http_failure_cannot_hide_behind_complete_startup() {
    for suffix in [
        "storage_http_status schema=v1 spiffs_available=false http_ready=true redacted=true\n",
        "storage_http_status schema=v1 spiffs_available=true http_ready=false redacted=true\n",
        "storage_http_failure schema=v1 phase=http_routes error=handlers_full redacted=true\n",
    ] {
        let assessment = assess(&(healthy() + suffix), Some(&identity()));
        assert!(assessment.execution_present);
        assert!(assessment.startup_failed);
        assert!(!assessment.qualified());
    }
}
