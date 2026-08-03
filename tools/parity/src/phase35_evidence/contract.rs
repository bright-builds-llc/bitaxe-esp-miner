use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExactPackageCapability {
    pub(crate) source_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) reference_clean: bool,
    pub(crate) manifest_schema: String,
    pub(crate) manifest_digest: String,
    pub(crate) executable_image_digest: String,
    pub(crate) factory_image_digest: String,
    pub(crate) package_digest: String,
    pub(crate) runtime_identity_digest: String,
    pub(crate) current_head_verified: bool,
    pub(crate) capability_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DetectorRunCapability {
    pub(crate) board_category: String,
    pub(crate) detector_capability_digest: String,
    pub(crate) physical_identity_digest: String,
    pub(crate) board_info_verified: bool,
    pub(crate) single_candidate_verified: bool,
    pub(crate) run_id_digest: String,
    pub(crate) capability_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RootAdmissionFacts {
    pub(crate) root_contract_digest: String,
    pub(crate) target_lock_digest: String,
    pub(crate) lifecycle_id: String,
    pub(crate) lifecycle_verified: bool,
    pub(crate) current_head_rechecked: bool,
    pub(crate) reference_cleanliness_rechecked: bool,
    pub(crate) runtime_identity_rechecked: bool,
    pub(crate) no_actuation_verified: bool,
    pub(crate) inventory_verified: bool,
    pub(crate) chronology_verified: bool,
    pub(crate) restoration_verified: bool,
    pub(crate) cleanup_verified: bool,
    pub(crate) redaction_verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvidenceEventCategory {
    RootAdmitted,
    BootAObserved,
    PatchResponded,
    StorageConfirmed,
    RebootStarted,
    BootBObserved,
    NoActuationVerified,
    RestorationConfirmed,
    CleanupConfirmed,
}

impl EvidenceEventCategory {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::RootAdmitted => "root_admitted",
            Self::BootAObserved => "boot_a_observed",
            Self::PatchResponded => "patch_responded",
            Self::StorageConfirmed => "storage_confirmed",
            Self::RebootStarted => "reboot_started",
            Self::BootBObserved => "boot_b_observed",
            Self::NoActuationVerified => "no_actuation_verified",
            Self::RestorationConfirmed => "restoration_confirmed",
            Self::CleanupConfirmed => "cleanup_confirmed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceEventInput {
    pub(crate) sequence: u64,
    pub(crate) category: EvidenceEventCategory,
    pub(crate) monotonic_millis: u64,
    pub(crate) payload_digest: String,
    pub(crate) predecessor_event_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InventoryEntryInput {
    pub(crate) role: String,
    pub(crate) path: String,
    pub(crate) sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceEpochInput {
    pub(crate) boot_ordinal: u64,
    pub(crate) boot_session_digest: String,
    pub(crate) started_millis: u64,
    pub(crate) ended_millis: u64,
    pub(crate) system_info_document: String,
    pub(crate) websocket_document: String,
    pub(crate) retained_log_document: String,
    pub(crate) storage_revision: u64,
    pub(crate) storage_value_digest: String,
    pub(crate) reset_category: String,
    pub(crate) package_capability_digest: String,
    pub(crate) detector_capability_digest: String,
    pub(crate) root_contract_digest: String,
    pub(crate) target_lock_digest: String,
    pub(crate) run_id_digest: String,
    pub(crate) runtime_identity_digest: String,
    pub(crate) physical_identity_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase35EvidenceRootInput {
    pub(crate) schema_version: String,
    pub(crate) exact_package: ExactPackageCapability,
    pub(crate) detector_run: DetectorRunCapability,
    pub(crate) admission_facts: RootAdmissionFacts,
    pub(crate) events: Vec<EvidenceEventInput>,
    pub(crate) inventory: Vec<InventoryEntryInput>,
    pub(crate) boot_a: EvidenceEpochInput,
    pub(crate) boot_b: EvidenceEpochInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryArtifact {
    pub(super) bytes: Vec<u8>,
    pub(super) file_kind: ArtifactFileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArtifactFileKind {
    Regular,
    Symlink,
    Other,
}

impl InventoryArtifact {
    #[cfg(test)]
    pub(crate) fn regular(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            file_kind: ArtifactFileKind::Regular,
        }
    }

    #[cfg(test)]
    pub(crate) fn symlink() -> Self {
        Self {
            bytes: Vec::new(),
            file_kind: ArtifactFileKind::Symlink,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RedactedEpochFacts {
    pub(super) session_digest: String,
    pub(super) ordinal: u64,
    pub(super) revision: u64,
    pub(super) duration_millis: u64,
    pub(super) setting_digest: String,
    pub(super) coherent_snapshot_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedPhase35Evidence {
    pub(super) root_digest: String,
    pub(super) package_capability_digest: String,
    pub(super) detector_capability_digest: String,
    pub(super) run_digest: String,
    pub(super) board_category: String,
    pub(super) event_count: u64,
    pub(super) inventory_count: u64,
    pub(super) boot_a: RedactedEpochFacts,
    pub(super) boot_b: RedactedEpochFacts,
    pub(super) exact_package: ExactPackageCapability,
    pub(super) detector_run: DetectorRunCapability,
    pub(super) admission_facts: RootAdmissionFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ShareablePhase35Projection {
    pub(super) schema: &'static str,
    pub(super) root_digest: String,
    pub(super) package_capability_digest: String,
    pub(super) detector_capability_digest: String,
    pub(super) run_digest: String,
    pub(super) board_category: String,
    pub(super) event_count: u64,
    pub(super) inventory_count: u64,
    pub(super) boot_a: RedactedEpochFacts,
    pub(super) boot_b: RedactedEpochFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum Phase35EvidenceError {
    #[error("unsupported Phase 35 evidence schema")]
    UnsupportedSchema,
    #[error("invalid cryptographic digest")]
    InvalidDigest,
    #[error("manifest schema is not manifest-v3")]
    ManifestV3Mismatch,
    #[error("executable image identity mismatch")]
    ExecutableImageMismatch,
    #[error("factory image identity mismatch")]
    FactoryImageMismatch,
    #[error("package identity mismatch")]
    PackageMismatch,
    #[error("runtime identity mismatch")]
    RuntimeIdentityMismatch,
    #[error("invalid exact-package capability")]
    InvalidPackageCapability,
    #[error("invalid detector capability")]
    InvalidDetectorCapability,
    #[error("root contract mismatch")]
    RootContractMismatch,
    #[error("board category is not Ultra 205")]
    WrongBoard,
    #[error("evidence inventory mismatch")]
    InventoryMismatch,
    #[error("unsafe evidence path")]
    UnsafePath,
    #[error("evidence symlink is forbidden")]
    Symlink,
    #[error("evidence chronology violation")]
    ChronologyViolation,
    #[error("event predecessor violation")]
    PredecessorViolation,
    #[error("duplicate evidence checkpoint")]
    DuplicateCheckpoint,
    #[error("missing evidence checkpoint")]
    MissingCheckpoint,
    #[error("mixed boot-local snapshot session")]
    MixedSession,
    #[error("boot ordinal mismatch")]
    BootOrdinalMismatch,
    #[error("unapproved reset category")]
    UnapprovedReset,
    #[error("physical identity drift")]
    PhysicalIdentityDrift,
    #[error("storage value or revision mismatch")]
    ValueRevisionMismatch,
    #[error("current HEAD is stale")]
    StaleCurrentHead,
    #[error("reference checkout is dirty")]
    DirtyReference,
    #[error("Phase 35 lifecycle mismatch")]
    LifecycleMismatch,
    #[error("no-actuation proof failed")]
    NoActuationFailure,
    #[error("restoration proof failed")]
    RestorationFailure,
    #[error("cleanup proof failed")]
    CleanupFailure,
    #[error("shareable projection contains a forbidden field")]
    ForbiddenProjectionField,
}
