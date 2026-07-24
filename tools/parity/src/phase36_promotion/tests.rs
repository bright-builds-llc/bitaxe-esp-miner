use std::collections::BTreeSet;

use serde_json::Value;

use super::checklist::parse_checklist_rows;
use super::types::digest_serializable;
use super::*;
use crate::phase35_evidence::tests::EligibleFixture;
use crate::phase36_evidence::effects::{
    IndependentEffectObservationSource, ValidatedIndependentEffectInterval,
};
use crate::phase36_evidence::runtime_identity::{
    ExactPackageIdentityJoin, RuntimeIdentityObservationSource, ValidatedObservedRuntimeIdentity,
};
use crate::phase36_evidence::{
    validate_substantive_snapshot_documents, SubstantiveEvidenceAdmission,
};

const CHECKLIST: &str = include_str!("../../../../docs/parity/checklist.md");
const SUBSTANCE: &str = include_str!("../../fixtures/phase36/substance-eligible.json");

fn substantive_prerequisites() -> (
    crate::phase36_evidence::ValidatedSensorSubstance,
    crate::phase36_evidence::ValidatedRuntimeHealthSubstance,
    crate::phase36_evidence::SubstantiveSnapshotJoin,
) {
    let value: Value = serde_json::from_str(SUBSTANCE).expect("fixture must parse");
    let json = serde_json::to_string(&value).expect("fixture must serialize");
    let revision = value["operatorSnapshotRevision"]
        .as_u64()
        .expect("revision must be numeric");
    let session = value["bootSession"]
        .as_str()
        .expect("session must be textual");
    let api = format!(
        "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
    );
    let websocket = format!(
        "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
    );
    let retained = format!(
        "operator_snapshot session={session} revision={revision} redacted=true\nsubstantive_snapshot_json: {json}\n"
    );
    let SubstantiveEvidenceAdmission::Validated { evidence } =
        validate_substantive_snapshot_documents(&api, &websocket, &retained)
            .expect("fixture must validate")
    else {
        panic!("fixture must be substantive");
    };
    (evidence.sensors, evidence.runtime_health, evidence.join)
}

fn observed_runtime_identity() -> ValidatedObservedRuntimeIdentity {
    let exact_package = ExactPackageIdentityJoin {
        manifest_digest: "1".repeat(64),
        executable_image_digest: "2".repeat(64),
        factory_image_digest: "3".repeat(64),
        firmware_elf_digest: "4".repeat(64),
        package_digest: "5".repeat(64),
    };
    let observation_source = RuntimeIdentityObservationSource::DeviceSessionReplay;
    let same_physical_device = true;
    let boot_b_session_digest = "6".repeat(64);
    let boot_b_ordinal = 32;
    let source_commit_digest = "7".repeat(64);
    let reference_commit_digest = "8".repeat(64);
    let application_elf_digest = "4".repeat(64);
    let claim_fact_digest = digest_serializable(&(
        observation_source,
        same_physical_device,
        &boot_b_session_digest,
        boot_b_ordinal,
        &source_commit_digest,
        &reference_commit_digest,
        &application_elf_digest,
        &exact_package,
    ))
    .expect("identity digest must serialize");
    ValidatedObservedRuntimeIdentity {
        observation_source,
        same_physical_device,
        boot_b_session_digest,
        boot_b_ordinal,
        source_commit_digest,
        reference_commit_digest,
        application_elf_digest,
        exact_package,
        claim_fact_digest,
    }
}

fn independent_effect() -> ValidatedIndependentEffectInterval {
    let observation_source = IndependentEffectObservationSource::IndependentLedger;
    let start_millis = 100;
    let end_millis = 1_000;
    let ledger_digest = "9".repeat(64);
    let claim_fact_digest =
        digest_serializable(&(observation_source, start_millis, end_millis, &ledger_digest))
            .expect("effect digest must serialize");
    ValidatedIndependentEffectInterval {
        observation_source,
        start_millis,
        end_millis,
        duration_millis: end_millis - start_millis,
        effect_count: 8,
        ledger_digest,
        claim_fact_digest,
    }
}

