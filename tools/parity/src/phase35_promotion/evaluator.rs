use std::collections::{BTreeMap, BTreeSet};

use crate::phase35_evidence::{sha256_hex, ValidatedPhase35Evidence, PHASE35_LIFECYCLE_ID};

use super::checklist::{parse_checklist_rows, render_projected_checklist, ChecklistSnapshot};
use super::types::{
    Phase35ClaimScope, Phase35EvidenceSource, Phase35LiveRechecks, Phase35NonPromotionReason,
    Phase35PromotionDecision, Phase35PromotionError, Phase35PromotionMatrix, PHASE35_HEALTH_ROW,
    PHASE35_HOSTNAME_ROW, PHASE35_IDENTITY_ROW, PHASE35_PROMOTABLE_ROWS, PHASE35_SNAPSHOT_ROW,
};

pub(crate) fn evaluate_phase35_promotion(
    evidence: &ValidatedPhase35Evidence,
    checklist: &ChecklistSnapshot,
) -> Result<Phase35PromotionMatrix, Phase35PromotionError> {
    validate_live_rechecks(evidence, &checklist.live)?;
    let root_digest = evidence.root_digest().to_owned();
    let scope_decisions = Phase35ClaimScope::ALL
        .into_iter()
        .map(|scope| (scope, decision_for_scope(scope, &root_digest)))
        .collect::<Vec<_>>();
    validate_scope_decisions(&scope_decisions, &root_digest)?;

    let projected_checklist = render_projected_checklist(checklist, &root_digest)?;
    let projected_rows = parse_checklist_rows(&projected_checklist)?;
    let preserved_row_fingerprints = preserved_rows(checklist, &projected_rows)?;
    Ok(Phase35PromotionMatrix {
        evidence_root_digest: root_digest,
        checklist_fingerprint_before: checklist.fingerprint.clone(),
        checklist_fingerprint_after: sha256_hex(projected_checklist.as_bytes()),
        scope_decisions,
        preserved_row_fingerprints,
        projected_checklist,
    })
}

fn preserved_rows(
    checklist: &ChecklistSnapshot,
    projected: &BTreeMap<String, super::checklist::ChecklistRowSnapshot>,
) -> Result<BTreeMap<String, String>, Phase35PromotionError> {
    let mut fingerprints = BTreeMap::new();
    for (row_id, original) in &checklist.rows {
        let projected_row = projected.get(row_id).ok_or_else(|| {
            Phase35PromotionError::Incomplete(format!("projected row disappeared: {row_id}"))
        })?;
        if PHASE35_PROMOTABLE_ROWS.contains(&row_id.as_str()) {
            continue;
        }
        if projected_row.raw_line != original.raw_line {
            return ineligible(Phase35NonPromotionReason::ChecklistDrift);
        }
        fingerprints.insert(row_id.clone(), original.fingerprint.clone());
    }
    Ok(fingerprints)
}

fn validate_live_rechecks(
    evidence: &ValidatedPhase35Evidence,
    live: &Phase35LiveRechecks,
) -> Result<(), Phase35PromotionError> {
    let package = evidence.exact_package();
    let detector = evidence.detector_run();
    let facts = evidence.admission_facts();
    if live.evidence_sources != [Phase35EvidenceSource::ProtectedEvidenceRoot]
        || live
            .evidence_sources
            .iter()
            .any(|source| Phase35EvidenceSource::REJECTED.contains(source))
    {
        return ineligible(Phase35NonPromotionReason::IneligibleEvidenceCategory);
    }
    if live.lifecycle_id != PHASE35_LIFECYCLE_ID
        || facts.lifecycle_id != PHASE35_LIFECYCLE_ID
        || !facts.lifecycle_verified
    {
        return ineligible(Phase35NonPromotionReason::LifecycleMismatch);
    }
    if live.current_head != package.source_commit
        || !package.current_head_verified
        || !facts.current_head_rechecked
    {
        return ineligible(Phase35NonPromotionReason::StaleCurrentHead);
    }
    if !live.reference_clean
        || !package.reference_clean
        || !facts.reference_cleanliness_rechecked
        || live.reference_commit != package.reference_commit
    {
        return ineligible(Phase35NonPromotionReason::DirtyOrWrongReference);
    }
    if live.manifest_schema != "manifest-v3"
        || live.manifest_schema != package.manifest_schema
        || live.manifest_digest != package.manifest_digest
    {
        return ineligible(Phase35NonPromotionReason::ManifestV3Mismatch);
    }
    check_digest(
        &live.executable_image_digest,
        &package.executable_image_digest,
        Phase35NonPromotionReason::ExecutableImageMismatch,
    )?;
    check_digest(
        &live.factory_image_digest,
        &package.factory_image_digest,
        Phase35NonPromotionReason::FactoryImageMismatch,
    )?;
    check_digest(
        &live.package_digest,
        &package.package_digest,
        Phase35NonPromotionReason::PackageIdentityMismatch,
    )?;
    check_digest(
        &live.runtime_identity_digest,
        &package.runtime_identity_digest,
        Phase35NonPromotionReason::RuntimeIdentityMismatch,
    )?;
    if !live.detector_single_candidate
        || !live.detector_board_info
        || live.board_category != "205"
        || detector.board_category != "205"
        || live.detector_capability_digest != detector.capability_digest
    {
        return ineligible(Phase35NonPromotionReason::DetectorCapabilityMismatch);
    }
    if !live.root_event_chain_verified
        || live.root_contract_digest != evidence.root_digest()
        || facts.root_contract_digest != evidence.root_digest()
        || !facts.chronology_verified
        || !facts.inventory_verified
    {
        return ineligible(Phase35NonPromotionReason::RootOrEventChainMismatch);
    }
    if !live.no_actuation_verified || !facts.no_actuation_verified {
        return ineligible(Phase35NonPromotionReason::NoActuationFailure);
    }
    Ok(())
}

