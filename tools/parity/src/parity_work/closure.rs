use std::fs;

use anyhow::{bail, Context, Result};
use camino::Utf8Path;

use crate::phase35_evidence::sha256_hex;

pub(super) const CLOSURE_FILE: &str = "CLOSURE.md";

pub(super) fn result_closes_plan(plan_root: &Utf8Path) -> Result<bool> {
    let result_exists = plan_root.join("RESULT.md").exists();
    let closure_exists = plan_root.join(CLOSURE_FILE).exists();
    if result_exists && closure_exists {
        bail!("parity work plan {plan_root} cannot contain both RESULT.md and CLOSURE.md");
    }
    Ok(result_exists)
}

pub(super) fn closes_plan(
    plan_root: &Utf8Path,
    plan_document: &str,
    plan_row_id: &str,
    initial_status: &str,
) -> Result<bool> {
    let closure_path = plan_root.join(CLOSURE_FILE);
    if !closure_path.exists() {
        return Ok(false);
    }
    let closure = fs::read_to_string(closure_path.as_std_path())
        .with_context(|| format!("failed to read parity plan closure {closure_path}"))?;
    let closure_row = metadata_value(&closure, "- Parity row: `", "parity row")?;
    let final_status = metadata_value(&closure, "- Final status: `", "final status")?;
    let outcome = metadata_value(&closure, "- Outcome: `", "outcome")?;
    let verification_claimed =
        metadata_value(&closure, "- Verification claimed: `", "verification claim")?;
    let plan_digest = metadata_value(&closure, "- Plan SHA-256: `", "plan digest")?;
    let active_task = metadata_value(&closure, "- Active task: `", "active task")?;

    if closure_row != plan_row_id {
        bail!("parity plan closure row {closure_row} does not match plan row {plan_row_id}");
    }
    if final_status != initial_status {
        bail!("parity plan closure status {final_status} must match the immutable plan status");
    }
    if !matches!(
        final_status.as_str(),
        "not-started" | "in-progress" | "implemented"
    ) {
        bail!("parity plan closure requires a non-verified unfinished status");
    }
    if !matches!(outcome.as_str(), "blocked" | "cancelled" | "superseded") {
        bail!("parity plan closure has unsupported outcome {outcome}");
    }
    if verification_claimed != "no" {
        bail!("parity plan closure must declare verification claimed as no");
    }
    let expected_digest = sha256_hex(plan_document.as_bytes());
    if plan_digest != expected_digest {
        bail!("parity plan closure digest does not match immutable PLAN.md");
    }
    if !is_task_id(&active_task) {
        bail!("parity plan closure has invalid active task ID");
    }

    for section in ["Closure reason", "Next safe action", "Non-claims"] {
        require_concrete_section(&closure, section)?;
    }
    Ok(true)
}

fn metadata_value(document: &str, prefix: &str, label: &str) -> Result<String> {
    let values = document
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix('`'))
        })
        .collect::<Vec<_>>();
    if values.len() != 1 || !is_concrete(values[0]) {
        bail!("parity plan closure requires exactly one concrete {label}");
    }
    Ok(values[0].to_owned())
}

