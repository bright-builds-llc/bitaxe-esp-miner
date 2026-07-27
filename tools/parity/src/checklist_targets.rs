//! Typed validation for Rust-owned parity checklist targets.

use std::fs;

use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChecklistTarget {
    RepoPath(Utf8PathBuf),
    BazelLabel { package: Utf8PathBuf },
}

pub(crate) fn validate_targets(workspace: &Utf8Path, target_cell: &str) -> Vec<String> {
    let tokens = code_spans(target_cell);
    if tokens.is_empty() {
        return vec!["Rust-Owned Target must contain at least one code span".to_owned()];
    }
    tokens
        .into_iter()
        .filter_map(|token| {
            match parse_target(&token).and_then(|target| validate_target(workspace, &target)) {
                Ok(()) => None,
                Err(error) => Some(format!("{token}: {error}")),
            }
        })
        .collect()
}

fn code_spans(cell: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut remainder = cell;
    while let Some(start) = remainder.find('`') {
        let after_start = &remainder[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        spans.push(after_start[..end].to_owned());
        remainder = &after_start[end + 1..];
    }
    spans
}

fn parse_target(token: &str) -> Result<ChecklistTarget, String> {
    if let Some(label) = token.strip_prefix("//") {
        let Some((package, name)) = label.split_once(':') else {
            return Err("Bazel label must use //package:target syntax".to_owned());
        };
        if package.is_empty()
            || name.is_empty()
            || !package.split('/').all(valid_segment)
            || !valid_segment(name)
        {
            return Err("Bazel label contains an invalid package or target".to_owned());
        }
        return Ok(ChecklistTarget::BazelLabel {
            package: Utf8PathBuf::from(package),
        });
    }
    let path = Utf8PathBuf::from(token);
    if path.is_absolute()
        || path.as_str().is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                camino::Utf8Component::ParentDir | camino::Utf8Component::CurDir
            )
        })
    {
        return Err("repository target must be a normalized relative path".to_owned());
    }
    Ok(ChecklistTarget::RepoPath(path))
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._+-".contains(character))
}

fn validate_target(workspace: &Utf8Path, target: &ChecklistTarget) -> Result<(), String> {
    let relative = match target {
        ChecklistTarget::RepoPath(path) => path,
        ChecklistTarget::BazelLabel { package } => {
            let build = workspace.join(package).join("BUILD.bazel");
            validate_existing_path(workspace, &build)?;
            return Ok(());
        }
    };
    validate_existing_path(workspace, &workspace.join(relative))
}

fn validate_existing_path(workspace: &Utf8Path, path: &Utf8Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path.as_std_path()).map_err(|_| "target does not exist".to_owned())?;
    if metadata.file_type().is_symlink() {
        return Err("target must not be a symbolic link".to_owned());
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err("target must be a regular file or directory".to_owned());
    }
    let canonical_workspace = canonical(workspace)?;
    let canonical_target = canonical(path)?;
    if !canonical_target.starts_with(&canonical_workspace) {
        return Err("target resolves outside the workspace".to_owned());
    }
    Ok(())
}

fn canonical(path: &Utf8Path) -> Result<Utf8PathBuf, String> {
    let canonical = fs::canonicalize(path.as_std_path())
        .map_err(|_| "target could not be canonicalized".to_owned())?;
    Utf8PathBuf::from_path_buf(canonical).map_err(|_| "target path is not UTF-8".to_owned())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory {
        path: Utf8PathBuf,
    }

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "bitaxe-parity-{label}-{}-{ordinal}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self {
                path: Utf8PathBuf::from_path_buf(path).expect("UTF-8 temp path"),
            }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.as_std_path()).expect("remove test directory");
        }
    }

    #[test]
    fn existing_file_directory_and_bazel_package_pass() {
        // Arrange
        let directory = TestDirectory::new("valid-targets");
        let workspace = &directory.path;
        fs::create_dir_all(workspace.join("pkg").as_std_path()).expect("package directory");
        fs::write(workspace.join("pkg/file.rs").as_std_path(), "").expect("source file");
        fs::write(workspace.join("pkg/BUILD.bazel").as_std_path(), "").expect("BUILD file");

        // Act
        let errors = validate_targets(workspace, "`pkg/file.rs`, `pkg`, `//pkg:firmware`");

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_traversal_malformed_and_symlink_targets_fail() {
        // Arrange
        let directory = TestDirectory::new("invalid-targets");
        let workspace = &directory.path;
        let outside = TestDirectory::new("outside-targets");
        symlink(
            outside.path.as_std_path(),
            workspace.join("link").as_std_path(),
        )
        .expect("symlink");

        // Act
        let cases = [
            "`missing.rs`",
            "`../outside`",
            "`//missing-label`",
            "`link`",
        ];

        // Assert
        for case in cases {
            assert!(!validate_targets(workspace, case).is_empty(), "{case}");
        }
    }
}
