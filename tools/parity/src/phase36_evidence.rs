//! Versioned, read-only successor classification for immutable Phase 35 evidence.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use camino::Utf8Path;
use serde::Serialize;

mod contract;
pub mod effects;
pub mod runtime_identity;
pub mod substance;

use crate::phase35_evidence::sha256_hex;

pub use contract::ComponentInsufficiency;
pub(crate) use contract::{
    Attempt31Sufficiency, EffectIntervalState, EffectObservationSource,
    ImmutableArtifactAssessment, Phase36ArtifactRole, Phase36ClaimDigests, Phase36Classification,
    Phase36EvidenceEnvelope, Phase36EvidenceError, PowerSensorFacts, ProvenanceJoinFacts,
    RuntimeHealthCategory, RuntimeHealthFacts, RuntimeIdentityObservationSource, SensorReason,
    SensorTruthState, ShareablePhase36FactsV1, SufficiencyResult, PHASE36_CONTRACT_DIGEST,
    PHASE36_INPUT_DOCUMENT, PHASE36_SCHEMA, SHAREABLE_PHASE36_FACTS_SCHEMA,
};
pub use substance::{
    validate_substantive_snapshot_documents, ObservationState, SubstantiveEvidenceAdmission,
    SubstantiveSnapshotJoin, ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};

pub(crate) fn classify_phase36_envelope(
    envelope: &Phase36EvidenceEnvelope,
) -> Result<Phase36Classification, Phase36EvidenceError> {
    validate_identity(envelope)?;
    validate_artifact_references(envelope)?;
    validate_shareable_facts(&envelope.shareable_facts)?;
    let expected_sufficiency = derive_sufficiency(&envelope.shareable_facts);
    if envelope.attempt31_sufficiency != expected_sufficiency {
        return Err(Phase36EvidenceError::SufficiencyResultMismatch);
    }
    let immutable_artifact_assessment =
        ImmutableArtifactAssessment::from_sufficiency(&expected_sufficiency);
    Ok(Phase36Classification {
        schema_version: PHASE36_SCHEMA,
        phase35_root_reference: envelope.phase35_root_reference.clone(),
        evaluation_identity: envelope.evaluation_identity.clone(),
        immutable_artifact_assessment,
        shareable_facts: envelope.shareable_facts.clone(),
    })
}

pub(crate) fn load_and_classify_phase36_root(
    protected_root: &Utf8Path,
) -> Result<Phase36Classification, Phase36EvidenceError> {
    validate_protected_root(protected_root)?;
    let input_path = protected_root.join(PHASE36_INPUT_DOCUMENT);
    let metadata = fs::symlink_metadata(input_path.as_std_path()).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Phase36EvidenceError::ProtectedInputMissing
        } else {
            Phase36EvidenceError::ProtectedRootInvalid
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(Phase36EvidenceError::ProtectedInputSymlink);
    }
    if !metadata.is_file() {
        return Err(Phase36EvidenceError::ProtectedRootInvalid);
    }
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(Phase36EvidenceError::WrongPermissions);
    }
    let bytes = fs::read(input_path.as_std_path())
        .map_err(|_| Phase36EvidenceError::ProtectedRootInvalid)?;
    let envelope = serde_json::from_slice::<Phase36EvidenceEnvelope>(&bytes)
        .map_err(|_| Phase36EvidenceError::PartialPublicOutput)?;
    classify_phase36_envelope(&envelope)
}

pub(crate) fn computed_claim_digests(
    facts: &ShareablePhase36FactsV1,
) -> Result<Phase36ClaimDigests, Phase36EvidenceError> {
    Ok(Phase36ClaimDigests {
        snapshot_substance: digest_serializable(&(
            &facts.power,
            &facts.temperature,
            &facts.tachometer,
            &facts.provenance_join,
        ))?,
        runtime_health: digest_serializable(&(&facts.runtime_health, &facts.provenance_join))?,
        runtime_identity: digest_serializable(&facts.runtime_identity)?,
        independent_no_actuation: digest_serializable(&facts.independent_effects)?,
    })
}

fn validate_identity(envelope: &Phase36EvidenceEnvelope) -> Result<(), Phase36EvidenceError> {
    if envelope.schema_version != PHASE36_SCHEMA
        || envelope.shareable_facts.schema_version != SHAREABLE_PHASE36_FACTS_SCHEMA
    {
        return Err(Phase36EvidenceError::UnsupportedSchema);
    }
    let root = &envelope.phase35_root_reference;
    if !is_lower_hex(&root.root_digest, 64)
        || !is_lower_hex(&root.phase35_generation_digest, 64)
        || !is_lower_hex(&root.evidence_source_commit, 40)
        || !is_lower_hex(&envelope.evaluation_identity.evaluator_commit, 40)
    {
        return Err(Phase36EvidenceError::InvalidDigest);
    }
    if envelope.evaluation_identity.successor_contract_digest != PHASE36_CONTRACT_DIGEST {
        return Err(Phase36EvidenceError::EvaluatorIdentityMismatch);
    }
    Ok(())
}

