use std::collections::BTreeMap;

use serde::Serialize;
use thiserror::Error;

#[cfg(test)]
use crate::phase35_evidence::{ValidatedPhase35Evidence, PHASE35_LIFECYCLE_ID};

pub(crate) const PHASE35_HOSTNAME_ROW: &str = "V12-HOSTNAME-205";
pub(crate) const PHASE35_IDENTITY_ROW: &str = "V12-PACKAGE-IDENTITY-205";
pub(crate) const PHASE35_SNAPSHOT_ROW: &str = "V12-OPERATOR-SNAPSHOT-205";
pub(crate) const PHASE35_HEALTH_ROW: &str = "V12-RUNTIME-HEALTH-205";

pub(crate) const PHASE35_PROMOTABLE_ROWS: [&str; 4] = [
    PHASE35_HOSTNAME_ROW,
    PHASE35_IDENTITY_ROW,
    PHASE35_SNAPSHOT_ROW,
    PHASE35_HEALTH_ROW,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase35ClaimScope {
    PassiveHostnameDurability,
    ExactSourceReferencePackageIdentity,
    CoherentOperatorSnapshot,
    PassiveRuntimeHealthProjection,
    ActiveControl,
    SelfTestEffects,
    WatchdogIntervention,
    MiningStratumAsic,
    ArchivedPhase28_1_1,
    Credentials,
    DirectUartOrPins,
    OtaOrRecovery,
    OtherBoards,
    LifecycleTestOnlyProof,
    BroaderOrUnmappedRows,
}

impl Phase35ClaimScope {
    pub(crate) const ALL: [Self; 15] = [
        Self::PassiveHostnameDurability,
        Self::ExactSourceReferencePackageIdentity,
        Self::CoherentOperatorSnapshot,
        Self::PassiveRuntimeHealthProjection,
        Self::ActiveControl,
        Self::SelfTestEffects,
        Self::WatchdogIntervention,
        Self::MiningStratumAsic,
        Self::ArchivedPhase28_1_1,
        Self::Credentials,
        Self::DirectUartOrPins,
        Self::OtaOrRecovery,
        Self::OtherBoards,
        Self::LifecycleTestOnlyProof,
        Self::BroaderOrUnmappedRows,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase35NonPromotionReason {
    ActiveControlExcluded,
    SelfTestEffectsExcluded,
    WatchdogInterventionExcluded,
    MiningStratumAsicExcluded,
    ArchivedPhase28_1_1Excluded,
    CredentialsExcluded,
    DirectUartOrPinsExcluded,
    OtaOrRecoveryExcluded,
    OtherBoardsExcluded,
    LifecycleTestOnlyProofExcluded,
    BroaderOrUnmappedRowExcluded,
    IneligibleEvidenceCategory,
    LifecycleMismatch,
    StaleCurrentHead,
    DirtyOrWrongReference,
    ManifestV3Mismatch,
    ExecutableImageMismatch,
    FactoryImageMismatch,
    PackageIdentityMismatch,
    RuntimeIdentityMismatch,
    DetectorCapabilityMismatch,
    RootOrEventChainMismatch,
    NoActuationFailure,
    ChecklistDrift,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub(crate) enum Phase35PromotionDecision {
    Promote {
        row_id: String,
        evidence_root_digest: String,
    },
    DoNotPromote {
        scope: Phase35ClaimScope,
        reason: Phase35NonPromotionReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase35EvidenceSource {
    ProtectedEvidenceRoot,
    LifecycleArtifact,
    PlanArtifact,
    SummaryArtifact,
    TestArtifact,
    VerificationArtifact,
    SecurityArtifact,
}

impl Phase35EvidenceSource {
    pub(crate) const REJECTED: [Self; 6] = [
        Self::LifecycleArtifact,
        Self::PlanArtifact,
        Self::SummaryArtifact,
        Self::TestArtifact,
        Self::VerificationArtifact,
        Self::SecurityArtifact,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase35LiveRechecks {
    pub(crate) lifecycle_id: String,
    pub(crate) current_head: String,
    pub(crate) reference_commit: String,
    pub(crate) reference_clean: bool,
    pub(crate) manifest_schema: String,
    pub(crate) manifest_digest: String,
    pub(crate) executable_image_digest: String,
    pub(crate) factory_image_digest: String,
    pub(crate) package_digest: String,
    pub(crate) runtime_identity_digest: String,
    pub(crate) detector_capability_digest: String,
    pub(crate) detector_single_candidate: bool,
    pub(crate) detector_board_info: bool,
    pub(crate) board_category: String,
    pub(crate) root_contract_digest: String,
    pub(crate) root_event_chain_verified: bool,
    pub(crate) no_actuation_verified: bool,
    pub(crate) evidence_sources: Vec<Phase35EvidenceSource>,
}

impl Phase35LiveRechecks {
    #[cfg(test)]
    pub(crate) fn matching(validated: &ValidatedPhase35Evidence) -> Self {
        let package = validated.exact_package();
        let detector = validated.detector_run();
        Self {
            lifecycle_id: PHASE35_LIFECYCLE_ID.to_owned(),
            current_head: package.source_commit.clone(),
            reference_commit: package.reference_commit.clone(),
            reference_clean: true,
            manifest_schema: package.manifest_schema.clone(),
            manifest_digest: package.manifest_digest.clone(),
            executable_image_digest: package.executable_image_digest.clone(),
            factory_image_digest: package.factory_image_digest.clone(),
            package_digest: package.package_digest.clone(),
            runtime_identity_digest: package.runtime_identity_digest.clone(),
            detector_capability_digest: detector.capability_digest.clone(),
            detector_single_candidate: true,
            detector_board_info: true,
            board_category: "205".to_owned(),
            root_contract_digest: validated.root_digest().to_owned(),
            root_event_chain_verified: true,
            no_actuation_verified: true,
            evidence_sources: vec![Phase35EvidenceSource::ProtectedEvidenceRoot],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase35PromotionMatrix {
    pub(crate) evidence_root_digest: String,
    pub(crate) checklist_fingerprint_before: String,
    pub(crate) checklist_fingerprint_after: String,
    pub(crate) scope_decisions: Vec<(Phase35ClaimScope, Phase35PromotionDecision)>,
    pub(crate) preserved_row_fingerprints: BTreeMap<String, String>,
    #[serde(skip)]
    pub(crate) projected_checklist: String,
}

impl Phase35PromotionMatrix {
    pub(crate) fn promoted_row_ids(&self) -> Vec<&str> {
        self.scope_decisions
            .iter()
            .filter_map(|(_, decision)| match decision {
                Phase35PromotionDecision::Promote { row_id, .. } => Some(row_id.as_str()),
                Phase35PromotionDecision::DoNotPromote { .. } => None,
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub(crate) enum Phase35PromotionError {
    #[error("Phase 35 promotion is ineligible: {0:?}")]
    Ineligible(Phase35NonPromotionReason),
    #[error("invalid Phase 35 checklist snapshot: {0}")]
    Checklist(String),
    #[error("incomplete Phase 35 promotion matrix: {0}")]
    Incomplete(String),
}
