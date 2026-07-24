use std::collections::{BTreeMap, BTreeSet};

use crate::phase35_evidence::sha256_hex;
use crate::phase36_evidence::effects::IndependentEffectObservationSource;
use crate::phase36_evidence::runtime_identity::RuntimeIdentityObservationSource;

use super::checklist::{
    parse_checklist_rows, render_projected_checklist, Phase36ChecklistSnapshot,
};
use super::types::{
    digest_serializable, is_lower_hex, Phase36ClaimDecision, Phase36ClaimPrerequisites,
    Phase36ClaimScope, Phase36DecisionReason, Phase36GenerationResolver, Phase36PromotionError,
    Phase36PromotionMatrix, PHASE36_AFFECTED_ROWS,
};

const PHASE36_PROMOTION_SCHEMA: &str = "phase36-promotion-matrix-v1";

pub(crate) fn evaluate_phase36_promotion(
    prerequisites: &Phase36ClaimPrerequisites,
    checklist: &Phase36ChecklistSnapshot,
) -> Result<Phase36PromotionMatrix, Phase36PromotionError> {
    validate_provenance(prerequisites)?;
    validate_prerequisite_digests(prerequisites)?;
    let decisions = Phase36ClaimScope::ALL
        .into_iter()
        .map(|scope| decision_for_scope(scope, prerequisites, checklist))
        .collect::<Result<Vec<_>, _>>()?;
    validate_scope_decisions(&decisions)?;
    let projected_checklist = render_projected_checklist(checklist, &decisions)?;
    let projected_rows = parse_checklist_rows(&projected_checklist)?;
    let preserved_row_fingerprints = preserved_rows(checklist, &projected_rows)?;
    let checklist_fingerprint_after = sha256_hex(projected_checklist.as_bytes());
    let generation_digest = digest_serializable(&(
        PHASE36_PROMOTION_SCHEMA,
        &prerequisites.phase35_root_digest,
        &prerequisites.superseded_phase35_generation_digest,
        &prerequisites.evaluator_digest,
        &checklist.fingerprint,
        &checklist_fingerprint_after,
        &decisions,
        &preserved_row_fingerprints,
    ))?;
    let resolver = Phase36GenerationResolver {
        phase35_root_digest: prerequisites.phase35_root_digest.clone(),
        superseded_phase35_generation_digest: prerequisites
            .superseded_phase35_generation_digest
            .clone(),
        authoritative_phase36_generation_digest: generation_digest,
    };
    Ok(Phase36PromotionMatrix {
        schema_version: PHASE36_PROMOTION_SCHEMA,
        phase35_root_digest: prerequisites.phase35_root_digest.clone(),
        superseded_phase35_generation_digest: prerequisites
            .superseded_phase35_generation_digest
            .clone(),
        evaluator_digest: prerequisites.evaluator_digest.clone(),
        checklist_fingerprint_before: checklist.fingerprint.clone(),
        checklist_fingerprint_after,
        scope_decisions: decisions,
        preserved_row_fingerprints,
        resolver,
        projected_checklist,
    })
}

fn validate_provenance(
    prerequisites: &Phase36ClaimPrerequisites,
) -> Result<(), Phase36PromotionError> {
    if !is_lower_hex(&prerequisites.phase35_root_digest, 64)
        || !is_lower_hex(&prerequisites.superseded_phase35_generation_digest, 64)
        || !is_lower_hex(&prerequisites.evaluator_digest, 64)
    {
        return Err(Phase36PromotionError::InvalidPrerequisiteDigest);
    }
    if prerequisites
        .maybe_hostname
        .as_ref()
        .is_some_and(|facts| facts.phase35_root_digest != prerequisites.phase35_root_digest)
    {
        return Err(Phase36PromotionError::InvalidPrerequisiteDigest);
    }
    Ok(())
}

