use super::super::*;
use super::promotion::write_promotion_roots;
use super::support::{create_workspace, snapshot, write_phase27_source};
use crate::operator_evidence::ShareOutcome;

#[test]
fn unknown_destination_file_fails_closed() {
    // Arrange
    let workspace = create_workspace("unknown-destination-file");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("initial generation should pass");
    fs::write(
        workspace
            .join("destination/operator-note.txt")
            .as_std_path(),
        "owned",
    )
    .expect("unknown file should write");
    let before = snapshot(&workspace.join("destination"));

    // Act
    let result = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    );

    // Assert
    assert!(result
        .expect_err("unknown file should fail")
        .to_string()
        .contains("unknown entry"));
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

#[test]
fn destination_manifest_must_match_exact_generator_schema() {
    // Arrange
    let workspace = create_workspace("manifest-schema");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("initial generation should pass");
    let manifest = workspace.join("destination/.phase28-evidence-manifest");
    fs::write(manifest.as_std_path(), "phase28-evidence-v1\n")
        .expect("manifest should be tampered");
    let before = snapshot(&workspace.join("destination"));

    // Act
    let error = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect_err("unknown manifest content should fail closed");

    // Assert
    assert!(error.to_string().contains("exact generator schema"));
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

#[test]
fn destination_rejects_directory_with_allowed_file_name() {
    // Arrange
    let workspace = create_workspace("allowed-name-directory");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("initial generation should pass");
    let api_path = workspace.join("destination/api.md");
    fs::remove_file(api_path.as_std_path()).expect("API slot should be removed");
    fs::create_dir(api_path.as_std_path()).expect("allowed-name directory should be created");

    // Act
    let error = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect_err("allowed-name directory should fail closed");

    // Assert
    assert!(error.to_string().contains("non-symlink regular file"));
    assert!(api_path.is_dir());
}

#[cfg(unix)]
#[test]
fn destination_rejects_symlink_entry_with_allowed_file_name() {
    use std::os::unix::fs::symlink;

    // Arrange
    let workspace = create_workspace("allowed-name-symlink");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("initial generation should pass");
    let api_path = workspace.join("destination/api.md");
    fs::remove_file(api_path.as_std_path()).expect("API slot should be removed");
    symlink("command.md", api_path.as_std_path()).expect("allowed-name symlink should be created");

    // Act
    let error = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect_err("allowed-name symlink should fail closed");

    // Assert
    assert!(error.to_string().contains("non-symlink regular file"));
    assert!(fs::symlink_metadata(api_path.as_std_path())
        .expect("symlink should remain")
        .file_type()
        .is_symlink());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn destination_replacement_after_lock_is_rejected_before_exchange() {
    // Arrange
    let workspace = create_workspace("destination-replacement-race");
    let (destination, staging) = write_promotion_roots(&workspace);
    let context = PromotionContext::acquire_for_test(&destination)
        .expect("promotion lock should be acquired");
    let displaced = workspace.join("displaced-destination");
    fs::rename(destination.as_std_path(), displaced.as_std_path())
        .expect("validated destination should be displaced");
    fs::create_dir(destination.as_std_path()).expect("substitute destination should be created");
    fs::write(
        destination.join("marker").as_std_path(),
        "substitute-generation",
    )
    .expect("substitute marker should write");

    // Act
    let error = promote_staging(
        &destination,
        &staging,
        ConsolidationOptions::default(),
        &context,
    )
    .expect_err("destination identity replacement should fail closed");

    // Assert
    assert!(error
        .to_string()
        .contains("identity changed before promotion"));
    assert_eq!(
        fs::read_to_string(destination.join("marker").as_std_path()).expect("marker should read"),
        "substitute-generation"
    );
    assert_eq!(
        fs::read_to_string(staging.join("marker").as_std_path()).expect("marker should read"),
        "replacement-generation"
    );
    assert_eq!(
        fs::read_to_string(displaced.join("marker").as_std_path()).expect("marker should read"),
        "previous-generation"
    );
}

#[test]
fn destination_promotion_lock_rejects_concurrent_owner() {
    // Arrange
    let workspace = create_workspace("destination-lock-contention");
    let destination = workspace.join("destination");
    let first =
        PromotionContext::acquire_for_test(&destination).expect("first lock should be acquired");

    // Act
    let error = match PromotionContext::acquire_for_test(&destination) {
        Ok(_) => panic!("second lock owner should fail closed"),
        Err(error) => error,
    };
    drop(first);
    let reacquired = PromotionContext::acquire_for_test(&destination);

    // Assert
    assert!(error.to_string().contains("promotion lock is already held"));
    assert!(reacquired.is_ok());
}
