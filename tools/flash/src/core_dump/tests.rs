use super::*;

fn enabled_task(section: &str) -> String {
    format!("## {section}\n### task-str005-start-panic-diagnosis | now\nDevelopment core-dump acquisition: enabled (recovery evidence prerequisite satisfied).\n")
}

#[test]
fn only_unique_enabled_active_task_is_admitted() {
    // Arrange
    let enabled = enabled_task("Active");
    // Act / Assert
    assert!(contract::admit_task(&enabled).is_ok());
    assert!(contract::admit_task(&enabled_task("Future")).is_err());
    assert!(contract::admit_task(&enabled.replace("enabled", "disabled")).is_err());
    assert!(contract::admit_task(&(enabled.clone() + &enabled)).is_err());
    assert!(contract::admit_task(&(enabled + &enabled_task("Future"))).is_err());
}

fn partition_bytes(offset: &str, size: &str) -> Vec<u8> {
    let csv = format!("factory,app,factory,0x10000,1M,\ncoredump,data,coredump,{offset},{size},\n");
    let table = esp_idf_part::PartitionTable::try_from_str(csv).expect("valid fixture");
    let mut bytes = table.to_bin().expect("fixture binary");
    bytes.resize(4096, 0xff);
    bytes
}

#[test]
fn accepts_only_actual_known_dump_ranges_with_valid_checksum() {
    // Arrange
    let legacy = partition_bytes("0xf12000", "0x10000");
    let development = partition_bytes("0xf12000", "0xee000");
    let wrong = partition_bytes("0xf00000", "0x10000");
    let mut corrupt = legacy.clone();
    corrupt[10] ^= 1;
    // Act / Assert
    assert_eq!(
        contract::dump_partition(&legacy).expect("legacy"),
        (0xf12000, 0x10000)
    );
    assert_eq!(
        contract::dump_partition(&development).expect("development"),
        (0xf12000, 0xee000)
    );
    assert!(contract::dump_partition(&wrong).is_err());
    assert!(contract::dump_partition(&corrupt).is_err());
    assert!(contract::dump_partition(&legacy[..2048]).is_err());
}

#[test]
fn releases_after_failed_application_return() {
    // Arrange
    let steps = RefCell::new(Vec::new());
    // Act
    let (returned, cleanup) = recover_and_release(
        true,
        || {
            steps.borrow_mut().push("return");
            bail!("return_failed")
        },
        || {
            steps.borrow_mut().push("release");
            Ok(())
        },
    );
    // Assert
    assert!(returned.is_err());
    assert!(cleanup.is_ok());
    assert_eq!(*steps.borrow(), ["return", "release"]);
}

#[test]
fn failed_rom_admission_does_not_attempt_application_reset() {
    // Arrange
    let steps = RefCell::new(Vec::new());
    // Act
    let (returned, cleanup) = recover_and_release(
        false,
        || {
            steps.borrow_mut().push("return");
            Ok(())
        },
        || {
            steps.borrow_mut().push("release");
            Ok(())
        },
    );
    // Assert
    assert!(returned.is_ok() && cleanup.is_ok());
    assert_eq!(*steps.borrow(), ["release"]);
}

#[test]
fn existing_or_symlink_private_roots_are_rejected() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("core-dump-test-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("fixture directory");
    let root = Utf8PathBuf::from_path_buf(directory).expect("UTF-8 fixture");
    // Act / Assert
    assert!(contract::create_root(&root).is_err());
    assert!(contract::create_root(&root.join("../escape")).is_err());
    #[cfg(unix)]
    {
        let link = root.join("link");
        std::os::unix::fs::symlink(&root, &link).expect("fixture symlink");
        assert!(contract::create_root(&link).is_err());
        fs::remove_file(link).expect("cleanup link");
    }
    fs::remove_dir(root).expect("cleanup directory");
}