fn check_digest(
    actual: &str,
    expected: &str,
    reason: Phase35NonPromotionReason,
) -> Result<(), Phase35PromotionError> {
    if actual == expected {
        return Ok(());
    }
    ineligible(reason)
}

fn ineligible<T>(reason: Phase35NonPromotionReason) -> Result<T, Phase35PromotionError> {
    Err(Phase35PromotionError::Ineligible(reason))
}

fn decision_for_scope(scope: Phase35ClaimScope, root_digest: &str) -> Phase35PromotionDecision {
    let maybe_row = match scope {
        Phase35ClaimScope::PassiveHostnameDurability => Some(PHASE35_HOSTNAME_ROW),
        Phase35ClaimScope::ExactSourceReferencePackageIdentity => Some(PHASE35_IDENTITY_ROW),
        Phase35ClaimScope::CoherentOperatorSnapshot => Some(PHASE35_SNAPSHOT_ROW),
        Phase35ClaimScope::PassiveRuntimeHealthProjection => Some(PHASE35_HEALTH_ROW),
        Phase35ClaimScope::ActiveControl
        | Phase35ClaimScope::SelfTestEffects
        | Phase35ClaimScope::WatchdogIntervention
        | Phase35ClaimScope::MiningStratumAsic
        | Phase35ClaimScope::ArchivedPhase28_1_1
        | Phase35ClaimScope::Credentials
        | Phase35ClaimScope::DirectUartOrPins
        | Phase35ClaimScope::OtaOrRecovery
        | Phase35ClaimScope::OtherBoards
        | Phase35ClaimScope::LifecycleTestOnlyProof
        | Phase35ClaimScope::BroaderOrUnmappedRows => None,
    };
    if let Some(row_id) = maybe_row {
        return Phase35PromotionDecision::Promote {
            row_id: row_id.to_owned(),
            evidence_root_digest: root_digest.to_owned(),
        };
    }
    Phase35PromotionDecision::DoNotPromote {
        scope,
        reason: reason_for_excluded_scope(scope),
    }
}

fn reason_for_excluded_scope(scope: Phase35ClaimScope) -> Phase35NonPromotionReason {
    match scope {
        Phase35ClaimScope::ActiveControl => Phase35NonPromotionReason::ActiveControlExcluded,
        Phase35ClaimScope::SelfTestEffects => Phase35NonPromotionReason::SelfTestEffectsExcluded,
        Phase35ClaimScope::WatchdogIntervention => {
            Phase35NonPromotionReason::WatchdogInterventionExcluded
        }
        Phase35ClaimScope::MiningStratumAsic => {
            Phase35NonPromotionReason::MiningStratumAsicExcluded
        }
        Phase35ClaimScope::ArchivedPhase28_1_1 => {
            Phase35NonPromotionReason::ArchivedPhase28_1_1Excluded
        }
        Phase35ClaimScope::Credentials => Phase35NonPromotionReason::CredentialsExcluded,
        Phase35ClaimScope::DirectUartOrPins => Phase35NonPromotionReason::DirectUartOrPinsExcluded,
        Phase35ClaimScope::OtaOrRecovery => Phase35NonPromotionReason::OtaOrRecoveryExcluded,
        Phase35ClaimScope::OtherBoards => Phase35NonPromotionReason::OtherBoardsExcluded,
        Phase35ClaimScope::LifecycleTestOnlyProof => {
            Phase35NonPromotionReason::LifecycleTestOnlyProofExcluded
        }
        Phase35ClaimScope::BroaderOrUnmappedRows => {
            Phase35NonPromotionReason::BroaderOrUnmappedRowExcluded
        }
        _ => unreachable!("promotable scope is handled before exclusion mapping"),
    }
}

fn validate_scope_decisions(
    decisions: &[(Phase35ClaimScope, Phase35PromotionDecision)],
    root_digest: &str,
) -> Result<(), Phase35PromotionError> {
    let scopes = decisions
        .iter()
        .map(|(scope, _)| *scope)
        .collect::<BTreeSet<_>>();
    if scopes != Phase35ClaimScope::ALL.into_iter().collect() || scopes.len() != decisions.len() {
        return Err(Phase35PromotionError::Incomplete(
            "every claim scope must have exactly one decision".to_owned(),
        ));
    }
    let promoted = decisions
        .iter()
        .filter_map(|(_, decision)| match decision {
            Phase35PromotionDecision::Promote {
                row_id,
                evidence_root_digest,
            } => Some((row_id.as_str(), evidence_root_digest.as_str())),
            Phase35PromotionDecision::DoNotPromote { .. } => None,
        })
        .collect::<Vec<_>>();
    if promoted.len() != PHASE35_PROMOTABLE_ROWS.len()
        || promoted.iter().any(|(_, digest)| *digest != root_digest)
        || promoted
            .iter()
            .map(|(row, _)| *row)
            .collect::<BTreeSet<_>>()
            != PHASE35_PROMOTABLE_ROWS.into_iter().collect()
    {
        return Err(Phase35PromotionError::Incomplete(
            "promotions must map the exact allowlist to one root digest".to_owned(),
        ));
    }
    Ok(())
}
