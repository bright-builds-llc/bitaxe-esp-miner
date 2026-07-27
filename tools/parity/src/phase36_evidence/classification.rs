use super::*;

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

pub(super) fn load_and_classify_with_authority(
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
