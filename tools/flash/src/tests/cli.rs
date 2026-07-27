use super::*;

#[test]
fn parses_key_value_aliases_for_flash() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "flash",
        "board=205",
        "dry-run=true",
        "redact-evidence=true",
        "port=/dev/cu.usbmodem101",
        "image=/tmp/bitaxe-ultra205.elf",
    ];

    // Act
    let cli = parse_cli(args).expect("cli");

    // Assert
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    assert_eq!(command.common.board, BoardId::Ultra205);
    assert_eq!(command.common.port.as_deref(), Some("/dev/cu.usbmodem101"));
    assert!(command.common.dry_run);
    assert!(command.common.redact_evidence);
    assert_eq!(
        command.image.as_deref(),
        Some(Utf8Path::new("/tmp/bitaxe-ultra205.elf"))
    );
}

#[test]
fn phase36_flash_argument_shape_uses_supported_redacted_evidence() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "flash",
        "--board",
        "205",
        "--port",
        "/dev/private-device",
        "--manifest",
        "/tmp/package.json",
        "--image",
        "/tmp/factory.bin",
        "--redact-evidence",
        "--evidence-dir",
        "/tmp/private-stage",
        "--wifi-credentials",
        "/tmp/wifi.json",
    ];

    // Act
    let cli = parse_cli(args).expect("Phase 36 flash arguments should parse");

    // Assert
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    assert!(command.common.redact_evidence);
    assert_eq!(command.common.evidence_mode, None);
    assert_eq!(
        command.common.evidence_dir.as_deref(),
        Some(Utf8Path::new("/tmp/private-stage"))
    );
    assert_eq!(
        command.wifi_credentials.as_deref(),
        Some(Utf8Path::new("/tmp/wifi.json"))
    );
}

#[test]
fn phase36_retired_dual_plus_redaction_shape_fails_in_real_parser() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "flash",
        "--board",
        "205",
        "--port",
        "/dev/private-device",
        "--manifest",
        "/tmp/package.json",
        "--image",
        "/tmp/factory.bin",
        "--evidence-mode",
        "dual",
        "--redact-evidence",
        "--evidence-dir",
        "/tmp/private-stage",
        "--wifi-credentials",
        "/tmp/wifi.json",
    ];

    // Act
    let result = parse_cli(args);

    // Assert
    let error = result.expect_err("retired Phase 36 argument shape must fail");
    assert!(format!("{error:#}").contains("cannot be used with"));
}

#[test]
fn flash_monitor_parses_capture_timeout_alias() {
    // Arrange
    let hyphenated_args = [
        "bitaxe-flash",
        "flash-monitor",
        "port=/dev/cu.usbmodem101",
        "capture-timeout-seconds=30",
    ];
    let underscored_args = [
        "bitaxe-flash",
        "flash-monitor",
        "port=/dev/cu.usbmodem101",
        "capture_timeout_seconds=30",
    ];

    // Act
    let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
    let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

    // Assert
    let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
        panic!("expected flash-monitor command");
    };
    let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(hyphenated_command.capture_timeout_seconds, 30);
    assert_eq!(underscored_command.capture_timeout_seconds, 30);
}

#[test]
fn flash_monitor_parses_redact_evidence_aliases() {
    // Arrange
    let hyphenated_args = [
        "bitaxe-flash",
        "flash-monitor",
        "port=/dev/cu.usbmodem101",
        "redact-evidence=true",
    ];
    let underscored_args = [
        "bitaxe-flash",
        "flash-monitor",
        "port=/dev/cu.usbmodem101",
        "redact_evidence=true",
    ];

    // Act
    let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
    let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

    // Assert
    let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
        panic!("expected flash-monitor command");
    };
    let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
        panic!("expected flash-monitor command");
    };
    assert!(hyphenated_command.common.redact_evidence);
    assert!(underscored_command.common.redact_evidence);
}

#[test]
fn flash_monitor_parses_dual_evidence_mode_aliases() {
    // Arrange
    let hyphenated_args = [
        "bitaxe-flash",
        "flash-monitor",
        "evidence-dir=/tmp/evidence",
        "evidence-mode=dual",
    ];
    let underscored_args = [
        "bitaxe-flash",
        "flash-monitor",
        "evidence_dir=/tmp/evidence",
        "evidence_mode=dual",
    ];

    // Act
    let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
    let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

    // Assert
    let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
        panic!("expected flash-monitor command");
    };
    let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(
        hyphenated_command.common.evidence_mode,
        Some(EvidenceMode::Dual)
    );
    assert_eq!(
        underscored_command.common.evidence_mode,
        Some(EvidenceMode::Dual)
    );
}

#[test]
fn finalize_evidence_parses_software_only_inputs() {
    // Arrange
    let digest = "a".repeat(64);
    let args = [
        "bitaxe-flash".to_owned(),
        "finalize-evidence".to_owned(),
        "evidence_dir=scratch/private-evidence".to_owned(),
        format!("expected_private_sha256={digest}"),
    ];

    // Act
    let cli = parse_cli(args).expect("finalize cli");

    // Assert
    let CliCommand::FinalizeEvidence(command) = cli.command else {
        panic!("expected finalize-evidence command");
    };
    assert_eq!(
        command.evidence_dir,
        Utf8PathBuf::from("scratch/private-evidence")
    );
    assert_eq!(command.expected_private_sha256, digest);
}

