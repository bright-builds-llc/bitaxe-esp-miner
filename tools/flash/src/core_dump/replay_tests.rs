use super::*;

#[test]
fn core_dump_proof_claim_survives_failure_and_rejects_reformatted_copy() {
    // Arrange
    let (_directory, mut command, environment) = fixture();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&command.recovery_proof).expect("proof")).expect("JSON");
    // Act
    assert!(run(&command, &environment).is_err());
    let calls = environment.executed_commands().len();
    let copied = environment.workspace_dir.join("copied-proof.json");
    write_private_new_bytes(
        &copied,
        &serde_json::to_vec_pretty(&original).expect("reformatted proof"),
    )
    .expect("copy");
    command.recovery_proof = copied;
    command.private_root = Utf8PathBuf::from("replayed-attempt");
    let result = run(&command, &environment);
    // Assert
    assert_eq!(
        result.expect_err("replay rejected").to_string(),
        "core_proof_already_consumed"
    );
    assert_eq!(environment.executed_commands().len(), calls);
    assert_eq!(environment.cleanup_calls.get(), 1);
    assert!(!environment.workspace_dir.join("replayed-attempt").exists());
    let root = environment
        .workspace_dir
        .join("scratch/core-dump-proof-claims");
    let entries = fs::read_dir(&root)
        .expect("claims")
        .collect::<std::io::Result<Vec<_>>>()
        .expect("claim entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(
        fs::metadata(&root).expect("mode").permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(entries[0].path())
            .expect("mode")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn core_dump_proof_claim_is_shared_by_read_and_clear() {
    // Arrange
    let (_directory, mut command, mut environment) = clear_fixture();
    environment.execute_failure = true;
    // Act
    assert!(run(&command.common, &environment).is_err());
    let calls = environment.executed_commands().len();
    command.common.private_root = Utf8PathBuf::from("replayed-clear");
    let result = crate::core_dump::clearing::run(&command, &environment);
    // Assert
    assert_eq!(
        result.expect_err("read to clear replay").to_string(),
        "core_proof_already_consumed"
    );
    assert_eq!(environment.executed_commands().len(), calls);
    assert!(!environment.workspace_dir.join("replayed-clear").exists());
}

#[test]
fn core_dump_new_collection_proof_has_independent_claim() {
    // Arrange
    let (_directory, mut command, environment) = fixture();
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&command.recovery_proof).expect("proof")).expect("JSON");
    value["observed_at_unix_ms"] =
        serde_json::json!(value["observed_at_unix_ms"].as_u64().expect("timestamp") - 1000);
    fs::write(
        &command.recovery_proof,
        serde_json::to_vec(&value).expect("first collection"),
    )
    .expect("first collection");
    // Act
    assert!(run(&command, &environment).is_err());
    let calls = environment.executed_commands().len();
    value["observed_at_unix_ms"] =
        serde_json::json!(value["observed_at_unix_ms"].as_u64().expect("timestamp") + 1);
    let fresh = environment.workspace_dir.join("new-collection.json");
    write_private_new_bytes(&fresh, &serde_json::to_vec(&value).expect("new proof"))
        .expect("new collection");
    command.recovery_proof = fresh;
    command.private_root = Utf8PathBuf::from("fresh-attempt");
    assert!(run(&command, &environment).is_err());
    // Assert
    assert_eq!(environment.executed_commands().len(), calls + 1);
    assert_eq!(environment.cleanup_calls.get(), 2);
    assert!(environment
        .workspace_dir
        .join("fresh-attempt/result.private.json")
        .exists());
}

#[test]
fn core_dump_claim_directory_symlink_is_rejected_before_usb() {
    // Arrange
    let (_directory, command, environment) = fixture();
    fs::create_dir(environment.workspace_dir.join("scratch")).expect("scratch");
    std::os::unix::fs::symlink(
        &environment.workspace_dir,
        environment
            .workspace_dir
            .join("scratch/core-dump-proof-claims"),
    )
    .expect("symlink");
    // Act
    let result = run(&command, &environment);
    // Assert
    assert!(result.is_err());
    assert!(environment.executed_commands().is_empty());
    assert_eq!(environment.cleanup_calls.get(), 0);
    assert!(!environment.workspace_dir.join("attempt").exists());
}
