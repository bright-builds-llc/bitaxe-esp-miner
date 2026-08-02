//! Hash-bound, documentation-only parity checklist revisions.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use camino::Utf8Path;
use serde::{Deserialize, Serialize};

use crate::operator_evidence::read_phase36_authoritative_snapshot;
use crate::phase35_evidence::sha256_hex;

mod comprehensive;

pub(crate) const CURRENT_REVISION_ID: &str = "2026-07-28-runtime-display-documentation";
pub(crate) const CURRENT_REVISION_SPEC: &str =
    "docs/parity/checklist-revisions/2026-07-28-runtime-display-documentation.json";
pub(crate) const CURRENT_REVISION_ROOT: &str =
    "docs/parity/evidence/checklist-revisions/2026-07-28-runtime-display-documentation";
const ACTIVE_CHECKLIST: &str = "docs/parity/checklist.md";
const PHASE36_ROOT: &str =
    "docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion";
const SNAPSHOT_FILE: &str = "checklist.md";
const MANIFEST_FILE: &str = "manifest.json";
const SPEC_SCHEMA: &str = "parity-checklist-documentation-revision-v1";
const MANIFEST_SCHEMA: &str = "parity-checklist-documentation-manifest-v1";

#[derive(Debug, Clone, Copy)]
struct RevisionAuthority {
    revision_id: &'static str,
    spec_path: &'static str,
    revision_root: &'static str,
}

