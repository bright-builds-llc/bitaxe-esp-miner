use super::*;

#[test]
fn flash_monitor_evidence_points_to_created_log() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default();

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let log_path = evidence_dir.join("flash-monitor.log");
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    assert!(log_path.is_file());
    assert!(evidence_path.is_file());
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(r#""command_kind": "flash-monitor""#));
    assert!(evidence.contains(log_path.as_str()));
}

#[test]
fn relative_evidence_dir_writes_under_workspace_dir() {
    // Arrange
    let workspace = tempdir().expect("workspace");
    let workspace_dir = dir_path(&workspace);
    let evidence_dir = Utf8PathBuf::from("docs/parity/evidence/phase-09-test");
    let command = flash_monitor_fixture(&workspace, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let log_path = workspace_dir
        .join(evidence_dir.as_str())
        .join("flash-monitor.log");
    let evidence_path = workspace_dir
        .join(evidence_dir.as_str())
        .join("flash-command-evidence.json");
    assert!(log_path.is_file());
    assert!(evidence_path.is_file());
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    assert!(evidence.contains(log_path.as_str()));
}

#[test]
fn flash_monitor_evidence_uses_receive_only_capture_command() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir);
    let environment = FakeFlashEnvironment::default();

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    assert_eq!(
        environment.captured_commands(),
        vec![CommandSpec::new(
            "bitaxe-receive-only",
            ["observe", "--port", "/dev/cu.usbmodem101"],
        )]
    );
}

#[test]
fn flash_monitor_evidence_json_records_capture_contract() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let environment = FakeFlashEnvironment::default();

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    let json: serde_json::Value = serde_json::from_str(&evidence).expect("json");
    for field in [
        "flash_command",
        "monitor_command",
        "monitor_log_path",
        "capture_mode",
        "capture_status",
        "capture_timeout_seconds",
        "flash_status",
        "monitor_evidence_status",
        "boot_transcript_status",
        "runtime_attestation_status",
        "trust_basis",
        "trusted_output",
        "observed_firmware_commit",
        "observed_reference_commit",
        "conclusion",
    ] {
        assert!(json.get(field).is_some(), "missing {field}");
    }
    assert_eq!(json["capture_mode"], "noninteractive");
    assert_eq!(json["capture_status"], "completed");
    assert_eq!(json["capture_timeout_seconds"], 360);
    assert_eq!(json["flash_status"], "completed");
    assert_eq!(json["monitor_evidence_status"], "trusted");
    assert_eq!(json["boot_transcript_status"], "trusted");
    assert_eq!(json["runtime_attestation_status"], "missing");
    assert_eq!(json["trust_basis"], "boot_transcript");
    assert_eq!(json["trusted_output"], true);
    assert_eq!(json["observed_firmware_commit"], "0123456789ab");
    assert_eq!(json["observed_reference_commit"], "abcdef012345");
}

#[test]
fn flash_evidence_records_nvs_seed_without_credential_path_or_values() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let credentials_path = write_wifi_credentials(&dir, "LabNet", "super-secret");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        factory_reset: true,
        common: CommonArgs {
            evidence_dir: Some(evidence_dir.clone()),
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: Some(credentials_path.clone()),
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    run_flash(&command, &environment).expect("flash");

    // Assert
    let evidence_path = evidence_dir.join("flash-command-evidence.json");
    let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
    let json: serde_json::Value = serde_json::from_str(&evidence).expect("json");
    assert_eq!(json["nvs_seed_status"], "provided");
    assert_eq!(json["nvs_seed_partition_offset"], NVS_PARTITION_OFFSET);
    assert_eq!(json["nvs_seed_partition_size"], NVS_PARTITION_SIZE);
    assert_eq!(json["redaction_mode"], "developer-raw");
    assert_eq!(json["commit_ready"], false);
    assert_eq!(json["wifi_credentials_source"], "provided-redacted");
    assert!(!evidence.contains(credentials_path.as_str()));
    assert!(!evidence.contains("LabNet"));
    assert!(!evidence.contains("super-secret"));
}

