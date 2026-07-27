use super::*;

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

pub(super) struct Phase36Authority {
    pub(super) phase35_root_digest: String,
    pub(super) phase35_generation_digest: String,
    pub(super) maybe_role_digests: Option<[String; 4]>,
}

impl Phase36Authority {
    pub(super) fn production() -> Result<Self, Phase36EvidenceError> {
        const GENERATION: &str = include_str!(
            "../../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json"
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
    pub(super) fn synthetic(
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

pub(super) struct AuthenticatedArtifactGraph {
    files: Vec<ProtectedFile>,
}

impl AuthenticatedArtifactGraph {
    pub(super) fn verify_unchanged(&self) -> Result<(), Phase36EvidenceError> {
        for file in &self.files {
            file.verify_unchanged().map_err(map_protected_error)?;
        }
        Ok(())
    }
}

pub(super) fn authenticate_artifact_graph(
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
