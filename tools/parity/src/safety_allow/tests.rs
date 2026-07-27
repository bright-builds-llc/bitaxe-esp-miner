use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn safety_allow_rejects_non_205_board() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["board"] = serde_json::json!("601");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "board must be 205");
}

#[test]
fn safety_allow_rejects_detector_port_mismatch() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "detector port mismatch");
}

#[test]
fn safety_allow_rejects_failed_board_info() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["board_info_status"] = serde_json::json!("failed");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "board-info must pass");
}

#[test]
fn safety_allow_rejects_package_identity_mismatch() {
    // Arrange
    let (manifest, mut package_manifest) = manifest_with_change(|_json| {});
    package_manifest["source_commit"] = serde_json::json!("stale-source");

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "package identity mismatch");
}

#[test]
fn safety_allow_active_claim_tiers_require_hardware_regression() {
    // Arrange
    let active_tiers = [
        "bounded-actuation",
        "fault-stimulus",
        "self-test-hardware",
        "load-stress",
        "runtime-display-input",
    ];

    for claim_tier in active_tiers {
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["claim_tier"] = serde_json::json!(claim_tier);
            json["evidence_class"] = serde_json::json!("hardware-smoke");
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "hardware-regression");
    }
}

#[test]
fn safety_allow_allows_failure_paths_fault_stimulus_with_hardware_regression() {
    // Arrange
    let (claim_tier, evidence_class) = ("fault-stimulus", "hardware-regression");
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("failure-paths");
        json["claim_tier"] = serde_json::json!(claim_tier);
        json["evidence_class"] = serde_json::json!(evidence_class);
        json["allowed_command"] =
                serde_json::json!("scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths");
        json["allowed_inputs"] = serde_json::json!({
            "stimulus": "fan-rpm-unavailable",
            "expected_fault": "fan_fault",
            "abort_condition": "missing_safe_state_marker",
            "restore_path": "just flash board=205 port=/dev/cu.usbmodem1101",
            "projection_status": "api-and-websocket-observed",
            "final_safe_state_marker": "safe_state: mining=disabled"
        });
        json["evidence_dir"] = serde_json::json!(
            "docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths"
        );
        json["checklist_rows"] = serde_json::json!(["SAFE-04"]);
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

#[test]
fn safety_allow_rejects_failure_paths_fault_stimulus_without_hardware_regression() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("failure-paths");
        json["claim_tier"] = serde_json::json!("fault-stimulus");
        json["evidence_class"] = serde_json::json!("hardware-smoke");
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "hardware-regression");
}

#[test]
fn safety_allow_rejects_failure_paths_unrelated_claim_tier() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("failure-paths");
        json["claim_tier"] = serde_json::json!("read-only-observation");
        json["evidence_class"] = serde_json::json!("hardware-smoke");
        json["allowed_command"] = serde_json::json!(
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths"
            );
        json["abort_conditions"] = serde_json::json!([]);
        json["recovery_steps"] = serde_json::json!([]);
        json["post_action_safe_state_markers"] = serde_json::json!([]);
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "does not allow claim_tier `read-only-observation`");
}

#[test]
fn safety_allow_rejects_failure_paths_fault_stimulus_missing_required_inputs() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("failure-paths");
        json["claim_tier"] = serde_json::json!("fault-stimulus");
        json["evidence_class"] = serde_json::json!("hardware-regression");
        json["allowed_command"] = serde_json::json!(
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths"
            );
        json["allowed_inputs"] = serde_json::json!({
            "restore_path": "just flash board=205 port=/dev/cu.usbmodem1101"
        });
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "allowed_inputs.stimulus");
    assert_error_contains(&report, "allowed_inputs.expected_fault");
    assert_error_contains(&report, "allowed_inputs.abort_condition");
    assert_error_contains(&report, "allowed_inputs.projection_status");
    assert_error_contains(&report, "allowed_inputs.final_safe_state_marker");
}

