use std::collections::BTreeSet;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use super::{parse_rows, publish_active_checklist, read};
use crate::phase35_evidence::sha256_hex;
use crate::{parity_work, LocalEnvironment, ReportEnvironment, TransitionItemArgs};

mod migration;

use migration::{read_binding, require_policy, validate_receipt_binding};

const TRANSITIONS_ROOT: &str = "docs/parity/checklist-transitions";
const BASELINE_FILE: &str = "baseline.md";
const SCHEMA: &str = "parity-checklist-transition-v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TransitionReceipt {
    schema_version: String,
    transition_id: String,
    predecessor_sha256: String,
    result_sha256: String,
    row_id: String,
    reference_commit: String,
    plan_path: String,
    plan_sha256: String,
    result_path: Option<String>,
    result_document_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    migration_ledger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    migration_ledger_sha256: Option<String>,
    before_rust_owned_target: String,
    after_rust_owned_target: String,
    before_status: String,
    after_status: String,
    before_evidence: String,
    after_evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    before_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after_notes: Option<String>,
}

pub(super) fn base_document(workspace: &Utf8Path, active: &str) -> Result<String, String> {
    let path = workspace.join(TRANSITIONS_ROOT).join(BASELINE_FILE);
    if !path.exists() {
        return Ok(active.to_owned());
    }
    read(&path, "parity transition baseline")
}

