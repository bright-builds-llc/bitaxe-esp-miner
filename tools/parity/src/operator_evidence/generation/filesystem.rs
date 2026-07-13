use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Component;
use std::sync::atomic::Ordering;

use camino::{Utf8Path, Utf8PathBuf};

use super::{
    ConsolidationOptions, GenerationError, GenerationResult, PromotionFailurePoint, MANIFEST_FILE,
    STAGING_SEQUENCE,
};
use crate::operator_evidence::OperatorEvidenceSlot;

pub(super) fn validate_managed_destination(destination: &Utf8Path) -> GenerationResult<()> {
    if !destination.join(MANIFEST_FILE).is_file() {
        return Err(GenerationError::InvalidInput(format!(
            "existing destination lacks generator-owned manifest: {destination}"
        )));
    }
    let allowed = OperatorEvidenceSlot::ALL
        .into_iter()
        .map(|slot| slot.file_name())
        .chain(std::iter::once(MANIFEST_FILE))
        .collect::<BTreeSet<_>>();
    for entry in fs::read_dir(destination.as_std_path()).map_err(|source| {
        io_error(
            format!("failed to inspect destination {destination}"),
            source,
        )
    })? {
        let entry =
            entry.map_err(|source| io_error("failed to inspect destination entry", source))?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            GenerationError::InvalidInput("destination contains a non-UTF-8 file name".to_owned())
        })?;
        if !allowed.contains(name) {
            return Err(GenerationError::InvalidInput(format!(
                "existing destination contains unknown file {name:?}"
            )));
        }
    }
    Ok(())
}

pub(super) fn promote_staging(
    destination: &Utf8Path,
    staging: &Utf8Path,
    options: ConsolidationOptions,
) -> GenerationResult<()> {
    let parent = destination.parent().ok_or_else(|| {
        GenerationError::InvalidInput("evidence destination has no parent".to_owned())
    })?;
    if !destination.exists() {
        fs::rename(staging.as_std_path(), destination.as_std_path()).map_err(|source| {
            io_error(format!("failed to promote staging root {staging}"), source)
        })?;
        return sync_directory(parent);
    }

    atomic_exchange(staging, destination)?;
    if options.maybe_failure == Some(PromotionFailurePoint::AfterExchange)
        || options.maybe_failure == Some(PromotionFailurePoint::DuringParentSync)
        || options.maybe_failure == Some(PromotionFailurePoint::DuringRollback)
    {
        return rollback_exchange(destination, staging, parent, options);
    }
    sync_directory(parent).or_else(|error| {
        rollback_exchange(destination, staging, parent, options).and(Err(error))
    })?;

    if options.maybe_failure == Some(PromotionFailurePoint::DuringOldGenerationCleanup) {
        return Err(GenerationError::RecoveryRequired {
            destination: destination.to_owned(),
            retained_old_generation: staging.to_owned(),
            detail: "old-generation cleanup was not attempted after injected failure".to_owned(),
        });
    }
    fs::remove_dir_all(staging.as_std_path())
        .map_err(|source| io_error(format!("failed to remove old generation {staging}"), source))?;
    sync_directory(parent)
}

pub(super) fn rollback_exchange(
    destination: &Utf8Path,
    staging: &Utf8Path,
    parent: &Utf8Path,
    options: ConsolidationOptions,
) -> GenerationResult<()> {
    if options.maybe_failure == Some(PromotionFailurePoint::DuringRollback) {
        return Err(GenerationError::RecoveryRequired {
            destination: destination.to_owned(),
            retained_old_generation: staging.to_owned(),
            detail: "rollback failed; both complete generations retained".to_owned(),
        });
    }
    atomic_exchange(destination, staging).map_err(|error| GenerationError::RecoveryRequired {
        destination: destination.to_owned(),
        retained_old_generation: staging.to_owned(),
        detail: format!("rollback exchange failed: {error}"),
    })?;
    sync_directory(parent)?;
    fs::remove_dir_all(staging.as_std_path()).map_err(|source| {
        GenerationError::RecoveryRequired {
            destination: destination.to_owned(),
            retained_old_generation: staging.to_owned(),
            detail: format!("rollback succeeded but staged replacement cleanup failed: {source}"),
        }
    })?;
    sync_directory(parent)?;
    Err(GenerationError::Injected(
        options
            .maybe_failure
            .unwrap_or(PromotionFailurePoint::DuringParentSync),
    ))
}

