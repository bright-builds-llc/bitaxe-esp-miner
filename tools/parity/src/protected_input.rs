//! Descriptor-bound reads from an owner-private evidence root.

use std::ffi::CString;
use std::fs;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{FileExt, MetadataExt, OpenOptionsExt, PermissionsExt};

use camino::Utf8Path;
use thiserror::Error;

use crate::phase35_evidence::sha256_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum ProtectedInputError {
    #[error("protected_root_invalid")]
    RootInvalid,
    #[error("protected_root_symlink")]
    RootSymlink,
    #[error("unsafe_artifact_path")]
    UnsafePath,
    #[error("protected_input_missing")]
    Missing,
    #[error("protected_input_symlink")]
    Symlink,
    #[error("wrong_permissions")]
    WrongPermissions,
    #[error("protected_input_changed")]
    Changed,
    #[error("protected_input_not_utf8")]
    NotUtf8,
}

#[derive(Debug)]
pub(crate) struct ProtectedRoot {
    directory: fs::File,
}

impl ProtectedRoot {
    pub(crate) fn open(root: &Utf8Path) -> Result<Self, ProtectedInputError> {
        if fs::symlink_metadata(root.as_std_path())
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(ProtectedInputError::RootSymlink);
        }
        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(root.as_std_path())
            .map_err(|error| {
                if error.raw_os_error() == Some(libc::ELOOP) {
                    ProtectedInputError::RootSymlink
                } else {
                    ProtectedInputError::RootInvalid
                }
            })?;
        validate_directory(&directory)?;
        Ok(Self { directory })
    }

    pub(crate) fn open_file(
        &self,
        relative: &Utf8Path,
    ) -> Result<ProtectedFile, ProtectedInputError> {
        if !safe_relative_path(relative) {
            return Err(ProtectedInputError::UnsafePath);
        }
        let mut components = relative.components().peekable();
        let mut directory = self
            .directory
            .try_clone()
            .map_err(|_| ProtectedInputError::RootInvalid)?;
        let file = loop {
            let camino::Utf8Component::Normal(name) =
                components.next().ok_or(ProtectedInputError::UnsafePath)?
            else {
                return Err(ProtectedInputError::UnsafePath);
            };
            if components.peek().is_some() {
                directory = open_directory_at(&directory, name)?;
                validate_directory(&directory)?;
                continue;
            }
            break open_file_at(&directory, name)?;
        };
        ProtectedFile::capture(file)
    }
}

#[derive(Debug)]
pub(crate) struct ProtectedFile {
    file: fs::File,
    bytes: Vec<u8>,
    digest: String,
    identity: FileIdentity,
}

impl ProtectedFile {
    fn capture(file: fs::File) -> Result<Self, ProtectedInputError> {
        let identity_before =
            FileIdentity::capture(&file).map_err(|_| ProtectedInputError::Changed)?;
        validate_file(&file)?;
        let bytes = read_descriptor(&file).map_err(|_| ProtectedInputError::Changed)?;
        let identity = FileIdentity::capture(&file).map_err(|_| ProtectedInputError::Changed)?;
        if identity != identity_before || bytes.len() as u64 != identity.length {
            return Err(ProtectedInputError::Changed);
        }
        Ok(Self {
            file,
            digest: sha256_hex(&bytes),
            bytes,
            identity,
        })
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn text(&self) -> Result<&str, ProtectedInputError> {
        std::str::from_utf8(&self.bytes).map_err(|_| ProtectedInputError::NotUtf8)
    }

    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }

    pub(crate) fn verify_unchanged(&self) -> Result<(), ProtectedInputError> {
        let identity =
            FileIdentity::capture(&self.file).map_err(|_| ProtectedInputError::Changed)?;
        let bytes = read_descriptor(&self.file).map_err(|_| ProtectedInputError::Changed)?;
        if identity != self.identity || sha256_hex(&bytes) != self.digest {
            return Err(ProtectedInputError::Changed);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    length: u64,
}

impl FileIdentity {
    fn capture(file: &fs::File) -> std::io::Result<Self> {
        let metadata = file.metadata()?;
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner: metadata.uid(),
            mode: metadata.mode(),
            length: metadata.len(),
        })
    }
}

fn open_directory_at(parent: &fs::File, name: &str) -> Result<fs::File, ProtectedInputError> {
    open_at(
        parent,
        name,
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
    )
}

fn open_file_at(parent: &fs::File, name: &str) -> Result<fs::File, ProtectedInputError> {
    open_at(
        parent,
        name,
        libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
    )
}

fn open_at(
    parent: &fs::File,
    name: &str,
    flags: libc::c_int,
) -> Result<fs::File, ProtectedInputError> {
    let name = CString::new(name.as_bytes()).map_err(|_| ProtectedInputError::UnsafePath)?;
    let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if descriptor < 0 {
        let error = std::io::Error::last_os_error();
        return Err(match error.raw_os_error() {
            Some(libc::ENOENT) => ProtectedInputError::Missing,
            Some(libc::ELOOP) => ProtectedInputError::Symlink,
            _ => ProtectedInputError::RootInvalid,
        });
    }
    Ok(unsafe { fs::File::from_raw_fd(descriptor) })
}

fn validate_directory(directory: &fs::File) -> Result<(), ProtectedInputError> {
    let metadata = directory
        .metadata()
        .map_err(|_| ProtectedInputError::RootInvalid)?;
    if !metadata.is_dir() {
        return Err(ProtectedInputError::RootInvalid);
    }
    if metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(ProtectedInputError::WrongPermissions);
    }
    Ok(())
}

fn validate_file(file: &fs::File) -> Result<(), ProtectedInputError> {
    let metadata = file
        .metadata()
        .map_err(|_| ProtectedInputError::RootInvalid)?;
    if !metadata.is_file() {
        return Err(ProtectedInputError::RootInvalid);
    }
    if metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(ProtectedInputError::WrongPermissions);
    }
    Ok(())
}

fn read_descriptor(file: &fs::File) -> std::io::Result<Vec<u8>> {
    let length = file.metadata()?.len();
    let mut bytes = vec![0; usize::try_from(length).map_err(|_| std::io::ErrorKind::FileTooLarge)?];
    let mut offset = 0;
    while offset < bytes.len() {
        let read = file.read_at(&mut bytes[offset..], offset as u64)?;
        if read == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        offset += read;
    }
    Ok(bytes)
}

pub(crate) fn safe_relative_path(path: &Utf8Path) -> bool {
    !path.as_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, camino::Utf8Component::Normal(_)))
}
