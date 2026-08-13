use super::*;

#[test]
fn parses_canonical_flags_for_flash() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "flash",
        "--board",
        "205",
        "--dry-run",
        "--redact-evidence",
        "--port",
        "/dev/cu.usbmodem101",
        "--image",
        "/tmp/bitaxe-ultra205.elf",
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
fn parses_closed_mining_campaign_flags() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "mining-campaign",
        "--stage",
        "live-share",
        "--profile",
        "conservative",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--manifest",
        "/tmp/package.json",
        "--wifi-credentials",
        "/tmp/wifi.json",
        "--pool-credentials",
        "/tmp/pool.json",
        "--evidence-dir",
        "/tmp/attempt-001",
        "--duration-seconds",
        "600",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("campaign cli");

    // Assert
    let CliCommand::MiningCampaign(command) = cli.command else {
        panic!("expected mining-campaign command");
    };
    assert_eq!(command.stage, MiningCampaignStage::LiveShare);
    assert_eq!(command.profile, Some(MiningCampaignProfile::Conservative));
    assert_eq!(command.duration_seconds, 600);
    assert_eq!(
        command.pool_credentials.as_deref(),
        Some(Utf8Path::new("/tmp/pool.json"))
    );
    assert!(command.redact_evidence);
}

#[test]
fn parses_private_identify_confirmation() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "confirm-identify",
        "--evidence-dir",
        "hardware-runs/api009/attempt-001/campaign",
        "--observation",
        "rendered",
    ];

    // Act
    let cli = parse_cli(args).expect("confirmation cli");

    // Assert
    let CliCommand::ConfirmIdentify(command) = cli.command else {
        panic!("expected confirm-identify command");
    };
    assert_eq!(command.observation, network::IdentifyObservation::Rendered);
    assert_eq!(
        command.evidence_dir,
        Utf8PathBuf::from("hardware-runs/api009/attempt-001/campaign")
    );
}

#[test]
fn parses_canonical_observation_campaign_flags() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "mining-campaign",
        "--stage",
        "observation",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--manifest",
        "/tmp/package.json",
        "--wifi-credentials",
        "/tmp/wifi.json",
        "--evidence-dir",
        "/tmp/attempt-011",
        "--duration-seconds",
        "360",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("observation campaign cli");

    // Assert
    let CliCommand::MiningCampaign(command) = cli.command else {
        panic!("expected mining-campaign command");
    };
    assert_eq!(command.stage, MiningCampaignStage::Observation);
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port.as_deref(), Some("/dev/cu.usbmodem101"));
    assert_eq!(
        command.manifest.as_deref(),
        Some(Utf8Path::new("/tmp/package.json"))
    );
    assert!(command.profile.is_none());
    assert!(command.pool_credentials.is_none());
    assert_eq!(command.duration_seconds, 360);
    assert!(command.redact_evidence);
}

#[test]
fn mining_campaign_parser_rejects_assignment_style_stage_token() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "mining-campaign",
        "stage=observation",
        "--wifi-credentials",
        "/tmp/wifi.json",
        "--evidence-dir",
        "/tmp/attempt-011",
        "--duration-seconds",
        "360",
        "--redact-evidence",
    ];

    // Act
    let error = parse_cli(args).expect_err("assignment-style stage must be rejected");

    // Assert
    let rendered = format!("{error:#}");
    assert!(rendered.contains("unexpected argument"));
    assert!(rendered.contains("stage=observation"));
}