pub(super) fn atomic_exchange(left: &Utf8Path, right: &Utf8Path) -> GenerationResult<()> {
    #[cfg(target_os = "linux")]
    {
        let left = c_path(left)?;
        let right = c_path(right)?;
        const RENAME_EXCHANGE: libc::c_uint = 2;
        // SAFETY: both C strings are NUL-terminated and remain alive for the syscall.
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                libc::AT_FDCWD,
                left.as_ptr(),
                libc::AT_FDCWD,
                right.as_ptr(),
                RENAME_EXCHANGE,
            )
        };
        if result == 0 {
            return Ok(());
        }
        Err(io_error(
            "renameat2 RENAME_EXCHANGE failed",
            io::Error::last_os_error(),
        ))
    }

    #[cfg(target_os = "macos")]
    {
        let left = c_path(left)?;
        let right = c_path(right)?;
        const RENAME_SWAP: libc::c_uint = 0x0000_0002;
        unsafe extern "C" {
            fn renamex_np(
                from: *const libc::c_char,
                to: *const libc::c_char,
                flags: libc::c_uint,
            ) -> libc::c_int;
        }
        // SAFETY: both C strings are NUL-terminated and remain alive for the call.
        let result = unsafe { renamex_np(left.as_ptr(), right.as_ptr(), RENAME_SWAP) };
        if result == 0 {
            return Ok(());
        }
        Err(io_error(
            "renamex_np RENAME_SWAP failed",
            io::Error::last_os_error(),
        ))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (left, right);
        Err(GenerationError::InvalidInput(
            "atomic directory exchange is unsupported on this platform".to_owned(),
        ))
    }
}

pub(super) fn c_path(path: &Utf8Path) -> GenerationResult<CString> {
    CString::new(path.as_str()).map_err(|_| {
        GenerationError::InvalidInput(format!("path contains an interior NUL byte: {path}"))
    })
}

pub(super) fn create_staging_directory(destination: &Utf8Path) -> GenerationResult<Utf8PathBuf> {
    let parent = destination.parent().ok_or_else(|| {
        GenerationError::InvalidInput("evidence destination has no parent".to_owned())
    })?;
    fs::create_dir_all(parent.as_std_path()).map_err(|source| {
        io_error(
            format!("failed to create destination parent {parent}"),
            source,
        )
    })?;
    let name = destination.file_name().ok_or_else(|| {
        GenerationError::InvalidInput("evidence destination has no file name".to_owned())
    })?;
    for _ in 0..32 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let staging = parent.join(format!(".{name}.staging-{}-{sequence}", std::process::id()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(staging.as_std_path()) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(io_error(
                    format!("failed to create staging root {staging}"),
                    source,
                ));
            }
        }
    }
    Err(GenerationError::InvalidInput(
        "could not allocate a unique staging directory".to_owned(),
    ))
}

pub(super) fn normalize_repo_relative(
    path: &Utf8Path,
    label: &str,
) -> GenerationResult<Utf8PathBuf> {
    if path.is_absolute() {
        return Err(GenerationError::InvalidInput(format!(
            "{label} must be repo-relative"
        )));
    }
    let mut normalized = Utf8PathBuf::new();
    for component in path.as_std_path().components() {
        match component {
            Component::Normal(value) => {
                let value = value.to_str().ok_or_else(|| {
                    GenerationError::InvalidInput(format!("{label} must be valid UTF-8"))
                })?;
                normalized.push(value);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(GenerationError::InvalidInput(format!(
                    "{label} must not contain traversal or root components"
                )));
            }
        }
    }
    if normalized.as_str().is_empty() {
        return Err(GenerationError::InvalidInput(format!(
            "{label} must not be empty"
        )));
    }
    Ok(normalized)
}

pub(super) fn reject_related_roots(
    source: &Utf8Path,
    destination: &Utf8Path,
) -> GenerationResult<()> {
    if source == destination || source.starts_with(destination) || destination.starts_with(source) {
        return Err(GenerationError::InvalidInput(
            "phase27 and phase28 roots must be distinct and non-nested".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn reject_symlink_managed_path(
    workspace_root: &Utf8Path,
    path: &Utf8Path,
) -> GenerationResult<()> {
    if !path.starts_with(workspace_root) {
        return Err(GenerationError::InvalidInput(
            "managed path escapes the workspace root".to_owned(),
        ));
    }
    let relative = path.strip_prefix(workspace_root).map_err(|_| {
        GenerationError::InvalidInput("managed path escapes the workspace root".to_owned())
    })?;
    let mut current = workspace_root.to_owned();
    for component in relative.components() {
        current.push(component.as_str());
        match fs::symlink_metadata(current.as_std_path()) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(GenerationError::InvalidInput(format!(
                    "symlink-managed roots are not allowed: {current}"
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(source) => {
                return Err(io_error(
                    format!("failed to inspect managed path {current}"),
                    source,
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn write_synced(path: &Utf8Path, contents: &str) -> GenerationResult<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path.as_std_path())
        .map_err(|source| io_error(format!("failed to create generated file {path}"), source))?;
    file.write_all(contents.as_bytes())
        .map_err(|source| io_error(format!("failed to write generated file {path}"), source))?;
    file.sync_all()
        .map_err(|source| io_error(format!("failed to sync generated file {path}"), source))
}

pub(super) fn sync_directory(path: &Utf8Path) -> GenerationResult<()> {
    File::open(path.as_std_path())
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(format!("failed to sync directory {path}"), source))
}

pub(super) fn io_error(action: impl Into<String>, source: io::Error) -> GenerationError {
    GenerationError::Io {
        action: action.into(),
        source,
    }
}
