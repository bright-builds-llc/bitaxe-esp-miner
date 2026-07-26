use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Component;

use camino::{Utf8Path, Utf8PathBuf};

use super::{GenerationError, GenerationResult};

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
