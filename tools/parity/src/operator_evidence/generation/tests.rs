use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::operator_evidence::ShareOutcome;

#[test]
fn completion_is_byte_identical_on_deterministic_rerun() {
    // Arrange
    let workspace = create_workspace("completion-rerun");
    let root = Utf8Path::new("evidence/phase27");

    // Act
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase27,
        root,
        WorkflowStatus::Blocked,
    )
    .expect("first completion should pass");
    let before = snapshot(&workspace.join(root));
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase27,
        root,
        WorkflowStatus::Blocked,
    )
    .expect("second completion should pass");

    // Assert
    assert_eq!(snapshot(&workspace.join(root)), before);
}

#[test]
fn completion_preserves_existing_observed_slot_bytes() {
    // Arrange
    let workspace = create_workspace("completion-preserves-observed");
    let root = workspace.join("evidence/phase27");
    fs::create_dir_all(root.as_std_path()).expect("root should be created");
    let observed = "observed-slot-bytes-remain-exact\n";
    fs::write(root.join("package.md").as_std_path(), observed).expect("slot should write");

    // Act
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase27,
        Utf8Path::new("evidence/phase27"),
        WorkflowStatus::Blocked,
    )
    .expect("completion should pass");

    // Assert
    assert_eq!(
        fs::read_to_string(root.join("package.md").as_std_path()).expect("slot should read"),
        observed
    );
}

#[test]
fn consolidation_rejects_absolute_and_related_roots_before_writes() {
    // Arrange
    let workspace = create_workspace("invalid-roots");

    // Act
    let absolute = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("/tmp/source"),
        Utf8Path::new("evidence/phase28"),
    );
    let nested = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("evidence/source"),
        Utf8Path::new("evidence/source/phase28"),
    );
    let equal = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("evidence/same"),
        Utf8Path::new("evidence/same"),
    );
    let parent = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("evidence/source/child"),
        Utf8Path::new("evidence/source"),
    );
    let traversal = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("evidence/../source"),
        Utf8Path::new("evidence/phase28"),
    );

    // Assert
    assert!(absolute
        .expect_err("absolute root should fail")
        .to_string()
        .contains("repo-relative"));
    assert!(nested
        .expect_err("nested roots should fail")
        .to_string()
        .contains("non-nested"));
    assert!(equal
        .expect_err("equal roots should fail")
        .to_string()
        .contains("non-nested"));
    assert!(parent
        .expect_err("parent root should fail")
        .to_string()
        .contains("non-nested"));
    assert!(traversal
        .expect_err("traversal should fail")
        .to_string()
        .contains("traversal"));
    assert!(!workspace.join("evidence/phase28").exists());
}

#[cfg(unix)]
#[test]
fn consolidation_rejects_symlink_managed_source() {
    use std::os::unix::fs::symlink;

    // Arrange
    let workspace = create_workspace("symlink-source");
    fs::create_dir_all(workspace.join("real-source").as_std_path()).expect("real source");
    symlink(
        workspace.join("real-source").as_std_path(),
        workspace.join("linked-source").as_std_path(),
    )
    .expect("symlink should be created");

    // Act
    let result = consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("linked-source"),
        Utf8Path::new("evidence/phase28"),
    );

    // Assert
    assert!(result
        .expect_err("symlink should fail")
        .to_string()
        .contains("symlink-managed"));
}

#[test]
fn consolidation_renders_cross_links_without_raw_source_sentinels() {
    // Arrange
    let workspace = create_workspace("cross-link-only");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    fs::write(
        workspace.join("source/detector.md").as_std_path(),
        "raw secret sentinel-password must never be copied",
    )
    .expect("sentinel source should write");

    // Act
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("./source"),
        Utf8Path::new("destination"),
    )
    .expect("consolidation should pass");
    let rendered = snapshot(&workspace.join("destination"));

    // Assert
    assert!(rendered.contains("source_phase27_root: source"));
    assert!(!rendered.contains("source_phase27_root: ./source"));
    assert!(!rendered.contains("sentinel-password"));
    assert!(!rendered.contains(workspace.as_str()));
}

#[test]
fn outcome_matrix_requires_support_and_renders_all_closed_outcomes() {
    for outcome in [
        ShareOutcome::Accepted,
        ShareOutcome::Rejected,
        ShareOutcome::BlockedSafePrerequisite,
    ] {
        // Arrange
        let workspace = create_workspace(outcome.as_str());
        write_phase27_source(&workspace.join("source"), outcome);

        // Act
        consolidate_phase28_evidence(
            &workspace,
            Utf8Path::new("source"),
            Utf8Path::new("destination"),
        )
        .expect("supported outcome should consolidate");
        let share =
            fs::read_to_string(workspace.join("destination/share-outcome.md").as_std_path())
                .expect("share outcome should read");

        // Assert
        assert!(share.contains(&format!("share_outcome: {}", outcome.as_str())));
    }
}

