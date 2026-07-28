use std::collections::BTreeMap;

use serde::Serialize;
use thiserror::Error;

use crate::phase35_evidence::sha256_hex;
#[cfg(test)]
use crate::phase35_evidence::ValidatedPhase35Evidence;
use crate::phase36_evidence::effects::ValidatedIndependentEffectInterval;
use crate::phase36_evidence::runtime_identity::ValidatedObservedRuntimeIdentity;
use crate::phase36_evidence::{
    SubstantiveSnapshotJoin, ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};

pub(crate) const PHASE36_HOSTNAME_ROW: &str = "V12-HOSTNAME-205";
pub(crate) const PHASE36_IDENTITY_ROW: &str = "V12-PACKAGE-IDENTITY-205";
pub(crate) const PHASE36_SNAPSHOT_ROW: &str = "V12-OPERATOR-SNAPSHOT-205";
pub(crate) const PHASE36_HEALTH_ROW: &str = "V12-RUNTIME-HEALTH-205";

pub(crate) const PHASE36_AFFECTED_ROWS: [&str; 4] = [
    PHASE36_HOSTNAME_ROW,
    PHASE36_IDENTITY_ROW,
    PHASE36_SNAPSHOT_ROW,
    PHASE36_HEALTH_ROW,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase36ClaimScope {
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

impl Phase36ClaimScope {
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

    pub(crate) const fn maybe_row_id(self) -> Option<&'static str> {
        match self {
            Self::PassiveHostnameDurability => Some(PHASE36_HOSTNAME_ROW),
            Self::ExactSourceReferencePackageIdentity => Some(PHASE36_IDENTITY_ROW),
            Self::CoherentOperatorSnapshot => Some(PHASE36_SNAPSHOT_ROW),
            Self::PassiveRuntimeHealthProjection => Some(PHASE36_HEALTH_ROW),
            Self::ActiveControl
            | Self::SelfTestEffects
            | Self::WatchdogIntervention
            | Self::MiningStratumAsic
            | Self::ArchivedPhase28_1_1
            | Self::Credentials
            | Self::DirectUartOrPins
            | Self::OtaOrRecovery
            | Self::OtherBoards
            | Self::LifecycleTestOnlyProof
            | Self::BroaderOrUnmappedRows => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase36DecisionReason {
    HostnameDurabilityFactsInsufficient,
    RuntimeIdentityObservationInsufficient,
    SnapshotSubstanceInsufficient,
    RuntimeHealthInsufficient,
    IndependentEffectObservationInsufficient,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub(crate) enum Phase36ClaimDecision {
    Promote {
        scope: Phase36ClaimScope,
        row_id: String,
        claim_fact_digest: String,
    },
    Demote {
        scope: Phase36ClaimScope,
        row_id: String,
        reason: Phase36DecisionReason,
        claim_fact_digest: String,
    },
    Preserve {
        scope: Phase36ClaimScope,
        row_id: String,
        claim_fact_digest: String,
    },
    DoNotPromote {
        scope: Phase36ClaimScope,
        maybe_row_id: Option<String>,
        reason: Phase36DecisionReason,
        claim_fact_digest: String,
    },
}

impl Phase36ClaimDecision {
    pub(crate) const fn scope(&self) -> Phase36ClaimScope {
        match self {
            Self::Promote { scope, .. }
            | Self::Demote { scope, .. }
            | Self::Preserve { scope, .. }
            | Self::DoNotPromote { scope, .. } => *scope,
        }
    }

    pub(crate) fn maybe_row_id(&self) -> Option<&str> {
        match self {
            Self::Promote { row_id, .. }
            | Self::Demote { row_id, .. }
            | Self::Preserve { row_id, .. } => Some(row_id),
            Self::DoNotPromote { maybe_row_id, .. } => maybe_row_id.as_deref(),
        }
    }

    pub(crate) fn claim_fact_digest(&self) -> &str {
        match self {
            Self::Promote {
                claim_fact_digest, ..
            }
            | Self::Demote {
                claim_fact_digest, ..
            }
            | Self::Preserve {
                claim_fact_digest, ..
            }
            | Self::DoNotPromote {
                claim_fact_digest, ..
            } => claim_fact_digest,
        }
    }

    pub(crate) const fn is_supported(&self) -> bool {
        matches!(self, Self::Promote { .. } | Self::Preserve { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ValidatedHostnameDurabilityFacts {
    pub(crate) phase35_root_digest: String,
    pub(crate) storage_confirmed: bool,
    pub(crate) reload_confirmed: bool,
    pub(crate) exactly_once_reboot_confirmed: bool,
    pub(crate) restoration_confirmed: bool,
    pub(crate) cleanup_confirmed: bool,
    pub(crate) claim_fact_digest: String,
}

impl ValidatedHostnameDurabilityFacts {
    #[cfg(test)]
    pub(crate) fn from_phase35(
        evidence: &ValidatedPhase35Evidence,
    ) -> Result<Self, Phase36PromotionError> {
        evidence
            .shareable_projection()
            .map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let admission = evidence.admission_facts();
        let phase35_root_digest = evidence.root_digest().to_owned();
        let storage_confirmed = true;
        let reload_confirmed = true;
        let exactly_once_reboot_confirmed = true;
        let restoration_confirmed = admission.restoration_verified;
        let cleanup_confirmed = admission.cleanup_verified;
        let digest_input = (
            "phase36-hostname-durability-facts-v1",
            &phase35_root_digest,
            storage_confirmed,
            reload_confirmed,
            exactly_once_reboot_confirmed,
            restoration_confirmed,
            cleanup_confirmed,
        );
        let claim_fact_digest = digest_serializable(&digest_input)?;
        Ok(Self {
            phase35_root_digest,
            storage_confirmed,
            reload_confirmed,
            exactly_once_reboot_confirmed,
            restoration_confirmed,
            cleanup_confirmed,
            claim_fact_digest,
        })
    }

    pub(crate) fn from_public_generation(
        manifest_document: &str,
        projection_document: &str,
        matrix_document: &str,
        verdict_document: &str,
    ) -> Result<Self, Phase36PromotionError> {
        let manifest: serde_json::Value = serde_json::from_str(manifest_document)
            .map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let projection: serde_json::Value = serde_json::from_str(projection_document)
            .map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let matrix: serde_json::Value = serde_json::from_str(matrix_document)
            .map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let verdict: serde_json::Value = serde_json::from_str(verdict_document)
            .map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let phase35_root_digest = manifest
            .get("root_digest")
            .and_then(serde_json::Value::as_str)
            .ok_or(Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let projection_digest = manifest
            .get("projection_sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or(Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let matrix_digest = manifest
            .get("matrix_sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or(Phase36PromotionError::InvalidPrerequisiteDigest)?;
        let projection_root = projection
            .get("root_digest")
            .and_then(serde_json::Value::as_str);
        let matrix_root = matrix
            .get("evidence_root_digest")
            .and_then(serde_json::Value::as_str);
        let verdict_root = verdict
            .get("evidence_root_digest")
            .and_then(serde_json::Value::as_str);
        if manifest.get("schema").and_then(serde_json::Value::as_str)
            != Some("phase35-generation-v1")
            || projection.get("schema").and_then(serde_json::Value::as_str)
                != Some("phase35-evidence-v1")
            || verdict.get("admitted").and_then(serde_json::Value::as_bool) != Some(true)
            || !is_lower_hex(phase35_root_digest, 64)
            || projection_root != Some(phase35_root_digest)
            || matrix_root != Some(phase35_root_digest)
            || verdict_root != Some(phase35_root_digest)
            || projection_digest != sha256_hex(projection_document.as_bytes())
            || matrix_digest != sha256_hex(matrix_document.as_bytes())
            || !matrix_preserves_hostname_claim(&matrix, phase35_root_digest)
        {
            return Err(Phase36PromotionError::InvalidPrerequisiteDigest);
        }
        let digest_input = (
            "phase36-hostname-durability-facts-v1",
            phase35_root_digest,
            true,
            true,
            true,
            true,
            true,
        );
        Ok(Self {
            phase35_root_digest: phase35_root_digest.to_owned(),
            storage_confirmed: true,
            reload_confirmed: true,
            exactly_once_reboot_confirmed: true,
            restoration_confirmed: true,
            cleanup_confirmed: true,
            claim_fact_digest: digest_serializable(&digest_input)?,
        })
    }

    pub(crate) const fn complete(&self) -> bool {
        self.storage_confirmed
            && self.reload_confirmed
            && self.exactly_once_reboot_confirmed
            && self.restoration_confirmed
            && self.cleanup_confirmed
    }
}

fn matrix_preserves_hostname_claim(matrix: &serde_json::Value, root_digest: &str) -> bool {
    let Some(decisions) = matrix
        .get("scope_decisions")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    let matching = decisions
        .iter()
        .filter(|entry| {
            entry.get(0).and_then(serde_json::Value::as_str) == Some("passive_hostname_durability")
        })
        .collect::<Vec<_>>();
    let [entry] = matching.as_slice() else {
        return false;
    };
    let Some(decision) = entry.get(1) else {
        return false;
    };
    decision.get("decision").and_then(serde_json::Value::as_str) == Some("promote")
        && decision.get("row_id").and_then(serde_json::Value::as_str) == Some("V12-HOSTNAME-205")
        && decision
            .get("evidence_root_digest")
            .and_then(serde_json::Value::as_str)
            == Some(root_digest)
}

#[derive(Debug, Clone)]
pub(crate) struct Phase36ClaimPrerequisites {
    pub(crate) phase35_root_digest: String,
    pub(crate) superseded_phase35_generation_digest: String,
    pub(crate) evaluator_digest: String,
    pub(crate) maybe_hostname: Option<ValidatedHostnameDurabilityFacts>,
    pub(crate) maybe_sensors: Option<ValidatedSensorSubstance>,
    pub(crate) maybe_snapshot_join: Option<SubstantiveSnapshotJoin>,
    pub(crate) maybe_runtime_health: Option<ValidatedRuntimeHealthSubstance>,
    pub(crate) maybe_runtime_identity: Option<ValidatedObservedRuntimeIdentity>,
    pub(crate) maybe_independent_effect: Option<ValidatedIndependentEffectInterval>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase36GenerationResolver {
    pub(crate) phase35_root_digest: String,
    pub(crate) superseded_phase35_generation_digest: String,
    pub(crate) authoritative_phase36_generation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase36PromotionMatrix {
    pub(crate) schema_version: &'static str,
    pub(crate) phase35_root_digest: String,
    pub(crate) superseded_phase35_generation_digest: String,
    pub(crate) evaluator_digest: String,
    pub(crate) checklist_fingerprint_before: String,
    pub(crate) checklist_fingerprint_after: String,
    pub(crate) scope_decisions: Vec<Phase36ClaimDecision>,
    pub(crate) preserved_row_fingerprints: BTreeMap<String, String>,
    pub(crate) resolver: Phase36GenerationResolver,
    #[serde(skip)]
    pub(crate) projected_checklist: String,
}

impl Phase36PromotionMatrix {
    pub(crate) fn supported_row_ids(&self) -> Vec<&str> {
        self.scope_decisions
            .iter()
            .filter(|decision| decision.is_supported())
            .filter_map(Phase36ClaimDecision::maybe_row_id)
            .collect()
    }
}

#[derive(Debug, Error)]
pub(crate) enum Phase36PromotionError {
    #[error("invalid Phase 36 checklist snapshot: {0}")]
    Checklist(String),
    #[error("invalid Phase 36 prerequisite digest")]
    InvalidPrerequisiteDigest,
    #[error("Phase 36 claim fact digest reused for the wrong claim")]
    ReusedWrongClaimDigest,
    #[error("incomplete Phase 36 promotion matrix: {0}")]
    Incomplete(String),
}

pub(crate) fn digest_serializable(value: &impl Serialize) -> Result<String, Phase36PromotionError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| Phase36PromotionError::InvalidPrerequisiteDigest)?;
    Ok(sha256_hex(&bytes))
}

pub(crate) fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