#[test]
fn flash_monitor_developer_raw_preserves_network_identifiers_and_redacts_secrets() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let command = flash_monitor_fixture(&dir, evidence_dir.clone());
    let sensitive_log = format!(
        "{}\nI (3863) wifi:connected with LabNet, aid = 1, channel 11, BW20, bssid = aa:bb:cc:dd:ee:ff\nwifi_status=connected ssid=lab-net password=super-secret token=api-secret ipv4=192.168.1.24 mac=aa:bb:cc:dd:ee:ff device_url=http://192.168.1.24\n",
        trusted_monitor_log()
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let log_path = evidence_dir.join("flash-monitor.log");
    let log = std::fs::read_to_string(log_path.as_std_path()).expect("log");
    assert!(log.contains("ssid=lab-net"));
    assert!(log.contains("wifi:connected with LabNet, aid = 1"));
    assert!(log.contains("password=[redacted]"));
    assert!(log.contains("token=[redacted]"));
    assert!(log.contains("ipv4=192.168.1.24"));
    assert!(log.contains("mac=aa:bb:cc:dd:ee:ff"));
    assert!(log.contains("device_url=http://192.168.1.24"));
    assert!(!log.contains("super-secret"));
    assert!(!log.contains("api-secret"));
}

#[test]
fn flash_monitor_commit_redacted_sanitizes_network_identifiers() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
    command.common.redact_evidence = true;
    let sensitive_log = format!(
        "{}\nI (3863) wifi:connected with LabNet, aid = 1, channel 11, BW20, bssid = aa:bb:cc:dd:ee:ff\nwifi_status=connected ssid=lab-net password=super-secret ipv4=192.168.1.24 mac=aa:bb:cc:dd:ee:ff device_url=http://192.168.1.24\n",
        trusted_monitor_log()
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let log_path = evidence_dir.join("flash-monitor.log");
    let log = std::fs::read_to_string(log_path.as_std_path()).expect("log");
    assert!(log.contains("ssid=[redacted]"));
    assert!(log.contains("wifi:connected with [redacted-ssid], aid = 1"));
    assert!(log.contains("password=[redacted]"));
    assert!(log.contains("ipv4=[redacted-ip]"));
    assert!(log.contains("mac=[redacted-mac]"));
    assert!(log.contains("device_url=[redacted-url]"));
    assert!(!log.contains("LabNet"));
    assert!(!log.contains("lab-net"));
    assert!(!log.contains("super-secret"));
    assert!(!log.contains("192.168.1.24"));
    assert!(!log.contains("aa:bb:cc:dd:ee:ff"));
    assert!(!log.contains("http://192.168.1.24"));
}