pub(super) fn validate_chain(
    workspace: &Utf8Path,
    baseline: &str,
    active: &str,
) -> Result<(), String> {
    let root = workspace.join(TRANSITIONS_ROOT);
    if !root.exists() {
        return require_same_document(baseline, active);
    }
    let baseline_path = root.join(BASELINE_FILE);
    if !baseline_path.is_file() {
        return Err("parity transition ledger is missing baseline.md".to_owned());
    }
    let stored_baseline = read(&baseline_path, "parity transition baseline")?;
    if stored_baseline != baseline {
        return Err("parity transition baseline does not match comprehensive authority".to_owned());
    }

    let mut receipt_paths = Vec::new();
    for entry in fs::read_dir(root.as_std_path())
        .map_err(|error| format!("failed to read parity transition ledger: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("failed to read parity transition entry: {error}"))?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| "parity transition path is not valid UTF-8".to_owned())?;
        if path.file_name() == Some(BASELINE_FILE) {
            continue;
        }
        if path.extension() != Some("json") || !path.is_file() {
            return Err(format!("unexpected parity transition ledger entry {path}"));
        }
        receipt_paths.push(path);
    }
    receipt_paths.sort();

    let mut current = baseline.to_owned();
    let mut transition_ids = BTreeSet::new();
    for path in receipt_paths {
        let document = read(&path, "parity transition receipt")?;
        let receipt: TransitionReceipt = serde_json::from_str(&document)
            .map_err(|error| format!("invalid parity transition receipt {path}: {error}"))?;
        validate_receipt(workspace, &path, &current, &receipt)?;
        if !transition_ids.insert(receipt.transition_id.clone()) {
            return Err("duplicate parity transition ID".to_owned());
        }
        current = apply_receipt(&current, &receipt)?;
    }
    require_same_document(&current, active)
}

pub(crate) fn transition_item(
    environment: &LocalEnvironment,
    args: &TransitionItemArgs,
) -> Result<String, String> {
    validate_transition_id(&args.transition_id)?;
    parity_work::validate_audit_path(&args.plan, "PLAN.md").map_err(|error| error.to_string())?;
    let plan_document = read(
        &environment.workspace_dir.join(&args.plan),
        "parity work plan",
    )?;
    let result_binding = match &args.maybe_result {
        Some(result_path) => {
            parity_work::validate_audit_path(result_path, "RESULT.md")
                .map_err(|error| error.to_string())?;
            let result_document = read(
                &environment.workspace_dir.join(result_path),
                "parity work result",
            )?;
            Some((
                result_path.to_string(),
                sha256_hex(result_document.as_bytes()),
            ))
        }
        None => None,
    };
    let migration_binding = read_binding(
        &environment.workspace_dir,
        args.maybe_migration_ledger.as_deref(),
    )?;

    let current = super::read_authoritative_checklist(&environment.workspace_dir)?;
    let rows = parse_rows(&current)?;
    let row = rows
        .get(&args.row_id)
        .ok_or_else(|| format!("parity row {} is missing", args.row_id))?;
    let before_status = normalize_status(&row.cells[4])?;
    let after_status = normalize_status(&args.to)?;
    require_policy(
        &before_status,
        &after_status,
        &args.row_id,
        migration_binding.as_ref().map(|binding| &binding.ledger),
    )?;
    if after_status == "verified" {
        if args.evidence.trim().eq_ignore_ascii_case("pending") || args.evidence.trim().is_empty() {
            return Err("verified parity transitions require non-pending evidence".to_owned());
        }
        if result_binding.is_none() {
            return Err("verified parity transitions require a RESULT.md binding".to_owned());
        }
    }

    let after_target = args
        .maybe_rust_owned_target
        .clone()
        .unwrap_or_else(|| row.cells[3].clone());
    let notes_binding = args
        .maybe_notes
        .as_deref()
        .map(|notes| validate_notes(notes).map(|notes| (row.cells[6].clone(), notes)))
        .transpose()?;
    let reference_commit = environment
        .reference_commit()
        .map_err(|error| error.to_string())?;
    let mut receipt = TransitionReceipt {
        schema_version: SCHEMA.to_owned(),
        transition_id: args.transition_id.clone(),
        predecessor_sha256: sha256_hex(current.as_bytes()),
        result_sha256: String::new(),
        row_id: args.row_id.clone(),
        reference_commit,
        plan_path: args.plan.to_string(),
        plan_sha256: sha256_hex(plan_document.as_bytes()),
        result_path: result_binding.as_ref().map(|(path, _)| path.clone()),
        result_document_sha256: result_binding.map(|(_, digest)| digest),
        migration_ledger_path: migration_binding
            .as_ref()
            .map(|binding| binding.path.clone()),
        migration_ledger_sha256: migration_binding.map(|binding| binding.digest),
        before_rust_owned_target: row.cells[3].clone(),
        after_rust_owned_target: after_target,
        before_status,
        after_status,
        before_evidence: row.cells[5].clone(),
        after_evidence: args.evidence.trim().to_owned(),
        before_notes: notes_binding.as_ref().map(|(before, _)| before.clone()),
        after_notes: notes_binding.map(|(_, after)| after),
    };
    require_changed_mutable_cell(&receipt)?;
    let projected = apply_receipt(&current, &receipt)?;
    receipt.result_sha256 = sha256_hex(projected.as_bytes());

    let root = environment.workspace_dir.join(TRANSITIONS_ROOT);
    let baseline_path = root.join(BASELINE_FILE);
    let receipt_path = root.join(format!("{}.json", receipt.transition_id));
    if receipt_path.exists() {
        return Err("parity transition receipt already exists".to_owned());
    }
    fs::create_dir_all(root.as_std_path())
        .map_err(|error| format!("failed to create parity transition ledger: {error}"))?;
    if !baseline_path.exists() {
        fs::write(baseline_path.as_std_path(), &current)
            .map_err(|error| format!("failed to write parity transition baseline: {error}"))?;
    }
    let receipt_document = format!(
        "{}\n",
        serde_json::to_string_pretty(&receipt)
            .map_err(|error| format!("failed to serialize parity transition receipt: {error}"))?
    );
    fs::write(receipt_path.as_std_path(), receipt_document)
        .map_err(|error| format!("failed to write parity transition receipt: {error}"))?;
    if let Err(error) = publish_active_checklist(&environment.workspace_dir, &projected) {
        let _ = fs::remove_file(receipt_path.as_std_path());
        return Err(error);
    }
    let validated = super::read_authoritative_checklist(&environment.workspace_dir)?;
    if validated != projected {
        return Err("published parity transition did not validate".to_owned());
    }
    Ok(format!(
        "transition_id={} row={} status={} checklist_sha256={}",
        receipt.transition_id, receipt.row_id, receipt.after_status, receipt.result_sha256
    ))
}

fn validate_receipt(
    workspace: &Utf8Path,
    path: &Utf8Path,
    predecessor: &str,
    receipt: &TransitionReceipt,
) -> Result<(), String> {
    validate_transition_id(&receipt.transition_id)?;
    if path.file_stem() != Some(receipt.transition_id.as_str())
        || receipt.schema_version != SCHEMA
        || receipt.predecessor_sha256 != sha256_hex(predecessor.as_bytes())
    {
        return Err(format!("parity transition binding mismatch for {path}"));
    }
    require_full_commit(&receipt.reference_commit)?;
    let plan_path = Utf8Path::new(&receipt.plan_path);
    parity_work::validate_audit_path(plan_path, "PLAN.md").map_err(|error| error.to_string())?;
    let plan = read(&workspace.join(plan_path), "parity transition work plan")?;
    if receipt.plan_sha256 != sha256_hex(plan.as_bytes()) {
        return Err("parity transition plan digest mismatch".to_owned());
    }
    match (&receipt.result_path, &receipt.result_document_sha256) {
        (Some(result_path), Some(result_digest)) => {
            let result_path = Utf8Path::new(result_path);
            parity_work::validate_audit_path(result_path, "RESULT.md")
                .map_err(|error| error.to_string())?;
            let result = read(&workspace.join(result_path), "parity transition result")?;
            if *result_digest != sha256_hex(result.as_bytes()) {
                return Err("parity transition result digest mismatch".to_owned());
            }
        }
        (None, None) => {}
        _ => return Err("parity transition result binding is incomplete".to_owned()),
    }
    let migration_ledger = validate_receipt_binding(
        workspace,
        receipt.migration_ledger_path.as_deref(),
        receipt.migration_ledger_sha256.as_deref(),
    )?;
    require_policy(
        &normalize_status(&receipt.before_status)?,
        &normalize_status(&receipt.after_status)?,
        &receipt.row_id,
        migration_ledger.as_ref(),
    )?;
    require_changed_mutable_cell(receipt)?;
    if receipt.after_status == "verified"
        && (receipt.result_path.is_none()
            || receipt
                .after_evidence
                .trim()
                .eq_ignore_ascii_case("pending")
            || receipt.after_evidence.trim().is_empty())
    {
        return Err("verified parity transition lacks result or evidence".to_owned());
    }
    let projected = apply_receipt(predecessor, receipt)?;
    if receipt.result_sha256 != sha256_hex(projected.as_bytes()) {
        return Err("parity transition result digest mismatch".to_owned());
    }
    Ok(())
}

fn apply_receipt(predecessor: &str, receipt: &TransitionReceipt) -> Result<String, String> {
    let rows = parse_rows(predecessor)?;
    let row = rows
        .get(&receipt.row_id)
        .ok_or_else(|| format!("parity transition row {} is missing", receipt.row_id))?;
    if row.cells[3] != receipt.before_rust_owned_target
        || normalize_status(&row.cells[4])? != receipt.before_status
        || row.cells[5] != receipt.before_evidence
    {
        return Err(format!(
            "parity transition before-state mismatch for {}",
            receipt.row_id
        ));
    }
    match (&receipt.before_notes, &receipt.after_notes) {
        (Some(before), Some(_)) if row.cells[6] != *before => {
            return Err(format!(
                "parity transition notes before-state mismatch for {}",
                receipt.row_id
            ));
        }
        (Some(_), Some(_)) | (None, None) => {}
        _ => return Err("parity transition notes binding is incomplete".to_owned()),
    }
    let mut lines = predecessor.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut cells = row.cells.clone();
    cells[3] = receipt.after_rust_owned_target.clone();
    cells[4] = receipt.after_status.clone();
    cells[5] = receipt.after_evidence.clone();
    if let Some(after_notes) = &receipt.after_notes {
        cells[6] = after_notes.clone();
    }
    lines[row.line_index] = format!("| {} |", cells.join(" | "));
    let mut projected = lines.join("\n");
    if predecessor.ends_with('\n') {
        projected.push('\n');
    }
    Ok(projected)
}

fn require_changed_mutable_cell(receipt: &TransitionReceipt) -> Result<(), String> {
    let notes_unchanged = match (&receipt.before_notes, &receipt.after_notes) {
        (None, None) => true,
        (Some(before), Some(after)) => before == after,
        _ => false,
    };
    if receipt.before_rust_owned_target == receipt.after_rust_owned_target
        && receipt.before_status == receipt.after_status
        && receipt.before_evidence == receipt.after_evidence
        && notes_unchanged
    {
        return Err("parity transition must change at least one mutable checklist cell".to_owned());
    }
    Ok(())
}

fn validate_notes(notes: &str) -> Result<String, String> {
    let notes = notes.trim();
    if notes.is_empty() || notes.contains('|') || notes.contains('\n') || notes.contains('\r') {
        return Err(
            "transition notes must be nonempty single-line Markdown without pipes".to_owned(),
        );
    }
    Ok(notes.to_owned())
}

fn normalize_status(status: &str) -> Result<String, String> {
    let normalized = status.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "not-started" | "in-progress" | "implemented" | "verified" | "deferred"
    ) {
        return Ok(normalized);
    }
    Err(format!("unknown parity status {status}"))
}

