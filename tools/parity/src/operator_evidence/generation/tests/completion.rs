use super::super::*;
use super::support::{create_workspace, snapshot};

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
fn completion_replaces_failed_generated_slots_on_passed_rerun() {
    // Arrange
    let workspace = create_workspace("completion-failed-to-passed");
    let relative_root = Utf8Path::new("evidence/phase25");
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase25,
        relative_root,
        WorkflowStatus::Failed,
    )
    .expect("failed completion should write closed placeholders");
    let conclusion = workspace.join(relative_root).join("conclusion.md");
    let before = fs::read_to_string(conclusion.as_std_path()).expect("conclusion should read");
    assert!(before.contains("workflow_status: failed"));
    assert!(before.contains("slot_status: blocked"));

    // Act
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase25,
        relative_root,
        WorkflowStatus::Passed,
    )
    .expect("passed rerun should replace generator-owned placeholders");
    let after = fs::read_to_string(conclusion.as_std_path()).expect("conclusion should read");

    // Assert
    assert!(after.contains("workflow_status: passed"));
    assert!(after.contains("slot_status: deferred"));
    assert!(!after.contains("workflow_status: failed"));
}

#[test]
fn completion_replaces_blocked_generated_slots_but_preserves_observed_bytes() {
    // Arrange
    let workspace = create_workspace("completion-blocked-to-passed");
    let relative_root = Utf8Path::new("evidence/phase27");
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase27,
        relative_root,
        WorkflowStatus::Blocked,
    )
    .expect("blocked completion should write closed placeholders");
    let root = workspace.join(relative_root);
    let observed = "wrapper-owned observed conclusion bytes\n";
    fs::write(root.join("conclusion.md").as_std_path(), observed)
        .expect("wrapper-owned slot should replace generated placeholder");

    // Act
    complete_operator_evidence(
        &workspace,
        OperatorEvidenceProfile::Phase27,
        relative_root,
        WorkflowStatus::Passed,
    )
    .expect("passed rerun should replace only generator-owned placeholders");
    let generated =
        fs::read_to_string(root.join("api.md").as_std_path()).expect("API slot should read");

    // Assert
    assert!(generated.contains("workflow_status: passed"));
    assert!(generated.contains("slot_status: deferred"));
    assert_eq!(
        fs::read_to_string(root.join("conclusion.md").as_std_path())
            .expect("conclusion should read"),
        observed
    );
}
