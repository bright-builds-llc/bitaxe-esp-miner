use super::super::*;
use super::support::{create_workspace, snapshot, write_phase27_source};
use crate::operator_evidence::ShareOutcome;

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
        let summary = fs::read_to_string(workspace.join("destination/summary.md").as_std_path())
            .expect("summary should read");
        let manifest = fs::read_to_string(
            workspace
                .join("destination/.phase28-evidence-manifest")
                .as_std_path(),
        )
        .expect("manifest should read");

        // Assert
        assert!(share.contains(&format!("share_outcome: {}", outcome.as_str())));
        assert!(summary.contains(&format!("share_outcome: {}", outcome.as_str())));
        assert!(manifest.lines().any(|line| line == "- summary.md"));
        assert!(share.contains("safe_stop_status: passed"));
        if matches!(outcome, ShareOutcome::Accepted | ShareOutcome::Rejected) {
            assert!(share.contains("asic_correlation_status: passed"));
        } else {
            assert!(share.contains("asic_bridge_status: blocked"));
            assert!(share.contains("source_safe_stop_status: complete"));
        }
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
