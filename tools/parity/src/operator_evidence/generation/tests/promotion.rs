use std::collections::BTreeSet;

use super::super::*;
use super::support::{create_workspace, find_staging_root, snapshot, write_phase27_source};
use crate::operator_evidence::ShareOutcome;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn post_exchange_failure_rolls_destination_back_byte_identically() {
    // Arrange
    let workspace = create_workspace("post-exchange-rollback");
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
    let before = snapshot(&workspace.join("destination"));

    // Act
    let result = consolidate_phase28_evidence_with_options(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
        ConsolidationOptions {
            maybe_failure: Some(PromotionFailurePoint::AfterExchange),
        },
    );

    // Assert
    assert!(result.is_err());
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn parent_sync_failure_returns_original_error_after_successful_rollback() {
    // Arrange
    let workspace = create_workspace("parent-sync-rollback");
    let (destination, staging) = write_promotion_roots(&workspace);
    let context = PromotionContext::acquire_for_test(&destination)
        .expect("promotion lock should be acquired");
    let mut filesystem = InjectedPromotionFilesystem::failing_sync_calls([1]);

    // Act
    let error = promote_staging_with_filesystem(
        &destination,
        &staging,
        ConsolidationOptions::default(),
        &mut filesystem,
        &context,
    )
    .expect_err("parent sync failure should fail promotion");

    // Assert
    assert!(matches!(error, GenerationError::Io { .. }));
    assert!(error.to_string().contains("injected sync failure 1"));
    assert_eq!(
        fs::read_to_string(destination.join("marker").as_std_path()).expect("marker should read"),
        "previous-generation"
    );
    assert!(!staging.exists());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn post_rollback_sync_failure_reports_recovery_with_both_generations() {
    // Arrange
    let workspace = create_workspace("rollback-sync-recovery");
    let (destination, staging) = write_promotion_roots(&workspace);
    let context = PromotionContext::acquire_for_test(&destination)
        .expect("promotion lock should be acquired");
    let mut filesystem = InjectedPromotionFilesystem::failing_sync_calls([1, 2]);

    // Act
    let error = promote_staging_with_filesystem(
        &destination,
        &staging,
        ConsolidationOptions::default(),
        &mut filesystem,
        &context,
    )
    .expect_err("rollback sync failure should require recovery");

    // Assert
    assert!(matches!(error, GenerationError::RecoveryRequired { .. }));
    let detail = error.to_string();
    assert!(detail.contains("injected sync failure 1"));
    assert!(detail.contains("rollback durability is uncertain"));
    assert_eq!(
        fs::read_to_string(destination.join("marker").as_std_path()).expect("marker should read"),
        "previous-generation"
    );
    assert_eq!(
        fs::read_to_string(staging.join("marker").as_std_path()).expect("marker should read"),
        "replacement-generation"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn post_promotion_cleanup_sync_failure_does_not_claim_removed_generation() {
    // Arrange
    let workspace = create_workspace("promotion-cleanup-sync-uncertain");
    let (destination, staging) = write_promotion_roots(&workspace);
    let context = PromotionContext::acquire_for_test(&destination)
        .expect("promotion lock should be acquired");
    let mut filesystem = InjectedPromotionFilesystem::failing_sync_calls([2]);

    // Act
    let error = promote_staging_with_filesystem(
        &destination,
        &staging,
        ConsolidationOptions::default(),
        &mut filesystem,
        &context,
    )
    .expect_err("post-cleanup sync failure should report uncertain durability");

    // Assert
    assert!(matches!(
        &error,
        GenerationError::DurabilityUncertain { .. }
    ));
    assert!(!staging.exists());
    assert!(!error.to_string().contains("retained_old_generation"));
    assert_eq!(
        fs::read_to_string(destination.join("marker").as_std_path()).expect("marker should read"),
        "replacement-generation"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn post_rollback_cleanup_sync_failure_does_not_claim_removed_generation() {
    // Arrange
    let workspace = create_workspace("rollback-cleanup-sync-uncertain");
    let (destination, staging) = write_promotion_roots(&workspace);
    let context = PromotionContext::acquire_for_test(&destination)
        .expect("promotion lock should be acquired");
    let mut filesystem = InjectedPromotionFilesystem::failing_sync_calls([1, 3]);

    // Act
    let error = promote_staging_with_filesystem(
        &destination,
        &staging,
        ConsolidationOptions::default(),
        &mut filesystem,
        &context,
    )
    .expect_err("post-rollback cleanup sync failure should report uncertain durability");

    // Assert
    assert!(matches!(
        &error,
        GenerationError::DurabilityUncertain { .. }
    ));
    assert!(!staging.exists());
    assert!(!error.to_string().contains("retained_old_generation"));
    assert_eq!(
        fs::read_to_string(destination.join("marker").as_std_path()).expect("marker should read"),
        "previous-generation"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn old_generation_cleanup_failure_retains_both_roots() {
    // Arrange
    let workspace = create_workspace("old-generation-cleanup");
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

    // Act
    let error = consolidate_phase28_evidence_with_options(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
        ConsolidationOptions {
            maybe_failure: Some(PromotionFailurePoint::DuringOldGenerationCleanup),
        },
    )
    .expect_err("cleanup failure should require recovery");

    // Assert
    assert!(matches!(error, GenerationError::RecoveryRequired { .. }));
    assert!(find_staging_root(&workspace).is_some());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct InjectedPromotionFilesystem {
    sync_calls: usize,
    failing_sync_calls: BTreeSet<usize>,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl InjectedPromotionFilesystem {
    fn failing_sync_calls(calls: impl IntoIterator<Item = usize>) -> Self {
        Self {
            sync_calls: 0,
            failing_sync_calls: calls.into_iter().collect(),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl PromotionFilesystem for InjectedPromotionFilesystem {
    fn rename(&mut self, source: &Utf8Path, destination: &Utf8Path) -> GenerationResult<()> {
        fs::rename(source.as_std_path(), destination.as_std_path())
            .map_err(|error| io_error("test rename failed", error))
    }

    fn exchange(&mut self, left: &Utf8Path, right: &Utf8Path) -> GenerationResult<()> {
        atomic_exchange(left, right)
    }

    fn sync_directory(&mut self, path: &Utf8Path) -> GenerationResult<()> {
        self.sync_calls += 1;
        if self.failing_sync_calls.contains(&self.sync_calls) {
            return Err(io_error(
                format!("injected sync failure {}", self.sync_calls),
                std::io::Error::other("injected test failure"),
            ));
        }
        sync_directory(path)
    }

    fn remove_directory(&mut self, path: &Utf8Path) -> GenerationResult<()> {
        fs::remove_dir_all(path.as_std_path())
            .map_err(|error| io_error("test remove failed", error))
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) fn write_promotion_roots(workspace: &Utf8Path) -> (Utf8PathBuf, Utf8PathBuf) {
    let destination = workspace.join("destination");
    let staging = workspace.join("staging");
    fs::create_dir_all(destination.as_std_path()).expect("destination should be created");
    fs::create_dir_all(staging.as_std_path()).expect("staging should be created");
    fs::write(
        destination.join("marker").as_std_path(),
        "previous-generation",
    )
    .expect("previous marker should write");
    fs::write(
        staging.join("marker").as_std_path(),
        "replacement-generation",
    )
    .expect("replacement marker should write");
    (destination, staging)
}