fn validate_artifact_references(
    envelope: &Phase36EvidenceEnvelope,
) -> Result<(), Phase36EvidenceError> {
    if envelope.immutable_artifacts.len() < Phase36ArtifactRole::ORDERED.len() {
        return Err(Phase36EvidenceError::MissingArtifactRole);
    }
    if envelope.immutable_artifacts.len() > Phase36ArtifactRole::ORDERED.len() {
        return Err(Phase36EvidenceError::ExtraArtifactRole);
    }
    for (index, artifact) in envelope.immutable_artifacts.iter().enumerate() {
        if artifact.role != Phase36ArtifactRole::ORDERED[index] {
            if envelope
                .immutable_artifacts
                .iter()
                .filter(|candidate| candidate.role == artifact.role)
                .count()
                > 1
            {
                return Err(Phase36EvidenceError::DuplicateArtifactRole);
            }
            return Err(Phase36EvidenceError::MissingArtifactRole);
        }
        validate_relative_path(&artifact.relative_path)?;
        if !is_lower_hex(&artifact.sha256, 64)
            || !is_lower_hex(&artifact.evidence_source_commit, 40)
        {
            return Err(Phase36EvidenceError::InvalidDigest);
        }
        if artifact.evidence_source_commit != envelope.phase35_root_reference.evidence_source_commit
        {
            return Err(Phase36EvidenceError::MixedEvidenceSourceCommits);
        }
    }
    let root_reference = &envelope.immutable_artifacts[0];
    if root_reference.sha256 != envelope.phase35_root_reference.root_digest {
        return Err(Phase36EvidenceError::Phase35RootReferenceMismatch);
    }
    let generation_reference = &envelope.immutable_artifacts[1];
    if generation_reference.sha256 != envelope.phase35_root_reference.phase35_generation_digest {
        return Err(Phase36EvidenceError::Phase35GenerationReferenceMismatch);
    }
    Ok(())
}

fn validate_shareable_facts(facts: &ShareablePhase36FactsV1) -> Result<(), Phase36EvidenceError> {
    validate_power(&facts.power)?;
    validate_scalar_sensor(
        facts.temperature.state,
        facts.temperature.maybe_millicelsius.is_some(),
        facts.temperature.producer_sequence,
        facts.temperature.acquisition_millis,
        facts.temperature.reason,
    )?;
    validate_scalar_sensor(
        facts.tachometer.state,
        facts.tachometer.maybe_rpm.is_some(),
        facts.tachometer.producer_sequence,
        facts.tachometer.acquisition_millis,
        facts.tachometer.reason,
    )?;
    validate_runtime_health(&facts.runtime_health)?;
    validate_provenance_join(&facts.provenance_join)?;
    let expected_digests = computed_claim_digests(facts)?;
    if facts.claim_digests != expected_digests {
        return Err(Phase36EvidenceError::PartialPublicOutput);
    }
    Ok(())
}

fn validate_power(power: &PowerSensorFacts) -> Result<(), Phase36EvidenceError> {
    let all_values = power.maybe_current_milliamps.is_some()
        && power.maybe_bus_millivolts.is_some()
        && power.maybe_power_milliwatts.is_some();
    let no_values = power.maybe_current_milliamps.is_none()
        && power.maybe_bus_millivolts.is_none()
        && power.maybe_power_milliwatts.is_none();
    validate_sensor_state(
        power.state,
        all_values,
        no_values,
        power.producer_sequence,
        power.acquisition_millis,
        power.reason,
    )
}

fn validate_scalar_sensor(
    state: SensorTruthState,
    has_value: bool,
    producer_sequence: u64,
    acquisition_millis: u64,
    reason: SensorReason,
) -> Result<(), Phase36EvidenceError> {
    validate_sensor_state(
        state,
        has_value,
        !has_value,
        producer_sequence,
        acquisition_millis,
        reason,
    )
}

fn validate_sensor_state(
    state: SensorTruthState,
    all_values: bool,
    no_values: bool,
    producer_sequence: u64,
    acquisition_millis: u64,
    reason: SensorReason,
) -> Result<(), Phase36EvidenceError> {
    let legal = match state {
        SensorTruthState::Fresh => {
            all_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::None
        }
        SensorTruthState::Stale => {
            all_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::ObservationExpired
        }
        SensorTruthState::Unavailable => {
            no_values
                && producer_sequence == 0
                && acquisition_millis == 0
                && reason == SensorReason::NeverObserved
        }
        SensorTruthState::Fault => {
            no_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::AcquisitionFailed
        }
    };
    if legal {
        return Ok(());
    }
    Err(Phase36EvidenceError::ContradictorySensorState)
}