#[test]
fn duplicate_dump_partitions_are_rejected() {
    // Arrange
    let table = esp_idf_part::PartitionTable::try_from_str(
        "factory,app,factory,0x10000,1M,\ncoredump,data,coredump,0xf12000,0x10000,\nduplicate,data,coredump,0xf22000,0x10000,\n",
    ).expect("valid duplicate subtype fixture");
    let mut bytes = table.to_bin().expect("fixture binary");
    bytes.resize(4096, 0xff);
    // Act / Assert
    assert!(contract::dump_partition(&bytes).is_err());
}

#[test]
fn successful_return_does_not_hide_release_failure() {
    // Arrange
    let released = RefCell::new(false);
    // Act
    let (returned, cleanup) = recover_and_release(
        true,
        || Ok(()),
        || {
            *released.borrow_mut() = true;
            bail!("release_failed")
        },
    );
    // Assert
    assert!(returned.is_ok());
    assert!(cleanup.is_err());
    assert!(*released.borrow());
}

#[test]
fn successor_core_gate_requires_one_explicit_active_owner() {
    // Arrange
    let successor = "## Active\n### task-str005-v2-accepted-share-probe | fixture\nRenew-image core-dump acquisition: enabled (fresh recovery required).\n";
    // Act / Assert
    assert!(contract::admit_task(successor).is_ok());
    assert!(contract::admit_mode(successor, true).is_err());
    assert!(contract::admit_task(&successor.replace("Active", "Future")).is_err());
    assert!(contract::admit_task(&(successor.to_owned() + &enabled_task("Active"))).is_err());
    assert!(contract::admit_task(&(successor.to_owned() + successor)).is_err());
}

#[test]
fn successor_core_clear_is_separately_scoped() {
    // Arrange
    let successor = "## Active\n### task-str005-v2-accepted-share-probe | fixture\nRenew-image core-dump clearing: enabled (private archive verified).\n";
    // Act / Assert
    assert!(contract::admit_mode(successor, true).is_ok());
    assert!(contract::admit_task(successor).is_err());
}

#[test]
fn heap_loss_read_is_admitted_without_clearing() {
    // Arrange
    let heap = "## Active\n### task-str005-heap-loss-diagnosis | fixture\nHeap-loss core-dump acquisition: enabled (fresh recovery required).\n";
    // Act / Assert
    assert!(contract::admit_task(heap).is_ok());
    assert!(contract::admit_mode(heap, true).is_err());
    assert!(contract::admit_task(&heap.replace("Active", "Future")).is_err());
    assert!(contract::admit_task(&(heap.to_owned() + &enabled_task("Active"))).is_err());
}

const LOCAL_LINE: &str =
    "Local mining core-dump acquisition: enabled (fresh stopped proof required).";

fn local_mining_task(line: &str) -> String {
    format!("## Active\n### task-handover-panic-diagnosis | now\n{line}\n")
}

#[test]
fn the_local_mining_owner_requires_the_stopped_proof() {
    // Arrange
    let local = local_mining_task(LOCAL_LINE);
    // Act / Assert
    assert_eq!(
        contract::admit_task(&local).expect("admitted"),
        contract::ProofKind::LocalMiningStopped
    );
    assert_eq!(
        contract::admit_task(&enabled_task("Active")).expect("admitted"),
        contract::ProofKind::Str005Recovery
    );
}

#[test]
fn two_enabled_core_dump_owners_are_ambiguous() {
    // Arrange
    let both = enabled_task("Active") + &local_mining_task(LOCAL_LINE);
    // Act / Assert
    assert!(contract::admit_task(&both).is_err());
}

#[test]
fn the_local_mining_clearing_line_never_admits_a_read() {
    // Arrange
    let clearing =
        local_mining_task("Local mining core-dump clearing: enabled (private archive verified).");
    // Act / Assert
    assert!(contract::admit_task(&clearing).is_err());
    assert!(contract::admit_mode(&clearing, true).is_ok());
}

