use std::fs;

use camino::Utf8PathBuf;

use super::find_open_plan;
use crate::ChecklistRow;

fn row(id: &str, status: &str) -> ChecklistRow {
    ChecklistRow {
        id: id.to_owned(),
        surface: id.to_owned(),
        reference_breadcrumb: "reference/source.c".to_owned(),
        rust_owned_target: "crate/src/lib.rs".to_owned(),
        rust_owned_target_markdown: "`crate/src/lib.rs`".to_owned(),
        status: status.to_owned(),
        evidence: "unit".to_owned(),
        notes: String::new(),
    }
}

#[test]
fn next_item_ignores_metadata_less_legacy_plan_for_verified_row() {
    // Arrange
    let workspace = temporary_workspace("legacy-verified");
    write_plan(&workspace, "# Plan\n\n- Parity row: `SELF-001`\n");
    let rows = vec![row("SELF-001", "verified")];

    // Act
    let maybe_open_plan = find_open_plan(&workspace, &rows).expect("open-plan scan");

    // Assert
    assert_eq!(maybe_open_plan, None);
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

#[test]
fn next_item_resumes_explicit_deferred_plan() {
    // Arrange
    let workspace = temporary_workspace("explicit-deferred");
    write_plan(
        &workspace,
        "# Plan\n\n- Parity row: `STR-005`\n- Initial status: `deferred`\n",
    );
    let rows = vec![row("STR-005", "deferred")];

    // Act
    let open_plan = find_open_plan(&workspace, &rows)
        .expect("open-plan scan")
        .expect("deferred plan must resume");

    // Assert
    assert_eq!(open_plan.row_id, "STR-005");
    assert_eq!(open_plan.plan_path, "docs/parity/work-plans/run/PLAN.md");
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

#[test]
fn next_item_resumes_plan_with_status_and_evidence_metadata() {
    // Arrange
    let workspace = temporary_workspace("status-with-evidence");
    let document = "# Plan\n\n- Parity row: `STR-005`\n- Initial status: `implemented | unit,golden,workflow`\n";
    write_plan(&workspace, document);

    // Act
    let open_plan = find_open_plan(&workspace, &[row("STR-005", "implemented")])
        .expect("status and evidence must be parsed separately")
        .expect("unfinished plan must remain open");

    // Assert
    assert_eq!(open_plan.row_id, "STR-005");
    assert_eq!(open_plan.plan_path, "docs/parity/work-plans/run/PLAN.md");
    assert_eq!(
        fs::read_to_string(workspace.join(&open_plan.plan_path)).expect("plan"),
        document
    );
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

#[test]
fn next_item_rejects_status_regression_with_evidence_metadata() {
    // Arrange
    let workspace = temporary_workspace("status-with-evidence-regression");
    write_plan(&workspace, "# Plan\n\n- Parity row: `STR-005`\n- Initial status: `implemented | unit,golden,workflow`\n");

    // Act
    let error = find_open_plan(&workspace, &[row("STR-005", "in-progress")])
        .expect_err("evidence suffix must not conceal a regression");

    // Assert
    assert!(error.to_string().contains("status regressed"));
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

#[test]
fn plan_initial_status_rejects_malformed_evidence_suffixes() {
    for suffix in ["", "unit,", "unit,,golden", "unknown", "unit | workflow"] {
        // Arrange
        let document = format!("- Initial status: `implemented | {suffix}`\n");

        // Act
        let error = super::parse_plan_initial_status(&document)
            .expect_err("malformed evidence suffix must fail");

        // Assert
        assert!(
            error
                .to_string()
                .contains("invalid initial-status evidence"),
            "{error}"
        );
    }
}

#[test]
fn next_item_skips_unrelated_task_plan_beside_open_parity_plan() {
    // Arrange
    let workspace = temporary_workspace("unrelated-task-plan");
    write_plan(
        &workspace,
        "# Plan\n\n- Parity row: `STR-005`\n- Initial status: `implemented`\n",
    );
    let other = workspace.join("docs/parity/work-plans/20260901T000000Z-USB-RECOVERY");
    fs::create_dir_all(&other).expect("task plan directory");
    fs::write(
        other.join("PLAN.md"),
        "# USB recovery\n\n- Active task: `task-usb-recovery`\n",
    )
    .expect("task plan");

    // Act
    let open_plan = find_open_plan(&workspace, &[row("STR-005", "implemented")])
        .expect("unrelated task plans must not block selection")
        .expect("parity plan must remain open");

    // Assert
    assert_eq!(open_plan.plan_path, "docs/parity/work-plans/run/PLAN.md");
    fs::remove_dir_all(workspace).expect("cleanup");
}

#[test]
fn next_item_rejects_row_named_plan_missing_parity_metadata() {
    // Arrange
    let workspace = temporary_workspace("missing-row-metadata");
    let plan = workspace.join("docs/parity/work-plans/20260901T000000Z-STR-005-RETRY");
    fs::create_dir_all(&plan).expect("plan directory");
    fs::write(plan.join("PLAN.md"), "# Incomplete plan\n").expect("plan");

    // Act
    let error = find_open_plan(&workspace, &[row("STR-005", "implemented")])
        .expect_err("row-named plans must declare their row");

    // Assert
    assert!(format!("{error:#}").contains("missing parity-row metadata"));
    fs::remove_dir_all(workspace).expect("cleanup");
}

#[test]
fn next_item_rejects_status_metadata_without_row_metadata() {
    // Arrange
    let workspace = temporary_workspace("status-missing-row");
    write_plan(&workspace, "# Plan\n\n- Initial status: `implemented`\n");

    // Act
    let error = find_open_plan(&workspace, &[row("STR-005", "implemented")])
        .expect_err("partial parity metadata must fail");

    // Assert
    assert!(format!("{error:#}").contains("missing parity-row metadata"));
    fs::remove_dir_all(workspace).expect("cleanup");
}

fn temporary_workspace(label: &str) -> Utf8PathBuf {
    let workspace = Utf8PathBuf::from_path_buf(
        std::env::temp_dir().join(format!("bitaxe-parity-{label}-plan-{}", std::process::id())),
    )
    .expect("temporary path must be UTF-8");
    let _ = fs::remove_dir_all(workspace.as_std_path());
    workspace
}

fn write_plan(workspace: &Utf8PathBuf, document: &str) {
    let plan_root = workspace.join("docs/parity/work-plans/run");
    fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
    fs::write(plan_root.join("PLAN.md").as_std_path(), document).expect("plan");
}
