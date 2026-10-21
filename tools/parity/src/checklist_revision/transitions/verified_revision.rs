//! Owner-authorized, additive revisions of `verified` rows.
//!
//! Automatic transitions out of `verified` stay forbidden. A hash-bound authority file under
//! `docs/parity/verified-revisions/` records an owner decision: the digest-bound evidence behind it,
//! and for each row exactly the evidence tokens to add (possibly none) and the digest of the new
//! notes. Such a revision keeps the status and the owned target and only appends to the notes, so no
//! earlier claim or non-claim is removed or reworded, and each grant applies once.

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use super::super::read;
use crate::phase35_evidence::sha256_hex;

const AUTHORITY_ROOT: &str = "docs/parity/verified-revisions";
const EVIDENCE_ROOT: &str = "docs/parity/evidence";
const SCHEMA: &str = "parity-verified-revision-authority-v1";

/// An authority file a transition binds, with the digest its receipt records.
pub(super) struct Binding {
    pub(super) path: String,
    pub(super) digest: String,
    pub(super) authority: Authority,
}

/// One owner decision: the rows it may refine, exactly how, and the evidence behind it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Authority {
    schema_version: String,
    authority_id: String,
    authorized_by: String,
    authorized_on: String,
    rationale: String,
    evidence: Vec<EvidenceFile>,
    rows: Vec<RowGrant>,
}

/// A committed evidence document, digest-bound so later edits fail closed.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceFile {
    path: String,
    sha256: String,
}

/// The single revision allowed for one row: the evidence tokens it adds and the exact new notes.
/// An empty `added_evidence` grants a passage-only revision for a row that already holds every
/// evidence token; its evidence cell must then stay unchanged. Pinning the notes digest makes the
/// grant one-shot, since a later append can never match it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RowGrant {
    row_id: String,
    added_evidence: Vec<String>,
    after_notes_sha256: String,
}

/// The cells a verified revision may touch, before and after.
pub(super) struct Revision<'cell> {
    pub(super) row_id: &'cell str,
    pub(super) before_status: &'cell str,
    pub(super) after_status: &'cell str,
    pub(super) before_target: &'cell str,
    pub(super) after_target: &'cell str,
    pub(super) before_evidence: &'cell str,
    pub(super) after_evidence: &'cell str,
    pub(super) maybe_before_notes: Option<&'cell str>,
    pub(super) maybe_after_notes: Option<&'cell str>,
}

/// Reads the authority a new transition names, if any.
pub(super) fn read_binding(
    workspace: &Utf8Path,
    maybe_path: Option<&Utf8Path>,
) -> Result<Option<Binding>, String> {
    let Some(path) = maybe_path else {
        return Ok(None);
    };
    let document = read_authority(workspace, path)?;
    Ok(Some(Binding {
        path: path.to_string(),
        digest: sha256_hex(document.as_bytes()),
        authority: parse(workspace, &document)?,
    }))
}

/// Re-reads a receipt's authority on replay; an edited authority or evidence file fails closed.
pub(super) fn validate_receipt_binding(
    workspace: &Utf8Path,
    maybe_path: Option<&str>,
    maybe_digest: Option<&str>,
) -> Result<Option<Authority>, String> {
    match (maybe_path, maybe_digest) {
        (None, None) => Ok(None),
        (Some(path), Some(digest)) => {
            let document = read_authority(workspace, &Utf8PathBuf::from(path))?;
            if digest != sha256_hex(document.as_bytes()) {
                return Err("verified revision authority digest mismatch".to_owned());
            }
            parse(workspace, &document).map(Some)
        }
        _ => Err("verified revision authority binding is incomplete".to_owned()),
    }
}

/// A verified row may change only as its grant pins: same status and target, exactly the granted
/// evidence tokens appended (or the evidence unchanged when none are granted), and the old notes
/// extended by a separated, printable suffix whose result matches the granted digest.
pub(super) fn require_authorized(authority: &Authority, revision: &Revision) -> Result<(), String> {
    if revision.before_status != "verified" || revision.after_status != "verified" {
        return Err("a verified revision keeps a verified row verified".to_owned());
    }
    let grant = authority
        .rows
        .iter()
        .find(|grant| grant.row_id == revision.row_id)
        .ok_or_else(|| {
            format!(
                "verified revision authority {} does not name {}",
                authority.authority_id, revision.row_id
            )
        })?;
    if revision.before_target != revision.after_target {
        return Err("a verified revision must not change the owned target".to_owned());
    }
    let expected_evidence = if grant.added_evidence.is_empty() {
        revision.before_evidence.to_owned()
    } else {
        format!(
            "{},{}",
            revision.before_evidence,
            grant.added_evidence.join(",")
        )
    };
    if revision.after_evidence != expected_evidence {
        return Err("a verified revision must add exactly the granted evidence".to_owned());
    }
    let (Some(before), Some(after)) = (revision.maybe_before_notes, revision.maybe_after_notes)
    else {
        return Err("a verified revision must append to the notes".to_owned());
    };
    let suffix = after
        .strip_prefix(before)
        .ok_or_else(|| "a verified revision may only append to the notes".to_owned())?;
    let printable = !suffix
        .chars()
        .any(|character| character.is_control() || is_format_character(character));
    if !suffix.starts_with(' ') || suffix.trim().is_empty() || !printable {
        return Err("appended notes must be a separated, printable sentence".to_owned());
    }
    if sha256_hex(after.as_bytes()) != grant.after_notes_sha256 {
        return Err("appended notes do not match the granted digest".to_owned());
    }
    Ok(())
}

