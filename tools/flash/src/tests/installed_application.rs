use super::*;
use crate::installed_application::admit_start_installed_task;

fn fixture() -> (
    tempfile::TempDir,
    StartInstalledCommand,
    FakeFlashEnvironment,
) {
    let directory = tempfile::tempdir().expect("private fixture");
    let workspace = Utf8PathBuf::from_path_buf(directory.path().to_owned()).expect("UTF-8 path");
    set_private_directory_mode(&workspace).expect("private parent");
    let command = StartInstalledCommand {
        manifest: Some(write_manifest_v4(&directory, DEFAULT_ELF_NAME)),
        board: BoardId::Ultra205,
        port: "/dev/test-only".to_owned(),
        expected_source_commit: SOURCE_COMMIT.to_owned(),
        expected_app_elf_sha256: APP_ELF_SHA256.to_owned(),
        private_root: Utf8PathBuf::from("attempt"),
        redact_evidence: true,
    };
    let tasks = task(&command);
    fs::write(workspace.join("TASKS.md"), tasks).expect("fixture active task");
    let environment = FakeFlashEnvironment {
        workspace_dir: workspace,
        ..FakeFlashEnvironment::default()
    };
    (directory, command, environment)
}

fn task(_command: &StartInstalledCommand) -> String {
    "## Active\n### task-fixed-usb-serial-qualification | 2026-09-04 | Test\n\nNo-write effect contract: `just native-usb-start-installed --board 205 --port <fresh-port> --manifest <frozen-manifest> --expected-source-commit <package-source> --expected-app-elf-sha256 <package-elf> --private-root <new-private-child> --redact-evidence`.\n".to_owned()
}

fn transcript(source: &str, digest: &str) -> Vec<u8> {
    use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};
    format!("{}\nusb_runtime_identity schema=v1 firmware_commit={source} app_elf_sha256={digest} redacted=true\n{}\n",
        WorkerUsbBootMarker::new(2, ResetReasonCategory::SoftwareCpu, 500).marker(),
        WorkerUsbBootMarker::new(2, ResetReasonCategory::SoftwareCpu, 2500).marker()).into_bytes()
}

#[test]
fn exact_installed_identity_completes_without_any_write() {
    // Arrange
    let (_directory, command, mut environment) = fixture();
    environment.maybe_installed_bytes = Some(transcript(
        &command.expected_source_commit,
        &command.expected_app_elf_sha256,
    ));
    // Act
    let result = run_start_installed(&command, &environment);
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(environment.workspace_dir.join("attempt/result.json")).expect("evidence"),
    )
    .expect("JSON");
    // Assert
    assert!(result.is_ok());
    assert_eq!(evidence["terminal_category"], "complete");
    assert_eq!(evidence["cleanup_complete"], true);
    assert_eq!(
        evidence["runtime"]["observed_source_commit"],
        command.expected_source_commit
    );
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![0]);
    assert!(environment.executed_commands.borrow().is_empty());
}

#[test]
fn wrong_installed_identity_is_retained_and_fails_after_cleanup() {
    // Arrange
    let (_directory, command, mut environment) = fixture();
    let observed_source = "3".repeat(40);
    environment.maybe_installed_bytes = Some(transcript(
        &observed_source,
        &command.expected_app_elf_sha256,
    ));
    // Act
    let result = run_start_installed(&command, &environment);
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(environment.workspace_dir.join("attempt/result.json")).expect("evidence"),
    )
    .expect("JSON");
    // Assert
    assert!(result.is_err());
    assert_eq!(
        evidence["terminal_category"],
        "runtime_identity_missing_or_mismatched"
    );
    assert_eq!(evidence["cleanup_complete"], true);
    assert_eq!(evidence["runtime"]["identity_match"], false);
    assert_eq!(
        evidence["runtime"]["observed_source_commit"],
        observed_source
    );
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![0]);
}