fn stopped_command() -> CoreDumpReadCommand {
    CoreDumpReadCommand {
        board: BoardId::Ultra205,
        port: "/dev/test-only".to_owned(),
        expected_physical_sha256: "6".repeat(64),
        expected_installed_source: "c".repeat(40),
        expected_installed_elf: "e".repeat(64),
        private_root: Utf8PathBuf::from("attempt"),
        recovery_proof: Utf8PathBuf::from("/unused"),
    }
}

const NOW: u64 = 1_000_000;

fn stopped_proof() -> serde_json::Value {
    serde_json::json!({"schema": "local-mining-stopped-proof-v1", "source_commit": "d".repeat(40),
        "firmware_commit": "c".repeat(40), "app_elf_sha256": "e".repeat(64), "physical_identity_sha256": "6".repeat(64),
        "observed_at_unix_ms": NOW - 5_000, "boot_session": "a".repeat(32), "mining_paused": true, "asic_stopped": true,
        "stopped_samples": 2, "mine_on_boot": false, "serial_ownership_released": true})
}

fn validate_stopped(value: &serde_json::Value, kind: contract::ProofKind) -> Result<()> {
    let bytes = serde_json::to_vec(value).expect("proof JSON");
    proof::validate(&bytes, &stopped_command(), &"d".repeat(40), NOW, kind)
}

#[test]
fn a_fresh_stopped_proof_admits_the_local_mining_read() {
    // Arrange / Act / Assert
    assert!(validate_stopped(&stopped_proof(), contract::ProofKind::LocalMiningStopped).is_ok());
}

#[test]
fn a_stopped_proof_fails_on_each_unsafe_or_mismatched_field() {
    // Arrange
    let cases = [
        ("observed_at_unix_ms", serde_json::json!(NOW - 121_000)),
        ("asic_stopped", serde_json::json!(false)),
        ("mining_paused", serde_json::json!(false)),
        ("stopped_samples", serde_json::json!(1)),
        ("mine_on_boot", serde_json::json!(true)),
        ("serial_ownership_released", serde_json::json!(false)),
        ("app_elf_sha256", serde_json::json!("f".repeat(64))),
        (
            "physical_identity_sha256",
            serde_json::json!("7".repeat(64)),
        ),
        ("source_commit", serde_json::json!("0".repeat(40))),
        ("firmware_commit", serde_json::json!("0".repeat(40))),
        ("schema", serde_json::json!("local-mining-stopped-proof-v0")),
        ("observed_at_unix_ms", serde_json::json!(NOW + 1)),
        ("boot_session", serde_json::json!("not-hex")),
    ];
    // Act / Assert
    for (field, value) in cases {
        let mut proof = stopped_proof();
        proof[field] = value;
        assert!(
            validate_stopped(&proof, contract::ProofKind::LocalMiningStopped).is_err(),
            "{field}"
        );
    }
}

#[test]
fn a_stopped_proof_is_fresh_through_exactly_120_seconds() {
    // Arrange
    let mut proof = stopped_proof();
    proof["observed_at_unix_ms"] = serde_json::json!(NOW - 120_000);
    // Act / Assert
    assert!(validate_stopped(&proof, contract::ProofKind::LocalMiningStopped).is_ok());
}

#[test]
fn the_local_mining_owner_refuses_a_recovery_proof() {
    // Arrange
    let recovery = serde_json::json!({"schema": "str005-current-recovery-proof-v1", "source_commit": "d".repeat(40)});
    // Act / Assert
    assert!(validate_stopped(&recovery, contract::ProofKind::LocalMiningStopped).is_err());
}

#[test]
fn a_proof_is_accepted_only_by_its_own_owner() {
    // Arrange
    let stopped = stopped_proof();
    // Act / Assert
    assert!(validate_stopped(&stopped, contract::ProofKind::Str005Recovery).is_err());
    let mut extra = stopped_proof();
    extra["ledger"] = serde_json::json!({});
    assert!(validate_stopped(&extra, contract::ProofKind::LocalMiningStopped).is_err());
}
