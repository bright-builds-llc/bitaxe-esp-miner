//! Versioned, read-only successor classification for immutable Phase 35 evidence.

use std::collections::BTreeMap;

use camino::Utf8Path;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;

mod contract;
pub mod effects;
pub mod runtime_identity;
pub mod substance;

use crate::phase35_evidence::{
    sha256_hex, validate_phase35_evidence, InventoryArtifact, Phase35EvidenceRootInput,
};
use crate::phase36_evidence::effects::{
    classify_independent_effect_document, IndependentEffectAdmission,
    IndependentEffectObservationSource,
};
use crate::phase36_evidence::runtime_identity::{
    validate_observed_runtime_identity_documents, ObservedRuntimeIdentityAdmission,
};
use crate::phase36_promotion::ValidatedHostnameDurabilityFacts;
use crate::protected_input::{ProtectedFile, ProtectedInputError, ProtectedRoot};

pub use contract::ComponentInsufficiency;
#[cfg(test)]
pub(crate) use contract::Phase36ClaimDigests;
pub(crate) use contract::{
    Attempt31Sufficiency, EffectIntervalState, EffectObservationSource,
    ImmutableArtifactAssessment, Phase36ArtifactRole, Phase36Classification,
    Phase36EvidenceEnvelope, Phase36EvidenceError, PowerSensorFacts, ProvenanceJoinFacts,
    RuntimeHealthCategory, RuntimeHealthFacts, RuntimeIdentityObservationSource, SensorReason,
    SensorTruthState, ShareablePhase36FactsV1, SufficiencyResult, PHASE36_INPUT_DOCUMENT,
    PHASE36_SCHEMA, SHAREABLE_PHASE36_FACTS_SCHEMA,
};
pub use substance::{
    validate_substantive_snapshot_components, validate_substantive_snapshot_documents,
    ObservationState, SubstantiveEvidenceAdmission, SubstantiveSnapshotJoin,
    ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
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
    let authority = Phase36Authority::production()?;
    load_and_classify_with_authority(protected_root, &authority)
}

fn load_and_classify_with_authority(
    protected_root: &Utf8Path,
    authority: &Phase36Authority,
) -> Result<Phase36Classification, Phase36EvidenceError> {
    let root = ProtectedRoot::open(protected_root).map_err(map_protected_error)?;
    let envelope_file = root
        .open_file(Utf8Path::new(PHASE36_INPUT_DOCUMENT))
        .map_err(map_protected_error)?;
    let envelope = serde_json::from_slice::<Phase36EvidenceEnvelope>(envelope_file.bytes())
        .map_err(|_| Phase36EvidenceError::PartialPublicOutput)?;
    let classification = classify_phase36_envelope(&envelope)?;
    let artifacts = authenticate_artifact_graph(&root, &envelope, authority)?;
    artifacts.verify_unchanged()?;
    envelope_file
        .verify_unchanged()
        .map_err(map_protected_error)?;
    Ok(classification)
}

