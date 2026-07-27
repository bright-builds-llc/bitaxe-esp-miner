use super::*;

#[test]
fn trusted_timeout_capture_is_accepted() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment =
        FakeFlashEnvironment::default().with_capture_status(CaptureProcessStatus::TimedOut);

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    assert!(result.is_ok());
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""capture_status": "timed_out_after_trusted_output""#));
}

#[test]
fn untrusted_timeout_capture_fails_after_writing_json() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default()
        .with_capture_status(CaptureProcessStatus::TimedOut)
        .with_log_contents("untrusted monitor log\n");

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("evidence capture failed and is not trusted"));
    assert!(error.contains("flash_status=completed"));
    assert!(error.contains("monitor_evidence_status=untrusted"));
    assert!(!error.contains("rerun: just flash-monitor"));
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""capture_status": "timed_out_without_trusted_output""#));
}

#[test]
fn late_attached_exact_runtime_attestation_is_accepted_without_boot_transcript() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let late_attach_log = runtime_attestation_log();
    let environment = FakeFlashEnvironment::default()
        .with_capture_status(CaptureProcessStatus::TimedOut)
        .with_log_contents(&late_attach_log);

    // Act
    run_flash_monitor(&command, &environment).expect("runtime attestation should be trusted");

    // Assert
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    let json: serde_json::Value = serde_json::from_str(&evidence).expect("evidence JSON");
    assert_eq!(json["flash_status"], "completed");
    assert_eq!(json["boot_transcript_status"], "missing");
    assert_eq!(json["runtime_attestation_status"], "trusted");
    assert_eq!(json["trust_basis"], "runtime_attestation");
    assert_eq!(json["trusted_output"], true);
}

#[test]
fn dual_timeout_defers_late_attach_evidence_to_private_classifier() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let late_attach_log = [
        "runtime_boot_identity session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 reset_reason=power_on uptime_ms=10000 redacted=true",
        "runtime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 device_url=http://fixture-target redacted=true",
        "runtime_boot_identity session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 reset_reason=power_on uptime_ms=20000 redacted=true",
    ]
    .join("\n");
    let environment = FakeFlashEnvironment::default()
        .with_capture_status(CaptureProcessStatus::TimedOut)
        .with_log_contents(&late_attach_log);

    // Act
    run_flash_monitor(&command, &environment).expect("deferred dual capture");

    // Assert
    let private_path = evidence_dir.join("flash-monitor.classifier-input.log");
    let private = fs::read_to_string(private_path.as_std_path()).expect("private log");
    bitaxe_api::phase33_evidence::classify_phase33_baseline(&private)
        .expect("authoritative classification");
    assert!(!evidence_dir.join("flash-monitor.log").exists());
    assert!(!evidence_dir.join("flash-command-evidence.json").exists());
    let private_record = fs::read_to_string(
        evidence_dir
            .join("flash-command-evidence.private.json")
            .as_std_path(),
    )
    .expect("private record");
    let private_json: serde_json::Value =
        serde_json::from_str(&private_record).expect("private JSON");
    assert_eq!(
        private_json["capture_status"],
        "timed_out_pending_private_classification"
    );
    assert_eq!(private_json["trusted_output"], false);
    assert_eq!(private_json["commit_ready"], false);

    // Act
    run_finalize_evidence(
        &FinalizeEvidenceCommand {
            evidence_dir: evidence_dir.clone(),
            expected_private_sha256: sha256_bytes(private.as_bytes()),
        },
        &environment,
    )
    .expect("finalize classified evidence");

    // Assert
    let admitted_record = fs::read_to_string(
        evidence_dir
            .join("flash-command-evidence.json")
            .as_std_path(),
    )
    .expect("admitted record");
    let admitted_json: serde_json::Value =
        serde_json::from_str(&admitted_record).expect("admitted JSON");
    assert_eq!(
        admitted_json["capture_status"],
        "timed_out_after_private_classification"
    );
    assert_eq!(admitted_json["trusted_output"], false);
    assert_eq!(admitted_json["commit_ready"], true);
}

#[test]
fn dual_mode_does_not_defer_untrusted_process_failure() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let environment = FakeFlashEnvironment::default()
        .with_capture_status(CaptureProcessStatus::ExitedFailure(
            "exit status 1".to_owned(),
        ))
        .with_log_contents(
            "runtime_boot_identity session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 reset_reason=power_on uptime_ms=10000 redacted=true\n",
        );

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!(
        "{:#}",
        result.expect_err("failed child must remain terminal")
    );
    assert!(error.contains("dual_evidence=failed reason=capture_not_accepted"));
    assert!(!evidence_dir.join("flash-monitor.log").exists());
    assert!(!evidence_dir.join("flash-command-evidence.json").exists());
}

