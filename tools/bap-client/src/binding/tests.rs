use super::*;

const COMMIT: &str = "645b6953fcb67f381e51e8d522edc30088fcd8c2";
const ELF: &str = "2cd38c51f75ee90aed12d6e1c0f2ce6d919fdb5504aaf6811a74a64c832c2a11";

fn package() -> Package {
    Package {
        source_commit: COMMIT.to_owned(),
        app_elf_sha256: ELF.to_owned(),
    }
}

fn observer_report(ordinal: u64, uptime_ms: u64) -> String {
    format!(
        "usb_reboot_loop: stable_boot\nmarker_count: 9\nreconnect_count: 0\n\
         latest_boot_ordinal: {ordinal}\nlatest_reset_reason: software_cpu\n\
         latest_uptime_ms: {uptime_ms}\nrust_panic_receipt: missing\n\
         allocation_failure_receipt: missing\nruntime_firmware_commit: {COMMIT}\n\
         runtime_app_elf_sha256: {ELF}\nworker_start_failure_observed: false\n\
         runtime_identity: exact_match\n"
    )
}

fn runtime(ordinal: u64, uptime_ms: u64) -> RuntimeObservation {
    parse_runtime(&observer_report(ordinal, uptime_ms)).expect("observer report")
}

fn detector(digest: &str) -> DetectorObservation {
    parse_detector(&format!(
        "port: /dev/cu.usbmodem1101\nusb_profile: serial_jtag_runtime\n\
         physical_identity_sha256: {digest}\nexecution_owner: unknown\nrom_admitted: false\n"
    ))
    .expect("detector report")
}

fn probe(outcome: &str) -> Value {
    json!({"ordinal": 6, "outcome": outcome, "source": {"commit": COMMIT, "clean": true}})
}

fn judge(
    probe: &Value,
    after_digest: &str,
    before: &RuntimeObservation,
    after: &RuntimeObservation,
    host_elapsed_ms: u64,
) -> Option<&'static str> {
    let package = package();
    let detector_before = detector("aa");
    let detector_after = detector(after_digest);
    first_failure(&BindingInputs {
        package: &package,
        probe,
        detector_before: &detector_before,
        detector_after: &detector_after,
        runtime_before: before,
        runtime_after: after,
        host_elapsed_ms,
    })
}

#[test]
fn one_continuing_boot_of_the_package_binds_the_probe() {
    // Arrange
    let (before, after) = (runtime(3, 180_000), runtime(3, 240_000));
    // Act
    let maybe_failure = judge(&probe("complete"), "aa", &before, &after, 61_000);
    // Assert
    assert_eq!(maybe_failure, None);
}

#[test]
fn an_incomplete_probe_never_binds() {
    // Arrange
    let (before, after) = (runtime(3, 180_000), runtime(3, 240_000));
    // Act
    let maybe_failure = judge(
        &probe("stop_hardware_blocker"),
        "aa",
        &before,
        &after,
        60_000,
    );
    // Assert
    assert_eq!(maybe_failure, Some("probe_not_complete"));
}

#[test]
fn a_different_physical_device_is_refused() {
    // Arrange
    let (before, after) = (runtime(3, 180_000), runtime(3, 240_000));
    // Act
    let maybe_failure = judge(&probe("complete"), "bb", &before, &after, 60_000);
    // Assert
    assert_eq!(maybe_failure, Some("physical_identity_changed"));
}

#[test]
fn another_build_is_refused() {
    // Arrange
    let report = observer_report(3, 180_000).replace(ELF, &"c".repeat(64));
    let before = parse_runtime(&report).expect("observer report");
    // Act
    let maybe_failure = judge(
        &probe("complete"),
        "aa",
        &before,
        &runtime(3, 240_000),
        60_000,
    );
    // Assert
    assert_eq!(maybe_failure, Some("runtime_before_not_package"));
}

#[test]
fn a_reset_between_observations_is_refused() {
    // Arrange
    let (before, after) = (runtime(3, 180_000), runtime(4, 240_000));
    // Act
    let maybe_failure = judge(&probe("complete"), "aa", &before, &after, 60_000);
    // Assert
    assert_eq!(maybe_failure, Some("boot_changed"));
}

#[test]
fn a_power_cycle_that_restarts_ordinal_one_is_refused_by_uptime() {
    // Arrange: ordinal 1 before and after, but uptime restarted near zero.
    let (before, after) = (runtime(1, 180_000), runtime(1, 40_000));
    // Act
    let maybe_failure = judge(&probe("complete"), "aa", &before, &after, 60_000);
    // Assert
    assert_eq!(maybe_failure, Some("uptime_not_continuous"));
}

#[test]
fn an_uptime_advance_unlike_host_time_is_refused() {
    // Arrange
    let (before, after) = (runtime(3, 180_000), runtime(3, 200_000));
    // Act
    let maybe_failure = judge(&probe("complete"), "aa", &before, &after, 60_000);
    // Assert
    assert_eq!(maybe_failure, Some("uptime_not_continuous"));
}

#[test]
fn a_missing_runtime_identity_is_malformed_input_not_a_match() {
    // Arrange
    let report = observer_report(3, 180_000).replace(
        &format!("runtime_firmware_commit: {COMMIT}\nruntime_app_elf_sha256: {ELF}\n"),
        "runtime_identity: missing\n",
    );
    // Act
    let result = parse_runtime(&report);
    // Assert
    assert_eq!(result, Err("runtime_identity_missing"));
}

#[test]
fn a_dirty_package_never_binds() {
    // Arrange
    let manifest = json!({"source_commit": COMMIT, "app_elf_sha256": ELF,
        "build_identity": {"source_dirty": true}});
    // Act
    let result = parse_package(&manifest);
    // Assert
    assert_eq!(result, Err("package_not_clean"));
}

#[test]
fn a_clean_package_manifest_names_its_image() {
    // Arrange: the package manifest's shape, with cleanliness under build_identity.
    let manifest = json!({"source_commit": COMMIT, "app_elf_sha256": ELF,
        "build_identity": {"source_dirty": false}});
    // Act
    let result = parse_package(&manifest);
    // Assert
    assert_eq!(result, Ok(package()));
}

#[test]
fn the_record_carries_no_physical_identity() {
    // Arrange
    let (package, probe) = (package(), probe("complete"));
    let (before, after) = (runtime(3, 180_000), runtime(3, 240_000));
    let digest = "f".repeat(64);
    let observed = detector(&digest);
    let inputs = BindingInputs {
        package: &package,
        probe: &probe,
        detector_before: &observed,
        detector_after: &observed,
        runtime_before: &before,
        runtime_after: &after,
        host_elapsed_ms: 60_000,
    };
    // Act
    let text = record(&inputs).to_string();
    // Assert
    assert!(text.contains("\"outcome\":\"bound\"") && !text.contains(&digest));
}
