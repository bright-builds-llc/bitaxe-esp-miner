use super::classification::classify_phase36_envelope;
use super::*;

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