#[test]
fn pre_exchange_failure_leaves_existing_destination_byte_identical() {
    // Arrange
    let workspace = create_workspace("pre-exchange-retains");
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
            maybe_failure: Some(PromotionFailurePoint::BeforeExchange),
        },
    );

    // Assert
    assert!(result.is_err());
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

#[test]
fn all_pre_exchange_failure_points_leave_destination_byte_identical() {
    for failure in [
        PromotionFailurePoint::BeforeStagingSync,
        PromotionFailurePoint::BeforeExchange,
    ] {
        // Arrange
        let workspace = create_workspace(&format!("pre-{failure:?}"));
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
                maybe_failure: Some(failure),
            },
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(snapshot(&workspace.join("destination")), before);
    }
}

#[test]
fn identical_consolidation_inputs_generate_byte_identical_destination() {
    // Arrange
    let workspace = create_workspace("deterministic-consolidation");
    write_phase27_source(
        &workspace.join("source"),
        ShareOutcome::BlockedSafePrerequisite,
    );
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("first generation should pass");
    let before = snapshot(&workspace.join("destination"));

    // Act
    consolidate_phase28_evidence(
        &workspace,
        Utf8Path::new("source"),
        Utf8Path::new("destination"),
    )
    .expect("second generation should pass");

    // Assert
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

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
        .contains("unknown file"));
    assert_eq!(snapshot(&workspace.join("destination")), before);
}

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
fn parent_sync_failure_rolls_back_and_rollback_failure_retains_both_roots() {
    for failure in [
        PromotionFailurePoint::DuringParentSync,
        PromotionFailurePoint::DuringRollback,
        PromotionFailurePoint::DuringOldGenerationCleanup,
    ] {
        // Arrange
        let workspace = create_workspace(&format!("post-{failure:?}"));
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
                maybe_failure: Some(failure),
            },
        );

        // Assert
        let error = result.expect_err("injected post-exchange failure should fail");
        if failure == PromotionFailurePoint::DuringParentSync {
            assert_eq!(snapshot(&workspace.join("destination")), before);
        } else {
            assert!(matches!(error, GenerationError::RecoveryRequired { .. }));
            assert!(find_staging_root(&workspace).is_some());
        }
    }
}

fn create_workspace(name: &str) -> Utf8PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("phase29-generation-{name}-{unique}"));
    fs::create_dir_all(&root).expect("workspace should be created");
    Utf8PathBuf::from_path_buf(root).expect("temp path should be UTF-8")
}

fn write_phase27_source(root: &Utf8Path, outcome: ShareOutcome) {
    fs::create_dir_all(root.as_std_path()).expect("source root should be created");
    let (slot_status, maybe_support) = match outcome {
        ShareOutcome::Accepted | ShareOutcome::Rejected => (
            "passed",
            "asic_correlation_status: passed\nsafe_stop_status: passed\n",
        ),
        ShareOutcome::LiveSubmitResponseObserved => {
            unreachable!("Phase 28 source fixtures do not use Phase 25 outcomes")
        }
        ShareOutcome::BlockedSafePrerequisite => (
            "blocked",
            "asic_bridge_status: blocked\nsafe_stop_status: blocked\n",
        ),
    };
    let common = format!(
        "board: 205\nredaction_status: passed\nraw_artifacts_committed: no\nraw_pool_values_committed: no\nshare_outcome: {}\n{maybe_support}",
        outcome.as_str()
    );
    for file in [
        "summary.md",
        "share-outcome.md",
        "redaction-review.md",
        "conclusion.md",
    ] {
        fs::write(root.join(file).as_std_path(), &common).expect("source file should write");
    }
    fs::write(
        root.join("share-outcome.md").as_std_path(),
        format!("slot: share-outcome\nslot_status: {slot_status}\n{common}"),
    )
    .expect("share source should write");
}

fn snapshot(root: &Utf8Path) -> String {
    let mut entries = fs::read_dir(root.as_std_path())
        .expect("root should read")
        .map(|entry| entry.expect("entry should read"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    let mut output = String::new();
    for entry in entries {
        if !entry.path().is_file() {
            continue;
        }
        output.push_str(&entry.file_name().to_string_lossy());
        output.push('\n');
        output.push_str(&fs::read_to_string(entry.path()).expect("file should read"));
    }
    output
}

fn find_staging_root(workspace: &Utf8Path) -> Option<Utf8PathBuf> {
    fs::read_dir(workspace.as_std_path())
        .expect("workspace should read")
        .filter_map(Result::ok)
        .find_map(|entry| {
            let name = entry.file_name();
            name.to_string_lossy()
                .starts_with(".destination.staging-")
                .then(|| Utf8PathBuf::from_path_buf(entry.path()).expect("path should be UTF-8"))
        })
}
