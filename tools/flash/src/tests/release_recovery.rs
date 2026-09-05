use super::*;

const PLAN_DOCUMENT: &str =
    include_str!("../../../../docs/parity/work-plans/20260817T005227Z-REL-003/PLAN.md");

#[test]
fn legacy_release_recovery_stops_before_erase_when_restore_would_reset_nvs() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let (workspace, command) = release_fixture(&dir);
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());

    // Act
    let error = run_release_recovery(&command, &environment)
        .expect_err("legacy provisioning is not a state-preserving restore");

    // Assert
    assert!(
        format!("{error:#}").contains("ordinary_update_preserves_nvs_use_explicit_factory_reset")
    );
    assert!(environment.executed_commands().is_empty());
    assert!(environment.generated_nvs_partitions().is_empty());
    assert!(!workspace.join(RELEASE_RECOVERY_PROJECTION).exists());
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
    write_manifest_v4_contents(&manifest, DEFAULT_ELF_NAME, FACTORY_IMAGE_NAME);
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
