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
        recovery_proof: workspace.join("current-recovery.json"),
    };
    write_private_new_bytes(
        &command.recovery_proof,
        &serde_json::to_vec(&proof_value()).expect("proof JSON"),
    )
    .expect("private proof");
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
    let mut proof = proof_value();
    proof["physical_identity_sha256"] = serde_json::json!("7".repeat(64));
    fs::write(
        &command.recovery_proof,
        serde_json::to_vec(&proof).expect("proof JSON"),
    )
    .expect("proof update");
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

fn proof_value() -> serde_json::Value {
    serde_json::json!({"schema":"str005-current-recovery-proof-v1", "source_commit":SOURCE_COMMIT,
        "gate_commit":"a".repeat(40), "firmware_commit":SOURCE_COMMIT, "app_elf_sha256":APP_ELF_SHA256,
        "physical_identity_sha256":"6".repeat(64), "observed_at_unix_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock").as_millis() as u64,
        "ledger":{"schema":"worker-qualification-ledger-v1","next_ordinal":18,"last_completed_ordinal":17,"total_charged_ms":1560000,"pending":false},
        "original_budget":{"schema":"worker-budget-review-v1","campaign_match":true,"reserved_mask":7,"completed_mask":7,"charged_ms":240000,"pending":false},
        "safe_baseline":true,"restoration_confirmed":true,"device_lease_inactive":true,"serial_ownership_released":true,"preservation_matches":true,"mine_on_boot":false,"current_v2_idle":true})
}

fn clear_fixture() -> (
    TempDir,
    crate::core_dump::clearing::CoreDumpClearCommand,
    FakeFlashEnvironment,
) {
    let (directory, common, mut environment) = fixture();
    environment.execute_failure = false;
    let tasks = environment.workspace_dir.join("TASKS.md");
    let mut content = fs::read_to_string(&tasks).expect("tasks");
    content.push_str("Development core-dump clearing: enabled (private archive verified).\n");
    fs::write(tasks, content).expect("enable clear");
    let preserved_dump = environment.workspace_dir.join("preserved.private.bin");
    let dump = vec![0x42; 0x10000];
    write_private_new_bytes(&preserved_dump, &dump).expect("private archive");
    let table = esp_idf_part::PartitionTable::try_from_str(
        "factory,app,factory,0x10000,1M,\ncoredump,data,coredump,0xf12000,0x10000,\n",
    )
    .expect("table");
    let mut bytes = table.to_bin().expect("binary table");
    bytes.resize(4096, 0xff);
    environment
        .core_dump_reads
        .borrow_mut()
        .extend([bytes, dump.clone(), vec![0xff; 0x10000]]);
    (
        directory,
        crate::core_dump::clearing::CoreDumpClearCommand {
            common,
            preserved_dump,
            preserved_sha256: sha256_bytes(&dump),
        },
        environment,
    )
}

#[test]
fn core_dump_clear_requires_exact_archive_then_verifies_erasure_and_releases() {
    // Arrange
    let (_directory, command, environment) = clear_fixture();
    // Act
    crate::core_dump::clearing::run(&command, &environment).expect("verified clear");
    // Assert
    let record = receipt(&environment);
    assert_eq!(record["clearing_complete"], true);
    assert_eq!(record["application_identity_restored"], true);
    assert_eq!(environment.cleanup_calls.get(), 1);
    let commands = environment.executed_commands();
    assert_eq!(
        commands
            .iter()
            .map(|c| c.args[0].as_str())
            .collect::<Vec<_>>(),
        [
            "read-flash",
            "read-flash",
            "board-info",
            "erase-region",
            "read-flash"
        ]
    );
    assert_eq!(
        &commands[3].args[commands[3].args.len() - 2..],
        ["0xf12000", "0x10000"]
    );
}

#[test]
fn core_dump_clear_drift_never_erases_and_still_returns_application() {
    // Arrange
    let (_directory, command, environment) = clear_fixture();
    environment.core_dump_reads.borrow_mut()[1][10] ^= 1;
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert!(!environment
        .executed_commands()
        .iter()
        .any(|c| c.args[0] == "erase-region"));
    assert_eq!(
        receipt(&environment)["first_failure_stage"],
        "preserved_dump_match"
    );
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(*environment.capture_lifecycle.borrow(), ["reset"]);
}

