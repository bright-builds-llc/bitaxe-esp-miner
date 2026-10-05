use super::*;

#[test]
fn task_gate_admits_exact_enabled_line_in_active_block() {
    // Arrange
    let tasks = tasks("## Active", "USB reset endurance hardware: enabled.");

    // Act
    let result = admit_reset_endurance_task(&tasks);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn task_gate_refuses_missing_inexact_or_inactive_enablement() {
    for tasks in [
        tasks("## Active", "USB reset endurance hardware: disabled."),
        tasks("## Active", "USB reset endurance hardware: enabled"),
        tasks("## Future", "USB reset endurance hardware: enabled."),
        String::new(),
    ] {
        // Act
        let result = admit_reset_endurance_task(&tasks);

        // Assert
        assert!(result.is_err());
    }
}

#[test]
fn task_gate_refuses_duplicate_task_blocks() {
    // Arrange
    let enabled = tasks("## Active", "USB reset endurance hardware: enabled.");
    let duplicated = format!("{enabled}\n{enabled}");

    // Act
    let result = admit_reset_endurance_task(&duplicated);

    // Assert
    assert!(result.is_err());
}

#[test]
fn config_enforces_cycle_bounds() {
    for (cycles, valid) in [(0, false), (1, true), (500, true), (501, false)] {
        // Act
        let result = ResetEnduranceConfig::new(&physical(), COMMIT, &elf(), cycles, 20);

        // Assert
        assert_eq!(result.is_ok(), valid, "cycles={cycles}");
    }
}

#[test]
fn config_enforces_observe_timeout_bounds() {
    for (seconds, valid) in [(4, false), (5, true), (60, true), (61, false)] {
        // Act
        let result = ResetEnduranceConfig::new(&physical(), COMMIT, &elf(), 1, seconds);

        // Assert
        assert_eq!(result.is_ok(), valid, "seconds={seconds}");
    }
}

#[test]
fn config_rejects_non_canonical_identity_digests() {
    for (physical, commit) in [
        ("C".repeat(64), COMMIT.to_owned()),
        ("c".repeat(63), COMMIT.to_owned()),
        ("c".repeat(64), COMMIT.to_uppercase()),
        ("c".repeat(64), "0".repeat(39)),
    ] {
        // Act
        let result = ResetEnduranceConfig::new(&physical, &commit, &elf(), 1, 20);

        // Assert
        assert!(result.is_err());
    }
}

#[test]
fn scanner_completes_on_discriminator_and_identity_split_across_reads() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);
    let input = format!("{}{}", discriminator(7), identity_line(COMMIT));
    let (head, tail) = input.split_at(input.len() - 37);

    // Act
    let first = scanner.feed(head.as_bytes());
    let second = scanner.feed(tail.as_bytes());

    // Assert
    assert_eq!(
        (first, second),
        (IdentityVerdict::Pending, IdentityVerdict::Complete)
    );
}

#[test]
fn scanner_accepts_crlf_terminated_lines() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);
    let input = format!("{}{}", identity_line(COMMIT), discriminator(7)).replace('\n', "\r\n");

    // Act
    let verdict = scanner.feed(input.as_bytes());

    // Assert
    assert_eq!(verdict, IdentityVerdict::Complete);
}

#[test]
fn scanner_identity_without_discriminator_stays_pending() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);

    // Act
    let verdict = scanner.feed(identity_line(COMMIT).as_bytes());

    // Assert
    assert_eq!(
        (verdict, scanner.identity_observed()),
        (IdentityVerdict::Pending, true)
    );
}

#[test]
fn scanner_rejects_two_distinct_ordinals_in_one_window() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);
    let input = format!("{}{}", discriminator(7), discriminator(8));

    // Act
    let verdict = scanner.feed(input.as_bytes());

    // Assert
    assert_eq!(verdict, IdentityVerdict::OrdinalAmbiguous);
}

#[test]
fn scanner_reports_mismatch_for_a_different_valid_identity() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);

    // Act
    let verdict = scanner.feed(identity_line(&"a".repeat(40)).as_bytes());

    // Assert
    assert_eq!(verdict, IdentityVerdict::Mismatch);
}

#[test]
fn scanner_keeps_waiting_after_malformed_or_unterminated_identity() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);
    let unterminated = identity_line(COMMIT).trim_end().to_owned();

    // Act
    scanner.feed(b"usb_runtime_identity schema=v1 firmware_commit=broken\n");
    let verdict = scanner.feed(unterminated.as_bytes());

    // Assert
    assert_eq!(verdict, IdentityVerdict::Pending);
}

#[test]
fn scanner_records_the_first_reset_reason_and_every_byte() {
    // Arrange
    let expected = UsbRuntimeIdentity::new(COMMIT, &elf()).expect("identity");
    let mut scanner = IdentityScanner::new(&expected);
    let input = "usb_reboot_discriminator schema=v1 boot_ordinal=9 reset_reason=watchdog uptime_ms=1 redacted=true\nusb_reboot_discriminator schema=v1 boot_ordinal=9 reset_reason=panic uptime_ms=2 redacted=true\n";

    // Act
    scanner.feed(input.as_bytes());

    // Assert
    let boot = scanner.maybe_boot().expect("boot marker");
    assert_eq!(
        (
            boot.reset_reason().label(),
            boot.boot_ordinal(),
            scanner.bytes_observed()
        ),
        ("watchdog", 9, u64::try_from(input.len()).expect("length"))
    );
}
