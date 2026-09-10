use anyhow::{bail, Result};

use super::{parse_plan_metadata_value, plan_status_rank};
use crate::{normalize, ChecklistRow};

pub(super) fn is_parity_plan(document: &str, directory_name: &str, rows: &[ChecklistRow]) -> bool {
    if document.lines().any(|line| {
        let line = line.trim();
        line.starts_with("- Parity row") || line.starts_with("- Initial status")
    }) {
        return true;
    }
    // The directory is shared with task-only plans. A row-named directory must
    // still be validated if its required metadata was accidentally omitted.
    let Some((_, name)) = directory_name.split_once('-') else {
        return false;
    };
    rows.iter().any(|row| {
        name.strip_prefix(&row.id)
            .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('-'))
    })
}

pub(super) fn parse_plan_initial_status(document: &str) -> Result<String> {
    let metadata = parse_plan_metadata_value(document, "- Initial status: `", "initial-status")?;
    let status = match metadata.split_once('|') {
        Some((status, evidence)) => {
            // Some immutable plans recorded both checklist cells in this field.
            // Recognizing evidence labels here does not validate parity evidence.
            if !evidence.split(',').all(|label| {
                matches!(
                    label.trim(),
                    "unit"
                        | "golden"
                        | "api-compare"
                        | "hardware-smoke"
                        | "hardware-regression"
                        | "workflow"
                        | "deferred"
                )
            }) {
                bail!("open parity plan has invalid initial-status evidence {evidence}");
            }
            status
        }
        None => metadata.as_str(),
    };
    let initial_status = normalize(status);
    if plan_status_rank(&initial_status).is_none() {
        bail!("open parity plan has non-actionable initial status {initial_status}");
    }
    Ok(initial_status)
}
