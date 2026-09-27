use super::*;
use crate::core_dump::{run, CoreDumpReadCommand};

fn fixture() -> (TempDir, CoreDumpReadCommand, FakeFlashEnvironment) {
    let directory = tempdir().expect("fixture");
    let workspace = Utf8PathBuf::from_path_buf(
        fs::canonicalize(directory.path()).expect("canonical private fixture"),
    )
    .expect("UTF-8");
    set_private_directory_mode(&workspace).expect("private parent");
    let initialized = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&workspace)
        .status()
        .expect("initialize isolated fixture repo");
    assert!(initialized.success());
    fs::write(workspace.join(".git/info/exclude"), "*\n").expect("ignore private fixture files");
    fs::write(workspace.join("TASKS.md"), "## Active\n### task-str005-start-panic-diagnosis | fixture\nDevelopment core-dump acquisition: enabled (recovery evidence prerequisite satisfied).\n").expect("fixture task");
    let command = CoreDumpReadCommand {
        board: BoardId::Ultra205,
        port: "/dev/test-only".to_owned(),
        expected_physical_sha256: "6".repeat(64),
        expected_installed_source: SOURCE_COMMIT.to_owned(),
        expected_installed_elf: APP_ELF_SHA256.to_owned(),
        private_root: Utf8PathBuf::from("attempt"),
    };
    use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};
    let transcript = format!("{}\nusb_runtime_identity schema=v1 firmware_commit={SOURCE_COMMIT} app_elf_sha256={APP_ELF_SHA256} redacted=true\n{}\n",
        WorkerUsbBootMarker::new(2, ResetReasonCategory::SoftwareCpu, 500).marker(),
        WorkerUsbBootMarker::new(2, ResetReasonCategory::SoftwareCpu, 2500).marker());
    let environment = FakeFlashEnvironment {
        workspace_dir: workspace,
        maybe_installed_bytes: Some(transcript.into_bytes()),
        execute_failure: true,
        ..FakeFlashEnvironment::default()
    };
    (directory, command, environment)
}

fn receipt(environment: &FakeFlashEnvironment) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(
            environment
                .workspace_dir
                .join("attempt/result.private.json"),
        )
        .expect("private receipt"),
    )
    .expect("JSON receipt")
}

#[test]
fn core_dump_read_failure_returns_application_and_releases_session() {
    // Arrange
    let (_directory, command, environment) = fixture();
    // Act
    let outcome = run(&command, &environment);
    let record = receipt(&environment);
    // Assert
    assert!(outcome.is_err());
    assert_eq!(record["first_failure_stage"], "partition_table_read");
    assert_eq!(record["application_identity_restored"], true);
    assert_eq!(record["cleanup_complete"], true);
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(*environment.capture_lifecycle.borrow(), ["reset"]);
    let commands = environment.executed_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].args[0], "read-flash");
    assert!(commands[0]
        .args
        .windows(2)
        .any(|pair| pair == ["0x8000", "0x1000"]));
    assert!(!commands[0]
        .args
        .iter()
        .any(|arg| arg.contains("erase") || arg.contains("write")));
}

#[test]
fn core_dump_failed_return_still_releases_and_preserves_read_failure() {
    // Arrange
    let (_directory, command, mut environment) = fixture();
    environment.application_exit_failure = true;
    // Act
    let outcome = run(&command, &environment);
    let record = receipt(&environment);
    // Assert
    assert!(outcome.is_err());
    assert_eq!(record["first_failure_stage"], "partition_table_read");
    assert_eq!(record["application_identity_restored"], false);
    assert_eq!(record["cleanup_complete"], true);
    assert_eq!(environment.cleanup_calls.get(), 1);
}

#[test]
fn core_dump_physical_mismatch_never_reads_or_resets() {
    // Arrange
    let (_directory, mut command, environment) = fixture();
    command.expected_physical_sha256 = "7".repeat(64);
    // Act
    let outcome = run(&command, &environment);
    let record = receipt(&environment);
    // Assert
    assert!(outcome.is_err());
    assert_eq!(record["first_failure_stage"], "physical_identity");
    assert_eq!(record["rom_admitted"], false);
    assert!(environment.executed_commands().is_empty());
    assert!(environment.capture_lifecycle.borrow().is_empty());
    assert_eq!(environment.cleanup_calls.get(), 1);
}
