use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;

use super::filesystem::{
    atomic_exchange, io_error, normalize_repo_relative, reject_symlink_managed_path,
    sync_directory, write_synced,
};
use super::ownership::PromotionContext;
use super::{GenerationError, GenerationResult};
use crate::phase35_evidence::sha256_hex;

const PROJECTION_FILE: &str = "projection.json";
const MATRIX_FILE: &str = "decision-matrix.json";
const CHECKLIST_FILE: &str = "checklist.md";
const MANIFEST_FILE: &str = ".phase35-generation-manifest.json";
const VERDICT_FILE: &str = "admitted.json";

#[derive(Debug, Clone)]
pub(crate) struct Phase35GenerationDocuments {
    pub(crate) projection_json: String,
    pub(crate) matrix_json: String,
    pub(crate) projected_checklist: String,
    pub(crate) expected_checklist_fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase35PublicationFailurePoint {
    BeforeValidation,
    AfterValidationBeforeExchange,
    DuringExchange,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Phase35PublicationOptions {
    pub(crate) maybe_failure: Option<Phase35PublicationFailurePoint>,
}

pub(crate) fn publish_phase35_generation(
    workspace_root: &Utf8Path,
    staging_root: &Utf8Path,
    destination_root: &Utf8Path,
    checklist_path: &Utf8Path,
    documents: &Phase35GenerationDocuments,
    options: Phase35PublicationOptions,
) -> GenerationResult<()> {
    let staging = workspace_root.join(normalize_repo_relative(staging_root, "staging root")?);
    let destination = workspace_root.join(normalize_repo_relative(
        destination_root,
        "destination root",
    )?);
    let checklist = workspace_root.join(normalize_repo_relative(checklist_path, "checklist path")?);
    if staging == destination
        || staging.starts_with(&destination)
        || destination.starts_with(&staging)
    {
        return Err(GenerationError::InvalidInput(
            "Phase 35 staging and destination roots must be distinct".to_owned(),
        ));
    }
    for managed in [&staging, &destination, &checklist] {
        reject_symlink_managed_path(workspace_root, managed)?;
    }
    if options.maybe_failure == Some(Phase35PublicationFailurePoint::BeforeValidation) {
        return Err(GenerationError::Phase35Injected(
            Phase35PublicationFailurePoint::BeforeValidation,
        ));
    }
    create_private_staging(&staging)?;
    let generation_result = stage_and_validate(&staging, &checklist, documents);
    if let Err(error) = generation_result {
        let _ = fs::remove_dir_all(staging.as_std_path());
        return Err(error);
    }
    if options.maybe_failure == Some(Phase35PublicationFailurePoint::AfterValidationBeforeExchange)
    {
        fs::remove_dir_all(staging.as_std_path()).map_err(|error| {
            io_error(
                format!("failed to remove rejected staging root {staging}"),
                error,
            )
        })?;
        return Err(GenerationError::Phase35Injected(
            Phase35PublicationFailurePoint::AfterValidationBeforeExchange,
        ));
    }

    let checklist_replacement = checklist_replacement_path(&checklist)?;
    write_synced(&checklist_replacement, &documents.projected_checklist)?;
    set_private_file_mode(&checklist_replacement)?;
    let context = PromotionContext::acquire_unvalidated(&destination)?;
    let destination_existed = context.destination_identity().is_some();
    exchange_generation(&destination, &staging, &context)?;

    if options.maybe_failure == Some(Phase35PublicationFailurePoint::DuringExchange) {
        rollback_generation(&destination, &staging, destination_existed)?;
        remove_file_if_present(&checklist_replacement)?;
        return Err(GenerationError::Phase35Injected(
            Phase35PublicationFailurePoint::DuringExchange,
        ));
    }
    if let Err(error) = atomic_exchange(&checklist, &checklist_replacement) {
        rollback_generation(&destination, &staging, destination_existed)?;
        remove_file_if_present(&checklist_replacement)?;
        return Err(error);
    }

    let destination_parent = destination.parent().ok_or_else(|| {
        GenerationError::InvalidInput("destination root has no parent".to_owned())
    })?;
    let checklist_parent = checklist
        .parent()
        .ok_or_else(|| GenerationError::InvalidInput("checklist path has no parent".to_owned()))?;
    if let Err(error) =
        sync_directory(destination_parent).and_then(|()| sync_directory(checklist_parent))
    {
        atomic_exchange(&checklist, &checklist_replacement)?;
        rollback_generation(&destination, &staging, destination_existed)?;
        return Err(error);
    }

    if destination_existed {
        fs::remove_dir_all(staging.as_std_path()).map_err(|error| {
            GenerationError::RecoveryRequired {
                destination: destination.clone(),
                retained_old_generation: staging.clone(),
                detail: format!("old Phase 35 generation cleanup failed: {error}"),
            }
        })?;
    }
    remove_file_if_present(&checklist_replacement)?;
    sync_directory(destination_parent)?;
    sync_directory(checklist_parent)
}

fn create_private_staging(staging: &Utf8Path) -> GenerationResult<()> {
    if staging.exists() {
        return Err(GenerationError::InvalidInput(format!(
            "Phase 35 staging root already exists: {staging}"
        )));
    }
    let parent = staging
        .parent()
        .ok_or_else(|| GenerationError::InvalidInput("staging root has no parent".to_owned()))?;
    fs::create_dir_all(parent.as_std_path())
        .map_err(|error| io_error(format!("failed to create staging parent {parent}"), error))?;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(staging.as_std_path())
        .map_err(|error| io_error(format!("failed to create staging root {staging}"), error))
}

fn stage_and_validate(
    staging: &Utf8Path,
    checklist: &Utf8Path,
    documents: &Phase35GenerationDocuments,
) -> GenerationResult<()> {
    let current_checklist = fs::read_to_string(checklist.as_std_path())
        .map_err(|error| io_error(format!("failed to read checklist {checklist}"), error))?;
    if sha256_hex(current_checklist.as_bytes()) != documents.expected_checklist_fingerprint {
        return Err(GenerationError::Validation(vec![
            "checklist changed after the promotion snapshot".to_owned(),
        ]));
    }
    let projection: Value = serde_json::from_str(&documents.projection_json)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    let matrix: Value = serde_json::from_str(&documents.matrix_json)
        .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    let projection_root = required_string(&projection, "root_digest")?;
    let matrix_root = required_string(&matrix, "evidence_root_digest")?;
    let before = required_string(&matrix, "checklist_fingerprint_before")?;
    let after = required_string(&matrix, "checklist_fingerprint_after")?;
    if projection_root != matrix_root
        || before != documents.expected_checklist_fingerprint
        || after != sha256_hex(documents.projected_checklist.as_bytes())
    {
        return Err(GenerationError::Validation(vec![
            "Phase 35 generation fingerprints or root digest disagree".to_owned(),
        ]));
    }
    for (name, contents) in [
        (PROJECTION_FILE, documents.projection_json.as_str()),
        (MATRIX_FILE, documents.matrix_json.as_str()),
        (CHECKLIST_FILE, documents.projected_checklist.as_str()),
    ] {
        let path = staging.join(name);
        write_synced(&path, contents)?;
        set_private_file_mode(&path)?;
    }
    let manifest = serde_json::to_string_pretty(&serde_json::json!({
        "schema": "phase35-generation-v1",
        "root_digest": matrix_root,
        "projection_sha256": sha256_hex(documents.projection_json.as_bytes()),
        "matrix_sha256": sha256_hex(documents.matrix_json.as_bytes()),
        "checklist_sha256": after,
    }))
    .map_err(|error| GenerationError::Validation(vec![error.to_string()]))?;
    write_synced(&staging.join(MANIFEST_FILE), &manifest)?;
    write_synced(
        &staging.join(VERDICT_FILE),
        &format!("{{\"admitted\":true,\"evidence_root_digest\":\"{matrix_root}\"}}\n"),
    )?;
    set_private_file_mode(&staging.join(MANIFEST_FILE))?;
    set_private_file_mode(&staging.join(VERDICT_FILE))?;
    sync_directory(staging)
}

fn required_string<'a>(value: &'a Value, field: &str) -> GenerationResult<&'a str> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| {
        GenerationError::Validation(vec![format!(
            "Phase 35 generation is missing string field {field}"
        )])
    })
}

