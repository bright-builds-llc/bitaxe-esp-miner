use std::collections::BTreeSet;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use serde_json::Value;

use super::filesystem::{
    normalize_repo_relative, reject_symlink_managed_path, sync_directory, write_synced,
};
use super::{GenerationError, GenerationResult};
use crate::phase35_evidence::sha256_hex;
use crate::phase36_promotion::{
    current_phase36_evaluator_digest, evaluate_phase36_promotion, Phase36ChecklistSnapshot,
    Phase36ClaimPrerequisites, Phase36PromotionMatrix,
};

mod transaction;

use transaction::{
    create_private_staging, read_text, recover_derived_checklist, regular_file_inventory,
    remove_directory_if_present, transactional_exchange, validate_existing_destination,
};

const PROJECTION_FILE: &str = "typed-fact-projection.json";
const MATRIX_FILE: &str = "decision-matrix.json";
const VERDICT_FILE: &str = "verdict.json";
const MANIFEST_FILE: &str = "manifest.json";
const CHECKLIST_SNAPSHOT_FILE: &str = "checklist.md";
const PROJECTION_SCHEMA: &str = "phase36-typed-fact-projection-v1";
const VERDICT_SCHEMA: &str = "phase36-publication-verdict-v1";
const MANIFEST_SCHEMA: &str = "phase36-generation-v1";
const PRIOR_MANIFEST_SCHEMA: &str = "phase35-generation-v1";
const OWNED_FILES: [&str; 5] = [
    PROJECTION_FILE,
    MATRIX_FILE,
    VERDICT_FILE,
    CHECKLIST_SNAPSHOT_FILE,
    MANIFEST_FILE,
];
const LEGACY_OWNED_FILES: [&str; 4] = [PROJECTION_FILE, MATRIX_FILE, VERDICT_FILE, MANIFEST_FILE];

#[derive(Debug, Clone)]
pub(crate) struct Phase36GenerationDocuments {
    pub(crate) prerequisites: Phase36ClaimPrerequisites,
    pub(crate) matrix: Phase36PromotionMatrix,
}

impl Phase36GenerationDocuments {
    pub(crate) const fn new(
        prerequisites: Phase36ClaimPrerequisites,
        matrix: Phase36PromotionMatrix,
    ) -> Self {
        Self {
            prerequisites,
            matrix,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase36PublicationFailurePoint {
    BeforeStaging,
    AfterProjectionWrite,
    AfterMatrixWrite,
    AfterVerdictWrite,
    AfterChecklistSnapshotWrite,
    AfterManifestWrite,
    BeforeValidation,
    AfterValidation,
    AfterStagingSync,
    AfterChecklistReplacementWrite,
    BeforeGenerationExchange,
    AfterGenerationExchange,
    BeforeChecklistExchange,
    AfterChecklistExchange,
    #[allow(dead_code)] // Constructed only by rollback fault-injection tests.
    AfterParentSync,
}

impl Phase36PublicationFailurePoint {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 15] = [
        Self::BeforeStaging,
        Self::AfterProjectionWrite,
        Self::AfterMatrixWrite,
        Self::AfterVerdictWrite,
        Self::AfterChecklistSnapshotWrite,
        Self::AfterManifestWrite,
        Self::BeforeValidation,
        Self::AfterValidation,
        Self::AfterStagingSync,
        Self::AfterChecklistReplacementWrite,
        Self::BeforeGenerationExchange,
        Self::AfterGenerationExchange,
        Self::BeforeChecklistExchange,
        Self::AfterChecklistExchange,
        Self::AfterParentSync,
    ];
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Phase36PublicationOptions {
    pub(crate) maybe_failure: Option<Phase36PublicationFailurePoint>,
    #[cfg(test)]
    pub(crate) crash_after_authority_exchange: bool,
}

#[derive(Serialize)]
pub(super) struct TypedFactProjection<'a> {
    schema_version: &'static str,
    phase35_root_digest: &'a str,
    superseded_phase35_generation_digest: &'a str,
    evaluator_digest: &'a str,
    hostname_durability: &'a Option<crate::phase36_promotion::ValidatedHostnameDurabilityFacts>,
    sensor_substance: &'a Option<crate::phase36_evidence::ValidatedSensorSubstance>,
    snapshot_join: &'a Option<crate::phase36_evidence::SubstantiveSnapshotJoin>,
    runtime_health: &'a Option<crate::phase36_evidence::ValidatedRuntimeHealthSubstance>,
    runtime_identity:
        &'a Option<crate::phase36_evidence::runtime_identity::ValidatedObservedRuntimeIdentity>,
    independent_effect:
        &'a Option<crate::phase36_evidence::effects::ValidatedIndependentEffectInterval>,
}

#[derive(Serialize)]
struct PublicationVerdict<'a> {
    schema_version: &'static str,
    authoritative_generation_digest: &'a str,
    supported_rows: Vec<&'a str>,
    complete_matrix: bool,
}

#[derive(Serialize)]
struct GenerationManifest<'a> {
    schema_version: &'static str,
    phase35_root_digest: &'a str,
    superseded_phase35_generation_digest: &'a str,
    authoritative_phase36_generation_digest: &'a str,
    evaluator_digest: &'a str,
    checklist_fingerprint_before: &'a str,
    checklist_fingerprint_after: &'a str,
    projection_sha256: String,
    matrix_sha256: String,
    verdict_sha256: String,
    checklist_sha256: String,
}

