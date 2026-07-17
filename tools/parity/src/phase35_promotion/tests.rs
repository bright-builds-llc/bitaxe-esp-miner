use super::checklist::parse_checklist_rows;
use super::*;
use crate::phase35_evidence::tests::EligibleFixture;
use crate::phase35_evidence::ValidatedPhase35Evidence;

const CHECKLIST: &str = include_str!("../../../../docs/parity/checklist.md");

fn eligible() -> (ValidatedPhase35Evidence, ChecklistSnapshot) {
    let evidence = EligibleFixture::new()
        .validate()
        .expect("synthetic eligible evidence should validate");
    let live = Phase35LiveRechecks::matching(&evidence);
    let checklist = ChecklistSnapshot::capture(CHECKLIST.to_owned(), live)
        .expect("repository checklist should parse");
    (evidence, checklist)
}

#[test]
fn phase35_promotion_is_complete_and_uses_one_root_digest() {
    // Arrange
    let (evidence, checklist) = eligible();

    // Act
    let matrix = evaluate_phase35_promotion(&evidence, &checklist)
        .expect("exact eligible root should promote");

    // Assert
    assert_eq!(matrix.scope_decisions.len(), Phase35ClaimScope::ALL.len());
    assert_eq!(
        Phase35ClaimScope::BroaderOrUnmappedRows as usize + 1,
        Phase35ClaimScope::ALL.len()
    );
    assert_eq!(matrix.promoted_row_ids(), PHASE35_PROMOTABLE_ROWS);
    assert!(matrix
        .scope_decisions
        .iter()
        .all(|(_, decision)| match decision {
            Phase35PromotionDecision::Promote {
                evidence_root_digest,
                ..
            } => evidence_root_digest == evidence.root_digest(),
            Phase35PromotionDecision::DoNotPromote { .. } => true,
        }));
}

#[test]
fn phase35_promotion_preserves_every_non_allowlisted_row_byte_identically() {
    // Arrange
    let (evidence, checklist) = eligible();
    let original_rows = parse_checklist_rows(CHECKLIST).expect("checklist parses");

    // Act
    let matrix = evaluate_phase35_promotion(&evidence, &checklist)
        .expect("exact eligible root should promote");
    let projected_rows =
        parse_checklist_rows(&matrix.projected_checklist).expect("projection parses");

    // Assert
    for (row_id, original) in original_rows {
        if PHASE35_PROMOTABLE_ROWS.contains(&row_id.as_str()) {
            continue;
        }
        assert_eq!(projected_rows[&row_id].raw_line, original.raw_line);
    }
}

#[test]
fn phase35_promotion_rejects_administrative_artifact_sources() {
    let sources = [
        Phase35EvidenceSource::LifecycleArtifact,
        Phase35EvidenceSource::PlanArtifact,
        Phase35EvidenceSource::SummaryArtifact,
        Phase35EvidenceSource::TestArtifact,
        Phase35EvidenceSource::VerificationArtifact,
        Phase35EvidenceSource::SecurityArtifact,
    ];
    for source in sources {
        // Arrange
        let (evidence, mut checklist) = eligible();
        checklist.live.evidence_sources = vec![source];

        // Act
        let error = evaluate_phase35_promotion(&evidence, &checklist)
            .expect_err("administrative artifact must not promote");

        // Assert
        assert!(matches!(
            error,
            Phase35PromotionError::Ineligible(
                Phase35NonPromotionReason::IneligibleEvidenceCategory
            )
        ));
    }
}

#[test]
fn phase35_promotion_rechecks_every_live_gate() {
    type Mutation = fn(&mut Phase35LiveRechecks);
    let cases: [(Mutation, Phase35NonPromotionReason); 15] = [
        (
            |live| live.current_head = "0".repeat(40),
            Phase35NonPromotionReason::StaleCurrentHead,
        ),
        (
            |live| live.reference_clean = false,
            Phase35NonPromotionReason::DirtyOrWrongReference,
        ),
        (
            |live| live.reference_commit = "0".repeat(40),
            Phase35NonPromotionReason::DirtyOrWrongReference,
        ),
        (
            |live| live.manifest_schema = "manifest-v2".to_owned(),
            Phase35NonPromotionReason::ManifestV3Mismatch,
        ),
        (
            |live| live.executable_image_digest = "0".repeat(64),
            Phase35NonPromotionReason::ExecutableImageMismatch,
        ),
        (
            |live| live.manifest_digest = "0".repeat(64),
            Phase35NonPromotionReason::ManifestV3Mismatch,
        ),
        (
            |live| live.factory_image_digest = "0".repeat(64),
            Phase35NonPromotionReason::FactoryImageMismatch,
        ),
        (
            |live| live.package_digest = "0".repeat(64),
            Phase35NonPromotionReason::PackageIdentityMismatch,
        ),
        (
            |live| live.runtime_identity_digest = "0".repeat(64),
            Phase35NonPromotionReason::RuntimeIdentityMismatch,
        ),
        (
            |live| live.detector_single_candidate = false,
            Phase35NonPromotionReason::DetectorCapabilityMismatch,
        ),
        (
            |live| live.detector_capability_digest = "0".repeat(64),
            Phase35NonPromotionReason::DetectorCapabilityMismatch,
        ),
        (
            |live| live.root_event_chain_verified = false,
            Phase35NonPromotionReason::RootOrEventChainMismatch,
        ),
        (
            |live| live.lifecycle_id = "stale".to_owned(),
            Phase35NonPromotionReason::LifecycleMismatch,
        ),
        (
            |live| live.no_actuation_verified = false,
            Phase35NonPromotionReason::NoActuationFailure,
        ),
        (
            |live| live.board_category = "601".to_owned(),
            Phase35NonPromotionReason::DetectorCapabilityMismatch,
        ),
    ];
    for (mutate, expected) in cases {
        // Arrange
        let (evidence, mut checklist) = eligible();
        mutate(&mut checklist.live);

        // Act
        let error = evaluate_phase35_promotion(&evidence, &checklist)
            .expect_err("live gate drift must fail closed");

        // Assert
        assert!(matches!(error, Phase35PromotionError::Ineligible(reason) if reason == expected));
    }
}

#[test]
fn phase35_promotion_rejects_missing_duplicate_and_unclassified_rows() {
    // Arrange
    let (evidence, checklist) = eligible();
    let missing = checklist
        .contents
        .lines()
        .filter(|line| !line.contains(types::PHASE35_HOSTNAME_ROW))
        .collect::<Vec<_>>()
        .join("\n");
    let duplicate = format!(
        "{}\n{}",
        checklist.contents,
        checklist.rows[types::PHASE35_HOSTNAME_ROW].raw_line
    );

    // Act
    let missing_result = ChecklistSnapshot::capture(missing, checklist.live.clone());
    let duplicate_result = ChecklistSnapshot::capture(duplicate, checklist.live.clone());
    let matrix = evaluate_phase35_promotion(&evidence, &checklist)
        .expect("baseline matrix should remain complete");

    // Assert
    assert!(missing_result.is_err());
    assert!(duplicate_result.is_err());
    assert_eq!(
        matrix.preserved_row_fingerprints.len() + PHASE35_PROMOTABLE_ROWS.len(),
        checklist.rows.len()
    );
}
