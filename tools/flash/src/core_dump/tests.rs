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
