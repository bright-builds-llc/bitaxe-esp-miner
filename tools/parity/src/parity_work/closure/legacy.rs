use anyhow::{bail, Result};
use camino::Utf8Path;

use crate::phase35_evidence::sha256_hex;

struct LegacyClosureAdmission {
    plan_directory: &'static str,
    plan_sha256: &'static str,
    closure_sha256: &'static str,
    row_id: &'static str,
    final_status: &'static str,
    task_id: &'static str,
    terminal_decision: &'static str,
}

const ADMISSIONS: [LegacyClosureAdmission; 2] = [
    LegacyClosureAdmission {
        plan_directory: "20260826T210025Z-STR-005-NOISE-DIAGNOSTIC",
        plan_sha256: "5c5dcc8b030cd07acb60b00d8414d72bc4ad854550d70dad4b66381940629eec",
        closure_sha256: "b6d17064a63514de9f5f52d536dee2c40c7c99309c82b073ddd58470ff802cbb",
        row_id: "STR-005",
        final_status: "implemented",
        task_id: "task-str005-noise-handshake-diagnostic",
        terminal_decision: "stop_repeated_boundary",
    },
    LegacyClosureAdmission {
        plan_directory: "20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY",
        plan_sha256: "3bbdf04402a0a51c4d380ef4efa65b4ee3d434bf865970c161a7faf0760b6658",
        closure_sha256: "0dc3b0e5300bafdfb229f0a531c6775605e88305bf6bd5410e78955f878cc7d3",
        row_id: "STR-005",
        final_status: "implemented",
        task_id: "task-str005-preconnect-noise-and-verification",
        terminal_decision: "stop_repeated_boundary",
    },
];

const LEGACY_FINAL_STATUS_PREFIX: &str = "- Final parity status: `";
const CANONICAL_ONLY_PREFIXES: [&str; 5] = [
    "- Final status: `",
    "- Outcome: `",
    "- Verification claimed: `",
    "- Plan SHA-256: `",
    "- Active task: `",
];

pub(super) fn closes_plan(
    plan_root: &Utf8Path,
    plan_document: &str,
    closure: &str,
    plan_row_id: &str,
    initial_status: &str,
) -> Result<bool> {
    if !has_metadata(closure, LEGACY_FINAL_STATUS_PREFIX) {
        return Ok(false);
    }
    if CANONICAL_ONLY_PREFIXES
        .iter()
        .any(|prefix| has_metadata(closure, prefix))
    {
        bail!("parity plan closure cannot mix canonical and legacy metadata");
    }

    let plan_directory = admitted_plan_directory(plan_root)?;
    let Some(admission) = ADMISSIONS
        .iter()
        .find(|candidate| candidate.plan_directory == plan_directory)
    else {
        bail!("parity plan legacy closure is not admitted");
    };

    require_match(
        "plan digest",
        &sha256_hex(plan_document.as_bytes()),
        admission.plan_sha256,
    )?;
    require_match(
        "closure digest",
        &sha256_hex(closure.as_bytes()),
        admission.closure_sha256,
    )?;
    require_match(
        "plan identity",
        &super::metadata_value(closure, "- Plan: `", "plan identity")?,
        admission.plan_directory,
    )?;
    require_match(
        "task identity",
        &super::metadata_value(closure, "- Task: `", "task identity")?,
        admission.task_id,
    )?;
    require_match(
        "row identity",
        &super::metadata_value(closure, "- Parity row: `", "parity row")?,
        admission.row_id,
    )?;
    require_match(
        "final status",
        &super::metadata_value(closure, LEGACY_FINAL_STATUS_PREFIX, "final parity status")?,
        admission.final_status,
    )?;
    require_match(
        "terminal decision",
        &super::metadata_value(closure, "- Terminal decision: `", "terminal decision")?,
        admission.terminal_decision,
    )?;
    require_match("plan row", plan_row_id, admission.row_id)?;
    require_match("initial status", initial_status, admission.final_status)?;

    Ok(true)
}

