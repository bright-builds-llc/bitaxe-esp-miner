use std::fs;
use std::os::unix::fs::PermissionsExt;

use camino::Utf8Path;

use super::super::session::create_supervisor_root;

fn parent(mode: u32) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(mode)).expect("parent mode");
    directory
}

#[test]
fn creates_a_fresh_mode_0700_root_under_a_private_parent() {
    // Arrange
    let parent = parent(0o700);
    let root = Utf8Path::from_path(parent.path())
        .expect("UTF-8")
        .join("attempt");

    // Act
    create_supervisor_root(&root).expect("fresh root");

    // Assert
    let mode = fs::metadata(root.as_std_path())
        .expect("root")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn refuses_an_existing_root_or_a_permissive_parent() {
    // Arrange
    let private = parent(0o700);
    let existing = Utf8Path::from_path(private.path())
        .expect("UTF-8")
        .join("attempt");
    fs::create_dir(existing.as_std_path()).expect("existing root");
    let permissive = parent(0o755);
    let under_permissive = Utf8Path::from_path(permissive.path())
        .expect("UTF-8")
        .join("attempt");

    // Act
    let existing_result = create_supervisor_root(&existing);
    let permissive_result = create_supervisor_root(&under_permissive);

    // Assert
    assert!(existing_result.is_err() && permissive_result.is_err());
}