#[cfg(test)]
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
        || !is_lower_hex(&envelope.evaluation_identity.evaluator_digest, 64)
    {
        return Err(Phase36EvidenceError::InvalidDigest);
    }
    if envelope.evaluation_identity.evaluator_digest != current_phase36_evidence_evaluator_digest()
        || envelope.evaluation_identity.successor_contract_digest
            != current_phase36_evidence_contract_digest()
    {
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
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubstantiveArtifact {
    schema_version: String,
    api_document: String,
    websocket_document: String,
    retained_document: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeIdentityArtifact {
    schema_version: String,
    exact_package_document: String,
    request_document: String,
    event_ledger_document: String,
    private_result_document: String,
    public_projection_document: String,
}

struct Phase36Authority {
    phase35_root_digest: String,
    phase35_generation_digest: String,
    maybe_role_digests: Option<[String; 4]>,
}

impl Phase36Authority {
    fn production() -> Result<Self, Phase36EvidenceError> {
        const GENERATION: &str = include_str!(
            "../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json"
        );
        let manifest: serde_json::Value =
            serde_json::from_str(GENERATION).map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
        let phase35_root_digest = manifest
            .get("root_digest")
            .and_then(serde_json::Value::as_str)
            .filter(|digest| is_lower_hex(digest, 64))
            .ok_or(Phase36EvidenceError::ArtifactInvalid)?
            .to_owned();
        Ok(Self {
            phase35_root_digest,
            phase35_generation_digest: sha256_hex(GENERATION.as_bytes()),
            maybe_role_digests: None,
        })
    }

    #[cfg(test)]
    fn synthetic(
        phase35_root_digest: String,
        phase35_generation_digest: String,
        role_digests: [String; 4],
    ) -> Self {
        Self {
            phase35_root_digest,
            phase35_generation_digest,
            maybe_role_digests: Some(role_digests),
        }
    }
}

struct AuthenticatedArtifactGraph {
    files: Vec<ProtectedFile>,
}

impl AuthenticatedArtifactGraph {
    fn verify_unchanged(&self) -> Result<(), Phase36EvidenceError> {
        for file in &self.files {
            file.verify_unchanged().map_err(map_protected_error)?;
        }
        Ok(())
    }
}

fn authenticate_artifact_graph(
    root: &ProtectedRoot,
    envelope: &Phase36EvidenceEnvelope,
    authority: &Phase36Authority,
) -> Result<AuthenticatedArtifactGraph, Phase36EvidenceError> {
    let mut files = Vec::new();
    for reference in &envelope.immutable_artifacts {
        let file = root
            .open_file(Utf8Path::new(&reference.relative_path))
            .map_err(map_protected_error)?;
        if file.digest() != reference.sha256 {
            return Err(Phase36EvidenceError::ArtifactDigestMismatch);
        }
        files.push(file);
    }

    let phase35_input = serde_json::from_slice::<Phase35EvidenceRootInput>(files[0].bytes())
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    let phase35_parent = Utf8Path::new(&envelope.immutable_artifacts[0].relative_path)
        .parent()
        .ok_or(Phase36EvidenceError::UnsafeArtifactPath)?;
    let mut inventory = BTreeMap::new();
    for entry in &phase35_input.inventory {
        validate_relative_path(&entry.path)?;
        let relative = phase35_parent.join(&entry.path);
        let file = root.open_file(&relative).map_err(map_protected_error)?;
        inventory.insert(
            entry.path.clone(),
            InventoryArtifact::regular(file.bytes().to_vec()),
        );
        files.push(file);
    }
    let validated_phase35 = validate_phase35_evidence(&phase35_input, &inventory)
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if validated_phase35.root_digest() != envelope.phase35_root_reference.root_digest {
        return Err(Phase36EvidenceError::Phase35RootReferenceMismatch);
    }
    if phase35_input.exact_package.source_commit
        != envelope.phase35_root_reference.evidence_source_commit
    {
        return Err(Phase36EvidenceError::MixedEvidenceSourceCommits);
    }
    if validated_phase35.root_digest() != authority.phase35_root_digest {
        return Err(Phase36EvidenceError::Phase35RootReferenceMismatch);
    }
    if files[1].digest() != authority.phase35_generation_digest {
        return Err(Phase36EvidenceError::Phase35GenerationReferenceMismatch);
    }

    validate_phase35_generation(root, envelope, &mut files)?;
    validate_substantive_roles(envelope, &files[2], &files[3])?;
    validate_runtime_identity_role(envelope, &files[4])?;
    validate_effect_role(envelope, &files[5])?;
    validate_role_authority(envelope, authority)?;

    Ok(AuthenticatedArtifactGraph { files })
}

fn validate_phase35_generation(
    root: &ProtectedRoot,
    envelope: &Phase36EvidenceEnvelope,
    files: &mut Vec<ProtectedFile>,
) -> Result<(), Phase36EvidenceError> {
    if files[1].digest() != envelope.phase35_root_reference.phase35_generation_digest {
        return Err(Phase36EvidenceError::Phase35GenerationReferenceMismatch);
    }
    let parent = Utf8Path::new(&envelope.immutable_artifacts[1].relative_path)
        .parent()
        .ok_or(Phase36EvidenceError::UnsafeArtifactPath)?;
    let projection = root
        .open_file(&parent.join("projection.json"))
        .map_err(map_protected_error)?;
    let matrix = root
        .open_file(&parent.join("decision-matrix.json"))
        .map_err(map_protected_error)?;
    let verdict = root
        .open_file(&parent.join("admitted.json"))
        .map_err(map_protected_error)?;
    let checklist = root
        .open_file(&parent.join("checklist.md"))
        .map_err(map_protected_error)?;
    let manifest_text = files[1].text().map_err(map_protected_error)?;
    let manifest: serde_json::Value =
        serde_json::from_str(manifest_text).map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if manifest
        .get("checklist_sha256")
        .and_then(serde_json::Value::as_str)
        != Some(checklist.digest())
    {
        return Err(Phase36EvidenceError::ArtifactDigestMismatch);
    }
    let hostname = ValidatedHostnameDurabilityFacts::from_public_generation(
        manifest_text,
        projection.text().map_err(map_protected_error)?,
        matrix.text().map_err(map_protected_error)?,
        verdict.text().map_err(map_protected_error)?,
    )
    .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if hostname.phase35_root_digest != envelope.phase35_root_reference.root_digest {
        return Err(Phase36EvidenceError::Phase35GenerationReferenceMismatch);
    }
    files.extend([projection, matrix, verdict, checklist]);
    Ok(())
}

fn validate_role_authority(
    envelope: &Phase36EvidenceEnvelope,
    authority: &Phase36Authority,
) -> Result<(), Phase36EvidenceError> {
    let role_digests = authority
        .maybe_role_digests
        .as_ref()
        .ok_or(Phase36EvidenceError::ArtifactInvalid)?;
    for (reference, digest) in envelope.immutable_artifacts[2..].iter().zip(role_digests) {
        if reference.sha256 != *digest {
            return Err(Phase36EvidenceError::ArtifactInvalid);
        }
    }
    Ok(())
}

fn validate_substantive_roles(
    envelope: &Phase36EvidenceEnvelope,
    snapshot_file: &ProtectedFile,
    health_file: &ProtectedFile,
) -> Result<(), Phase36EvidenceError> {
    let snapshot: SubstantiveArtifact = serde_json::from_slice(snapshot_file.bytes())
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if snapshot.schema_version != "phase36-snapshot-substance-artifact-v1" {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    }
    let snapshot_components = validate_substantive_snapshot_components(
        &snapshot.api_document,
        &snapshot.websocket_document,
        &snapshot.retained_document,
    )
    .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    let sensors = snapshot_components
        .maybe_sensors
        .ok_or(Phase36EvidenceError::ArtifactInvalid)?;
    validate_sensor_projection(
        &envelope.shareable_facts,
        &sensors,
        &snapshot_components.join,
    )?;
    if envelope.shareable_facts.claim_digests.snapshot_substance != sensors.claim_fact_digest {
        return Err(Phase36EvidenceError::PartialPublicOutput);
    }

    let health: SubstantiveArtifact = serde_json::from_slice(health_file.bytes())
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if health.schema_version != "phase36-runtime-health-artifact-v1" {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    }
    let health_components = validate_substantive_snapshot_components(
        &health.api_document,
        &health.websocket_document,
        &health.retained_document,
    )
    .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    let runtime_health = health_components
        .maybe_runtime_health
        .ok_or(Phase36EvidenceError::ArtifactInvalid)?;
    validate_health_projection(
        &envelope.shareable_facts,
        &runtime_health,
        &health_components.join,
    )?;
    if envelope.shareable_facts.claim_digests.runtime_health != runtime_health.claim_fact_digest {
        return Err(Phase36EvidenceError::PartialPublicOutput);
    }
    Ok(())
}

fn validate_runtime_identity_role(
    envelope: &Phase36EvidenceEnvelope,
    runtime_identity_file: &ProtectedFile,
) -> Result<(), Phase36EvidenceError> {
    let artifact: RuntimeIdentityArtifact = serde_json::from_slice(runtime_identity_file.bytes())
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if artifact.schema_version != "phase36-runtime-identity-artifact-v1" {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    }
    let package_value: serde_json::Value = serde_json::from_str(&artifact.exact_package_document)
        .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    if package_value
        .get("source_commit")
        .and_then(serde_json::Value::as_str)
        != Some(&envelope.phase35_root_reference.evidence_source_commit)
    {
        return Err(Phase36EvidenceError::MixedEvidenceSourceCommits);
    }
    let admission = validate_observed_runtime_identity_documents(
        &artifact.exact_package_document,
        Some(&artifact.request_document),
        Some(&artifact.event_ledger_document),
        Some(&artifact.private_result_document),
        Some(&artifact.public_projection_document),
    )
    .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    let ObservedRuntimeIdentityAdmission::Validated { identity } = admission else {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    };
    let facts = &envelope.shareable_facts.runtime_identity;
    if identity.observation_source
        != runtime_identity::RuntimeIdentityObservationSource::DeviceSessionReplay
        || !identity.same_physical_device
        || facts.observation_source != RuntimeIdentityObservationSource::DeviceSessionReplay
        || !facts.same_physical_device
        || !facts.source_commit_observed
        || !facts.reference_commit_observed
        || !facts.application_elf_observed
        || !facts.exact_package_joined
    {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    }
    if envelope.shareable_facts.claim_digests.runtime_identity != identity.claim_fact_digest {
        return Err(Phase36EvidenceError::PartialPublicOutput);
    }
    Ok(())
}

fn validate_effect_role(
    envelope: &Phase36EvidenceEnvelope,
    effect_file: &ProtectedFile,
) -> Result<(), Phase36EvidenceError> {
    let admission = classify_independent_effect_document(
        Some(effect_file.text().map_err(map_protected_error)?),
        None,
    )
    .map_err(|_| Phase36EvidenceError::ArtifactInvalid)?;
    let IndependentEffectAdmission::Validated { interval } = admission else {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    };
    let facts = &envelope.shareable_facts.independent_effects;
    if interval.observation_source != IndependentEffectObservationSource::IndependentLedger
        || facts.observation_source != EffectObservationSource::IndependentLedger
        || facts.interval_state != EffectIntervalState::Complete
        || !facts.all_effect_paths_covered
        || facts.prohibited_effect_observed
    {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    }
    if envelope
        .shareable_facts
        .claim_digests
        .independent_no_actuation
        != interval.claim_fact_digest
    {
        return Err(Phase36EvidenceError::PartialPublicOutput);
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
    if [
        &facts.claim_digests.snapshot_substance,
        &facts.claim_digests.runtime_health,
        &facts.claim_digests.runtime_identity,
        &facts.claim_digests.independent_no_actuation,
    ]
    .into_iter()
    .any(|digest| !is_lower_hex(digest, 64))
    {
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

fn validate_sensor_projection(
    facts: &ShareablePhase36FactsV1,
    sensors: &ValidatedSensorSubstance,
    join: &SubstantiveSnapshotJoin,
) -> Result<(), Phase36EvidenceError> {
    let power_stamp = fresh_stamp(&sensors.power.state)?;
    let temperature_stamp = fresh_stamp(&sensors.temperature.state)?;
    let tachometer_stamp = fresh_stamp(&sensors.tachometer.state)?;
    let tachometer_milliunits = sensors
        .tachometer
        .maybe_value_milliunits
        .filter(|value| value % 1_000 == 0)
        .ok_or(Phase36EvidenceError::ArtifactInvalid)?;
    let matches = facts.power.state == SensorTruthState::Fresh
        && facts.power.maybe_current_milliamps == sensors.power.maybe_current_milliamps
        && facts.power.maybe_bus_millivolts == sensors.power.maybe_bus_millivolts
        && facts.power.maybe_power_milliwatts == sensors.power.maybe_power_milliwatts
        && facts.power.producer_sequence == power_stamp.sequence
        && facts.power.acquisition_millis == power_stamp.acquired_at_ms
        && facts.temperature.state == SensorTruthState::Fresh
        && facts.temperature.maybe_millicelsius == sensors.temperature.maybe_value_milliunits
        && facts.temperature.producer_sequence == temperature_stamp.sequence
        && facts.temperature.acquisition_millis == temperature_stamp.acquired_at_ms
        && facts.tachometer.state == SensorTruthState::Fresh
        && facts.tachometer.maybe_rpm == u64::try_from(tachometer_milliunits / 1_000).ok()
        && facts.tachometer.producer_sequence == tachometer_stamp.sequence
        && facts.tachometer.acquisition_millis == tachometer_stamp.acquired_at_ms
        && facts.provenance_join.boot_session_digest == join.operator_boot_session_digest
        && facts.provenance_join.operator_snapshot_revision == join.operator_snapshot_revision
        && facts.provenance_join.sensor_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    if matches {
        Ok(())
    } else {
        Err(Phase36EvidenceError::ArtifactInvalid)
    }
}

fn fresh_stamp(
    state: &ObservationState,
) -> Result<&substance::ObservationStamp, Phase36EvidenceError> {
    let ObservationState::Fresh { stamp } = state else {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    };
    Ok(stamp)
}

fn validate_health_projection(
    facts: &ShareablePhase36FactsV1,
    health: &ValidatedRuntimeHealthSubstance,
    join: &SubstantiveSnapshotJoin,
) -> Result<(), Phase36EvidenceError> {
    let lifecycle_matches = matches!(
        health.lifecycle_state,
        substance::RuntimeLifecycleState::Idle | substance::RuntimeLifecycleState::Passed
    ) && facts.runtime_health.lifecycle_state
        == contract::RuntimeLifecycleState::Ready;
    let checkpoint_matches = health
        .maybe_checkpoint_category
        .as_ref()
        .is_some_and(|category| category.as_str() == "telemetry")
        && facts.runtime_health.checkpoint_category == contract::CheckpointCategory::ServiceLoop;
    let matches = lifecycle_matches
        && checkpoint_matches
        && health.supervisor_availability == substance::SupervisorAvailability::Available
        && facts.runtime_health.supervisor_availability
            == contract::SupervisorAvailability::Available
        && health.checkpoint_health == substance::CheckpointHealth::Healthy
        && facts.runtime_health.health_category == RuntimeHealthCategory::Healthy
        && health.maybe_checkpoint_sequence == Some(facts.runtime_health.checkpoint_sequence)
        && health.maybe_checkpoint_age_millis == Some(facts.runtime_health.checkpoint_age_millis)
        && facts.runtime_health.watchdog_participation == contract::WatchdogParticipation::Unproved
        && facts.provenance_join.boot_session_digest == join.operator_boot_session_digest
        && facts.provenance_join.operator_snapshot_revision == join.operator_snapshot_revision
        && facts.provenance_join.runtime_health_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    if matches {
        Ok(())
    } else {
        Err(Phase36EvidenceError::ArtifactInvalid)
    }
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

#[cfg(test)]
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

pub(crate) fn current_phase36_evidence_evaluator_digest() -> String {
    sha256_hex(
        [
            "phase36-evidence-evaluator-v1\0",
            include_str!("phase36_evidence.rs"),
            include_str!("phase36_evidence/substance.rs"),
            include_str!("phase36_evidence/substance/types.rs"),
            include_str!("phase36_evidence/runtime_identity.rs"),
            include_str!("phase36_evidence/runtime_identity/ledger.rs"),
            include_str!("phase36_evidence/effects.rs"),
            include_str!("phase35_evidence.rs"),
            include_str!("phase35_evidence/contract.rs"),
            include_str!("phase35_evidence/digests.rs"),
            include_str!("phase35_evidence/inventory.rs"),
            include_str!("phase35_evidence/projection.rs"),
            include_str!("protected_input.rs"),
        ]
        .concat()
        .as_bytes(),
    )
}

pub(crate) fn current_phase36_evidence_contract_digest() -> String {
    sha256_hex(
        [
            "phase36-evidence-contract-v1\0",
            include_str!("phase36_evidence/contract.rs"),
            &current_phase36_evidence_evaluator_digest(),
        ]
        .concat()
        .as_bytes(),
    )
}

fn map_protected_error(error: ProtectedInputError) -> Phase36EvidenceError {
    match error {
        ProtectedInputError::RootInvalid => Phase36EvidenceError::ProtectedRootInvalid,
        ProtectedInputError::RootSymlink => Phase36EvidenceError::ProtectedRootSymlink,
        ProtectedInputError::UnsafePath => Phase36EvidenceError::UnsafeArtifactPath,
        ProtectedInputError::Missing => Phase36EvidenceError::ProtectedInputMissing,
        ProtectedInputError::Symlink => Phase36EvidenceError::ProtectedInputSymlink,
        ProtectedInputError::WrongPermissions => Phase36EvidenceError::WrongPermissions,
        ProtectedInputError::Changed => Phase36EvidenceError::ProtectedInputChanged,
        ProtectedInputError::NotUtf8 => Phase36EvidenceError::ArtifactInvalid,
    }
}

#[cfg(test)]
pub(crate) mod tests;
