//! Hash-bound authority for the comprehensive reference inventory revision.

use std::collections::BTreeSet;

use camino::Utf8Path;
use serde::Deserialize;

use super::{parse_rows, read, CURRENT_REVISION_ROOT, SNAPSHOT_FILE};
use crate::phase35_evidence::sha256_hex;

const REVISION_SPEC: &str =
    "docs/parity/checklist-revisions/2026-08-02-comprehensive-reference-inventory.json";
const SCHEMA: &str = "parity-checklist-comprehensive-revision-v1";
const REVISION_ID: &str = "2026-08-02-comprehensive-reference-inventory";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComprehensiveRevisionSpec {
    schema_version: String,
    revision_id: String,
    predecessor_path: String,
    predecessor_sha256: String,
    checklist_sha256: String,
    inventory_path: String,
    inventory_sha256: String,
    added_rows: Vec<String>,
    refined_rows: Vec<String>,
}

pub(super) fn validate(
    workspace: &Utf8Path,
    predecessor: &str,
    active: &str,
) -> Result<(), String> {
    let spec_document = read(
        &workspace.join(REVISION_SPEC),
        "comprehensive checklist revision specification",
    )?;
    let spec: ComprehensiveRevisionSpec = serde_json::from_str(&spec_document)
        .map_err(|error| format!("invalid comprehensive checklist revision: {error}"))?;
    let inventory = read(
        &workspace.join(&spec.inventory_path),
        "reference surface inventory",
    )?;
    let expected_predecessor = format!("{CURRENT_REVISION_ROOT}/{SNAPSHOT_FILE}");

    if spec.schema_version != SCHEMA
        || spec.revision_id != REVISION_ID
        || spec.predecessor_path != expected_predecessor
        || spec.predecessor_sha256 != sha256_hex(predecessor.as_bytes())
        || spec.checklist_sha256 != sha256_hex(active.as_bytes())
        || spec.inventory_sha256 != sha256_hex(inventory.as_bytes())
    {
        return Err("comprehensive checklist revision binding mismatch".to_owned());
    }

    let before = parse_rows(predecessor)?;
    let after = parse_rows(active)?;
    let added = unique_declared_rows(&spec.added_rows, "added")?;
    let refined = unique_declared_rows(&spec.refined_rows, "refined")?;
    if !added.is_disjoint(&refined) {
        return Err("comprehensive checklist revision row declarations overlap".to_owned());
    }

    for (row_id, before_row) in &before {
        let after_row = after
            .get(row_id)
            .ok_or_else(|| format!("comprehensive checklist revision removed row {row_id}"))?;
        if before_row.cells[4] != after_row.cells[4] || before_row.cells[5] != after_row.cells[5] {
            return Err(format!(
                "comprehensive checklist revision changed status or evidence for {row_id}"
            ));
        }
        if before_row.cells != after_row.cells && !refined.contains(row_id.as_str()) {
            return Err(format!(
                "comprehensive checklist revision changed undeclared row {row_id}"
            ));
        }
    }

    let actual_added = after
        .keys()
        .filter(|row_id| !before.contains_key(*row_id))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_added != added {
        return Err("comprehensive checklist added-row declaration mismatch".to_owned());
    }
    if refined.iter().any(|row_id| !before.contains_key(*row_id)) {
        return Err("comprehensive checklist refined row is absent from predecessor".to_owned());
    }

    Ok(())
}

fn unique_declared_rows<'row>(
    rows: &'row [String],
    label: &str,
) -> Result<BTreeSet<&'row str>, String> {
    let unique = rows.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if rows.is_empty() || unique.len() != rows.len() {
        return Err(format!(
            "comprehensive checklist {label} row declarations must be nonempty and unique"
        ));
    }
    Ok(unique)
}