#[test]
fn failed_exit_remains_primary_when_cleanup_also_fails() {
    // Arrange
    let (_directory, command, mut environment) = fixture();
    environment.application_exit_failure = true;
    environment.cleanup_failure = true;
    // Act
    let result = run_start_installed(&command, &environment);
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(environment.workspace_dir.join("attempt/result.json")).expect("evidence"),
    )
    .expect("JSON");
    // Assert
    assert!(result.is_err());
    assert_eq!(evidence["terminal_category"], "rom_exit_failed");
    assert_eq!(evidence["cleanup_complete"], false);
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![0]);
}

#[test]
fn task_contract_does_not_require_a_self_referential_future_build_hash() {
    // Arrange
    let (_directory, command, _environment) = fixture();
    // Act / Assert
    assert!(admit_start_installed_task(&task(&command)).is_ok());
}

#[test]
fn unrelated_task_cannot_supply_the_installed_identity() {
    // Arrange
    let (_directory, command, _environment) = fixture();
    let input = format!(
        "## Active\n### task-fixed-usb-serial-qualification | date\nNo contract here.\n{}",
        task(&command).replace("task-fixed-usb-serial-qualification", "task-unrelated")
    );
    // Act / Assert
    assert!(admit_start_installed_task(&input).is_err());
}

#[test]
fn future_or_duplicated_task_is_not_active_admission() {
    // Arrange
    let (_directory, command, _environment) = fixture();
    let active = task(&command);
    // Act / Assert
    assert!(admit_start_installed_task(&active.replace("## Active", "## Future")).is_err());
    assert!(admit_start_installed_task(&format!("{active}{active}")).is_err());
}

#[test]
fn malformed_identity_fails_before_root_or_session_creation() {
    // Arrange
    let (_directory, mut command, environment) = fixture();
    command.expected_source_commit = "not-an-identity".to_owned();
    // Act
    let result = run_start_installed(&command, &environment);
    // Assert
    assert!(result.is_err());
    assert_eq!(environment.installed_session_calls.get(), 0);
    assert!(!environment.workspace_dir.join("attempt").exists());
}

#[test]
fn existing_private_root_prevents_any_session() {
    // Arrange
    let (_directory, command, environment) = fixture();
    fs::create_dir(environment.workspace_dir.join("attempt")).expect("preexisting root");
    // Act
    let result = run_start_installed(&command, &environment);
    // Assert
    assert!(result.is_err());
    assert_eq!(environment.installed_session_calls.get(), 0);
}

#[test]
fn observation_failure_after_one_exit_is_not_retried_and_cleanup_is_recorded() {
    // Arrange
    let (_directory, command, environment) = fixture();
    // Act
    let result = run_start_installed(&command, &environment);
    let evidence = fs::read_to_string(environment.workspace_dir.join("attempt/result.json"))
        .expect("evidence");
    // Assert
    assert!(result.is_err());
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![0]);
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert!(environment.executed_commands.borrow().is_empty());
    assert!(evidence.contains("runtime_observation_failed"));
    assert!(!evidence.contains("/dev/test-only"));
    assert!(!evidence.contains("adapter_unavailable"));
}

#[test]
fn no_reset_rom_probe_cannot_write_flash_or_reset_the_device() {
    // Arrange / Act
    let args = installed_rom_probe_args("admitted");
    // Assert
    assert_eq!(
        args,
        [
            "board-info",
            "--chip",
            "esp32s3",
            "--port",
            "admitted",
            "--non-interactive",
            "--before",
            "no-reset",
            "--after",
            "no-reset"
        ]
    );
}

#[test]
fn failed_factory_write_does_not_start_the_application() {
    // Arrange
    let directory = tempfile::tempdir().expect("fixture");
    let command = FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(write_manifest_v4(&directory, DEFAULT_ELF_NAME)),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment {
        execute_failure: true,
        ..FakeFlashEnvironment::default()
    };
    // Act
    let result = run_flash(&command, &environment);
    // Assert
    assert!(result.is_err());
    assert!(environment
        .application_exit_write_counts
        .borrow()
        .is_empty());
}
