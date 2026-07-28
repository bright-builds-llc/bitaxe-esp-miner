use super::*;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn admitted_result_file_requires_regular_mode_0600_known_json() {
    // Arrange
    let fixture = EffectResultFileFixture::new();
    let document = effect_result_document(
        "exact_package_flash",
        "completed",
        None,
        &"a".repeat(64),
        &"b".repeat(64),
    );
    fixture.write(&document, 0o600);

    // Act
    let result = maybe_read_effect_result(&fixture.path);

    // Assert
    let result = result.expect("valid closed result file");
    assert_eq!(result.schema_version, "phase36-effect-result-v1");
    assert_eq!(result.operation, Phase36AllowedOperation::ExactPackageFlash);
    assert_eq!(result.status, Phase36EffectStatus::Completed);
    assert_eq!(result.failure, None);
}

#[test]
fn result_file_admission_fails_closed_for_missing_wrong_mode_and_invalid_json() {
    // Arrange
    let missing = EffectResultFileFixture::new();
    let wrong_mode = EffectResultFileFixture::new();
    wrong_mode.write(b"{}", 0o644);
    let malformed = EffectResultFileFixture::new();
    malformed.write(b"{", 0o600);
    let unknown_field = EffectResultFileFixture::new();
    let mut document: serde_json::Value = serde_json::from_slice(&effect_result_document(
        "cleanup",
        "failed_no_device_effect",
        Some("cleanup_failed"),
        &"a".repeat(64),
        &"b".repeat(64),
    ))
    .expect("fixture json");
    document["unexpected"] = serde_json::json!(true);
    unknown_field.write(
        &serde_json::to_vec(&document).expect("fixture serialization"),
        0o600,
    );

    // Act
    let results = [
        maybe_read_effect_result(&missing.path),
        maybe_read_effect_result(&wrong_mode.path),
        maybe_read_effect_result(&malformed.path),
        maybe_read_effect_result(&unknown_field.path),
    ];

    // Assert
    assert!(results.into_iter().all(|result| result.is_none()));
}

#[test]
fn result_file_admission_rejects_directories_and_symlinks() {
    // Arrange
    let directory = EffectResultFileFixture::new();
    std::fs::create_dir(&directory.path).expect("result directory");
    std::fs::set_permissions(&directory.path, std::fs::Permissions::from_mode(0o700))
        .expect("result directory permissions");
    let symlink_fixture = EffectResultFileFixture::new();
    let target = symlink_fixture.root.join("target.json");
    std::fs::write(&target, b"{}").expect("symlink target");
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600))
        .expect("symlink target permissions");
    symlink(&target, &symlink_fixture.path).expect("result symlink");

    // Act
    let directory_result = maybe_read_effect_result(&directory.path);
    let symlink_result = maybe_read_effect_result(&symlink_fixture.path);

    // Assert
    assert!(directory_result.is_none());
    assert!(symlink_result.is_none());
}

fn effect_result_document(
    operation: &str,
    status: &str,
    maybe_failure: Option<&str>,
    package_digest: &str,
    factory_digest: &str,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema_version": "phase36-effect-result-v1",
        "operation": operation,
        "status": status,
        "failure": maybe_failure,
        "package_identity_digest": package_digest,
        "factory_image_digest": factory_digest,
    }))
    .expect("effect result fixture serialization")
}

struct EffectResultFileFixture {
    root: Utf8PathBuf,
    path: Utf8PathBuf,
}

impl EffectResultFileFixture {
    fn new() -> Self {
        static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "phase36-effect-result-{}-{nanos}-{fixture_id}",
            std::process::id()
        )))
        .expect("UTF-8 temp path");
        std::fs::create_dir(&root).expect("fixture root");
        let path = root.join("effect-result.json");
        Self { root, path }
    }

    fn write(&self, bytes: &[u8], mode: u32) {
        std::fs::write(&self.path, bytes).expect("effect result file");
        std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(mode))
            .expect("effect result permissions");
    }
}

impl Drop for EffectResultFileFixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("fixture cleanup");
    }
}
