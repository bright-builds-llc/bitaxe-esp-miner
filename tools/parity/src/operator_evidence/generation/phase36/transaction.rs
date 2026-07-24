use std::collections::BTreeSet;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};

use super::{
    injected, validate_legacy_generation, validate_staged_generation, GenerationError,
    GenerationResult, Phase36PublicationFailurePoint, Phase36PublicationOptions, PublicationPaths,
    CHECKLIST_SNAPSHOT_FILE, LEGACY_OWNED_FILES, OWNED_FILES,
};
use crate::operator_evidence::generation::filesystem::{
    atomic_exchange, io_error, sync_directory, write_synced,
};
use crate::operator_evidence::generation::ownership::PromotionContext;

pub(super) fn transactional_exchange(
    paths: &PublicationPaths,
    projected_checklist: &str,
    options: Phase36PublicationOptions,
) -> GenerationResult<()> {
    let replacement = checklist_replacement_path(&paths.checklist)?;
    write_synced(&replacement, projected_checklist)?;
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::AfterChecklistReplacementWrite)
    {
        cleanup_rejected(paths, &replacement)?;
        return injected(Phase36PublicationFailurePoint::AfterChecklistReplacementWrite);
    }
    let context = match PromotionContext::acquire_unvalidated(&paths.destination) {
        Ok(context) => context,
        Err(error) => {
            cleanup_rejected(paths, &replacement)?;
            return Err(error);
        }
    };
    let destination_existed = context.destination_identity().is_some();
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::BeforeGenerationExchange) {
        cleanup_rejected(paths, &replacement)?;
        return injected(Phase36PublicationFailurePoint::BeforeGenerationExchange);
    }
    exchange_generation(&paths.destination, &paths.staging, &context)?;
    #[cfg(test)]
    if options.crash_after_authority_exchange {
        unsafe { libc::_exit(86) }
    }
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::AfterGenerationExchange) {
        rollback_transaction(paths, &replacement, destination_existed, false)?;
        return injected(Phase36PublicationFailurePoint::AfterGenerationExchange);
    }
    if options.maybe_failure == Some(Phase36PublicationFailurePoint::BeforeChecklistExchange) {
        rollback_transaction(paths, &replacement, destination_existed, false)?;
        return injected(Phase36PublicationFailurePoint::BeforeChecklistExchange);
    }
    if let Err(error) = atomic_exchange(&paths.checklist, &replacement) {
        rollback_transaction(paths, &replacement, destination_existed, false)?;
        return Err(error);
    }
    let maybe_failure = options.maybe_failure.filter(|point| {
        matches!(
            point,
            Phase36PublicationFailurePoint::AfterChecklistExchange
                | Phase36PublicationFailurePoint::AfterParentSync
        )
    });
    let sync_result =
        if maybe_failure == Some(Phase36PublicationFailurePoint::AfterChecklistExchange) {
            Ok(())
        } else {
            sync_parents(paths)
        };
    if let Err(error) = sync_result {
        rollback_transaction(paths, &replacement, destination_existed, true)?;
        return Err(error);
    }
    if let Some(point) = maybe_failure {
        rollback_transaction(paths, &replacement, destination_existed, true)?;
        return injected(point);
    }
    if destination_existed {
        remove_directory_if_present(&paths.staging)?;
    }
    remove_file_if_present(&replacement)?;
    sync_parents(paths)
}

pub(super) fn recover_derived_checklist(paths: &PublicationPaths) -> GenerationResult<()> {
    if !paths.destination.exists() {
        return Ok(());
    }
    if regular_file_inventory(&paths.destination)?
        == LEGACY_OWNED_FILES
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    {
        validate_legacy_generation(&paths.destination)?;
        return Ok(());
    }
    validate_existing_destination(&paths.destination)?;
    let authoritative = read_text(
        &paths.destination.join(CHECKLIST_SNAPSHOT_FILE),
        "Phase 36 authoritative checklist snapshot",
    )?;
    let current = read_text(&paths.checklist, "derived checklist")?;
    if current != authoritative {
        let replacement = checklist_replacement_path(&paths.checklist)?;
        write_synced(&replacement, &authoritative)?;
        atomic_exchange(&paths.checklist, &replacement)?;
        remove_file_if_present(&replacement)?;
        sync_directory(parent(&paths.checklist, "checklist")?)?;
    }
    if paths.staging.exists() {
        validate_staged_generation(&paths.staging)?;
        remove_directory_if_present(&paths.staging)?;
        sync_directory(parent(&paths.destination, "destination root")?)?;
    }
    Ok(())
}