#[test]
fn monitor_capture_states_project_to_the_legacy_wire_contract() {
    // Arrange
    let cases = [
        (
            MonitorCaptureState::NotRequested,
            (
                "not_applicable",
                CaptureStatus::DryRun,
                "not_requested",
                false,
            ),
        ),
        (
            MonitorCaptureState::DryRun,
            ("dry_run", CaptureStatus::DryRun, "not_captured", false),
        ),
        (
            MonitorCaptureState::Trusted {
                completion: TrustedCaptureCompletion::Completed,
                basis: MonitorTrustBasis::BootTranscript,
            },
            ("noninteractive", CaptureStatus::Completed, "trusted", true),
        ),
        (
            MonitorCaptureState::Trusted {
                completion: TrustedCaptureCompletion::Completed,
                basis: MonitorTrustBasis::RuntimeAttestation,
            },
            ("noninteractive", CaptureStatus::Completed, "trusted", true),
        ),
        (
            MonitorCaptureState::Trusted {
                completion: TrustedCaptureCompletion::TimedOut,
                basis: MonitorTrustBasis::BootTranscript,
            },
            (
                "noninteractive",
                CaptureStatus::TimedOutAfterTrustedOutput,
                "trusted",
                true,
            ),
        ),
        (
            MonitorCaptureState::Trusted {
                completion: TrustedCaptureCompletion::TimedOut,
                basis: MonitorTrustBasis::RuntimeAttestation,
            },
            (
                "noninteractive",
                CaptureStatus::TimedOutAfterTrustedOutput,
                "trusted",
                true,
            ),
        ),
        (
            MonitorCaptureState::PendingPrivateClassification,
            (
                "noninteractive",
                CaptureStatus::TimedOutPendingPrivateClassification,
                "pending_private_classification",
                false,
            ),
        ),
        (
            MonitorCaptureState::AdmittedPrivateClassification,
            (
                "noninteractive",
                CaptureStatus::TimedOutAfterPrivateClassification,
                "pending_private_classification",
                false,
            ),
        ),
        (
            MonitorCaptureState::Untrusted {
                timed_out: true,
                conclusion: "timeout".to_owned(),
            },
            (
                "noninteractive",
                CaptureStatus::TimedOutWithoutTrustedOutput,
                "untrusted",
                false,
            ),
        ),
        (
            MonitorCaptureState::Untrusted {
                timed_out: false,
                conclusion: "failure".to_owned(),
            },
            ("noninteractive", CaptureStatus::Failed, "untrusted", false),
        ),
    ];

    for (state, expected) in cases {
        // Act
        let projection = state.projection();

        // Assert
        assert_eq!(
            (
                projection.capture_mode,
                projection.capture_status,
                projection.monitor_evidence_status,
                projection.trusted_output,
            ),
            expected,
            "{state:?}"
        );
    }
}

#[test]
fn flash_monitor_rejects_conflicting_evidence_modes() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "flash-monitor",
        "--evidence-dir",
        "/tmp/evidence",
        "--evidence-mode",
        "dual",
        "--redact-evidence",
    ];

    // Act
    let result = parse_cli(args);

    // Assert
    let error = result.expect_err("conflicting modes");
    assert!(format!("{error:#}").contains("cannot be used with"));
}

#[test]
fn non_flash_monitor_commands_reject_dual_mode() {
    // Arrange
    let flash_args = [
        "bitaxe-flash",
        "flash",
        "--evidence-mode",
        "dual",
        "--evidence-dir",
        "/tmp/evidence",
    ];
    let monitor_args = [
        "bitaxe-flash",
        "monitor",
        "--evidence-mode",
        "dual",
        "--evidence-dir",
        "/tmp/evidence",
    ];

    // Act
    let flash_result = parse_cli(flash_args);
    let monitor_result = parse_cli(monitor_args);

    // Assert
    assert!(format!("{:#}", flash_result.expect_err("flash dual")).contains("only"));
    assert!(format!("{:#}", monitor_result.expect_err("monitor dual")).contains("only"));
}

#[test]
fn dual_console_value_never_exposes_operational_input() {
    // Arrange
    let operational = "/Users/operator/private.log --port /dev/cu.usbmodem101";

    // Act
    let dual_value = operational_console_value(operational, false);
    let legacy_value = operational_console_value(operational, true);

    // Assert
    assert_eq!(dual_value, PROTECTED_OPERATIONAL);
    assert_eq!(legacy_value, operational);
    assert!(!dual_value.contains("/Users"));
    assert!(!dual_value.contains("/dev/"));
}

#[test]
fn parses_wifi_credentials_aliases_for_flash_and_flash_monitor() {
    // Arrange
    let flash_args = [
        "bitaxe-flash",
        "flash",
        "port=/dev/cu.usbmodem101",
        "wifi-credentials=/tmp/wifi.json",
    ];
    let flash_monitor_args = [
        "bitaxe-flash",
        "flash-monitor",
        "port=/dev/cu.usbmodem101",
        "wifi_credentials=/tmp/wifi.json",
    ];

    // Act
    let flash_cli = parse_cli(flash_args).expect("flash cli");
    let flash_monitor_cli = parse_cli(flash_monitor_args).expect("flash-monitor cli");

    // Assert
    let CliCommand::Flash(flash_command) = flash_cli.command else {
        panic!("expected flash command");
    };
    let CliCommand::FlashMonitor(flash_monitor_command) = flash_monitor_cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(
        flash_command.wifi_credentials.as_deref(),
        Some(Utf8Path::new("/tmp/wifi.json"))
    );
    assert_eq!(
        flash_monitor_command.wifi_credentials.as_deref(),
        Some(Utf8Path::new("/tmp/wifi.json"))
    );
}