#[test]
fn core_dump_clear_failed_readback_keeps_first_failure_through_failed_return() {
    // Arrange
    let (_directory, command, mut environment) = clear_fixture();
    environment.core_dump_reads.borrow_mut()[2][0] = 0;
    environment.application_exit_failure = true;
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert_eq!(
        receipt(&environment)["first_failure_stage"],
        "dump_erase_readback"
    );
    assert_eq!(
        receipt(&environment)["application_identity_restored"],
        false
    );
    assert_eq!(environment.cleanup_calls.get(), 1);
}

#[test]
fn core_dump_bad_archive_digest_rejects_before_usb() {
    // Arrange
    let (_directory, mut command, environment) = clear_fixture();
    command.preserved_sha256 = "f".repeat(64);
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert!(environment.executed_commands().is_empty());
    assert_eq!(environment.cleanup_calls.get(), 0);
    assert!(!environment.workspace_dir.join("attempt").exists());
}

#[test]
fn core_dump_stale_recovery_proof_rejects_before_usb() {
    // Arrange
    let (_directory, command, environment) = fixture();
    let mut value = proof_value();
    value["observed_at_unix_ms"] = serde_json::json!(0);
    fs::write(
        &command.recovery_proof,
        serde_json::to_vec(&value).expect("proof JSON"),
    )
    .expect("proof");
    // Act
    assert!(run(&command, &environment).is_err());
    // Assert
    assert!(environment.executed_commands().is_empty());
    assert_eq!(environment.cleanup_calls.get(), 0);
}

#[test]
fn core_dump_proof_rejects_invalid_identity_state_and_closed_shapes() {
    for field in [
        "source_commit",
        "gate_commit",
        "firmware_commit",
        "app_elf_sha256",
        "physical_identity_sha256",
        "safe_baseline",
        "restoration_confirmed",
        "device_lease_inactive",
        "serial_ownership_released",
        "preservation_matches",
        "current_v2_idle",
        "mine_on_boot",
        "extra",
    ] {
        // Arrange
        let (_directory, command, environment) = fixture();
        let mut value = proof_value();
        value[field] = if field == "mine_on_boot" {
            serde_json::json!(true)
        } else {
            serde_json::json!(false)
        };
        fs::write(
            &command.recovery_proof,
            serde_json::to_vec(&value).expect("proof JSON"),
        )
        .expect("proof");
        // Act / Assert
        assert!(run(&command, &environment).is_err(), "accepted {field}");
        assert!(environment.executed_commands().is_empty());
        assert_eq!(environment.cleanup_calls.get(), 0);
    }
}

#[test]
fn core_dump_proof_rejects_future_time_and_pending_accounting() {
    for (field, invalid) in [
        ("observed_at_unix_ms", serde_json::json!(u64::MAX)),
        (
            "ledger",
            serde_json::json!({"schema":"worker-qualification-ledger-v1","next_ordinal":18,"last_completed_ordinal":17,"total_charged_ms":1560000,"pending":true}),
        ),
        (
            "original_budget",
            serde_json::json!({"schema":"worker-budget-review-v1","campaign_match":true,"reserved_mask":7,"completed_mask":7,"charged_ms":240000,"pending":true}),
        ),
    ] {
        // Arrange
        let (_directory, command, environment) = fixture();
        let mut value = proof_value();
        value[field] = invalid;
        fs::write(
            &command.recovery_proof,
            serde_json::to_vec(&value).expect("proof JSON"),
        )
        .expect("proof");
        // Act / Assert
        assert!(run(&command, &environment).is_err());
        assert!(environment.executed_commands().is_empty());
    }
}

#[test]
fn core_dump_private_input_mode_and_symlink_fail_before_usb() {
    // Arrange
    let (_directory, command, environment) = fixture();
    #[cfg(unix)]
    {
        fs::set_permissions(&command.recovery_proof, fs::Permissions::from_mode(0o644))
            .expect("fixture mode");
        // Act / Assert
        assert!(run(&command, &environment).is_err());
        fs::set_permissions(&command.recovery_proof, fs::Permissions::from_mode(0o600))
            .expect("fixture mode");
        let actual = environment.workspace_dir.join("actual-proof.json");
        fs::rename(&command.recovery_proof, &actual).expect("move proof");
        std::os::unix::fs::symlink(&actual, &command.recovery_proof).expect("symlink proof");
        assert!(run(&command, &environment).is_err());
        assert!(environment.executed_commands().is_empty());
    }
}

