use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use super::super::read;
use crate::phase35_evidence::sha256_hex;

const AUTOMATION_MIGRATION_LEDGER: &str = "docs/parity/automation-migration.json";

pub(super) struct Binding {
    pub(super) path: String,
    pub(super) digest: String,
    pub(super) ledger: Ledger,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ledger {
    schema_version: String,
    cutover: String,
    legacy_consumers_accepted: bool,
    historical_evidence_rewritten: bool,
    mappings: Vec<Mapping>,
    downgraded_rows: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    legacy: String,
    replacement: String,
    equivalence: String,
    parity_rows: Vec<String>,
}

pub(super) fn read_binding(
    workspace: &Utf8Path,
    maybe_path: Option<&Utf8Path>,
) -> Result<Option<Binding>, String> {
    let Some(path) = maybe_path else {
        return Ok(None);
    };
    require_canonical_path(path)?;
    let document = read(&workspace.join(path), "automation migration ledger")?;
    let ledger = parse_ledger(&document)?;
    Ok(Some(Binding {
        path: path.to_string(),
        digest: sha256_hex(document.as_bytes()),
        ledger,
    }))
}

pub(super) fn validate_receipt_binding(
    workspace: &Utf8Path,
    maybe_path: Option<&str>,
    maybe_digest: Option<&str>,
) -> Result<Option<Ledger>, String> {
    match (maybe_path, maybe_digest) {
        (None, None) => Ok(None),
        (Some(path), Some(digest)) => {
            let path = Utf8PathBuf::from(path);
            require_canonical_path(&path)?;
            let document = read(&workspace.join(path), "automation migration ledger")?;
            if digest != sha256_hex(document.as_bytes()) {
                return Err("automation migration ledger digest mismatch".to_owned());
            }
            parse_ledger(&document).map(Some)
        }
        _ => Err("automation migration ledger binding is incomplete".to_owned()),
    }
}

pub(super) fn require_policy(
    before: &str,
    after: &str,
    row_id: &str,
    maybe_migration: Option<&Ledger>,
) -> Result<(), String> {
    if require_monotonic(before, after).is_ok() {
        return Ok(());
    }
    let Some(migration) = maybe_migration else {
        return require_monotonic(before, after);
    };
    let mapped = migration
        .mappings
        .iter()
        .any(|mapping| mapping.parity_rows.iter().any(|row| row == row_id));
    if !mapped {
        return Err(format!(
            "automation migration ledger does not map parity row {row_id}"
        ));
    }
    if before == after {
        return Ok(());
    }
    if before == "verified"
        && after == "implemented"
        && migration.downgraded_rows.iter().any(|row| row == row_id)
    {
        return Ok(());
    }
    Err(format!(
        "automation migration ledger does not authorize {row_id} transition {before} -> {after}"
    ))
}

fn parse_ledger(document: &str) -> Result<Ledger, String> {
    let ledger: Ledger = serde_json::from_str(document)
        .map_err(|error| format!("invalid automation migration ledger: {error}"))?;
    if ledger.schema_version != "bitaxe-automation-migration-v1"
        || ledger.cutover != "typed-automation-deep-refactor"
        || ledger.legacy_consumers_accepted
        || ledger.historical_evidence_rewritten
    {
        return Err("automation migration ledger policy mismatch".to_owned());
    }
    if ledger.mappings.iter().any(|mapping| {
        mapping.legacy.trim().is_empty()
            || mapping.replacement.trim().is_empty()
            || mapping.equivalence.trim().is_empty()
            || mapping.parity_rows.is_empty()
    }) {
        return Err("automation migration ledger contains an incomplete mapping".to_owned());
    }
    Ok(ledger)
}

fn require_canonical_path(path: &Utf8Path) -> Result<(), String> {
    if path == Utf8Path::new(AUTOMATION_MIGRATION_LEDGER) {
        return Ok(());
    }
    Err(format!(
        "migration ledger must be {AUTOMATION_MIGRATION_LEDGER}"
    ))
}

fn require_monotonic(before: &str, after: &str) -> Result<(), String> {
    if matches!(before, "verified" | "deferred") {
        return Err(format!(
            "automatic transitions out of {before} are forbidden"
        ));
    }
    let rank = |status: &str| match status {
        "not-started" => Some(0),
        "in-progress" => Some(1),
        "implemented" => Some(2),
        "verified" => Some(3),
        _ => None,
    };
    let before_rank = rank(before).ok_or_else(|| format!("status {before} is not actionable"))?;
    let after_rank = rank(after).ok_or_else(|| format!("status {after} is not actionable"))?;
    if after_rank <= before_rank {
        return Err("parity transitions must move monotonically toward verified".to_owned());
    }
    Ok(())
}