const PREDECESSOR_REVISIONS: [RevisionAuthority; 2] = [
    RevisionAuthority {
        revision_id: "2026-07-26-source-pointer-refresh",
        spec_path: "docs/parity/checklist-revisions/2026-07-26-source-pointer-refresh.json",
        revision_root: "docs/parity/evidence/checklist-revisions/2026-07-26-source-pointer-refresh",
    },
    RevisionAuthority {
        revision_id: "2026-07-27-module-ownership-refactor",
        spec_path: "docs/parity/checklist-revisions/2026-07-27-module-ownership-refactor.json",
        revision_root:
            "docs/parity/evidence/checklist-revisions/2026-07-27-module-ownership-refactor",
    },
];
const CURRENT_PREDECESSOR_ROOT: &str =
    PREDECESSOR_REVISIONS[PREDECESSOR_REVISIONS.len() - 1].revision_root;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RevisionSpec {
    schema_version: String,
    revision_id: String,
    predecessor_sha256: String,
    changes: Vec<RowChange>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RowChange {
    row_id: String,
    before_rust_owned_target: String,
    after_rust_owned_target: String,
    after_notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RevisionManifest {
    schema_version: String,
    revision_id: String,
    predecessor_path: String,
    predecessor_sha256: String,
    change_spec_path: String,
    change_spec_sha256: String,
    affected_rows: Vec<String>,
    checklist_sha256: String,
}

#[derive(Debug, Clone)]
struct MarkdownRow {
    line_index: usize,
    cells: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicationOutcome {
    pub(crate) revision_id: String,
    pub(crate) affected_rows: usize,
    pub(crate) checklist_sha256: String,
}

pub(crate) fn publish_current_revision(
    workspace: &Utf8Path,
    change_spec: &Utf8Path,
) -> Result<PublicationOutcome, String> {
    if change_spec != Utf8Path::new(CURRENT_REVISION_SPEC) {
        return Err("change specification is not the configured current revision".to_owned());
    }
    let spec_path = workspace.join(change_spec);
    let spec_document = read(&spec_path, "checklist revision specification")?;
    let spec = parse_spec(&spec_document)?;
    require_current_revision(&spec)?;
    let predecessor = read_predecessor_authority(workspace)?;
    let predecessor_sha256 = sha256_hex(predecessor.as_bytes());
    if predecessor_sha256 != spec.predecessor_sha256 {
        return Err("checklist revision predecessor digest mismatch".to_owned());
    }
    let projected = apply_spec(&predecessor, &spec)?;
    let checklist_sha256 = sha256_hex(projected.as_bytes());
    let affected_rows = spec
        .changes
        .iter()
        .map(|change| change.row_id.clone())
        .collect::<Vec<_>>();
    let manifest = RevisionManifest {
        schema_version: MANIFEST_SCHEMA.to_owned(),
        revision_id: spec.revision_id.clone(),
        predecessor_path: format!("{CURRENT_PREDECESSOR_ROOT}/{SNAPSHOT_FILE}"),
        predecessor_sha256,
        change_spec_path: CURRENT_REVISION_SPEC.to_owned(),
        change_spec_sha256: sha256_hex(spec_document.as_bytes()),
        affected_rows,
        checklist_sha256: checklist_sha256.clone(),
    };
    let manifest_document = format!(
        "{}\n",
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| format!("failed to serialize checklist revision manifest: {error}"))?
    );
    publish_documents(workspace, &projected, &manifest_document)?;
    let validated = read_authoritative_checklist(workspace)?;
    if validated != projected {
        return Err("published checklist revision did not validate".to_owned());
    }
    Ok(PublicationOutcome {
        revision_id: spec.revision_id,
        affected_rows: spec.changes.len(),
        checklist_sha256,
    })
}

pub(crate) fn read_authoritative_checklist(workspace: &Utf8Path) -> Result<String, String> {
    let predecessor = read_predecessor_authority(workspace)?;
    let spec_path = workspace.join(CURRENT_REVISION_SPEC);
    let spec_document = read(&spec_path, "checklist revision specification")?;
    let spec = parse_spec(&spec_document)?;
    require_revision(&spec, CURRENT_REVISION_ID)?;
    let predecessor_sha256 = sha256_hex(predecessor.as_bytes());
    if predecessor_sha256 != spec.predecessor_sha256 {
        return Err("checklist revision predecessor digest mismatch".to_owned());
    }
    let expected = apply_spec(&predecessor, &spec)?;

    let revision_root = workspace.join(CURRENT_REVISION_ROOT);
    let snapshot = read(
        &revision_root.join(SNAPSHOT_FILE),
        "authoritative checklist revision snapshot",
    )?;
    let manifest_document = read(
        &revision_root.join(MANIFEST_FILE),
        "checklist revision manifest",
    )?;
    let manifest: RevisionManifest = serde_json::from_str(&manifest_document)
        .map_err(|error| format!("invalid checklist revision manifest: {error}"))?;
    validate_manifest(
        &manifest,
        &spec,
        &spec_document,
        &predecessor,
        &snapshot,
        &format!("{CURRENT_PREDECESSOR_ROOT}/{SNAPSHOT_FILE}"),
        CURRENT_REVISION_SPEC,
    )?;
    if snapshot != expected {
        return Err(
            "checklist revision snapshot does not match its change specification".to_owned(),
        );
    }
    let active = read(&workspace.join(ACTIVE_CHECKLIST), "active parity checklist")?;
    comprehensive::validate(workspace, &snapshot, &active)?;
    Ok(active)
}

#[cfg(test)]
fn validate_active_snapshot(active: &str, snapshot: &str) -> Result<(), String> {
    if active != snapshot {
        return Err(
            "active parity checklist does not match the authoritative revision snapshot".to_owned(),
        );
    }
    Ok(())
}

fn parse_spec(document: &str) -> Result<RevisionSpec, String> {
    let spec: RevisionSpec = serde_json::from_str(document)
        .map_err(|error| format!("invalid checklist revision specification: {error}"))?;
    if spec.schema_version != SPEC_SCHEMA {
        return Err("unsupported checklist revision specification schema".to_owned());
    }
    if spec.changes.is_empty() {
        return Err("checklist revision must contain at least one row change".to_owned());
    }
    let unique_rows = spec
        .changes
        .iter()
        .map(|change| change.row_id.as_str())
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != spec.changes.len() {
        return Err("checklist revision contains duplicate row changes".to_owned());
    }
    Ok(spec)
}

fn require_current_revision(spec: &RevisionSpec) -> Result<(), String> {
    require_revision(spec, CURRENT_REVISION_ID)
}

fn require_revision(spec: &RevisionSpec, expected_revision_id: &str) -> Result<(), String> {
    if spec.revision_id != expected_revision_id {
        return Err("checklist revision ID does not match the configured authority".to_owned());
    }
    Ok(())
}

fn read_predecessor_authority(workspace: &Utf8Path) -> Result<String, String> {
    let mut predecessor =
        read_phase36_authoritative_snapshot(workspace, Utf8Path::new(PHASE36_ROOT))
            .map_err(|error| error.to_string())?;
    let mut predecessor_path = format!("{PHASE36_ROOT}/{SNAPSHOT_FILE}");
    for authority in PREDECESSOR_REVISIONS {
        predecessor = read_revision_authority(
            workspace,
            authority.revision_id,
            authority.spec_path,
            authority.revision_root,
            &predecessor_path,
            &predecessor,
        )?;
        predecessor_path = format!("{}/{SNAPSHOT_FILE}", authority.revision_root);
    }
    Ok(predecessor)
}

fn read_revision_authority(
    workspace: &Utf8Path,
    revision_id: &str,
    spec_path: &str,
    revision_root: &str,
    predecessor_path: &str,
    predecessor: &str,
) -> Result<String, String> {
    let spec_document = read(
        &workspace.join(spec_path),
        "predecessor checklist revision specification",
    )?;
    let spec = parse_spec(&spec_document)?;
    require_revision(&spec, revision_id)?;
    if sha256_hex(predecessor.as_bytes()) != spec.predecessor_sha256 {
        return Err("checklist revision predecessor digest mismatch".to_owned());
    }
    let expected = apply_spec(predecessor, &spec)?;
    let revision_root = workspace.join(revision_root);
    let snapshot = read(
        &revision_root.join(SNAPSHOT_FILE),
        "predecessor checklist revision snapshot",
    )?;
    let manifest_document = read(
        &revision_root.join(MANIFEST_FILE),
        "predecessor checklist revision manifest",
    )?;
    let manifest: RevisionManifest = serde_json::from_str(&manifest_document)
        .map_err(|error| format!("invalid checklist revision manifest: {error}"))?;
    validate_manifest(
        &manifest,
        &spec,
        &spec_document,
        predecessor,
        &snapshot,
        predecessor_path,
        spec_path,
    )?;
    if snapshot != expected {
        return Err(
            "checklist revision snapshot does not match its change specification".to_owned(),
        );
    }
    Ok(snapshot)
}

fn apply_spec(predecessor: &str, spec: &RevisionSpec) -> Result<String, String> {
    let rows = parse_rows(predecessor)?;
    let mut lines = predecessor.lines().map(str::to_owned).collect::<Vec<_>>();
    for change in &spec.changes {
        let row = rows
            .get(&change.row_id)
            .ok_or_else(|| format!("checklist revision row {} is missing", change.row_id))?;
        if row.cells[3] != change.before_rust_owned_target {
            return Err(format!(
                "checklist revision before-state mismatch for {}",
                change.row_id
            ));
        }
        let mut cells = row.cells.clone();
        cells[3] = change.after_rust_owned_target.clone();
        if let Some(after_notes) = &change.after_notes {
            cells[6] = after_notes.clone();
        }
        lines[row.line_index] = format!("| {} |", cells.join(" | "));
    }
    let mut projected = lines.join("\n");
    if predecessor.ends_with('\n') {
        projected.push('\n');
    }
    verify_only_documentation_changed(predecessor, &projected, spec)?;
    Ok(projected)
}

fn parse_rows(checklist: &str) -> Result<BTreeMap<String, MarkdownRow>, String> {
    let mut rows = BTreeMap::new();
    for (line_index, line) in checklist.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_owned())
            .collect::<Vec<_>>();
        if cells.first().is_some_and(|cell| cell == "ID")
            || cells.iter().all(|cell| {
                !cell.is_empty()
                    && cell
                        .chars()
                        .all(|character| matches!(character, '-' | ':' | ' '))
            })
        {
            continue;
        }
        if cells.len() != 7 {
            return Err(format!(
                "invalid checklist row at line {}: expected 7 columns",
                line_index + 1
            ));
        }
        let row_id = cells[0].trim_matches('`').to_owned();
        if rows
            .insert(row_id.clone(), MarkdownRow { line_index, cells })
            .is_some()
        {
            return Err(format!("duplicate checklist row {row_id}"));
        }
    }
    Ok(rows)
}