#[test]
fn core_dump_clear_requires_separate_active_enable_line() {
    // Arrange
    let (_directory, command, environment) = clear_fixture();
    let tasks = environment.workspace_dir.join("TASKS.md");
    let content = fs::read_to_string(&tasks).expect("tasks").replace(
        "Development core-dump clearing: enabled (private archive verified).",
        "",
    );
    fs::write(tasks, content).expect("task");
    // Act / Assert
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn core_dump_clear_cli_requires_recovery_proof_and_archive() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "core-dump-clear",
        "--port",
        "/dev/test-only",
        "--expected-physical-sha256",
        &"a".repeat(64),
        "--expected-installed-source",
        SOURCE_COMMIT,
        "--expected-installed-elf",
        APP_ELF_SHA256,
        "--private-root",
        "/private/attempt",
    ];
    // Act / Assert
    assert!(Cli::try_parse_from(args).is_err());
}

#[test]
fn core_dump_clear_failed_fresh_rom_proof_never_erases() {
    // Arrange
    let (_directory, command, mut environment) = clear_fixture();
    environment.maybe_execute_failure_command = Some("board-info".to_owned());
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert!(!environment
        .executed_commands()
        .iter()
        .any(|command| command.args[0] == "erase-region"));
    assert_eq!(environment.cleanup_calls.get(), 1);
}

#[test]
fn core_dump_clear_failed_erase_is_never_reissued_and_returns_application() {
    // Arrange
    let (_directory, command, mut environment) = clear_fixture();
    environment.maybe_execute_failure_command = Some("erase-region".to_owned());
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert_eq!(
        environment
            .executed_commands()
            .iter()
            .filter(|command| command.args[0] == "erase-region")
            .count(),
        1
    );
    assert_eq!(receipt(&environment)["first_failure_stage"], "dump_erase");
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(*environment.capture_lifecycle.borrow(), ["reset"]);
}

#[test]
fn core_dump_clear_commands_never_request_additional_reset() {
    // Arrange
    let proof = installed_rom_probe_args("/dev/test-only");
    let erase = crate::core_dump::clearing::erase_command("/dev/test-only", 0xf12000, 0x10000);
    // Act / Assert
    for args in [&proof, &erase.args] {
        assert!(args.windows(2).any(|pair| pair == ["--before", "no-reset"]));
        assert!(args.windows(2).any(|pair| pair == ["--after", "no-reset"]));
        assert!(!args
            .iter()
            .any(|value| value == "usb-reset" || value == "hard-reset"));
    }
}

#[test]
fn core_dump_clear_fresh_inspection_follows_same_physical_session_reenumeration() {
    // Arrange
    let (_directory, command, mut environment) = clear_fixture();
    environment.maybe_rom_reenumeration =
        Some(("/dev/re-enumerated-fixture".to_owned(), "6".repeat(64)));
    // Act
    crate::core_dump::clearing::run(&command, &environment).expect("same physical new node");
    // Assert
    let inspections = environment.physical_inspection_ports.borrow();
    assert_eq!(
        inspections.first().map(String::as_str),
        Some("/dev/test-only")
    );
    assert!(inspections[1..]
        .iter()
        .all(|port| port == "/dev/re-enumerated-fixture"));
    assert_eq!(receipt(&environment)["clearing_complete"], true);
    assert_eq!(environment.cleanup_calls.get(), 1);
}

#[test]
fn core_dump_reenumerated_physical_drift_rejects_before_read_or_erase() {
    // Arrange
    let (_directory, command, mut environment) = clear_fixture();
    environment.maybe_rom_reenumeration =
        Some(("/dev/re-enumerated-fixture".to_owned(), "7".repeat(64)));
    // Act
    assert!(crate::core_dump::clearing::run(&command, &environment).is_err());
    // Assert
    assert!(environment.executed_commands().is_empty());
    assert!(environment.capture_lifecycle.borrow().is_empty());
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert_eq!(receipt(&environment)["clearing_complete"], false);
}