fn validate_prerequisite_digests(
    prerequisites: &Phase36ClaimPrerequisites,
) -> Result<(), Phase36PromotionError> {
    let mut named_digests = Vec::new();
    if let Some(hostname) = prerequisites.maybe_hostname.as_ref() {
        let expected = digest_serializable(&(
            "phase36-hostname-durability-facts-v1",
            &hostname.phase35_root_digest,
            hostname.storage_confirmed,
            hostname.reload_confirmed,
            hostname.exactly_once_reboot_confirmed,
            hostname.restoration_confirmed,
            hostname.cleanup_confirmed,
        ))?;
        require_digest(
            "hostname",
            &hostname.claim_fact_digest,
            &expected,
            false,
            &mut named_digests,
        )?;
    }
    if let (Some(sensors), Some(join)) = (
        prerequisites.maybe_sensors.as_ref(),
        prerequisites.maybe_snapshot_join.as_ref(),
    ) {
        let expected = digest_serializable(&(
            &sensors.power,
            &sensors.temperature,
            &sensors.tachometer,
            join,
        ))?;
        require_digest(
            "snapshot",
            &sensors.claim_fact_digest,
            &expected,
            true,
            &mut named_digests,
        )?;
    }
    if let (Some(health), Some(join)) = (
        prerequisites.maybe_runtime_health.as_ref(),
        prerequisites.maybe_snapshot_join.as_ref(),
    ) {
        let mut digest_input = health.clone();
        digest_input.claim_fact_digest.clear();
        let expected = digest_serializable(&(&digest_input, join))?;
        require_digest(
            "runtime_health",
            &health.claim_fact_digest,
            &expected,
            true,
            &mut named_digests,
        )?;
    }
    if let Some(identity) = prerequisites.maybe_runtime_identity.as_ref() {
        let expected = digest_serializable(&(
            identity.observation_source,
            identity.same_physical_device,
            &identity.boot_b_session_digest,
            identity.boot_b_ordinal,
            &identity.source_commit_digest,
            &identity.reference_commit_digest,
            &identity.application_elf_digest,
            &identity.exact_package,
        ))?;
        require_digest(
            "runtime_identity",
            &identity.claim_fact_digest,
            &expected,
            true,
            &mut named_digests,
        )?;
    }
    if let Some(effect) = prerequisites.maybe_independent_effect.as_ref() {
        let expected = digest_serializable(&(
            effect.observation_source,
            effect.start_millis,
            effect.end_millis,
            &effect.ledger_digest,
        ))?;
        require_digest(
            "independent_effect",
            &effect.claim_fact_digest,
            &expected,
            true,
            &mut named_digests,
        )?;
    }
    let unique = named_digests
        .iter()
        .map(|(_, digest)| digest.as_str())
        .collect::<BTreeSet<_>>();
    if unique.len() != named_digests.len() {
        return Err(Phase36PromotionError::ReusedWrongClaimDigest);
    }
    Ok(())
}