pub(crate) fn prerequisites() -> Phase36ClaimPrerequisites {
    let evidence = EligibleFixture::new()
        .validate()
        .expect("Phase 35 fixture must validate");
    let hostname = ValidatedHostnameDurabilityFacts::from_phase35(&evidence)
        .expect("hostname facts must derive from validated Phase 35 evidence");
    let (sensors, runtime_health, join) = substantive_prerequisites();
    Phase36ClaimPrerequisites {
        phase35_root_digest: evidence.root_digest().to_owned(),
        superseded_phase35_generation_digest: "a".repeat(64),
        evaluator_digest: "b".repeat(64),
        maybe_hostname: Some(hostname),
        maybe_sensors: Some(sensors),
        maybe_snapshot_join: Some(join),
        maybe_runtime_health: Some(runtime_health),
        maybe_runtime_identity: Some(observed_runtime_identity()),
        maybe_independent_effect: Some(independent_effect()),
    }
}

fn checklist(contents: &str) -> Phase36ChecklistSnapshot {
    Phase36ChecklistSnapshot::capture(contents.to_owned()).expect("checklist must parse")
}

fn decision_for(
    matrix: &Phase36PromotionMatrix,
    scope: Phase36ClaimScope,
) -> &Phase36ClaimDecision {
    matrix
        .scope_decisions
        .iter()
        .find(|decision| decision.scope() == scope)
        .expect("scope must have a decision")
}

fn checklist_with_affected_status(status: &str) -> String {
    let rows = parse_checklist_rows(CHECKLIST).expect("checklist must parse");
    PHASE36_AFFECTED_ROWS
        .iter()
        .fold(CHECKLIST.to_owned(), |contents, row_id| {
            let row = &rows[*row_id];
            let mut cells = row.cells.clone();
            cells[4] = status.to_owned();
            contents.replace(&row.raw_line, &format!("| {} |", cells.join(" | ")))
        })
}

#[test]
fn phase36_promotion_supports_zero_through_four_promotions() {
    let verified_checklist = checklist_with_affected_status("verified");
    for promotion_count in 0..=4 {
        // Arrange
        let mut contents = verified_checklist.clone();
        for row_id in PHASE36_AFFECTED_ROWS.iter().take(promotion_count) {
            let row = parse_checklist_rows(&contents).expect("checklist must parse")[*row_id]
                .raw_line
                .clone();
            contents = contents.replace(&row, &row.replacen("| verified |", "| implemented |", 1));
        }

        // Act
        let matrix = evaluate_phase36_promotion(&prerequisites(), &checklist(&contents))
            .expect("matching prerequisites must evaluate");
        let promotions = matrix
            .scope_decisions
            .iter()
            .filter(|decision| matches!(decision, Phase36ClaimDecision::Promote { .. }))
            .count();

        // Assert
        assert_eq!(promotions, promotion_count);
        assert_eq!(matrix.supported_row_ids().len(), 4);
    }
}

