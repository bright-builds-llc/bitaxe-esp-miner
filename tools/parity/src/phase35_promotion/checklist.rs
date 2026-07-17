use std::collections::BTreeMap;

use crate::phase35_evidence::sha256_hex;

use super::types::{Phase35LiveRechecks, Phase35PromotionError, PHASE35_PROMOTABLE_ROWS};

#[derive(Debug, Clone)]
pub(super) struct ChecklistRowSnapshot {
    pub(super) line_index: usize,
    pub(super) raw_line: String,
    pub(super) cells: Vec<String>,
    pub(super) fingerprint: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ChecklistSnapshot {
    pub(super) contents: String,
    pub(super) fingerprint: String,
    pub(super) rows: BTreeMap<String, ChecklistRowSnapshot>,
    pub(super) live: Phase35LiveRechecks,
}

impl ChecklistSnapshot {
    pub(crate) fn capture(
        contents: String,
        live: Phase35LiveRechecks,
    ) -> Result<Self, Phase35PromotionError> {
        let rows = parse_checklist_rows(&contents)?;
        for row_id in PHASE35_PROMOTABLE_ROWS {
            if !rows.contains_key(row_id) {
                return Err(Phase35PromotionError::Checklist(format!(
                    "missing dedicated Phase 35 row {row_id}"
                )));
            }
        }
        Ok(Self {
            fingerprint: sha256_hex(contents.as_bytes()),
            contents,
            rows,
            live,
        })
    }
}

pub(super) fn render_projected_checklist(
    checklist: &ChecklistSnapshot,
    root_digest: &str,
) -> Result<String, Phase35PromotionError> {
    let mut lines = checklist
        .contents
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for row_id in PHASE35_PROMOTABLE_ROWS {
        let row = checklist.rows.get(row_id).ok_or_else(|| {
            Phase35PromotionError::Checklist(format!("missing dedicated row {row_id}"))
        })?;
        let mut cells = row.cells.clone();
        cells[4] = "verified".to_owned();
        cells[5] = "hardware-smoke".to_owned();
        cells[6] = format!(
            "Phase 35 proves only this exact passive board `205` category from admitted protected-root digest `{root_digest}`; scope is not widened beyond this row."
        );
        lines[row.line_index] = format!("| {} |", cells.join(" | "));
    }
    let mut output = lines.join("\n");
    if checklist.contents.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

pub(super) fn parse_checklist_rows(
    checklist: &str,
) -> Result<BTreeMap<String, ChecklistRowSnapshot>, Phase35PromotionError> {
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
        let separator = cells.iter().all(|cell| {
            !cell.is_empty()
                && cell
                    .chars()
                    .all(|character| matches!(character, '-' | ':' | ' '))
        });
        if cells.first().is_some_and(|cell| cell == "ID") || separator {
            continue;
        }
        if cells.len() != 7 {
            return Err(Phase35PromotionError::Checklist(format!(
                "line {} has {} columns",
                line_index + 1,
                cells.len()
            )));
        }
        let row_id = cells[0].trim_matches('`').to_owned();
        let snapshot = ChecklistRowSnapshot {
            line_index,
            raw_line: line.to_owned(),
            cells,
            fingerprint: sha256_hex(line.as_bytes()),
        };
        if rows.insert(row_id.clone(), snapshot).is_some() {
            return Err(Phase35PromotionError::Checklist(format!(
                "duplicate checklist row {row_id}"
            )));
        }
    }
    Ok(rows)
}
