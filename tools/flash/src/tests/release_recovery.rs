use super::*;

const PLAN_DOCUMENT: &str =
    include_str!("../../../../docs/parity/work-plans/20260817T005227Z-REL-003/PLAN.md");

#[test]
fn release_recovery_erases_once_then_restores_exact_safe_package() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let (workspace, command) = release_fixture(&dir);
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());

    // Act
    run_release_recovery(&command, &environment).expect("release recovery");

    // Assert
    let executed = environment.executed_commands();
    assert_eq!(executed[0], release_erase_command("/dev/cu.usbmodem101"));
    assert_eq!(
        executed
            .iter()
            .filter(|command| command.args.first().map(String::as_str) == Some("erase-flash"))
            .count(),
        1
    );
    assert_eq!(
        executed
            .iter()
            .filter(|command| command.args.first().map(String::as_str) == Some("write-bin"))
            .count(),
        2
    );
    assert_eq!(environment.cleanup_calls(), 1);
    let projection =
        std::fs::read_to_string(workspace.join(RELEASE_RECOVERY_PROJECTION).as_std_path())
            .expect("projection");
    let evidence: ReleaseRecoveryEvidence = serde_json::from_str(&projection).expect("evidence");
    assert_eq!(evidence.validate(), Ok(()));
    assert!(!evidence.recovery_flash_used);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(workspace.join(RELEASE_RECOVERY_PROJECTION).as_std_path())
                .expect("projection metadata")
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
    }
}

#[test]
fn factory_failure_uses_one_recovery_attempt_and_withholds_projection() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let (workspace, command) = release_fixture(&dir);
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(workspace.clone())
        .with_execute_failure_offset("0x0");

    // Act
    let error = run_release_recovery(&command, &environment)
        .expect_err("factory failure should close the attempt");

    // Assert
    assert!(format!("{error:#}").contains("restore_or_runtime_proof_failed"));
    assert_eq!(
        environment
            .executed_commands()
            .iter()
            .filter(|command| {
                command.args.first().map(String::as_str) == Some("write-bin")
                    && command.args.iter().any(|argument| argument == "0x0")
            })
            .count(),
        2
    );
    assert_eq!(environment.cleanup_calls(), 2);
    assert!(!workspace.join(RELEASE_RECOVERY_PROJECTION).exists());
    let result: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace
                .join(RELEASE_RECOVERY_PRIVATE_ROOT)
                .join("release-recovery-result.json")
                .as_std_path(),
        )
        .expect("failure result"),
    )
    .expect("failure result JSON");
    assert_eq!(result["recovery_flash_used"], true);
    assert_eq!(result["recovery_complete"], false);
    assert_eq!(result["cleanup_complete"], true);
}

#[test]
fn missing_runtime_proof_after_completed_restore_never_reflashes() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let (workspace, command) = release_fixture(&dir);
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(workspace.clone())
        .with_log_contents("untrusted runtime output\n");

    // Act
    let error = run_release_recovery(&command, &environment)
        .expect_err("missing runtime proof should close the attempt");

    // Assert
    assert!(format!("{error:#}").contains("restore_or_runtime_proof_failed"));
    assert_eq!(
        environment
            .executed_commands()
            .iter()
            .filter(|command| {
                command.args.first().map(String::as_str) == Some("write-bin")
                    && command.args.iter().any(|argument| argument == "0x0")
            })
            .count(),
        1
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(!workspace.join(RELEASE_RECOVERY_PROJECTION).exists());
    let result: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace
                .join(RELEASE_RECOVERY_PRIVATE_ROOT)
                .join("release-recovery-result.json")
                .as_std_path(),
        )
        .expect("failure result"),
    )
    .expect("failure result JSON");
    assert_eq!(result["terminal_category"], "runtime_proof_failed");
    assert_eq!(result["recovery_flash_used"], false);
}

#[test]
fn wifi_seed_failure_after_factory_restore_never_reflashes() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let (workspace, command) = release_fixture(&dir);
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(workspace.clone())
        .with_execute_failure_offset(NVS_PARTITION_OFFSET);

    // Act
    let error = run_release_recovery(&command, &environment)
        .expect_err("Wi-Fi seed failure should close the attempt");

    // Assert
    assert!(format!("{error:#}").contains("restore_or_runtime_proof_failed"));
    assert_eq!(
        environment
            .executed_commands()
            .iter()
            .filter(|command| {
                command.args.first().map(String::as_str) == Some("write-bin")
                    && command.args.iter().any(|argument| argument == "0x0")
            })
            .count(),
        1
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(!workspace.join(RELEASE_RECOVERY_PROJECTION).exists());
    let result: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace
                .join(RELEASE_RECOVERY_PRIVATE_ROOT)
                .join("release-recovery-result.json")
                .as_std_path(),
        )
        .expect("failure result"),
    )
    .expect("failure result JSON");
    assert_eq!(result["terminal_category"], "wifi_seed_restore_failed");
    assert_eq!(result["recovery_flash_used"], false);
}

fn release_fixture(dir: &TempDir) -> (Utf8PathBuf, ReleaseRecoveryCommand) {
    let workspace = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 workspace");
    let plan = workspace.join(RELEASE_RECOVERY_PLAN);
    write_mode_file(&plan, PLAN_DOCUMENT, 0o644);
    write_mode_file(
        &workspace.join("TASKS.md"),
        &format!(
            "### {RELEASE_RECOVERY_TASK} | fixture\nPlan: `{RELEASE_RECOVERY_PLAN}`.\n\
             Run `capture-release-recovery-evidence` to erase the complete flash; never reuse it or erase again.\n"
        ),
        0o644,
    );
    let wrapper = workspace.join("scratch/rel003-large-erase/wrapper-001");
    std::fs::create_dir_all(wrapper.as_std_path()).expect("create wrapper");
    set_private_directory_mode(&wrapper).expect("wrapper mode");
    write_mode_file(
        &wrapper.join("detector.stdout"),
        "configuration_candidate: test\nusb_session: ready\nport: /dev/cu.usbmodem101\n",
        0o600,
    );
    let manifest = workspace.join(RELEASE_RECOVERY_MANIFEST);
    write_manifest_v3_contents(&manifest, DEFAULT_ELF_NAME, FACTORY_IMAGE_NAME);
    let credential_fixture = write_wifi_credentials(dir, "LabNet", "super-secret");
    let credentials = workspace.join(RELEASE_RECOVERY_WIFI_CREDENTIALS);
    std::fs::rename(credential_fixture.as_std_path(), credentials.as_std_path())
        .expect("rename credentials fixture");
    let command = ReleaseRecoveryCommand {
        board: BoardId::Ultra205,
        private_root: Utf8PathBuf::from(RELEASE_RECOVERY_PRIVATE_ROOT),
        package_manifest: Utf8PathBuf::from(RELEASE_RECOVERY_MANIFEST),
        wifi_credentials: Utf8PathBuf::from(RELEASE_RECOVERY_WIFI_CREDENTIALS),
        detector_output: Utf8PathBuf::from(RELEASE_RECOVERY_DETECTOR_OUTPUT),
        plan: Utf8PathBuf::from(RELEASE_RECOVERY_PLAN),
        projection: Utf8PathBuf::from(RELEASE_RECOVERY_PROJECTION),
        capture_timeout_seconds: 360,
    };
    (workspace, command)
}

fn write_mode_file(path: &Utf8Path, contents: &str, mode: u32) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent.as_std_path()).expect("create parent");
    }
    std::fs::write(path.as_std_path(), contents).expect("write fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path.as_std_path(), std::fs::Permissions::from_mode(mode))
            .expect("set mode");
    }
}