fn require_concrete_section(document: &str, expected: &str) -> Result<()> {
    let heading = format!("## {expected}");
    let mut bodies = Vec::new();
    let mut maybe_body = None;
    for line in document.lines() {
        if line.starts_with("## ") {
            if let Some(body) = maybe_body.take() {
                bodies.push(body);
            }
            maybe_body = (line == heading).then(String::new);
        } else if let Some(body) = maybe_body.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(body) = maybe_body {
        bodies.push(body);
    }
    if bodies.len() != 1 || !is_concrete(bodies[0].trim()) {
        bail!("parity plan closure requires one non-empty {heading} section");
    }
    Ok(())
}

fn is_concrete(value: &str) -> bool {
    !value.trim().is_empty() && !value.contains('<') && !value.contains('>')
}

fn is_task_id(value: &str) -> bool {
    value.starts_with("task-")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_work::{find_open_plan, ranked_candidates};
    use crate::ChecklistRow;
    use camino::Utf8PathBuf;

    const PLAN: &str = "# Plan\n\n- Parity row: `API-010`\n- Initial status: `implemented`\n";

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

    fn workspace(name: &str) -> (Utf8PathBuf, Utf8PathBuf) {
        let root = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-closure-{name}-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(root.as_std_path());
        let plan_root = root.join("docs/parity/work-plans/run");
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::write(plan_root.join("PLAN.md").as_std_path(), PLAN).expect("plan");
        (root, plan_root)
    }

    fn closure_for(plan: &str) -> String {
        format!(
            "# Parity work closure\n\n- Parity row: `API-010`\n- Final status: `implemented`\n- Outcome: `blocked`\n- Verification claimed: `no`\n- Plan SHA-256: `{}`\n- Active task: `task-api-010`\n\n## Closure reason\n\nHardware evidence is unavailable.\n\n## Next safe action\n\nCreate a fresh task when hardware returns.\n\n## Non-claims\n\nNo verification is claimed.\n",
            sha256_hex(plan.as_bytes())
        )
    }

    fn valid_closure() -> String {
        closure_for(PLAN)
    }

    fn assert_invalid(name: &str, closure: String, expected: &str) {
        let (workspace, plan_root) = workspace(name);
        fs::write(plan_root.join(CLOSURE_FILE).as_std_path(), closure).expect("closure");
        let error = find_open_plan(&workspace, &[row("API-010", "implemented")])
            .expect_err("closure must fail closed");
        assert!(error.to_string().contains(expected), "{error:#}");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn matching_nonverified_closure_closes_plan_but_keeps_row_candidate() {
        // Arrange
        let (workspace, plan_root) = workspace("valid");
        fs::write(plan_root.join(CLOSURE_FILE).as_std_path(), valid_closure()).expect("closure");
        let rows = vec![row("API-010", "implemented")];

        // Act
        let maybe_open_plan = find_open_plan(&workspace, &rows).expect("open-plan scan");
        let candidates = ranked_candidates(&rows);

        // Assert
        assert_eq!(maybe_open_plan, None);
        assert_eq!(candidates[0].row_id, "API-010");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn historical_nonverified_closure_remains_valid_after_row_verification() {
        // Arrange
        let (workspace, plan_root) = workspace("later-verification");
        fs::write(plan_root.join(CLOSURE_FILE).as_std_path(), valid_closure()).expect("closure");

        // Act
        let maybe_open_plan = find_open_plan(&workspace, &[row("API-010", "verified")])
            .expect("historical closure should remain valid after a forward transition");

        // Assert
        assert_eq!(maybe_open_plan, None);
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn result_still_closes_plan_without_closure() {
        // Arrange
        let (workspace, plan_root) = workspace("result");
        fs::write(plan_root.join("RESULT.md").as_std_path(), "result\n").expect("result");

        // Act
        let maybe_open_plan = find_open_plan(&workspace, &[row("API-010", "implemented")])
            .expect("result-based closure should remain valid");

        // Assert
        assert_eq!(maybe_open_plan, None);
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn newest_closure_closes_explicitly_linked_open_lineage() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-closure-lineage-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let older_root = workspace.join("docs/parity/work-plans/20260101T000000Z-API-010");
        let newer_root = workspace.join("docs/parity/work-plans/20260102T000000Z-API-010");
        fs::create_dir_all(older_root.as_std_path()).expect("older root");
        fs::create_dir_all(newer_root.as_std_path()).expect("newer root");
        fs::write(older_root.join("PLAN.md").as_std_path(), PLAN).expect("older plan");
        let newer_plan = format!(
            "{PLAN}\nContinues `docs/parity/work-plans/20260101T000000Z-API-010/PLAN.md`.\n"
        );
        fs::write(newer_root.join("PLAN.md").as_std_path(), &newer_plan).expect("newer plan");
        fs::write(
            newer_root.join(CLOSURE_FILE).as_std_path(),
            closure_for(&newer_plan),
        )
        .expect("closure");

        // Act
        let maybe_open_plan = find_open_plan(&workspace, &[row("API-010", "implemented")])
            .expect("linked closure should validate");

        // Assert
        assert_eq!(maybe_open_plan, None);
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn terminal_lineage_is_retired_before_a_different_row_is_resumed() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-closed-lineage-new-row-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let older_root = workspace.join("docs/parity/work-plans/20260101T000000Z-API-010");
        let closed_root = workspace.join("docs/parity/work-plans/20260102T000000Z-API-010");
        let active_root = workspace.join("docs/parity/work-plans/20260103T000000Z-PWR-001");
        fs::create_dir_all(older_root.as_std_path()).expect("older root");
        fs::create_dir_all(closed_root.as_std_path()).expect("closed root");
        fs::create_dir_all(active_root.as_std_path()).expect("active root");
        fs::write(older_root.join("PLAN.md").as_std_path(), PLAN).expect("older plan");
        let closed_plan = format!(
            "{PLAN}\nContinues `docs/parity/work-plans/20260101T000000Z-API-010/PLAN.md`.\n"
        );
        fs::write(closed_root.join("PLAN.md").as_std_path(), &closed_plan).expect("closed plan");
        fs::write(
            closed_root.join(CLOSURE_FILE).as_std_path(),
            closure_for(&closed_plan),
        )
        .expect("closure");
        fs::write(
            active_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `PWR-001`\n- Initial status: `implemented`\n",
        )
        .expect("active plan");

        // Act
        let open_plan = find_open_plan(
            &workspace,
            &[row("API-010", "implemented"), row("PWR-001", "implemented")],
        )
        .expect("closed lineage must not conflict with a new row")
        .expect("PWR-001 should remain open");

        // Assert
        assert_eq!(open_plan.row_id, "PWR-001");
        assert_eq!(
            open_plan.plan_path,
            "docs/parity/work-plans/20260103T000000Z-PWR-001/PLAN.md"
        );
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn older_closure_does_not_hide_newer_linked_open_plan() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-closure-newer-open-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let older_root = workspace.join("docs/parity/work-plans/20260101T000000Z-API-010");
        let newer_root = workspace.join("docs/parity/work-plans/20260102T000000Z-API-010");
        fs::create_dir_all(older_root.as_std_path()).expect("older root");
        fs::create_dir_all(newer_root.as_std_path()).expect("newer root");
        fs::write(older_root.join("PLAN.md").as_std_path(), PLAN).expect("older plan");
        fs::write(older_root.join(CLOSURE_FILE).as_std_path(), valid_closure()).expect("closure");
        let newer_plan = format!(
            "{PLAN}\nContinues `docs/parity/work-plans/20260101T000000Z-API-010/PLAN.md`.\n"
        );
        fs::write(newer_root.join("PLAN.md").as_std_path(), newer_plan).expect("newer plan");

        // Act
        let open_plan = find_open_plan(&workspace, &[row("API-010", "implemented")])
            .expect("linked plans should validate")
            .expect("newer plan should remain open");

        // Assert
        assert_eq!(
            open_plan.plan_path,
            "docs/parity/work-plans/20260102T000000Z-API-010/PLAN.md"
        );
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn closure_rejects_missing_metadata() {
        // Arrange, Act, Assert
        assert_invalid(
            "missing-metadata",
            valid_closure().replace("- Active task: `task-api-010`\n", ""),
            "active task",
        );
    }

    #[test]
    fn closure_rejects_unsupported_outcome() {
        // Arrange, Act, Assert
        assert_invalid(
            "bad-outcome",
            valid_closure().replace("- Outcome: `blocked`", "- Outcome: `complete`"),
            "unsupported outcome",
        );
    }

    #[test]
    fn closure_rejects_verified_status() {
        // Arrange, Act, Assert
        let (workspace, plan_root) = workspace("verified");
        let closure = valid_closure().replace(
            "- Final status: `implemented`",
            "- Final status: `verified`",
        );
        fs::write(plan_root.join(CLOSURE_FILE).as_std_path(), closure).expect("closure");
        let error = closes_plan(&plan_root, PLAN, "API-010", "verified")
            .expect_err("verified closure must fail");
        assert!(error.to_string().contains("non-verified"));
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn closure_rejects_verification_claim() {
        // Arrange, Act, Assert
        assert_invalid(
            "verification-claim",
            valid_closure().replace(
                "- Verification claimed: `no`",
                "- Verification claimed: `yes`",
            ),
            "verification claimed as no",
        );
    }

    #[test]
    fn closure_rejects_row_mismatch() {
        // Arrange, Act, Assert
        assert_invalid(
            "row-mismatch",
            valid_closure().replace("- Parity row: `API-010`", "- Parity row: `CFG-001`"),
            "does not match plan row",
        );
    }

    #[test]
    fn closure_rejects_status_mismatch() {
        // Arrange, Act, Assert
        assert_invalid(
            "status-mismatch",
            valid_closure().replace(
                "- Final status: `implemented`",
                "- Final status: `in-progress`",
            ),
            "must match the immutable plan status",
        );
    }

    #[test]
    fn closure_rejects_plan_digest_mismatch() {
        // Arrange, Act, Assert
        assert_invalid(
            "digest-mismatch",
            valid_closure().replace(
                &sha256_hex(PLAN.as_bytes()),
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            "digest does not match",
        );
    }

    #[test]
    fn closure_rejects_empty_required_section() {
        // Arrange, Act, Assert
        assert_invalid(
            "empty-section",
            valid_closure().replace(
                "## Next safe action\n\nCreate a fresh task when hardware returns.\n",
                "## Next safe action\n",
            ),
            "Next safe action",
        );
    }

    #[test]
    fn closure_rejects_simultaneous_result() {
        // Arrange
        let (workspace, plan_root) = workspace("dual-artifacts");
        fs::write(plan_root.join(CLOSURE_FILE).as_std_path(), valid_closure()).expect("closure");
        fs::write(plan_root.join("RESULT.md").as_std_path(), "result\n").expect("result");

        // Act
        let error = find_open_plan(&workspace, &[row("API-010", "implemented")])
            .expect_err("dual terminal artifacts must fail");

        // Assert
        assert!(error.to_string().contains("both RESULT.md and CLOSURE.md"));
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }
}