fn admitted_plan_directory(plan_root: &Utf8Path) -> Result<&str> {
    let Some(plan_directory) = plan_root.file_name() else {
        bail!("parity plan legacy closure has no plan directory");
    };
    let Some(work_plans) = plan_root.parent() else {
        bail!("parity plan legacy closure is outside work-plans");
    };
    let Some(parity) = work_plans.parent() else {
        bail!("parity plan legacy closure is outside parity");
    };
    let Some(docs) = parity.parent() else {
        bail!("parity plan legacy closure is outside docs");
    };
    if work_plans.file_name() != Some("work-plans")
        || parity.file_name() != Some("parity")
        || docs.file_name() != Some("docs")
    {
        bail!("parity plan legacy closure path is not admitted");
    }
    Ok(plan_directory)
}

fn has_metadata(document: &str, prefix: &str) -> bool {
    document.lines().any(|line| line.trim().starts_with(prefix))
}

fn require_match(label: &str, actual: &str, expected: &str) -> Result<()> {
    if actual != expected {
        bail!("parity plan legacy closure {label} is not admitted");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use camino::{Utf8Path, Utf8PathBuf};

    use crate::parity_work::find_open_plan;
    use crate::ChecklistRow;

    const LEGACY_STR005_CLOSURES: [(&str, &str, &str); 2] = [
        (
            "20260826T210025Z-STR-005-NOISE-DIAGNOSTIC",
            include_str!(
                "../../../../../docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/PLAN.md"
            ),
            include_str!(
                "../../../../../docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/CLOSURE.md"
            ),
        ),
        (
            "20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY",
            include_str!(
                "../../../../../docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md"
            ),
            include_str!(
                "../../../../../docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/CLOSURE.md"
            ),
        ),
    ];
    const DECOMPOSITION_RUN: &str = "20260828T175218Z-STR-005-DECOMPOSITION";
    const DECOMPOSITION_PLAN: &str = include_str!(
        "../../../../../docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md"
    );
    const DECOMPOSITION_CLOSURE: &str = include_str!(
        "../../../../../docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/CLOSURE.md"
    );

    fn workspace(name: &str) -> Utf8PathBuf {
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-legacy-closure-{name}-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        workspace
    }

    fn write_plan(workspace: &Utf8Path, run: &str, plan: &str, closure: &str) -> Utf8PathBuf {
        let plan_root = workspace.join("docs/parity/work-plans").join(run);
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::write(plan_root.join("PLAN.md").as_std_path(), plan).expect("plan");
        fs::write(
            plan_root.join(super::super::CLOSURE_FILE).as_std_path(),
            closure,
        )
        .expect("closure");
        plan_root
    }

    fn str005_row() -> ChecklistRow {
        ChecklistRow {
            id: "STR-005".to_owned(),
            surface: "Stratum v2 protocol".to_owned(),
            reference_breadcrumb: "reference/source.c".to_owned(),
            rust_owned_target: "crates/bitaxe-stratum/src/v2".to_owned(),
            rust_owned_target_markdown: "`crates/bitaxe-stratum/src/v2`".to_owned(),
            status: "implemented".to_owned(),
            evidence: "unit,golden,workflow".to_owned(),
            notes: String::new(),
        }
    }

    #[test]
    fn exact_legacy_str005_closures_close_their_plans() {
        for (run, plan, closure) in LEGACY_STR005_CLOSURES {
            // Arrange
            let workspace = workspace(run);
            let plan_root = write_plan(&workspace, run, plan, closure);

            // Act
            let closed = super::super::closes_plan(&plan_root, plan, "STR-005", "implemented")
                .expect("admitted legacy closure");

            // Assert
            assert!(closed);
            fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
        }
    }

    #[test]
    fn legacy_str005_lineage_and_canonical_decomposition_are_terminal() {
        // Arrange
        let workspace = workspace("complete-lineage");
        for (run, plan, closure) in LEGACY_STR005_CLOSURES {
            write_plan(&workspace, run, plan, closure);
        }
        write_plan(
            &workspace,
            DECOMPOSITION_RUN,
            DECOMPOSITION_PLAN,
            DECOMPOSITION_CLOSURE,
        );

        // Act
        let maybe_open_plan =
            find_open_plan(&workspace, &[str005_row()]).expect("complete lineage scan");

        // Assert
        assert_eq!(maybe_open_plan, None);
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_plan_directory_drift() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("path-drift");
        let plan_root = write_plan(&workspace, &format!("{run}-COPY"), plan, closure);

        // Act
        let error = super::super::closes_plan(&plan_root, plan, "STR-005", "implemented")
            .expect_err("copied legacy closure must fail");

        // Assert
        assert!(error.to_string().contains("not admitted"), "{error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_plan_byte_drift() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("plan-drift");
        let changed_plan = format!("{plan}\n");
        let plan_root = write_plan(&workspace, run, &changed_plan, closure);

        // Act
        let error = super::super::closes_plan(&plan_root, &changed_plan, "STR-005", "implemented")
            .expect_err("changed legacy plan must fail");

        // Assert
        assert!(error.to_string().contains("plan digest"), "{error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_closure_byte_drift() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("closure-drift");
        let changed_closure = format!("{closure}\n");
        let plan_root = write_plan(&workspace, run, plan, &changed_closure);

        // Act
        let error = super::super::closes_plan(&plan_root, plan, "STR-005", "implemented")
            .expect_err("changed legacy closure must fail");

        // Assert
        assert!(error.to_string().contains("closure digest"), "{error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_metadata_drift() {
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        for (label, changed_closure) in [
            (
                "task",
                closure.replace(
                    "task-str005-noise-handshake-diagnostic",
                    "task-str005-noise-handshake-copy",
                ),
            ),
            (
                "row",
                closure.replace("- Parity row: `STR-005`", "- Parity row: `STR-006`"),
            ),
            (
                "status",
                closure.replace(
                    "- Final parity status: `implemented`",
                    "- Final parity status: `verified`",
                ),
            ),
            (
                "decision",
                closure.replace("stop_repeated_boundary", "stop_hardware_blocker"),
            ),
        ] {
            // Arrange
            let workspace = workspace(label);
            let plan_root = write_plan(&workspace, run, plan, &changed_closure);

            // Act
            let error = super::super::closes_plan(&plan_root, plan, "STR-005", "implemented")
                .expect_err("changed legacy metadata must fail");

            // Assert
            assert!(error.to_string().contains("closure digest"), "{error:#}");
            fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
        }
    }

    #[test]
    fn legacy_closure_rejects_mixed_canonical_metadata() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("mixed-schema");
        let mixed_closure = format!("{closure}\n- Final status: `implemented`\n");
        let plan_root = write_plan(&workspace, run, plan, &mixed_closure);

        // Act
        let error = super::super::closes_plan(&plan_root, plan, "STR-005", "implemented")
            .expect_err("mixed closure schema must fail");

        // Assert
        assert!(error.to_string().contains("cannot mix"), "{error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_caller_row_drift() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("caller-row-drift");
        let plan_root = write_plan(&workspace, run, plan, closure);

        // Act
        let row_error = super::super::closes_plan(&plan_root, plan, "STR-006", "implemented")
            .expect_err("caller row drift must fail");

        // Assert
        assert!(row_error.to_string().contains("plan row"), "{row_error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn legacy_closure_rejects_caller_status_drift() {
        // Arrange
        let (run, plan, closure) = LEGACY_STR005_CLOSURES[0];
        let workspace = workspace("caller-status-drift");
        let plan_root = write_plan(&workspace, run, plan, closure);

        // Act
        let status_error = super::super::closes_plan(&plan_root, plan, "STR-005", "in-progress")
            .expect_err("caller status drift must fail");

        // Assert
        assert!(
            status_error.to_string().contains("initial status"),
            "{status_error:#}"
        );
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }
}