pub(crate) fn publish_phase36_generation(
    workspace_root: &Utf8Path,
    staging_root: &Utf8Path,
    destination_root: &Utf8Path,
    checklist_path: &Utf8Path,
    prior_manifest_path: &Utf8Path,
    documents: &Phase36GenerationDocuments,
    options: Phase36PublicationOptions,
) -> GenerationResult<()> {
    let paths = PublicationPaths::resolve(
        workspace_root,
        staging_root,
        destination_root,
        checklist_path,
        prior_manifest_path,
    )?;
    recover_derived_checklist(&paths)?;
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::BeforeStaging) {
        return injected(Phase36PublicationFailurePoint::BeforeStaging);
    }
    validate_existing_destination(&paths.destination)?;
    let current_checklist = read_text(&paths.checklist, "checklist")?;
    let prior_manifest = read_text(&paths.prior_manifest, "Phase 35 manifest")?;
    let rendered = render_and_validate(documents, &current_checklist, &prior_manifest)?;
    create_private_staging(&paths.staging)?;
    if let Err(error) = stage_documents(&paths.staging, &rendered, options) {
        remove_directory_if_present(&paths.staging)?;
        return Err(error);
    }
    transactional_exchange(&paths, &rendered.projected_checklist, options)
}

struct PublicationPaths {
    staging: Utf8PathBuf,
    destination: Utf8PathBuf,
    checklist: Utf8PathBuf,
    prior_manifest: Utf8PathBuf,
}

impl PublicationPaths {
    fn resolve(
        workspace: &Utf8Path,
        staging: &Utf8Path,
        destination: &Utf8Path,
        checklist: &Utf8Path,
        prior_manifest: &Utf8Path,
    ) -> GenerationResult<Self> {
        let resolved = Self {
            staging: workspace.join(normalize_repo_relative(staging, "staging root")?),
            destination: workspace.join(normalize_repo_relative(destination, "destination root")?),
            checklist: workspace.join(normalize_repo_relative(checklist, "checklist path")?),
            prior_manifest: workspace.join(normalize_repo_relative(
                prior_manifest,
                "prior manifest path",
            )?),
        };
        if resolved.staging == resolved.destination
            || resolved.staging.starts_with(&resolved.destination)
            || resolved.destination.starts_with(&resolved.staging)
        {
            return Err(GenerationError::InvalidInput(
                "Phase 36 staging and destination roots must be distinct".to_owned(),
            ));
        }
        for path in [
            &resolved.staging,
            &resolved.destination,
            &resolved.checklist,
            &resolved.prior_manifest,
        ] {
            reject_symlink_managed_path(workspace, path)?;
        }
        Ok(resolved)
    }
}

struct RenderedGeneration {
    projection: String,
    matrix: String,
    verdict: String,
    manifest: String,
    projected_checklist: String,
}

