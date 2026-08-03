use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

const REQUEST_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn safety_allow_accepts_typed_voltage_workflow() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_| {});

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

#[test]
fn safety_allow_rejects_command_for_wrong_surface() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["workflow"]["command"] = serde_json::json!("verify-mining");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "workflow command does not match");
}

#[test]
fn safety_allow_rejects_invalid_request_digest() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["workflow"]["request_sha256"] = serde_json::json!("NOT-A-DIGEST");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "64 lowercase hexadecimal");
}

#[test]
fn safety_allow_rejects_detector_or_package_mismatch() {
    // Arrange
    let (manifest, mut package_manifest) = manifest_with_change(|json| {
        json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
    });
    package_manifest["source_commit"] = serde_json::json!("stale-source");

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "detector port mismatch");
    assert_error_contains(&report, "package identity mismatch");
}

#[test]
fn safety_allow_active_claim_requires_complete_stop_contract() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["abort_conditions"] = serde_json::json!([]);
        json["recovery_steps"] = serde_json::json!([]);
        json["post_action_safe_state_markers"] = serde_json::json!([]);
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "recovery_steps must not be empty");
    assert_error_contains(&report, "abort_conditions must not be empty");
    assert_error_contains(&report, "post_action_safe_state_markers must not be empty");
}

#[test]
fn safety_allow_filters_use_workflow_identity_and_digest() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_| {});
    let documents = SafetyAllowDocuments {
        manifest,
        package_manifest,
    };
    let filters = SafetyAllowFilters {
        maybe_surface: Some("voltage-control".to_owned()),
        maybe_workflow: Some("verify-hardware-surface".to_owned()),
        maybe_request_sha256: Some(REQUEST_SHA256.to_owned()),
    };

    // Act
    let report = validate_safety_allow_documents(&documents, &filters);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

#[test]
fn safety_allow_loads_checked_in_style_json_from_temporary_directory() {
    // Arrange
    let fixture = SafetyAllowFixture::new(valid_manifest_json_string(
        "package/bitaxe-ultra205-package.json",
    ));

    // Act
    let documents = load_safety_allow_documents(&fixture.workspace_dir, &fixture.manifest_path)
        .expect("fixture documents should load");
    let report = validate_safety_allow_manifest(&documents.manifest, &documents.package_manifest);

    // Assert
    assert!(report.passed(), "{report:#?}");
    std::fs::remove_dir_all(fixture.workspace_dir.as_std_path()).expect("fixture cleanup");
}

fn assert_error_contains(report: &SafetyAllowReport, expected: &str) {
    assert!(
        report
            .validation_errors
            .iter()
            .any(|error| error.contains(expected)),
        "expected validation error containing {expected:?}, got {report:#?}"
    );
}

fn manifest_with_change(change: impl FnOnce(&mut Value)) -> (SafetyAllowManifest, Value) {
    let package_manifest =
        serde_json::from_str(package_manifest_json_string()).expect("package json");
    let mut manifest_json: Value = serde_json::from_str(&valid_manifest_json_string(
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    ))
    .expect("manifest json");
    change(&mut manifest_json);
    let manifest = serde_json::from_value(manifest_json).expect("valid manifest shape");
    (manifest, package_manifest)
}

fn package_manifest_json_string() -> &'static str {
    r#"{"source_commit":"source-abc","reference_commit":"reference-def"}"#
}

fn valid_manifest_json_string(package_manifest: &str) -> String {
    r#"{
      "board":"205",
      "port":"/dev/cu.usbmodem1101",
      "detector_command":"just detect-ultra205",
      "detector_port":"/dev/cu.usbmodem1101",
      "board_info_command":"espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive",
      "board_info_status":"passed",
      "package_manifest":"__PACKAGE_MANIFEST__",
      "source_commit":"source-abc",
      "reference_commit":"reference-def",
      "surface":"voltage-control",
      "claim_tier":"bounded-actuation",
      "evidence_class":"hardware-regression",
      "workflow":{
        "schema_version":"bitaxe-workflow-identity-v1",
        "command":"verify-hardware-surface",
        "request_sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      },
      "constraints":{"setpoint_mv":[1200]},
      "abort_conditions":["detector_mismatch","board_info_failure","missing_safe_state_marker"],
      "recovery_steps":["same-package safe-state restore"],
      "post_action_safe_state_markers":["safe_state: mining=disabled","hardware_control=disabled"],
      "evidence_dir":"docs/parity/evidence/safety/voltage-control",
      "redaction_reviewer":"reviewer",
      "checklist_rows":["PWR-003","PWR-005"]
    }"#
        .replace("__PACKAGE_MANIFEST__", package_manifest)
}

struct SafetyAllowFixture {
    workspace_dir: Utf8PathBuf,
    manifest_path: Utf8PathBuf,
}

impl SafetyAllowFixture {
    fn new(manifest_json: String) -> Self {
        let workspace_dir = unique_temp_dir();
        let package_dir = workspace_dir.join("package");
        std::fs::create_dir_all(package_dir.as_std_path()).expect("package dir");
        std::fs::write(
            package_dir
                .join("bitaxe-ultra205-package.json")
                .as_std_path(),
            package_manifest_json_string(),
        )
        .expect("package manifest");
        let manifest_path = workspace_dir.join("allow.json");
        std::fs::write(manifest_path.as_std_path(), manifest_json).expect("allow manifest");
        Self {
            workspace_dir,
            manifest_path,
        }
    }
}

fn unique_temp_dir() -> Utf8PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bitaxe-safety-allow-{nanos}"));
    let utf8_path = Utf8PathBuf::from_path_buf(path).expect("temp path should be UTF-8");
    std::fs::create_dir_all(utf8_path.as_std_path()).expect("temp dir");
    utf8_path
}
