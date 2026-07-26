use std::fs::{self, File, OpenOptions};
use std::io;

use camino::{Utf8Path, Utf8PathBuf};

use super::filesystem::io_error;
use super::{GenerationError, GenerationResult};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PathIdentity {
    device: u64,
    inode: u64,
}

impl PathIdentity {
    fn capture_directory(path: &Utf8Path, label: &str) -> GenerationResult<Self> {
        let metadata = fs::symlink_metadata(path.as_std_path())
            .map_err(|error| io_error(format!("failed to inspect {label} {path}"), error))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(GenerationError::InvalidInput(format!(
                "{label} must be a non-symlink directory: {path}"
            )));
        }
        Ok(Self::from_metadata(&metadata))
    }

    fn capture_regular_file(path: &Utf8Path, label: &str) -> GenerationResult<Self> {
        let metadata = fs::symlink_metadata(path.as_std_path())
            .map_err(|error| io_error(format!("failed to inspect {label} {path}"), error))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(GenerationError::InvalidInput(format!(
                "{label} must be a non-symlink regular file: {path}"
            )));
        }
        Ok(Self::from_metadata(&metadata))
    }

    #[cfg(unix)]
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }

    #[cfg(not(unix))]
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        use std::time::UNIX_EPOCH;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |value| value.as_nanos() as u64);
        Self {
            device: metadata.len(),
            inode: modified,
        }
    }
}

pub(super) struct PromotionContext {
    parent: Utf8PathBuf,
    parent_identity: PathIdentity,
    lock: PromotionLock,
    destination_identity: Option<PathIdentity>,
}

impl PromotionContext {
    pub(super) fn acquire_unvalidated(destination: &Utf8Path) -> GenerationResult<Self> {
        let parent = destination.parent().ok_or_else(|| {
            GenerationError::InvalidInput("evidence destination has no parent".to_owned())
        })?;
        fs::create_dir_all(parent.as_std_path()).map_err(|error| {
            io_error(
                format!("failed to create evidence destination parent {parent}"),
                error,
            )
        })?;
        let parent_identity = PathIdentity::capture_directory(parent, "destination parent")?;
        let lock = PromotionLock::acquire(
            parent,
            destination.file_name().ok_or_else(|| {
                GenerationError::InvalidInput("evidence destination has no file name".to_owned())
            })?,
        )?;
        let destination_identity = match fs::symlink_metadata(destination.as_std_path()) {
            Ok(_) => Some(PathIdentity::capture_directory(
                destination,
                "existing destination",
            )?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(io_error(
                    format!("failed to inspect evidence destination {destination}"),
                    error,
                ));
            }
        };
        let context = Self {
            parent: parent.to_owned(),
            parent_identity,
            lock,
            destination_identity,
        };
        context.revalidate_lock_and_parent()?;
        Ok(context)
    }

    pub(super) const fn destination_identity(&self) -> Option<PathIdentity> {
        self.destination_identity
    }

    pub(super) fn validate_before_exchange(
        &self,
        destination: &Utf8Path,
        staging: &Utf8Path,
    ) -> GenerationResult<PathIdentity> {
        self.revalidate_lock_and_parent()?;
        self.revalidate_destination(destination)?;
        PathIdentity::capture_directory(staging, "staging destination")
    }

    pub(super) fn validate_initial_promotion(
        &self,
        destination: &Utf8Path,
        staging: &Utf8Path,
        staging_identity: PathIdentity,
    ) -> GenerationResult<()> {
        self.revalidate_lock_and_parent()?;
        revalidate_identity(destination, staging_identity, "promoted destination")?;
        require_missing(staging, "staging path after initial promotion")
    }

    pub(super) fn validate_swapped(
        &self,
        destination: &Utf8Path,
        staging: &Utf8Path,
        staging_identity: PathIdentity,
    ) -> GenerationResult<()> {
        self.revalidate_lock_and_parent()?;
        revalidate_identity(destination, staging_identity, "promoted destination")?;
        let previous_identity = self.destination_identity.ok_or_else(|| {
            GenerationError::InvalidInput(
                "existing destination identity is missing during exchange".to_owned(),
            )
        })?;
        revalidate_identity(staging, previous_identity, "retained previous generation")?;
        Ok(())
    }

    fn revalidate_lock_and_parent(&self) -> GenerationResult<()> {
        revalidate_identity(&self.parent, self.parent_identity, "destination parent")?;
        self.lock.revalidate()
    }

    fn revalidate_destination(&self, destination: &Utf8Path) -> GenerationResult<()> {
        match self.destination_identity {
            Some(identity) => revalidate_identity(destination, identity, "evidence destination"),
            None => require_missing(destination, "evidence destination"),
        }
    }
}

struct PromotionLock {
    path: Utf8PathBuf,
    identity: PathIdentity,
    _file: File,
}

impl PromotionLock {
    fn acquire(parent: &Utf8Path, destination_name: &str) -> GenerationResult<Self> {
        let path = parent.join(format!(".{destination_name}.promotion.lock"));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let file = options.open(path.as_std_path()).map_err(|error| {
            let detail = if error.kind() == io::ErrorKind::AlreadyExists {
                "promotion lock is already held"
            } else {
                "failed to create promotion lock"
            };
            io_error(format!("{detail} at {path}"), error)
        })?;
        file.sync_all()
            .map_err(|error| io_error(format!("failed to sync promotion lock {path}"), error))?;
        let identity = PathIdentity::capture_regular_file(&path, "promotion lock")?;
        Ok(Self {
            path,
            identity,
            _file: file,
        })
    }

    fn revalidate(&self) -> GenerationResult<()> {
        revalidate_identity(&self.path, self.identity, "promotion lock")
    }
}

impl Drop for PromotionLock {
    fn drop(&mut self) {
        if self.revalidate().is_ok() {
            let _ = fs::remove_file(self.path.as_std_path());
            if let Some(parent) = self.path.parent() {
                let _ = File::open(parent.as_std_path()).and_then(|directory| directory.sync_all());
            }
        }
    }
}

fn revalidate_identity(
    path: &Utf8Path,
    expected: PathIdentity,
    label: &str,
) -> GenerationResult<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .map_err(|error| io_error(format!("failed to revalidate {label} {path}"), error))?;
    if metadata.file_type().is_symlink() {
        return Err(GenerationError::InvalidInput(format!(
            "{label} was replaced by a symlink: {path}"
        )));
    }
    let actual = PathIdentity::from_metadata(&metadata);
    if actual != expected {
        return Err(GenerationError::InvalidInput(format!(
            "{label} identity changed before promotion: {path}"
        )));
    }
    Ok(())
}

fn require_missing(path: &Utf8Path, label: &str) -> GenerationResult<()> {
    match fs::symlink_metadata(path.as_std_path()) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(GenerationError::InvalidInput(format!(
            "{label} appeared after ownership validation: {path}"
        ))),
        Err(error) => Err(io_error(
            format!("failed to revalidate {label} {path}"),
            error,
        )),
    }
}