#[test]
fn safety_allow_allows_failure_paths_unsupported_pending_deferred_without_recovery() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("failure-paths");
        json["claim_tier"] = serde_json::json!("unsupported-pending");
        json["evidence_class"] = serde_json::json!("deferred");
        json["allowed_command"] =
                serde_json::json!("scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths");
        json["allowed_inputs"] = serde_json::json!({
            "blocked_by": "no compile-gated fault stimulus route"
        });
        json["abort_conditions"] = serde_json::json!([]);
        json["recovery_steps"] = serde_json::json!([]);
        json["post_action_safe_state_markers"] = serde_json::json!([]);
        json["evidence_dir"] = serde_json::json!(
            "docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths"
        );
        json["checklist_rows"] = serde_json::json!(["SAFE-04"]);
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert!(report.passed(), "{report:#?}");
}

#[test]
fn safety_allow_rejects_live_api_projection_without_explicit_target() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("live-api-websocket-telemetry");
        json["claim_tier"] = serde_json::json!("api-websocket-projection");
        json["evidence_class"] = serde_json::json!("hardware-smoke");
        json["allowed_command"] = serde_json::json!(
            "scripts/phase14-live-telemetry.sh --manifest allow.json --out-dir evidence/live-api"
        );
        json["allowed_inputs"] = serde_json::json!({
            "device_url_source": "missing",
            "network_scan": "disabled",
            "route_path": "/api/system/info",
            "duration_ms": 10000,
            "max_frames": 5
        });
        json["abort_conditions"] = serde_json::json!([]);
        json["recovery_steps"] = serde_json::json!([]);
        json["post_action_safe_state_markers"] = serde_json::json!([]);
    });

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(
        &report,
        "explicit DEVICE_URL or trusted raw origin-only target lock",
    );
}