fn render_and_validate(
    documents: &Phase36GenerationDocuments,
    current_checklist: &str,
    prior_manifest: &str,
) -> GenerationResult<RenderedGeneration> {
    if documents.prerequisites.evaluator_digest != current_phase36_evaluator_digest() {
        return validation("Phase 36 evaluator identity is stale");
    }
    validate_prior_manifest(prior_manifest, &documents.prerequisites)?;
    let checklist = Phase36ChecklistSnapshot::capture(current_checklist.to_owned())
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    let expected = evaluate_phase36_promotion(&documents.prerequisites, &checklist)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    if expected != documents.matrix {
        return validation("Phase 36 matrix does not match the current evaluator");
    }
    let projection = pretty_json(&typed_projection(&documents.prerequisites))?;
    validate_projection_shape(&projection)?;
    let matrix = pretty_json(&documents.matrix)?;
    let supported_rows = documents.matrix.supported_row_ids();
    let verdict = pretty_json(&PublicationVerdict {
        schema_version: VERDICT_SCHEMA,
        authoritative_generation_digest: &documents
            .matrix
            .resolver
            .authoritative_phase36_generation_digest,
        supported_rows,
        complete_matrix: true,
    })?;
    let manifest = pretty_json(&GenerationManifest {
        schema_version: MANIFEST_SCHEMA,
        phase35_root_digest: &documents.matrix.phase35_root_digest,
        superseded_phase35_generation_digest: &documents
            .matrix
            .superseded_phase35_generation_digest,
        authoritative_phase36_generation_digest: &documents
            .matrix
            .resolver
            .authoritative_phase36_generation_digest,
        evaluator_digest: &documents.matrix.evaluator_digest,
        checklist_fingerprint_before: &documents.matrix.checklist_fingerprint_before,
        checklist_fingerprint_after: &documents.matrix.checklist_fingerprint_after,
        projection_sha256: sha256_hex(projection.as_bytes()),
        matrix_sha256: sha256_hex(matrix.as_bytes()),
        verdict_sha256: sha256_hex(verdict.as_bytes()),
        checklist_sha256: sha256_hex(expected.projected_checklist.as_bytes()),
    })?;
    Ok(RenderedGeneration {
        projection,
        matrix,
        verdict,
        manifest,
        projected_checklist: expected.projected_checklist,
    })
}

pub(super) fn typed_projection(
    prerequisites: &Phase36ClaimPrerequisites,
) -> TypedFactProjection<'_> {
    TypedFactProjection {
        schema_version: PROJECTION_SCHEMA,
        phase35_root_digest: &prerequisites.phase35_root_digest,
        superseded_phase35_generation_digest: &prerequisites.superseded_phase35_generation_digest,
        evaluator_digest: &prerequisites.evaluator_digest,
        hostname_durability: &prerequisites.maybe_hostname,
        sensor_substance: &prerequisites.maybe_sensors,
        snapshot_join: &prerequisites.maybe_snapshot_join,
        runtime_health: &prerequisites.maybe_runtime_health,
        runtime_identity: &prerequisites.maybe_runtime_identity,
        independent_effect: &prerequisites.maybe_independent_effect,
    }
}

fn validate_prior_manifest(
    contents: &str,
    prerequisites: &Phase36ClaimPrerequisites,
) -> GenerationResult<()> {
    let value: Value = serde_json::from_str(contents)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    if value.get("schema").and_then(Value::as_str) != Some(PRIOR_MANIFEST_SCHEMA)
        || value.get("root_digest").and_then(Value::as_str)
            != Some(prerequisites.phase35_root_digest.as_str())
        || sha256_hex(contents.as_bytes()) != prerequisites.superseded_phase35_generation_digest
    {
        return validation("Phase 35 root or generation fingerprint changed");
    }
    Ok(())
}

fn validate_projection_shape(contents: &str) -> GenerationResult<()> {
    let value: Value = serde_json::from_str(contents)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    let expected = [
        "schema_version",
        "phase35_root_digest",
        "superseded_phase35_generation_digest",
        "evaluator_digest",
        "hostname_durability",
        "sensor_substance",
        "snapshot_join",
        "runtime_health",
        "runtime_identity",
        "independent_effect",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = value
        .as_object()
        .ok_or_else(|| {
            GenerationError::Validation(vec!["projection must be an object".to_owned()])
        })?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return validation("Phase 36 typed projection is incomplete or has extra fields");
    }
    for forbidden in [
        "password",
        "credential",
        "ssid",
        "device_url",
        "usb_path",
        "raw_document",
        "request_body",
        "response_body",
    ] {
        if contains_key(&value, forbidden) {
            return validation("Phase 36 projection contains a protected field");
        }
    }
    Ok(())
}

fn contains_key(value: &Value, forbidden: &str) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| key == forbidden || contains_key(value, forbidden)),
        Value::Array(values) => values.iter().any(|value| contains_key(value, forbidden)),
        _ => false,
    }
}

