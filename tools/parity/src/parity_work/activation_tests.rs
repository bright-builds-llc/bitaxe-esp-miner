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