/// Invisible formatting that could disguise text: zero-width and bidirectional controls.
fn is_format_character(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{061c}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2069}'
            | '\u{feff}'
    )
}

/// An authority granting one row, for receipt-level tests elsewhere in the transition ledger.
#[cfg(test)]
pub(super) fn authority_for(row_id: &str, added_evidence: &str, after_notes: &str) -> Authority {
    Authority {
        schema_version: SCHEMA.to_owned(),
        authority_id: "fixture".to_owned(),
        authorized_by: "owner".to_owned(),
        authorized_on: "2026-10-16".to_owned(),
        rationale: "Fixture.".to_owned(),
        evidence: vec![EvidenceFile {
            path: "docs/parity/evidence/fixture.md".to_owned(),
            sha256: "0".repeat(64),
        }],
        rows: vec![RowGrant {
            row_id: row_id.to_owned(),
            added_evidence: vec![added_evidence.to_owned()],
            after_notes_sha256: sha256_hex(after_notes.as_bytes()),
        }],
    }
}

fn read_authority(workspace: &Utf8Path, path: &Utf8Path) -> Result<String, String> {
    let in_root =
        path.parent() == Some(Utf8Path::new(AUTHORITY_ROOT)) && path.extension() == Some("json");
    if !in_root {
        return Err(format!(
            "verified revision authority must be a JSON file in {AUTHORITY_ROOT}"
        ));
    }
    read(&workspace.join(path), "verified revision authority")
}

fn parse(workspace: &Utf8Path, document: &str) -> Result<Authority, String> {
    let authority: Authority = serde_json::from_str(document)
        .map_err(|error| format!("invalid verified revision authority: {error}"))?;
    let filled = [
        &authority.authority_id,
        &authority.authorized_by,
        &authority.authorized_on,
        &authority.rationale,
    ]
    .iter()
    .all(|value| !value.trim().is_empty());
    let mut row_ids = authority
        .rows
        .iter()
        .map(|grant| grant.row_id.as_str())
        .collect::<Vec<_>>();
    row_ids.sort_unstable();
    row_ids.dedup();
    let grants_valid = authority.rows.iter().all(is_valid_grant);
    if authority.schema_version != SCHEMA
        || !filled
        || authority.evidence.is_empty()
        || authority.rows.is_empty()
        || row_ids.len() != authority.rows.len()
        || !grants_valid
    {
        return Err("verified revision authority is incomplete".to_owned());
    }
    for evidence in &authority.evidence {
        require_evidence_file(workspace, evidence)?;
    }
    Ok(authority)
}

fn require_evidence_file(workspace: &Utf8Path, evidence: &EvidenceFile) -> Result<(), String> {
    let path = Utf8Path::new(&evidence.path);
    let contained = path.starts_with(EVIDENCE_ROOT)
        && path
            .components()
            .all(|component| matches!(component, Utf8Component::Normal(_)));
    if !contained {
        return Err(format!(
            "verified revision evidence must be under {EVIDENCE_ROOT}"
        ));
    }
    let document = read(&workspace.join(path), "verified revision evidence")
        .map_err(|_| format!("verified revision evidence {} is missing", evidence.path))?;
    if sha256_hex(document.as_bytes()) != evidence.sha256 {
        return Err(format!(
            "verified revision evidence {} changed",
            evidence.path
        ));
    }
    Ok(())
}

/// Every granted token must be well formed; an empty list is a passage-only grant.
fn is_valid_grant(grant: &RowGrant) -> bool {
    grant
        .added_evidence
        .iter()
        .all(|token| is_evidence_token(token))
        && is_sha256(&grant.after_notes_sha256)
}

fn is_evidence_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| matches!(character, '0'..='9' | 'a'..='f'))
}

#[cfg(test)]
mod tests;