fn require_digest(
    name: &'static str,
    actual: &str,
    expected: &str,
    require_exact: bool,
    named_digests: &mut Vec<(&'static str, String)>,
) -> Result<(), Phase36PromotionError> {
    if !is_lower_hex(actual, 64) || (require_exact && actual != expected) {
        return Err(Phase36PromotionError::InvalidPrerequisiteDigest);
    }
    named_digests.push((name, actual.to_owned()));
    Ok(())
}

fn decision_for_scope(
    scope: Phase36ClaimScope,
    prerequisites: &Phase36ClaimPrerequisites,
    checklist: &Phase36ChecklistSnapshot,
) -> Result<Phase36ClaimDecision, Phase36PromotionError> {
    let Some(row_id) = scope.row_id() else {
        let reason = exclusion_reason(scope);
        return Ok(Phase36ClaimDecision::DoNotPromote {
            scope,
            maybe_row_id: None,
            reason,
            claim_fact_digest: decision_digest(scope, reason, &[])?,
        });
    };
    let row = checklist.rows.get(row_id).ok_or_else(|| {
        Phase36PromotionError::Checklist(format!("missing affected row {row_id}"))
    })?;
    let maybe_reason = missing_reason(scope, prerequisites);
    let admitted_digests = admitted_digests(scope, prerequisites);
    let reason_for_digest =
        maybe_reason.unwrap_or(Phase36DecisionReason::BroaderOrUnmappedRowExcluded);
    let claim_fact_digest = decision_digest(scope, reason_for_digest, &admitted_digests)?;
    let currently_verified = row.cells[4] == "verified";
    match (maybe_reason, currently_verified) {
        (None, true) => Ok(Phase36ClaimDecision::Preserve {
            scope,
            row_id: row_id.to_owned(),
            claim_fact_digest,
        }),
        (None, false) => Ok(Phase36ClaimDecision::Promote {
            scope,
            row_id: row_id.to_owned(),
            claim_fact_digest,
        }),
        (Some(reason), true) => Ok(Phase36ClaimDecision::Demote {
            scope,
            row_id: row_id.to_owned(),
            reason,
            claim_fact_digest,
        }),
        (Some(reason), false) => Ok(Phase36ClaimDecision::DoNotPromote {
            scope,
            maybe_row_id: Some(row_id.to_owned()),
            reason,
            claim_fact_digest,
        }),
    }
}

fn missing_reason(
    scope: Phase36ClaimScope,
    prerequisites: &Phase36ClaimPrerequisites,
) -> Option<Phase36DecisionReason> {
    let claim_specific = match scope {
        Phase36ClaimScope::PassiveHostnameDurability => prerequisites
            .maybe_hostname
            .as_ref()
            .filter(|facts| facts.complete())
            .is_none()
            .then_some(Phase36DecisionReason::HostnameDurabilityFactsInsufficient),
        Phase36ClaimScope::ExactSourceReferencePackageIdentity => prerequisites
            .maybe_runtime_identity
            .as_ref()
            .filter(|identity| {
                identity.same_physical_device
                    && matches!(
                        identity.observation_source,
                        RuntimeIdentityObservationSource::DeviceSessionReplay
                            | RuntimeIdentityObservationSource::TerminalResultProjection
                    )
            })
            .is_none()
            .then_some(Phase36DecisionReason::RuntimeIdentityObservationInsufficient),
        Phase36ClaimScope::CoherentOperatorSnapshot => prerequisites
            .maybe_sensors
            .as_ref()
            .zip(prerequisites.maybe_snapshot_join.as_ref())
            .is_none()
            .then_some(Phase36DecisionReason::SnapshotSubstanceInsufficient),
        Phase36ClaimScope::PassiveRuntimeHealthProjection => prerequisites
            .maybe_runtime_health
            .as_ref()
            .zip(prerequisites.maybe_snapshot_join.as_ref())
            .is_none()
            .then_some(Phase36DecisionReason::RuntimeHealthInsufficient),
        _ => unreachable!("excluded scopes are handled before prerequisites"),
    };
    if matches!(scope, Phase36ClaimScope::PassiveHostnameDurability) {
        return claim_specific;
    }
    claim_specific.or_else(|| {
        prerequisites
            .maybe_independent_effect
            .as_ref()
            .filter(|effect| {
                effect.observation_source == IndependentEffectObservationSource::IndependentLedger
                    && effect.effect_count == 8
                    && effect.duration_millis == effect.end_millis - effect.start_millis
            })
            .is_none()
            .then_some(Phase36DecisionReason::IndependentEffectObservationInsufficient)
    })
}

fn admitted_digests(
    scope: Phase36ClaimScope,
    prerequisites: &Phase36ClaimPrerequisites,
) -> Vec<&str> {
    let mut digests = Vec::new();
    match scope {
        Phase36ClaimScope::PassiveHostnameDurability => {
            if let Some(facts) = prerequisites.maybe_hostname.as_ref() {
                digests.push(facts.claim_fact_digest.as_str());
            }
        }
        Phase36ClaimScope::ExactSourceReferencePackageIdentity => {
            if let Some(identity) = prerequisites.maybe_runtime_identity.as_ref() {
                digests.push(identity.claim_fact_digest.as_str());
            }
        }
        Phase36ClaimScope::CoherentOperatorSnapshot => {
            if let Some(sensors) = prerequisites.maybe_sensors.as_ref() {
                digests.push(sensors.claim_fact_digest.as_str());
            }
        }
        Phase36ClaimScope::PassiveRuntimeHealthProjection => {
            if let Some(health) = prerequisites.maybe_runtime_health.as_ref() {
                digests.push(health.claim_fact_digest.as_str());
            }
        }
        _ => {}
    }
    if !matches!(scope, Phase36ClaimScope::PassiveHostnameDurability) {
        let Some(effect) = prerequisites.maybe_independent_effect.as_ref() else {
            return digests;
        };
        digests.push(effect.claim_fact_digest.as_str());
    }
    digests
}

fn decision_digest(
    scope: Phase36ClaimScope,
    reason: Phase36DecisionReason,
    admitted_digests: &[&str],
) -> Result<String, Phase36PromotionError> {
    digest_serializable(&(
        "phase36-claim-decision-facts-v1",
        scope,
        reason,
        admitted_digests,
    ))
}

fn exclusion_reason(scope: Phase36ClaimScope) -> Phase36DecisionReason {
    match scope {
        Phase36ClaimScope::ActiveControl => Phase36DecisionReason::ActiveControlExcluded,
        Phase36ClaimScope::SelfTestEffects => Phase36DecisionReason::SelfTestEffectsExcluded,
        Phase36ClaimScope::WatchdogIntervention => {
            Phase36DecisionReason::WatchdogInterventionExcluded
        }
        Phase36ClaimScope::MiningStratumAsic => Phase36DecisionReason::MiningStratumAsicExcluded,
        Phase36ClaimScope::ArchivedPhase28_1_1 => {
            Phase36DecisionReason::ArchivedPhase28_1_1Excluded
        }
        Phase36ClaimScope::Credentials => Phase36DecisionReason::CredentialsExcluded,
        Phase36ClaimScope::DirectUartOrPins => Phase36DecisionReason::DirectUartOrPinsExcluded,
        Phase36ClaimScope::OtaOrRecovery => Phase36DecisionReason::OtaOrRecoveryExcluded,
        Phase36ClaimScope::OtherBoards => Phase36DecisionReason::OtherBoardsExcluded,
        Phase36ClaimScope::LifecycleTestOnlyProof => {
            Phase36DecisionReason::LifecycleTestOnlyProofExcluded
        }
        Phase36ClaimScope::BroaderOrUnmappedRows => {
            Phase36DecisionReason::BroaderOrUnmappedRowExcluded
        }
        _ => unreachable!("affected scope does not have an exclusion reason"),
    }
}

fn validate_scope_decisions(
    decisions: &[Phase36ClaimDecision],
) -> Result<(), Phase36PromotionError> {
    let scopes = decisions
        .iter()
        .map(Phase36ClaimDecision::scope)
        .collect::<BTreeSet<_>>();
    if scopes != Phase36ClaimScope::ALL.into_iter().collect() || scopes.len() != decisions.len() {
        return Err(Phase36PromotionError::Incomplete(
            "every claim scope must have exactly one decision".to_owned(),
        ));
    }
    let affected_rows = decisions
        .iter()
        .filter_map(Phase36ClaimDecision::maybe_row_id)
        .collect::<BTreeSet<_>>();
    if affected_rows != PHASE36_AFFECTED_ROWS.into_iter().collect() {
        return Err(Phase36PromotionError::Incomplete(
            "every affected row must have exactly one decision".to_owned(),
        ));
    }
    if decisions
        .iter()
        .any(|decision| !is_lower_hex(decision.claim_fact_digest(), 64))
    {
        return Err(Phase36PromotionError::InvalidPrerequisiteDigest);
    }
    Ok(())
}

fn preserved_rows(
    checklist: &Phase36ChecklistSnapshot,
    projected: &BTreeMap<String, super::checklist::Phase36ChecklistRow>,
) -> Result<BTreeMap<String, String>, Phase36PromotionError> {
    let mut fingerprints = BTreeMap::new();
    for (row_id, original) in &checklist.rows {
        let projected_row = projected.get(row_id).ok_or_else(|| {
            Phase36PromotionError::Incomplete(format!("projected row disappeared: {row_id}"))
        })?;
        if PHASE36_AFFECTED_ROWS.contains(&row_id.as_str()) {
            continue;
        }
        if projected_row.raw_line != original.raw_line {
            return Err(Phase36PromotionError::Incomplete(format!(
                "unrelated row changed: {row_id}"
            )));
        }
        fingerprints.insert(row_id.clone(), original.fingerprint.clone());
    }
    Ok(fingerprints)
}