fn validate_runtime_health(health: &RuntimeHealthFacts) -> Result<(), Phase36EvidenceError> {
    let legal = match health.health_category {
        RuntimeHealthCategory::Healthy => {
            health.supervisor_availability == contract::SupervisorAvailability::Available
                && health.checkpoint_sequence > 0
                && health.checkpoint_age_millis <= 5_000
        }
        RuntimeHealthCategory::Stale => {
            health.supervisor_availability == contract::SupervisorAvailability::Available
                && health.checkpoint_sequence > 0
                && health.checkpoint_age_millis > 5_000
        }
        RuntimeHealthCategory::Unavailable => {
            health.supervisor_availability == contract::SupervisorAvailability::Unavailable
                && health.checkpoint_sequence == 0
        }
    };
    if legal {
        return Ok(());
    }
    Err(Phase36EvidenceError::ContradictoryRuntimeHealthState)
}

fn validate_provenance_join(provenance: &ProvenanceJoinFacts) -> Result<(), Phase36EvidenceError> {
    if is_lower_hex(&provenance.boot_session_digest, 64)
        && provenance.operator_snapshot_revision > 0
    {
        return Ok(());
    }
    Err(Phase36EvidenceError::MissingProvenanceJoin)
}

fn derive_sufficiency(facts: &ShareablePhase36FactsV1) -> Attempt31Sufficiency {
    let snapshot_sufficient = facts.provenance_join.sensor_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    let health_sufficient = facts.provenance_join.runtime_health_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    let identity_sufficient = facts.runtime_identity.observation_source
        != RuntimeIdentityObservationSource::PackageDerived
        && facts.runtime_identity.same_physical_device
        && facts.runtime_identity.source_commit_observed
        && facts.runtime_identity.reference_commit_observed
        && facts.runtime_identity.application_elf_observed
        && facts.runtime_identity.exact_package_joined;
    let effects_sufficient = facts.independent_effects.observation_source
        == EffectObservationSource::IndependentLedger
        && facts.independent_effects.interval_state == EffectIntervalState::Complete
        && facts.independent_effects.all_effect_paths_covered
        && !facts.independent_effects.prohibited_effect_observed;
    Attempt31Sufficiency {
        snapshot_substance: sufficiency(
            snapshot_sufficient,
            ComponentInsufficiency::SnapshotSubstance,
        ),
        runtime_health: sufficiency(health_sufficient, ComponentInsufficiency::RuntimeHealth),
        runtime_identity_observation: sufficiency(
            identity_sufficient,
            ComponentInsufficiency::RuntimeIdentityObservation,
        ),
        independent_effect_observation: sufficiency(
            effects_sufficient,
            ComponentInsufficiency::IndependentEffectObservation,
        ),
    }
}

fn sufficiency(sufficient: bool, category: ComponentInsufficiency) -> SufficiencyResult {
    if sufficient {
        SufficiencyResult::Sufficient
    } else {
        SufficiencyResult::Insufficient { category }
    }
}

fn validate_protected_root(protected_root: &Utf8Path) -> Result<(), Phase36EvidenceError> {
    let metadata = fs::symlink_metadata(protected_root.as_std_path())
        .map_err(|_| Phase36EvidenceError::ProtectedRootInvalid)?;
    if metadata.file_type().is_symlink() {
        return Err(Phase36EvidenceError::ProtectedRootSymlink);
    }
    if !metadata.is_dir() {
        return Err(Phase36EvidenceError::ProtectedRootInvalid);
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(Phase36EvidenceError::WrongPermissions);
    }
    Ok(())
}

fn validate_relative_path(relative_path: &str) -> Result<(), Phase36EvidenceError> {
    if relative_path.is_empty()
        || relative_path.starts_with('/')
        || relative_path.starts_with('\\')
        || relative_path.contains('\\')
        || relative_path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
        || Utf8Path::new(relative_path).is_absolute()
    {
        return Err(Phase36EvidenceError::UnsafeArtifactPath);
    }
    Ok(())
}

fn digest_serializable(value: &impl Serialize) -> Result<String, Phase36EvidenceError> {
    let bytes = serde_json::to_vec(value).map_err(|_| Phase36EvidenceError::PartialPublicOutput)?;
    Ok(sha256_hex(&bytes))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests;