fn rollback_transaction(
    paths: &PublicationPaths,
    replacement: &Utf8Path,
    destination_existed: bool,
    checklist_exchanged: bool,
) -> GenerationResult<()> {
    if checklist_exchanged {
        atomic_exchange(&paths.checklist, replacement)?;
    }
    rollback_generation(&paths.destination, &paths.staging, destination_existed)?;
    remove_directory_if_present(&paths.staging)?;
    remove_file_if_present(replacement)
}

fn cleanup_rejected(paths: &PublicationPaths, replacement: &Utf8Path) -> GenerationResult<()> {
    remove_file_if_present(replacement)?;
    remove_directory_if_present(&paths.staging)
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
            .map_err(|error| io_error("failed to promote Phase 36 generation", error))?;
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
            .map_err(|error| io_error("failed to roll back Phase 36 generation", error))
    }
}

pub(super) fn validate_existing_destination(destination: &Utf8Path) -> GenerationResult<()> {
    if !destination.exists() {
        return Ok(());
    }
    let inventory = regular_file_inventory(destination)?;
    if inventory
        == LEGACY_OWNED_FILES
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    {
        return validate_legacy_generation(destination);
    }
    if inventory
        != OWNED_FILES
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    {
        return Err(GenerationError::InvalidInput(
            "existing Phase 36 destination is not generator-owned".to_owned(),
        ));
    }
    validate_staged_generation(destination)
}

pub(super) fn regular_file_inventory(root: &Utf8Path) -> GenerationResult<BTreeSet<String>> {
    let mut inventory = BTreeSet::new();
    for entry in fs::read_dir(root.as_std_path())
        .map_err(|error| io_error(format!("failed to inspect {root}"), error))?
    {
        let entry =
            entry.map_err(|error| io_error("failed to inspect destination entry", error))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| io_error("failed to inspect destination entry type", error))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(GenerationError::InvalidInput(
                "Phase 36 destination contains a non-regular entry".to_owned(),
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| GenerationError::InvalidInput("non-UTF-8 destination entry".to_owned()))?;
        inventory.insert(name);
    }
    Ok(inventory)
}

pub(super) fn create_private_staging(staging: &Utf8Path) -> GenerationResult<()> {
    if staging.exists() {
        return Err(GenerationError::InvalidInput(
            "Phase 36 staging root already exists".to_owned(),
        ));
    }
    fs::create_dir_all(parent(staging, "staging root")?.as_std_path())
        .map_err(|error| io_error("failed to create staging parent", error))?;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(staging.as_std_path())
        .map_err(|error| io_error("failed to create Phase 36 staging root", error))
}

pub(super) fn read_text(path: &Utf8Path, label: &str) -> GenerationResult<String> {
    fs::read_to_string(path.as_std_path())
        .map_err(|error| io_error(format!("failed to read {label}"), error))
}

fn sync_parents(paths: &PublicationPaths) -> GenerationResult<()> {
    sync_directory(parent(&paths.destination, "destination root")?)
        .and_then(|()| sync_directory(parent(&paths.checklist, "checklist")?))
}

fn parent<'a>(path: &'a Utf8Path, label: &str) -> GenerationResult<&'a Utf8Path> {
    path.parent()
        .ok_or_else(|| GenerationError::InvalidInput(format!("{label} has no parent")))
}

fn checklist_replacement_path(checklist: &Utf8Path) -> GenerationResult<Utf8PathBuf> {
    let name = checklist
        .file_name()
        .ok_or_else(|| GenerationError::InvalidInput("checklist has no file name".to_owned()))?;
    Ok(parent(checklist, "checklist")?.join(format!(
        ".{name}.phase36-replacement-{}",
        std::process::id()
    )))
}

pub(super) fn remove_directory_if_present(path: &Utf8Path) -> GenerationResult<()> {
    match fs::remove_dir_all(path.as_std_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("failed to remove Phase 36 staging root", error)),
    }
}

fn remove_file_if_present(path: &Utf8Path) -> GenerationResult<()> {
    match fs::remove_file(path.as_std_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("failed to remove Phase 36 replacement", error)),
    }
}