fn validate_transition_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(
            "transition ID must contain only ASCII letters, digits, hyphens, or underscores"
                .to_owned(),
        );
    }
    Ok(())
}

fn require_full_commit(value: &str) -> Result<(), String> {
    if value.len() != 40 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err("parity transition reference commit is invalid".to_owned());
    }
    Ok(())
}

fn require_same_document(expected: &str, actual: &str) -> Result<(), String> {
    if expected != actual {
        return Err("active parity checklist does not match its transition ledger".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKLIST: &str = "\
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |\n\
| --- | --- | --- | --- | --- | --- | --- |\n\
| STR-001 | Socket | reference/source.c | crate/src/lib.rs | implemented | pending | Note. |\n";

    fn receipt() -> TransitionReceipt {
        TransitionReceipt {
            schema_version: SCHEMA.to_owned(),
            transition_id: "20260802T120000Z-STR-001".to_owned(),
            predecessor_sha256: sha256_hex(CHECKLIST.as_bytes()),
            result_sha256: String::new(),
            row_id: "STR-001".to_owned(),
            reference_commit: "a".repeat(40),
            plan_path: "docs/parity/work-plans/run/PLAN.md".to_owned(),
            plan_sha256: "b".repeat(64),
            result_path: Some("docs/parity/work-plans/run/RESULT.md".to_owned()),
            result_document_sha256: Some("c".repeat(64)),
            migration_ledger_path: None,
            migration_ledger_sha256: None,
            before_rust_owned_target: "crate/src/lib.rs".to_owned(),
            after_rust_owned_target: "crate/src/lib.rs".to_owned(),
            before_status: "implemented".to_owned(),
            after_status: "verified".to_owned(),
            before_evidence: "pending".to_owned(),
            after_evidence: "unit".to_owned(),
            before_notes: None,
            after_notes: None,
        }
    }

    fn valid_chain(label: &str) -> (Utf8PathBuf, String) {
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-transition-{label}-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let plan_root = workspace.join("docs/parity/work-plans/run");
        let ledger_root = workspace.join(TRANSITIONS_ROOT);
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::create_dir_all(ledger_root.as_std_path()).expect("ledger root");
        fs::write(plan_root.join("PLAN.md").as_std_path(), "plan\n").expect("plan");
        fs::write(plan_root.join("RESULT.md").as_std_path(), "result\n").expect("result");
        fs::write(ledger_root.join(BASELINE_FILE).as_std_path(), CHECKLIST).expect("baseline");

        let mut receipt = receipt();
        receipt.plan_sha256 = sha256_hex(b"plan\n");
        receipt.result_document_sha256 = Some(sha256_hex(b"result\n"));
        let projected = apply_receipt(CHECKLIST, &receipt).expect("projected checklist");
        receipt.result_sha256 = sha256_hex(projected.as_bytes());
        let receipt_document =
            serde_json::to_string_pretty(&receipt).expect("transition receipt JSON");
        fs::write(
            ledger_root
                .join(format!("{}.json", receipt.transition_id))
                .as_std_path(),
            receipt_document,
        )
        .expect("receipt");
        (workspace, projected)
    }

    #[test]
    fn receipt_changes_only_the_selected_mutable_cells() {
        // Arrange
        let receipt = receipt();

        // Act
        let projected = apply_receipt(CHECKLIST, &receipt).expect("transition should apply");

        // Assert
        assert!(projected.contains("| verified | unit | Note. |"));
        assert!(projected.contains("| STR-001 | Socket | reference/source.c |"));
    }

    #[test]
    fn receipt_projects_hash_bound_notes_when_requested() {
        // Arrange
        let mut receipt = receipt();
        receipt.before_notes = Some("Note.".to_owned());
        receipt.after_notes = Some("Verified evidence.".to_owned());

        // Act
        let projected = apply_receipt(CHECKLIST, &receipt).expect("transition should apply");

        // Assert
        assert!(projected.contains("| verified | unit | Verified evidence. |"));
        assert!(!projected.contains("| verified | unit | Note. |"));
    }

    #[test]
    fn receipt_rejects_incomplete_notes_binding() {
        // Arrange
        let mut receipt = receipt();
        receipt.after_notes = Some("Verified evidence.".to_owned());

        // Act
        let error = apply_receipt(CHECKLIST, &receipt)
            .expect_err("incomplete notes binding must fail closed");

        // Assert
        assert!(error.contains("notes binding is incomplete"));
    }

    #[test]
    fn transition_rejects_status_regression() {
        // Arrange
        let before = "implemented";
        let after = "in-progress";

        // Act
        let error = require_policy(before, after, "STR-001", None)
            .expect_err("regression must be rejected");

        // Assert
        assert!(error.contains("monotonically"));
    }

    #[test]
    fn transition_accepts_same_status_for_a_metadata_revision() {
        // Arrange
        let status = "implemented";

        // Act
        let result = require_policy(status, status, "STR-001", None);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn receipt_requires_at_least_one_mutable_cell_change() {
        // Arrange
        let mut receipt = receipt();
        receipt.after_status = receipt.before_status.clone();
        receipt.after_evidence = receipt.before_evidence.clone();

        // Act
        let error = require_changed_mutable_cell(&receipt)
            .expect_err("an exact no-op transition must be rejected");

        // Assert
        assert!(error.contains("at least one mutable checklist cell"));
    }

    #[test]
    fn receipt_accepts_same_status_when_the_target_changes() {
        // Arrange
        let mut receipt = receipt();
        receipt.after_status = receipt.before_status.clone();
        receipt.after_evidence = receipt.before_evidence.clone();
        receipt.after_rust_owned_target = "crate/src/new_owner.rs".to_owned();

        // Act
        let result = require_changed_mutable_cell(&receipt);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn transition_rejects_verified_source_rows() {
        // Arrange
        let before = "verified";
        let after = "verified";

        // Act
        let error = require_policy(before, after, "STR-001", None)
            .expect_err("verified source must be terminal");

        // Assert
        assert!(error.contains("forbidden"));
    }

    #[test]
    fn transition_chain_accepts_hash_bound_plan_and_result() {
        // Arrange
        let (workspace, projected) = valid_chain("accepted");

        // Act
        let result = validate_chain(&workspace, CHECKLIST, &projected);

        // Assert
        assert!(result.is_ok());
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn transition_chain_rejects_result_document_tampering() {
        // Arrange
        let (workspace, projected) = valid_chain("tampered-result");
        fs::write(
            workspace
                .join("docs/parity/work-plans/run/RESULT.md")
                .as_std_path(),
            "tampered\n",
        )
        .expect("tamper result");

        // Act
        let error =
            validate_chain(&workspace, CHECKLIST, &projected).expect_err("tampering must fail");

        // Assert
        assert!(error.contains("result digest mismatch"));
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }
}