#[test]
fn mining_campaign_parser_rejects_open_stage_or_profile_values() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "mining-campaign",
        "--stage",
        "unbounded",
        "--profile",
        "overclocked",
        "--wifi-credentials",
        "/tmp/wifi.json",
        "--evidence-dir",
        "/tmp/attempt-001",
        "--duration-seconds",
        "600",
        "--redact-evidence",
    ];

    // Act
    let error = parse_cli(args).expect_err("open campaign values must be rejected");

    // Assert
    let rendered = format!("{error:#}");
    assert!(rendered.contains("invalid value"));
    assert!(!rendered.contains("overclocked"));
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
fn flash_monitor_parses_only_canonical_flags() {
    // Arrange
    let canonical = [
        "bitaxe-flash",
        "flash-monitor",
        "--port",
        "/dev/cu.usbmodem101",
        "--capture-timeout-seconds",
        "30",
        "--redact-evidence",
    ];
    let legacy = [
        "bitaxe-flash",
        "flash-monitor",
        "capture_timeout_seconds=30",
    ];

    // Act
    let cli = parse_cli(canonical).expect("canonical cli");
    let legacy_error = parse_cli(legacy).expect_err("legacy syntax must fail");

    // Assert
    let CliCommand::FlashMonitor(command) = cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(command.capture_timeout_seconds, 30);
    assert!(command.common.redact_evidence);
    assert!(format!("{legacy_error:#}").contains("unexpected argument"));
}

#[test]
fn reconnect_probe_requires_wifi_credentials() {
    // Arrange
    let missing_credentials = ["bitaxe-flash", "flash-monitor", "--network-reconnect-probe"];
    let admitted = [
        "bitaxe-flash",
        "flash-monitor",
        "--network-reconnect-probe",
        "--wifi-credentials",
        "/tmp/wifi.json",
    ];

    // Act
    let error = parse_cli(missing_credentials).expect_err("credentials must be required");
    let cli = parse_cli(admitted).expect("probe cli");

    // Assert
    assert!(format!("{error:#}").contains("--wifi-credentials"));
    let CliCommand::FlashMonitor(command) = cli.command else {
        panic!("expected flash-monitor command");
    };
    assert!(command.network_reconnect_probe);
}

#[test]
fn thermal_fault_intent_requires_wifi_and_conflicts_with_reconnect_probe() {
    // Arrange
    let missing_credentials = [
        "bitaxe-flash",
        "flash-monitor",
        "--thermal-fault-stimulus-intent",
        THERMAL_FAULT_INTENT_RELATIVE_PATH,
    ];
    let conflicting = [
        "bitaxe-flash",
        "flash-monitor",
        "--thermal-fault-stimulus-intent",
        THERMAL_FAULT_INTENT_RELATIVE_PATH,
        "--network-reconnect-probe",
        "--wifi-credentials",
        "private-wifi.json",
    ];
    let admitted = [
        "bitaxe-flash",
        "flash-monitor",
        "--thermal-fault-stimulus-intent",
        THERMAL_FAULT_INTENT_RELATIVE_PATH,
        "--wifi-credentials",
        "private-wifi.json",
    ];

    // Act
    let missing = parse_cli(missing_credentials).expect_err("credentials are mandatory");
    let conflict = parse_cli(conflicting).expect_err("probe modes are exclusive");
    let cli = parse_cli(admitted).expect("closed thermal fault cli");

    // Assert
    assert!(format!("{missing:#}").contains("--wifi-credentials"));
    assert!(format!("{conflict:#}").contains("cannot be used with"));
    let CliCommand::FlashMonitor(command) = cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(
        command.thermal_fault_stimulus_intent.as_deref(),
        Some(Utf8Path::new(THERMAL_FAULT_INTENT_RELATIVE_PATH))
    );
}

#[test]
fn finalize_evidence_parses_software_only_inputs() {
    // Arrange
    let digest = "a".repeat(64);
    let args = [
        "bitaxe-flash".to_owned(),
        "finalize-evidence".to_owned(),
        "--evidence-dir".to_owned(),
        "scratch/private-evidence".to_owned(),
        "--expected-private-sha256".to_owned(),
        digest.clone(),
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
fn parses_canonical_wifi_credentials_for_flash_and_flash_monitor() {
    // Arrange
    let flash_args = [
        "bitaxe-flash",
        "flash",
        "--port",
        "/dev/cu.usbmodem101",
        "--wifi-credentials",
        "/tmp/wifi.json",
    ];
    let flash_monitor_args = [
        "bitaxe-flash",
        "flash-monitor",
        "--port",
        "/dev/cu.usbmodem101",
        "--wifi-credentials",
        "/tmp/wifi.json",
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