#[test]
fn safety_allow_rejects_missing_procedure_scope_fields() {
    // Arrange
    let cases = [
        ("recovery_steps", "recovery_steps must not be empty"),
        (
            "post_action_safe_state_markers",
            "post_action_safe_state_markers must not be empty",
        ),
        ("redaction_reviewer", "redaction_reviewer must not be empty"),
        ("checklist_rows", "checklist_rows must not be empty"),
    ];

    for (field, expected_error) in cases {
        let (manifest, package_manifest) = manifest_with_change(|json| match field {
            "recovery_steps" => json["recovery_steps"] = serde_json::json!([]),
            "post_action_safe_state_markers" => {
                json["post_action_safe_state_markers"] = serde_json::json!([])
            }
            "redaction_reviewer" => json["redaction_reviewer"] = serde_json::json!(""),
            "checklist_rows" => json["checklist_rows"] = serde_json::json!([]),
            _ => unreachable!("test case field should be handled"),
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, expected_error);
    }
}

#[test]
fn safety_allow_allows_surface_claim_tiers_with_matching_evidence_class() {
    // Arrange
    let cases = [
            (
                "safe-baseline",
                "safe-baseline",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/safe-baseline",
                serde_json::json!({ "safe_state_marker": "safe_state: mining=disabled" }),
            ),
            (
                "safe-baseline",
                "read-only-observation",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/safe-baseline",
                serde_json::json!({ "safe_state_marker": "safe_state: mining=disabled" }),
            ),
            (
                "live-api-websocket-telemetry",
                "api-websocket-projection",
                "hardware-smoke",
                "scripts/phase14-live-telemetry.sh --manifest allow.json --out-dir evidence/live-api --device-url http://[redacted]",
                serde_json::json!({
                    "device_url_source": "explicit DEVICE_URL",
                    "network_scan": "disabled",
                    "route_path": "/api/system/info",
                    "duration_ms": 10000,
                    "max_frames": 5
                }),
            ),
            (
                "failure-paths",
                "safe-unavailable",
                "hardware-smoke",
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths",
                serde_json::json!({ "reason": "fault route unavailable" }),
            ),
            (
                "failure-paths",
                "unsupported-pending",
                "deferred",
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths",
                serde_json::json!({ "reason": "no production-safe bounded fault-stimulus route" }),
            ),
            (
                "parity-redaction",
                "parity-redaction",
                "workflow",
                "rg -n -i secret docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence",
                serde_json::json!({ "scan": "secret patterns" }),
            ),
        ];

    for (surface, claim_tier, evidence_class, allowed_command, allowed_inputs) in cases {
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!(surface);
            json["claim_tier"] = serde_json::json!(claim_tier);
            json["evidence_class"] = serde_json::json!(evidence_class);
            json["allowed_command"] = serde_json::json!(allowed_command);
            json["allowed_inputs"] = allowed_inputs;
            json["abort_conditions"] = serde_json::json!([]);
            json["recovery_steps"] = serde_json::json!([]);
            json["post_action_safe_state_markers"] = serde_json::json!([]);
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert!(
            report.passed(),
            "{surface}/{claim_tier} should pass: {report:#?}"
        );
    }
}

#[test]
fn safety_allow_renders_passed_cli_contract_for_valid_manifest() {
    // Arrange
    let cli_command =
        "bazel run //tools/parity:report -- safety-allow --manifest <valid-test-manifest>";
    let (manifest, package_manifest) = manifest_with_change(|_json| {});

    // Act
    let report = validate_safety_allow_manifest(&manifest, &package_manifest);
    let output = render_safety_allow_report(&manifest, &report);

    // Assert
    assert!(cli_command.contains("safety-allow --manifest"));
    assert!(report.passed(), "{output}");
    assert!(output.contains("safety_allow_status: passed"));
    assert!(output.contains("surface: voltage-control"));
    assert!(output.contains("claim_tier: bounded-actuation"));
    assert!(output.contains("evidence_class: hardware-regression"));
    assert!(output.contains("checklist_rows: PWR-003,PWR-005"));
}

#[test]
fn safety_allow_filters_reject_mismatched_surface_and_command() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_json| {});
    let documents = SafetyAllowDocuments {
        manifest,
        package_manifest,
    };
    let filters = SafetyAllowFilters {
        maybe_surface: Some("thermal-fan".to_owned()),
        maybe_allowed_command: Some("scripts/other.sh --manifest allow.json".to_owned()),
    };

    // Act
    let report = validate_safety_allow_documents(&documents, &filters);

    // Assert
    assert_error_contains(&report, "surface filter mismatch");
    assert_error_contains(&report, "allowed command filter mismatch");
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
    assert_eq!(documents.manifest.board, "205");
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
    r#"{
  "source_commit": "source-abc",
  "reference_commit": "reference-def"
}"#
}

fn valid_manifest_json_string(package_manifest: &str) -> String {
    r#"{
            "board": "205",
            "port": "/dev/cu.usbmodem1101",
            "detector_command": "just detect-ultra205",
            "detector_port": "/dev/cu.usbmodem1101",
            "board_info_command": "espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive",
            "board_info_status": "passed",
            "package_manifest": "__PACKAGE_MANIFEST__",
            "source_commit": "source-abc",
            "reference_commit": "reference-def",
            "surface": "voltage-control",
            "claim_tier": "bounded-actuation",
            "evidence_class": "hardware-regression",
            "allowed_command": "scripts/phase14-power-voltage.sh --manifest allow.json --surface voltage-control --out-dir evidence/voltage-control",
            "allowed_inputs": {
                "setpoint_mv": [1200]
            },
            "abort_conditions": [
                "detector_mismatch",
                "board_info_failure",
                "missing_safe_state_marker"
            ],
            "recovery_steps": [
                "just flash board=205 port=/dev/cu.usbmodem1101"
            ],
            "post_action_safe_state_markers": [
                "safe_state: mining=disabled",
                "hardware_control=disabled"
            ],
            "evidence_dir": "docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control",
            "redaction_reviewer": "phase-14-reviewer",
            "checklist_rows": ["PWR-003", "PWR-005"]
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