fn stage_documents(
    staging: &Utf8Path,
    rendered: &RenderedGeneration,
    options: Phase36PublicationOptions,
) -> GenerationResult<()> {
    let documents = [
        (
            PROJECTION_FILE,
            rendered.projection.as_str(),
            Phase36PublicationFailurePoint::AfterProjectionWrite,
        ),
        (
            MATRIX_FILE,
            rendered.matrix.as_str(),
            Phase36PublicationFailurePoint::AfterMatrixWrite,
        ),
        (
            VERDICT_FILE,
            rendered.verdict.as_str(),
            Phase36PublicationFailurePoint::AfterVerdictWrite,
        ),
        (
            CHECKLIST_SNAPSHOT_FILE,
            rendered.projected_checklist.as_str(),
            Phase36PublicationFailurePoint::AfterChecklistSnapshotWrite,
        ),
        (
            MANIFEST_FILE,
            rendered.manifest.as_str(),
            Phase36PublicationFailurePoint::AfterManifestWrite,
        ),
    ];
    for (name, contents, failure_point) in documents {
        write_synced(&staging.join(name), contents)?;
        if options.maybe_failure == Some(failure_point) {
            return injected(failure_point);
        }
    }
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::BeforeValidation) {
        return injected(Phase36PublicationFailurePoint::BeforeValidation);
    }
    validate_staged_generation(staging)?;
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::AfterValidation) {
        return injected(Phase36PublicationFailurePoint::AfterValidation);
    }
    sync_directory(staging)?;
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::AfterStagingSync) {
        return injected(Phase36PublicationFailurePoint::AfterStagingSync);
    }
    Ok(())
}

fn validate_staged_generation(staging: &Utf8Path) -> GenerationResult<()> {
    let inventory = regular_file_inventory(staging)?;
    if inventory
        != OWNED_FILES
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    {
        return validation("Phase 36 staging inventory is incomplete");
    }
    let manifest: Value = serde_json::from_str(&read_text(
        &staging.join(MANIFEST_FILE),
        "Phase 36 manifest",
    )?)
    .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    if manifest.get("schema_version").and_then(Value::as_str) != Some(MANIFEST_SCHEMA) {
        return validation("Phase 36 manifest schema is invalid");
    }
    for (file, field) in [
        (PROJECTION_FILE, "projection_sha256"),
        (MATRIX_FILE, "matrix_sha256"),
        (VERDICT_FILE, "verdict_sha256"),
        (CHECKLIST_SNAPSHOT_FILE, "checklist_sha256"),
    ] {
        let contents = read_text(&staging.join(file), file)?;
        if manifest.get(field).and_then(Value::as_str)
            != Some(sha256_hex(contents.as_bytes()).as_str())
        {
            return validation("Phase 36 staged document fingerprint mismatch");
        }
    }
    Ok(())
}

fn validate_legacy_generation(root: &Utf8Path) -> GenerationResult<()> {
    let manifest: Value =
        serde_json::from_str(&read_text(&root.join(MANIFEST_FILE), "Phase 36 manifest")?)
            .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    if manifest.get("schema_version").and_then(Value::as_str) != Some(MANIFEST_SCHEMA) {
        return validation("legacy Phase 36 manifest schema is invalid");
    }
    for (file, field) in [
        (PROJECTION_FILE, "projection_sha256"),
        (MATRIX_FILE, "matrix_sha256"),
        (VERDICT_FILE, "verdict_sha256"),
    ] {
        let contents = read_text(&root.join(file), file)?;
        if manifest.get(field).and_then(Value::as_str)
            != Some(sha256_hex(contents.as_bytes()).as_str())
        {
            return validation("legacy Phase 36 document fingerprint mismatch");
        }
    }
    Ok(())
}

pub(crate) fn read_phase36_public_checklist(
    workspace_root: &Utf8Path,
    staging_root: &Utf8Path,
    destination_root: &Utf8Path,
    checklist_path: &Utf8Path,
    prior_manifest_path: &Utf8Path,
) -> GenerationResult<String> {
    let paths = PublicationPaths::resolve(
        workspace_root,
        staging_root,
        destination_root,
        checklist_path,
        prior_manifest_path,
    )?;
    recover_derived_checklist(&paths)?;
    read_text(&paths.checklist, "derived checklist")
}

fn pretty_json(value: &impl Serialize) -> GenerationResult<String> {
    let mut rendered = serde_json::to_string_pretty(value)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    rendered.push('\n');
    Ok(rendered)
}

fn validation<T>(message: &str) -> GenerationResult<T> {
    Err(GenerationError::Validation(vec![message.to_owned()]))
}

fn injected<T>(point: Phase36PublicationFailurePoint) -> GenerationResult<T> {
    Err(GenerationError::Phase36Injected(point))
}