#[test]
fn finalize_evidence_rejects_contradictory_legacy_capture_fields() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let environment = FakeFlashEnvironment::default()
        .with_capture_status(CaptureProcessStatus::TimedOut)
        .with_log_contents("late private capture without trusted markers\n");
    run_flash_monitor(&command, &environment).expect("stage private evidence");
    let private_log = evidence_dir.join("flash-monitor.classifier-input.log");
    let private_record = evidence_dir.join("flash-command-evidence.private.json");
    let mut json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(private_record.as_std_path()).expect("private record"),
    )
    .expect("private JSON");
    json["trusted_output"] = serde_json::Value::Bool(true);
    fs::write(
        private_record.as_std_path(),
        serde_json::to_vec_pretty(&json).expect("encode mutated private record"),
    )
    .expect("mutate private record");

    // Act
    let result = run_finalize_evidence(
        &FinalizeEvidenceCommand {
            evidence_dir: evidence_dir.clone(),
            expected_private_sha256: evidence::private_log_sha256(&private_log)
                .expect("private digest"),
        },
        &environment,
    );

    // Assert
    let error = format!(
        "{:#}",
        result.expect_err("contradictory capture fields must fail")
    );
    assert!(error.contains("private_record_invalid_state"));
    assert!(!evidence_dir.join("flash-monitor.log").exists());
    assert!(!evidence_dir.join("flash-command-evidence.json").exists());
}

#[test]
fn stale_firmware_commit_capture_fails_after_writing_json() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let stale_log = trusted_monitor_log().replace(
        "firmware_commit=0123456789ab",
        "firmware_commit=fedcba987654",
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&stale_log);

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("observed firmware_commit=fedcba987654"));
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""trusted_output": false"#));
    assert!(evidence.contains(r#""observed_firmware_commit": "fedcba987654""#));
}

#[test]
fn truncated_firmware_commit_capture_fails_after_writing_json() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let truncated_log =
        trusted_monitor_log().replace("firmware_commit=0123456789ab", "firmware_commit=0");
    let environment = FakeFlashEnvironment::default().with_log_contents(&truncated_log);

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("observed firmware_commit=0"));
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""trusted_output": false"#));
    assert!(evidence.contains(r#""observed_firmware_commit": "0""#));
}

#[test]
fn prefixed_firmware_commit_marker_capture_fails_after_writing_json() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let prefixed_log = trusted_monitor_log().replace(
        "firmware_commit=0123456789ab",
        "not_firmware_commit=0123456789ab",
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&prefixed_log);

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("missing trusted Ultra 205 boot markers"));
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""trusted_output": false"#));
    assert!(evidence.contains(r#""observed_firmware_commit": "Unavailable""#));
}

#[test]
fn monitor_failure_guidance_uses_repo_commands() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default().with_capture_status(
        CaptureProcessStatus::ExitedFailure("exit status 1".to_owned()),
    );

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("just detect-ultra205"));
    assert!(error.contains("just monitor port=/dev/cu.usbmodem101"));
    assert!(error.contains(&format!("evidence_dir={evidence_dir}")));
    assert!(error.contains("do not reflash automatically"));
    assert!(!error.contains("rerun: just flash-monitor"));
    let raw_timeout_command = ["timeout", "25", "espflash"].join(" ");
    assert!(!error.contains(&raw_timeout_command));
}

#[test]
fn write_failure_is_terminal_before_monitor_or_completed_flash_evidence() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default().with_execute_failure();

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = format!(
        "{:#}",
        result.expect_err("write failure must remain terminal")
    );
    assert!(error.contains("sentinel child failure"));
    assert!(!error.contains("flash_status=completed"));
    assert!(environment.captured_commands().is_empty());
    assert!(!evidence_dir.join("flash-command-evidence.json").exists());
}

#[test]
fn rejects_deferred_gamma_601_board() {
    // Arrange
    let input = "601";

    // Act
    let result = input.parse::<BoardId>();

    // Assert
    let error = result.expect_err("deferred board");
    assert!(error.contains("deferred"));
}

#[test]
fn accepts_ultra_205_board() {
    // Arrange
    let input = "205";

    // Act
    let result = input.parse::<BoardId>();

    // Assert
    assert_eq!(result.expect("board"), BoardId::Ultra205);
}

#[test]
fn detect_preserves_explicit_port_alias() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "detect",
        "board=205",
        "port=/dev/cu.usbmodem101",
    ];

    // Act
    let cli = parse_cli(args).expect("detect command");

    // Assert
    let CliCommand::Detect(command) = cli.command else {
        panic!("expected detect command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port.as_deref(), Some("/dev/cu.usbmodem101"));
}

#[test]
fn monitor_defaults_to_hardware_safe_capture_budget() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "monitor",
        "board=205",
        "port=/dev/cu.usbmodem101",
    ];

    // Act
    let cli = parse_cli(args).expect("monitor command");

    // Assert
    let CliCommand::Monitor(command) = cli.command else {
        panic!("expected monitor command");
    };
    assert_eq!(command.capture_timeout_seconds, 360);
}
