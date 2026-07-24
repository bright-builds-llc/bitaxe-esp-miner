use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) const PHASE36_SCHEMA: &str = "phase36-evidence-v1";
pub(crate) const SHAREABLE_PHASE36_FACTS_SCHEMA: &str = "phase36-shareable-facts-v1";
pub(crate) const PHASE36_INPUT_DOCUMENT: &str = "phase36.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase35RootReference {
    pub(crate) root_digest: String,
    pub(crate) evidence_source_commit: String,
    pub(crate) phase35_generation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase36EvaluationIdentity {
    pub(crate) evaluator_digest: String,
    pub(crate) successor_contract_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase36ArtifactRole {
    Phase35Root,
    Phase35Generation,
    SnapshotSubstance,
    RuntimeHealth,
    RuntimeIdentityObservation,
    IndependentEffectObservation,
}

impl Phase36ArtifactRole {
    pub(crate) const ORDERED: [Self; 6] = [
        Self::Phase35Root,
        Self::Phase35Generation,
        Self::SnapshotSubstance,
        Self::RuntimeHealth,
        Self::RuntimeIdentityObservation,
        Self::IndependentEffectObservation,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImmutableArtifactReference {
    pub(crate) role: Phase36ArtifactRole,
    pub(crate) relative_path: String,
    pub(crate) sha256: String,
    pub(crate) evidence_source_commit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub(crate) enum SufficiencyResult {
    Sufficient,
    Insufficient { category: ComponentInsufficiency },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ComponentInsufficiency {
    #[serde(rename = "snapshot_substance_insufficient")]
    SnapshotSubstance,
    #[serde(rename = "runtime_health_insufficient")]
    RuntimeHealth,
    #[serde(rename = "runtime_identity_observation_insufficient")]
    RuntimeIdentityObservation,
    #[serde(rename = "independent_effect_observation_insufficient")]
    IndependentEffectObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Attempt31Sufficiency {
    pub(crate) snapshot_substance: SufficiencyResult,
    pub(crate) runtime_health: SufficiencyResult,
    pub(crate) runtime_identity_observation: SufficiencyResult,
    pub(crate) independent_effect_observation: SufficiencyResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImmutableArtifactStatus {
    ImmutableArtifactsSufficient,
    ImmutableArtifactsInsufficient,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImmutableArtifactAssessment {
    pub(crate) status: ImmutableArtifactStatus,
    pub(crate) component_insufficiencies: Vec<ComponentInsufficiency>,
}

impl ImmutableArtifactAssessment {
    pub(crate) fn from_sufficiency(sufficiency: &Attempt31Sufficiency) -> Self {
        let component_insufficiencies = [
            sufficiency.snapshot_substance,
            sufficiency.runtime_health,
            sufficiency.runtime_identity_observation,
            sufficiency.independent_effect_observation,
        ]
        .into_iter()
        .filter_map(|result| match result {
            SufficiencyResult::Sufficient => None,
            SufficiencyResult::Insufficient { category } => Some(category),
        })
        .collect::<Vec<_>>();
        let status = if component_insufficiencies.is_empty() {
            ImmutableArtifactStatus::ImmutableArtifactsSufficient
        } else {
            ImmutableArtifactStatus::ImmutableArtifactsInsufficient
        };
        Self {
            status,
            component_insufficiencies,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SensorTruthState {
    Fresh,
    Stale,
    Unavailable,
    Fault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SensorReason {
    None,
    NeverObserved,
    AcquisitionFailed,
    ObservationExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PowerSensorFacts {
    pub(crate) state: SensorTruthState,
    pub(crate) maybe_current_milliamps: Option<i64>,
    pub(crate) maybe_bus_millivolts: Option<i64>,
    pub(crate) maybe_power_milliwatts: Option<i64>,
    pub(crate) producer_sequence: u64,
    pub(crate) acquisition_millis: u64,
    pub(crate) reason: SensorReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TemperatureSensorFacts {
    pub(crate) state: SensorTruthState,
    pub(crate) maybe_millicelsius: Option<i64>,
    pub(crate) producer_sequence: u64,
    pub(crate) acquisition_millis: u64,
    pub(crate) reason: SensorReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TachometerSensorFacts {
    pub(crate) state: SensorTruthState,
    pub(crate) maybe_rpm: Option<u64>,
    pub(crate) producer_sequence: u64,
    pub(crate) acquisition_millis: u64,
    pub(crate) reason: SensorReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeLifecycleState {
    Starting,
    Ready,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SupervisorAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckpointCategory {
    Startup,
    ServiceLoop,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeHealthCategory {
    Healthy,
    Stale,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WatchdogParticipation {
    Participating,
    NotParticipating,
    Unproved,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeHealthFacts {
    pub(crate) lifecycle_state: RuntimeLifecycleState,
    pub(crate) supervisor_availability: SupervisorAvailability,
    pub(crate) checkpoint_category: CheckpointCategory,
    pub(crate) checkpoint_sequence: u64,
    pub(crate) checkpoint_age_millis: u64,
    pub(crate) health_category: RuntimeHealthCategory,
    pub(crate) watchdog_participation: WatchdogParticipation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProvenanceJoinFacts {
    pub(crate) boot_session_digest: String,
    pub(crate) operator_snapshot_revision: u64,
    pub(crate) sensor_snapshot_joined: bool,
    pub(crate) runtime_health_snapshot_joined: bool,
    pub(crate) api_websocket_retained_joined: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeIdentityObservationSource {
    DeviceSessionReplay,
    TerminalResultProjection,
    PackageDerived,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeIdentityFacts {
    pub(crate) observation_source: RuntimeIdentityObservationSource,
    pub(crate) same_physical_device: bool,
    pub(crate) source_commit_observed: bool,
    pub(crate) reference_commit_observed: bool,
    pub(crate) application_elf_observed: bool,
    pub(crate) exact_package_joined: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EffectObservationSource {
    IndependentLedger,
    SupervisorAuthored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EffectIntervalState {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndependentEffectFacts {
    pub(crate) observation_source: EffectObservationSource,
    pub(crate) interval_state: EffectIntervalState,
    pub(crate) all_effect_paths_covered: bool,
    pub(crate) prohibited_effect_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase36ClaimDigests {
    pub(crate) snapshot_substance: String,
    pub(crate) runtime_health: String,
    pub(crate) runtime_identity: String,
    pub(crate) independent_no_actuation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShareablePhase36FactsV1 {
    pub(crate) schema_version: String,
    pub(crate) power: PowerSensorFacts,
    pub(crate) temperature: TemperatureSensorFacts,
    pub(crate) tachometer: TachometerSensorFacts,
    pub(crate) runtime_health: RuntimeHealthFacts,
    pub(crate) provenance_join: ProvenanceJoinFacts,
    pub(crate) runtime_identity: RuntimeIdentityFacts,
    pub(crate) independent_effects: IndependentEffectFacts,
    pub(crate) claim_digests: Phase36ClaimDigests,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase36EvidenceEnvelope {
    pub(crate) schema_version: String,
    pub(crate) phase35_root_reference: Phase35RootReference,
    pub(crate) evaluation_identity: Phase36EvaluationIdentity,
    pub(crate) immutable_artifacts: Vec<ImmutableArtifactReference>,
    pub(crate) attempt31_sufficiency: Attempt31Sufficiency,
    pub(crate) shareable_facts: ShareablePhase36FactsV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase36Classification {
    pub(crate) schema_version: &'static str,
    pub(crate) phase35_root_reference: Phase35RootReference,
    pub(crate) evaluation_identity: Phase36EvaluationIdentity,
    pub(crate) immutable_artifact_assessment: ImmutableArtifactAssessment,
    pub(crate) shareable_facts: ShareablePhase36FactsV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum Phase36EvidenceError {
    #[error("unsupported_schema")]
    UnsupportedSchema,
    #[error("invalid_digest")]
    InvalidDigest,
    #[error("phase35_root_reference_mismatch")]
    Phase35RootReferenceMismatch,
    #[error("phase35_generation_reference_mismatch")]
    Phase35GenerationReferenceMismatch,
    #[error("evaluator_identity_mismatch")]
    EvaluatorIdentityMismatch,
    #[error("missing_artifact_role")]
    MissingArtifactRole,
    #[error("extra_artifact_role")]
    ExtraArtifactRole,
    #[error("duplicate_artifact_role")]
    DuplicateArtifactRole,
    #[error("unsafe_artifact_path")]
    UnsafeArtifactPath,
    #[error("mixed_evidence_source_commits")]
    MixedEvidenceSourceCommits,
    #[error("contradictory_sensor_state")]
    ContradictorySensorState,
    #[error("contradictory_runtime_health_state")]
    ContradictoryRuntimeHealthState,
    #[error("missing_provenance_join")]
    MissingProvenanceJoin,
    #[error("partial_public_output")]
    PartialPublicOutput,
    #[error("sufficiency_result_mismatch")]
    SufficiencyResultMismatch,
    #[error("protected_root_invalid")]
    ProtectedRootInvalid,
    #[error("protected_root_symlink")]
    ProtectedRootSymlink,
    #[error("wrong_permissions")]
    WrongPermissions,
    #[error("protected_input_missing")]
    ProtectedInputMissing,
    #[error("protected_input_symlink")]
    ProtectedInputSymlink,
    #[error("protected_input_changed")]
    ProtectedInputChanged,
    #[error("immutable_artifact_digest_mismatch")]
    ArtifactDigestMismatch,
    #[error("immutable_artifact_invalid")]
    ArtifactInvalid,
}