fn verify_only_documentation_changed(
    before: &str,
    after: &str,
    spec: &RevisionSpec,
) -> Result<(), String> {
    let before_rows = parse_rows(before)?;
    let after_rows = parse_rows(after)?;
    if before_rows.keys().collect::<Vec<_>>() != after_rows.keys().collect::<Vec<_>>() {
        return Err("checklist revision changed the row set".to_owned());
    }
    let allowed = spec
        .changes
        .iter()
        .map(|change| change.row_id.as_str())
        .collect::<BTreeSet<_>>();
    for (row_id, before_row) in &before_rows {
        let after_row = after_rows
            .get(row_id)
            .ok_or_else(|| format!("checklist revision removed row {row_id}"))?;
        for column in [0, 1, 2, 4, 5] {
            if before_row.cells[column] != after_row.cells[column] {
                return Err(format!(
                    "checklist revision changed protected column {column} for {row_id}"
                ));
            }
        }
        if !allowed.contains(row_id.as_str()) && before_row.cells != after_row.cells {
            return Err(format!(
                "checklist revision changed undeclared row {row_id}"
            ));
        }
    }
    Ok(())
}

fn validate_manifest(
    manifest: &RevisionManifest,
    spec: &RevisionSpec,
    spec_document: &str,
    predecessor: &str,
    snapshot: &str,
    predecessor_path: &str,
    change_spec_path: &str,
) -> Result<(), String> {
    let expected_rows = spec
        .changes
        .iter()
        .map(|change| change.row_id.clone())
        .collect::<Vec<_>>();
    if manifest.schema_version != MANIFEST_SCHEMA
        || manifest.revision_id != spec.revision_id
        || manifest.predecessor_path != predecessor_path
        || manifest.predecessor_sha256 != sha256_hex(predecessor.as_bytes())
        || manifest.change_spec_path != change_spec_path
        || manifest.change_spec_sha256 != sha256_hex(spec_document.as_bytes())
        || manifest.affected_rows != expected_rows
        || manifest.checklist_sha256 != sha256_hex(snapshot.as_bytes())
    {
        return Err("checklist revision manifest binding mismatch".to_owned());
    }
    Ok(())
}

