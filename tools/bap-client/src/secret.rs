//! Sensitive setting values come only from a private file, never from the command
//! line, where process listings and shell history would keep them.

use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

/// Reads one value from a regular file the current user owns, readable by no one
/// else; a single trailing newline is dropped.
pub fn read_private_value(path: &Path) -> io::Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    // SAFETY: getuid has no preconditions and cannot fail.
    let user = unsafe { libc::getuid() };
    if !metadata.file_type().is_file()
        || metadata.uid() != user
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "value_file_must_be_private",
        ));
    }
    let text = fs::read_to_string(path)?;
    let value = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(&text);
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    fn write(dir: &Path, mode: u32, text: &str) -> std::path::PathBuf {
        let path = dir.join("value");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&path)
            .expect("file");
        file.write_all(text.as_bytes()).expect("write");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("mode");
        path
    }

    #[test]
    fn a_private_file_yields_its_value_without_the_newline() {
        // Arrange
        let dir = tempfile::tempdir().expect("dir");
        let path = write(dir.path(), 0o600, "network\n");
        // Act
        let value = read_private_value(&path);
        // Assert
        assert_eq!(value.expect("value"), "network");
    }

    #[test]
    fn a_group_or_world_readable_file_is_refused() {
        // Arrange
        let dir = tempfile::tempdir().expect("dir");
        let path = write(dir.path(), 0o644, "network\n");
        // Act
        let result = read_private_value(&path);
        // Assert
        assert_eq!(
            result.expect_err("refused").kind(),
            io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn a_symlink_is_refused() {
        // Arrange
        let dir = tempfile::tempdir().expect("dir");
        let target = write(dir.path(), 0o600, "network\n");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).expect("link");
        // Act
        let result = read_private_value(&link);
        // Assert
        assert!(result.is_err());
    }
}