#[test]
fn phase36_promotion_each_claim_specific_absence_changes_only_its_row() {
    type Mutation = fn(&mut Phase36ClaimPrerequisites);
    let cases: [(Phase36ClaimScope, Mutation); 4] = [
        (Phase36ClaimScope::PassiveHostnameDurability, |input| {
            input.maybe_hostname = None
        }),
        (
            Phase36ClaimScope::ExactSourceReferencePackageIdentity,
            |input| input.maybe_runtime_identity = None,
        ),
        (Phase36ClaimScope::CoherentOperatorSnapshot, |input| {
            input.maybe_sensors = None
        }),
        (Phase36ClaimScope::PassiveRuntimeHealthProjection, |input| {
            input.maybe_runtime_health = None
        }),
    ];
    let verified_checklist = checklist_with_affected_status("verified");
    let baseline = evaluate_phase36_promotion(&prerequisites(), &checklist(&verified_checklist))
        .expect("baseline must evaluate");
    let baseline_rows =
        parse_checklist_rows(&baseline.projected_checklist).expect("projection must parse");
    for (scope, mutate) in cases {
        // Arrange
        let mut input = prerequisites();
        mutate(&mut input);

        // Act
        let matrix = evaluate_phase36_promotion(&input, &checklist(&verified_checklist))
            .expect("insufficiency must produce a correction");
        let rows =
            parse_checklist_rows(&matrix.projected_checklist).expect("projection must parse");
        let changed = rows
            .iter()
            .filter_map(|(row_id, row)| {
                (row.raw_line != baseline_rows[row_id].raw_line).then_some(row_id.as_str())
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(changed, [scope.row_id().expect("affected scope has row")]);
        assert!(matches!(
            decision_for(&matrix, scope),
            Phase36ClaimDecision::Demote { .. }
        ));
    }
}

#[test]
fn phase36_promotion_missing_independent_effect_preserves_phase35_hostname_claim() {
    // Arrange
    let mut input = prerequisites();
    input.maybe_independent_effect = None;

    // Act
    let verified_checklist = checklist_with_affected_status("verified");
    let matrix = evaluate_phase36_promotion(&input, &checklist(&verified_checklist))
        .expect("shared insufficiency must produce exact corrections");

    // Assert
    assert!(matches!(
        decision_for(&matrix, Phase36ClaimScope::PassiveHostnameDurability),
        Phase36ClaimDecision::Preserve { .. }
    ));
    assert!(matrix
        .scope_decisions
        .iter()
        .filter(|decision| {
            decision.maybe_row_id().is_some()
                && decision.scope() != Phase36ClaimScope::PassiveHostnameDurability
        })
        .all(|decision| matches!(
            decision,
            Phase36ClaimDecision::Demote {
                reason: Phase36DecisionReason::IndependentEffectObservationInsufficient,
                ..
            }
        )));
}

#[test]
fn phase36_promotion_preserves_unrelated_rows_and_non_claims_byte_identically() {
    // Arrange
    let original_rows = parse_checklist_rows(CHECKLIST).expect("checklist must parse");

    // Act
    let matrix = evaluate_phase36_promotion(&prerequisites(), &checklist(CHECKLIST))
        .expect("baseline must evaluate");
    let projected_rows =
        parse_checklist_rows(&matrix.projected_checklist).expect("projection must parse");

    // Assert
    for (row_id, original) in original_rows {
        if PHASE36_AFFECTED_ROWS.contains(&row_id.as_str()) {
            continue;
        }
        assert_eq!(projected_rows[&row_id].raw_line, original.raw_line);
    }
    let excluded = matrix
        .scope_decisions
        .iter()
        .filter(|decision| decision.maybe_row_id().is_none())
        .map(Phase36ClaimDecision::scope)
        .collect::<BTreeSet<_>>();
    assert_eq!(excluded.len(), Phase36ClaimScope::ALL.len() - 4);
    assert!(matrix
        .scope_decisions
        .iter()
        .filter(|decision| decision.maybe_row_id().is_none())
        .all(|decision| matches!(decision, Phase36ClaimDecision::DoNotPromote { .. })));
}

#[test]
fn phase36_promotion_positive_decisions_have_exact_lowercase_fact_digests() {
    // Arrange
    let matrix = evaluate_phase36_promotion(&prerequisites(), &checklist(CHECKLIST))
        .expect("baseline must evaluate");

    // Act
    let digests = matrix
        .scope_decisions
        .iter()
        .filter(|decision| decision.is_supported())
        .map(Phase36ClaimDecision::claim_fact_digest)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(digests.len(), 4);
    assert!(digests.iter().all(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }));
}

#[test]
fn phase36_promotion_rejects_wrong_claim_digest_reuse() {
    // Arrange
    let mut input = prerequisites();
    let snapshot_digest = input
        .maybe_sensors
        .as_ref()
        .expect("sensors must exist")
        .claim_fact_digest
        .clone();
    input
        .maybe_runtime_identity
        .as_mut()
        .expect("identity must exist")
        .claim_fact_digest = snapshot_digest;

    // Act
    let result = evaluate_phase36_promotion(&input, &checklist(CHECKLIST));

    // Assert
    assert!(matches!(
        result,
        Err(Phase36PromotionError::InvalidPrerequisiteDigest)
            | Err(Phase36PromotionError::ReusedWrongClaimDigest)
    ));
}

#[test]
fn phase36_generation_resolver_retains_historical_fingerprints() {
    // Arrange
    let input = prerequisites();

    // Act
    let matrix =
        evaluate_phase36_promotion(&input, &checklist(CHECKLIST)).expect("baseline must evaluate");

    // Assert
    assert_eq!(
        matrix.resolver.phase35_root_digest,
        input.phase35_root_digest
    );
    assert_eq!(
        matrix.resolver.superseded_phase35_generation_digest,
        input.superseded_phase35_generation_digest
    );
    assert_eq!(
        matrix
            .resolver
            .authoritative_phase36_generation_digest
            .len(),
        64
    );
}