fn publish_documents(workspace: &Utf8Path, checklist: &str, manifest: &str) -> Result<(), String> {
    let destination = workspace.join(CURRENT_REVISION_ROOT);
    if destination.exists() {
        let existing_checklist = read(
            &destination.join(SNAPSHOT_FILE),
            "existing checklist revision snapshot",
        )?;
        let existing_manifest = read(
            &destination.join(MANIFEST_FILE),
            "existing checklist revision manifest",
        )?;
        if existing_checklist == checklist && existing_manifest == manifest {
            return publish_active_checklist(workspace, checklist);
        }
        return Err("checklist revision destination conflicts with publication".to_owned());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "checklist revision destination has no parent".to_owned())?;
    fs::create_dir_all(parent.as_std_path())
        .map_err(|error| format!("failed to create checklist revision parent: {error}"))?;
    let staging = parent.join(format!(".{CURRENT_REVISION_ID}.staging"));
    if staging.exists() {
        return Err("checklist revision staging directory already exists".to_owned());
    }
    fs::create_dir(staging.as_std_path())
        .map_err(|error| format!("failed to create checklist revision staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join(SNAPSHOT_FILE).as_std_path(), checklist)
            .map_err(|error| format!("failed to write checklist revision snapshot: {error}"))?;
        fs::write(staging.join(MANIFEST_FILE).as_std_path(), manifest)
            .map_err(|error| format!("failed to write checklist revision manifest: {error}"))?;
        fs::rename(staging.as_std_path(), destination.as_std_path())
            .map_err(|error| format!("failed to publish checklist revision: {error}"))?;

        publish_active_checklist(workspace, checklist)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(staging.as_std_path());
        let _ = fs::remove_dir_all(destination.as_std_path());
    }
    result
}

fn publish_active_checklist(workspace: &Utf8Path, checklist: &str) -> Result<(), String> {
    let active = workspace.join(ACTIVE_CHECKLIST);
    let replacement = active.with_file_name(format!(
        ".{}.revision-replacement",
        active
            .file_name()
            .ok_or_else(|| "active checklist has no file name".to_owned())?
    ));
    fs::write(replacement.as_std_path(), checklist)
        .map_err(|error| format!("failed to write active checklist replacement: {error}"))?;
    fs::rename(replacement.as_std_path(), active.as_std_path())
        .map_err(|error| format!("failed to publish active checklist: {error}"))
}

fn read(path: &Utf8Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path.as_std_path())
        .map_err(|error| format!("failed to read {label}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKLIST: &str = "\
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |\n\
| --- | --- | --- | --- | --- | --- | --- |\n\
| STR-001 | Socket | `reference/source.c` | `old.rs` | implemented | unit | Old note. |\n";

    fn spec() -> RevisionSpec {
        RevisionSpec {
            schema_version: SPEC_SCHEMA.to_owned(),
            revision_id: CURRENT_REVISION_ID.to_owned(),
            predecessor_sha256: sha256_hex(CHECKLIST.as_bytes()),
            changes: vec![RowChange {
                row_id: "STR-001".to_owned(),
                before_rust_owned_target: "`old.rs`".to_owned(),
                after_rust_owned_target: "`new.rs`".to_owned(),
                after_notes: Some("New note.".to_owned()),
            }],
        }
    }

    #[test]
    fn revision_changes_only_targets_and_notes() {
        // Arrange
        let spec = spec();

        // Act
        let revised = apply_spec(CHECKLIST, &spec).expect("revision should apply");

        // Assert
        assert!(revised.contains("`new.rs`"));
        assert!(revised.contains("New note."));
        assert!(revised.contains("| implemented | unit |"));
    }

    #[test]
    fn revision_rejects_before_state_drift() {
        // Arrange
        let mut spec = spec();
        spec.changes[0].before_rust_owned_target = "`wrong.rs`".to_owned();

        // Act
        let error = apply_spec(CHECKLIST, &spec).expect_err("drift must fail");

        // Assert
        assert!(error.contains("before-state mismatch"));
    }

    #[test]
    fn revision_validation_rejects_unauthorized_status_change() {
        // Arrange
        let spec = spec();
        let changed = CHECKLIST.replace("| implemented |", "| verified |");

        // Act
        let error = verify_only_documentation_changed(CHECKLIST, &changed, &spec)
            .expect_err("protected status must fail");

        // Assert
        assert!(error.contains("protected column"));
    }

    #[test]
    fn manifest_validation_rejects_digest_drift() {
        // Arrange
        let spec = spec();
        let spec_document = serde_json::to_string(&spec).expect("spec JSON");
        let snapshot = apply_spec(CHECKLIST, &spec).expect("snapshot");
        let manifest = RevisionManifest {
            schema_version: MANIFEST_SCHEMA.to_owned(),
            revision_id: CURRENT_REVISION_ID.to_owned(),
            predecessor_path: format!("{CURRENT_PREDECESSOR_ROOT}/{SNAPSHOT_FILE}"),
            predecessor_sha256: sha256_hex(CHECKLIST.as_bytes()),
            change_spec_path: CURRENT_REVISION_SPEC.to_owned(),
            change_spec_sha256: sha256_hex(spec_document.as_bytes()),
            affected_rows: vec!["STR-001".to_owned()],
            checklist_sha256: "0".repeat(64),
        };

        // Act
        let error = validate_manifest(
            &manifest,
            &spec,
            &spec_document,
            CHECKLIST,
            &snapshot,
            &format!("{CURRENT_PREDECESSOR_ROOT}/{SNAPSHOT_FILE}"),
            CURRENT_REVISION_SPEC,
        )
        .expect_err("digest drift must fail");

        // Assert
        assert!(error.contains("binding mismatch"));
    }

    #[test]
    fn active_checklist_must_equal_latest_snapshot() {
        // Arrange
        let snapshot = apply_spec(CHECKLIST, &spec()).expect("snapshot");

        // Act
        let error = validate_active_snapshot(CHECKLIST, &snapshot)
            .expect_err("stale root checklist must fail");

        // Assert
        assert!(error.contains("does not match"));
    }
}