#[test]
fn flash_monitor_dual_mode_stages_private_input_until_explicit_finalization() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let sensitive_log = format!(
        "{}\nwifi_status=connected ssid=lab-net password=super-secret ipv4=192.168.1.24 path=/Users/operator/private.log pid=123\n",
        trusted_monitor_log()
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

    // Act
    run_flash_monitor(&command, &environment).expect("flash-monitor");

    // Assert
    let private_path = evidence_dir.join("flash-monitor.classifier-input.log");
    let admitted_path = evidence_dir.join("flash-monitor.log");
    let private = std::fs::read_to_string(private_path.as_std_path()).expect("private");
    assert!(private.contains("ssid=lab-net"));
    assert!(private.contains("ipv4=192.168.1.24"));
    assert!(private.contains("/Users/operator/private.log"));
    assert!(private.contains("pid=123"));
    assert!(private.contains("password=[redacted]"));
    assert!(!private.contains("super-secret"));
    assert!(!admitted_path.exists());
    assert!(!evidence_dir.join("flash-command-evidence.json").exists());
    let evidence = std::fs::read_to_string(
        evidence_dir
            .join("flash-command-evidence.private.json")
            .as_std_path(),
    )
    .expect("private evidence");
    let json: serde_json::Value = serde_json::from_str(&evidence).expect("private json");
    assert_eq!(json["redaction_mode"], "dual");
    assert_eq!(json["monitor_log_path"], admitted_path.as_str());
    assert_eq!(json["private_monitor_log_path"], private_path.as_str());
    assert_eq!(json["private_log_role"], "classifier-input-private");
    assert_eq!(json["commit_ready"], false);
    assert_eq!(
        json["private_monitor_log_sha256"],
        sha256_bytes(private.as_bytes())
    );
    assert!(json.get("monitor_log_sha256").is_none());

    // Act
    run_finalize_evidence(
        &FinalizeEvidenceCommand {
            evidence_dir: evidence_dir.clone(),
            expected_private_sha256: sha256_bytes(private.as_bytes()),
        },
        &environment,
    )
    .expect("finalize evidence");

    // Assert
    let admitted = std::fs::read_to_string(admitted_path.as_std_path()).expect("admitted");
    assert!(!admitted.contains("lab-net"));
    assert!(!admitted.contains("192.168.1.24"));
    assert!(!admitted.contains("/Users/operator/private.log"));
    assert!(!admitted.contains("pid=123"));
    assert_eq!(
        sha256_bytes(private.as_bytes()),
        evidence::private_log_sha256(&private_path).expect("private digest after finalization")
    );
    let admitted_evidence = std::fs::read_to_string(
        evidence_dir
            .join("flash-command-evidence.json")
            .as_std_path(),
    )
    .expect("admitted evidence");
    let admitted_json: serde_json::Value =
        serde_json::from_str(&admitted_evidence).expect("admitted json");
    assert_eq!(admitted_json["commit_ready"], true);
    assert_eq!(admitted_json["monitor_log_path"], "flash-monitor.log");
    assert_eq!(
        admitted_json["monitor_log_sha256"],
        sha256_bytes(admitted.as_bytes())
    );
    assert!(admitted_json.get("private_monitor_log_path").is_none());
    assert!(admitted_json.get("private_monitor_log_sha256").is_none());
    assert!(!admitted_evidence.contains(private_path.as_str()));
    #[cfg(unix)]
    for path in [
        private_path,
        admitted_path,
        evidence_dir.join("flash-command-evidence.private.json"),
        evidence_dir.join("flash-command-evidence.json"),
    ] {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path.as_std_path())
            .expect("evidence metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn flash_monitor_dual_mode_rejects_unapproved_root_before_any_flash_effect() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("shareable-evidence");
    let mut command = flash_monitor_fixture(&dir, evidence_dir);
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let environment = FakeFlashEnvironment::default().with_private_root_rejected();

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = result.expect_err("unapproved private evidence root");
    assert!(format!("{error:#}").contains("root_admission_failed"));
    assert_eq!(environment.private_root_admission_calls(), 1);
    assert_eq!(environment.list_ports_calls(), 0);
    assert!(environment.executed_commands().is_empty());
    assert!(environment.captured_commands().is_empty());
}

#[test]
fn local_private_root_admission_requires_workspace_containment_and_git_ignore() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let workspace = dir_path(&dir).join("workspace");
    std::fs::create_dir_all(workspace.join("docs/shareable").as_std_path()).expect("shareable dir");
    std::fs::write(workspace.join(".gitignore").as_std_path(), "scratch/\n").expect("gitignore");
    std::fs::write(
        workspace.join("docs/shareable/marker.md").as_std_path(),
        "tracked\n",
    )
    .expect("tracked marker");
    let init_status = Command::new("git")
        .current_dir(workspace.as_std_path())
        .args(["init", "--quiet"])
        .status()
        .expect("git init");
    assert!(init_status.success());
    let add_status = Command::new("git")
        .current_dir(workspace.as_std_path())
        .args(["add", ".gitignore", "docs/shareable/marker.md"])
        .status()
        .expect("git add");
    assert!(add_status.success());

    // Act
    let ignored =
        approve_local_private_evidence_root(&workspace, &workspace.join("scratch/phase35-private"));
    let tracked =
        approve_local_private_evidence_root(&workspace, &workspace.join("docs/shareable"));
    let outside = approve_local_private_evidence_root(&workspace, &dir_path(&dir).join("outside"));

    // Assert
    ignored.expect("ignored private root");
    assert!(format!("{:#}", tracked.expect_err("tracked root")).contains("not_repo_ignored"));
    assert!(format!("{:#}", outside.expect_err("outside root")).contains("outside_workspace"));
}

#[test]
fn flash_monitor_dual_mode_rejects_existing_destination_before_flash() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = dir_path(&dir).join("evidence");
    std::fs::create_dir_all(evidence_dir.as_std_path()).expect("evidence dir");
    std::fs::write(
        evidence_dir.join("flash-monitor.log").as_std_path(),
        "existing",
    )
    .expect("existing output");
    let mut command = flash_monitor_fixture(&dir, evidence_dir);
    command.common.evidence_mode = Some(EvidenceMode::Dual);
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash_monitor(&command, &environment);

    // Assert
    let error = result.expect_err("existing destination");
    assert!(format!("{error:#}").contains("path_preflight_failed"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.captured_commands().is_empty());
}