fn exchange_generation(
    destination: &Utf8Path,
    staging: &Utf8Path,
    context: &PromotionContext,
) -> GenerationResult<()> {
    let staging_identity = context.validate_before_exchange(destination, staging)?;
    if context.destination_identity().is_some() {
        atomic_exchange(staging, destination)?;
        context.validate_swapped(destination, staging, staging_identity)
    } else {
        fs::rename(staging.as_std_path(), destination.as_std_path())
            .map_err(|error| io_error("failed to promote Phase 35 generation", error))?;
        context.validate_initial_promotion(destination, staging, staging_identity)
    }
}

fn rollback_generation(
    destination: &Utf8Path,
    staging: &Utf8Path,
    destination_existed: bool,
) -> GenerationResult<()> {
    if destination_existed {
        atomic_exchange(destination, staging)
    } else {
        fs::rename(destination.as_std_path(), staging.as_std_path())
            .map_err(|error| io_error("failed to roll back Phase 35 generation", error))
    }
}

fn checklist_replacement_path(checklist: &Utf8Path) -> GenerationResult<Utf8PathBuf> {
    let parent = checklist
        .parent()
        .ok_or_else(|| GenerationError::InvalidInput("checklist path has no parent".to_owned()))?;
    let name = checklist.file_name().ok_or_else(|| {
        GenerationError::InvalidInput("checklist path has no file name".to_owned())
    })?;
    Ok(parent.join(format!(
        ".{name}.phase35-replacement-{}",
        std::process::id()
    )))
}

fn remove_file_if_present(path: &Utf8Path) -> GenerationResult<()> {
    match fs::remove_file(path.as_std_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(format!("failed to remove {path}"), error)),
    }
}

fn set_private_file_mode(path: &Utf8Path) -> GenerationResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))
            .map_err(|error| io_error(format!("failed to secure {path}"), error))?;
    }
    Ok(())
}
