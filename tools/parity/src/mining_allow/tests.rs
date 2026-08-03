use super::*;

const REQUEST_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn mining_allow_accepts_typed_workflow() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_| {});

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

#[test]
fn mining_allow_rejects_wrong_workflow_and_digest() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["workflow"]["command"] = serde_json::json!("verify-hardware-surface");
        json["workflow"]["request_sha256"] = serde_json::json!("bad");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "workflow command must be verify-mining");
    assert_error_contains(&report, "64 lowercase hexadecimal");
}

#[test]
fn mining_allow_rejects_detector_and_package_mismatch() {
    // Arrange
    let (manifest, mut package_manifest) = manifest_with_change(|json| {
        json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
    });
    package_manifest["source_commit"] = serde_json::json!("stale-source");

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "detector port mismatch");
    assert_error_contains(&report, "package identity mismatch");
}

#[test]
fn mining_allow_requires_structured_constraints() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["constraints"] = serde_json::Value::Null;
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "constraints must be a JSON object");
}

#[test]
fn mining_allow_bounded_soak_is_bounded() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("bounded-soak");
        json["claim_tier"] = serde_json::json!("bounded-soak");
        json["evidence_class"] = serde_json::json!("soak");
        json["constraints"]["duration_seconds"] = serde_json::json!(601);
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "duration_seconds between 60 and 600");
}

#[test]
fn mining_allow_filters_require_identity_and_digest() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_| {});
    let documents = MiningAllowDocuments {
        manifest,
        package_manifest,
    };
    let filters = MiningAllowFilters {
        maybe_surface: Some("mining-smoke".to_owned()),
        maybe_workflow: Some("verify-mining".to_owned()),
        maybe_request_sha256: Some(REQUEST_SHA256.to_owned()),
    };

    // Act
    let report = validate_mining_allow_documents(&documents, &filters);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

fn assert_error_contains(report: &MiningAllowReport, expected: &str) {
    assert!(
        report
            .validation_errors
            .iter()
            .any(|error| error.contains(expected)),
        "expected validation error containing {expected:?}, got {report:#?}"
    );
}

fn manifest_with_change(change: impl FnOnce(&mut Value)) -> (MiningAllowManifest, Value) {
    let package_manifest: Value = serde_json::from_str(
        r#"{"source_commit":"source-abc","reference_commit":"reference-def"}"#,
    )
    .expect("package json");
    let mut manifest_json: Value = serde_json::from_str(&valid_manifest_json_string(
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    ))
    .expect("manifest json");
    change(&mut manifest_json);
    let manifest = serde_json::from_value(manifest_json).expect("valid manifest shape");
    (manifest, package_manifest)
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
      "surface":"mining-smoke",
      "claim_tier":"controlled-no-share",
      "evidence_class":"hardware-smoke",
      "workflow":{
        "schema_version":"bitaxe-workflow-identity-v1",
        "command":"verify-mining",
        "request_sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      },
      "constraints":{"pool_config":"local-owner-supplied","device_url":"explicit","duration_seconds":60},
      "abort_conditions":["detector_mismatch","board_info_failure","missing_trusted_wrapper_markers","redaction_uncertainty","unsafe_temperature_or_power","watchdog_unresponsive"],
      "recovery_steps":["same-package safe-state restore"],
      "post_action_safe_state_markers":["safe_state: mining=disabled","hardware_control=disabled","work_submission=disabled"],
      "prerequisite_artifacts":["docs/parity/evidence/mining/detector.json"],
      "evidence_dir":"docs/parity/evidence/mining/smoke",
      "redaction_reviewer":"reviewer",
      "checklist_rows":["STR-007","STR-008"]
    }"#
        .replace("__PACKAGE_MANIFEST__", package_manifest)
}
