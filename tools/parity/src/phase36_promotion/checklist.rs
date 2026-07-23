use std::collections::BTreeMap;

use crate::phase35_evidence::sha256_hex;

use super::types::{
    Phase36ClaimDecision, Phase36DecisionReason, Phase36PromotionError, PHASE36_AFFECTED_ROWS,
};

#[derive(Debug, Clone)]
pub(crate) struct Phase36ChecklistRow {
    pub(crate) line_index: usize,
    pub(crate) raw_line: String,
    pub(crate) cells: Vec<String>,
    pub(crate) fingerprint: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Phase36ChecklistSnapshot {
    pub(crate) contents: String,
    pub(crate) fingerprint: String,
    pub(crate) rows: BTreeMap<String, Phase36ChecklistRow>,
}

impl Phase36ChecklistSnapshot {
    pub(crate) fn capture(contents: String) -> Result<Self, Phase36PromotionError> {
        let rows = parse_checklist_rows(&contents)?;
        for row_id in PHASE36_AFFECTED_ROWS {
            if !rows.contains_key(row_id) {
                return Err(Phase36PromotionError::Checklist(format!(
                    "missing dedicated Phase 36 row {row_id}"
                )));
            }
        }
        Ok(Self {
            fingerprint: sha256_hex(contents.as_bytes()),
            contents,
            rows,
        })
    }
}

pub(crate) fn render_projected_checklist(
    checklist: &Phase36ChecklistSnapshot,
    decisions: &[Phase36ClaimDecision],
) -> Result<String, Phase36PromotionError> {
    let mut lines = checklist
        .contents
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for decision in decisions {
        let Some(row_id) = decision.maybe_row_id() else {
            continue;
        };
        let row = checklist.rows.get(row_id).ok_or_else(|| {
            Phase36PromotionError::Checklist(format!("missing affected row {row_id}"))
        })?;
        let mut cells = row.cells.clone();
        match decision {
            Phase36ClaimDecision::Promote {
                claim_fact_digest, ..
            }
            | Phase36ClaimDecision::Preserve {
                claim_fact_digest, ..
            } => {
                cells[4] = "verified".to_owned();
                cells[5] = "hardware-smoke".to_owned();
                cells[6] = format!(
                    "Phase 36 supports only this exact passive board `205` claim from reconstructable typed facts digest `{claim_fact_digest}`; prior Phase 35 scope and every non-claim remain unchanged."
                );
            }
            Phase36ClaimDecision::Demote {
                reason,
                claim_fact_digest,
                ..
            }
            | Phase36ClaimDecision::DoNotPromote {
                reason,
                claim_fact_digest,
                ..
            } => {
                cells[4] = "implemented".to_owned();
                cells[5] = "workflow".to_owned();
                cells[6] = format!(
                    "Phase 36 correction `{}` from exact admitted-fact digest `{claim_fact_digest}`; this row remains below verified and every non-claim is preserved.",
                    reason_label(*reason)
                );
            }
        }
        lines[row.line_index] = format!("| {} |", cells.join(" | "));
    }
    let mut output = lines.join("\n");
    if checklist.contents.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

pub(crate) fn parse_checklist_rows(
    checklist: &str,
) -> Result<BTreeMap<String, Phase36ChecklistRow>, Phase36PromotionError> {
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
            return Err(Phase36PromotionError::Checklist(format!(
                "line {} has {} columns",
                line_index + 1,
                cells.len()
            )));
        }
        let row_id = cells[0].trim_matches('`').to_owned();
        let snapshot = Phase36ChecklistRow {
            line_index,
            raw_line: line.to_owned(),
            cells,
            fingerprint: sha256_hex(line.as_bytes()),
        };
        if rows.insert(row_id.clone(), snapshot).is_some() {
            return Err(Phase36PromotionError::Checklist(format!(
                "duplicate checklist row {row_id}"
            )));
        }
    }
    Ok(rows)
}

const fn reason_label(reason: Phase36DecisionReason) -> &'static str {
    match reason {
        Phase36DecisionReason::HostnameDurabilityFactsInsufficient => {
            "hostname_durability_facts_insufficient"
        }
        Phase36DecisionReason::RuntimeIdentityObservationInsufficient => {
            "runtime_identity_observation_insufficient"
        }
        Phase36DecisionReason::SnapshotSubstanceInsufficient => "snapshot_substance_insufficient",
        Phase36DecisionReason::RuntimeHealthInsufficient => "runtime_health_insufficient",
        Phase36DecisionReason::IndependentEffectObservationInsufficient => {
            "independent_effect_observation_insufficient"
        }
        Phase36DecisionReason::ActiveControlExcluded => "active_control_excluded",
        Phase36DecisionReason::SelfTestEffectsExcluded => "self_test_effects_excluded",
        Phase36DecisionReason::WatchdogInterventionExcluded => "watchdog_intervention_excluded",
        Phase36DecisionReason::MiningStratumAsicExcluded => "mining_stratum_asic_excluded",
        Phase36DecisionReason::ArchivedPhase28_1_1Excluded => "archived_phase28_1_1_excluded",
        Phase36DecisionReason::CredentialsExcluded => "credentials_excluded",
        Phase36DecisionReason::DirectUartOrPinsExcluded => "direct_uart_or_pins_excluded",
        Phase36DecisionReason::OtaOrRecoveryExcluded => "ota_or_recovery_excluded",
        Phase36DecisionReason::OtherBoardsExcluded => "other_boards_excluded",
        Phase36DecisionReason::LifecycleTestOnlyProofExcluded => {
            "lifecycle_test_only_proof_excluded"
        }
        Phase36DecisionReason::BroaderOrUnmappedRowExcluded => "broader_or_unmapped_row_excluded",
    }
}
