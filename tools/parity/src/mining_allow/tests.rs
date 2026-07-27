use super::*;

#[test]
fn mining_allow_rejects_non_205_board() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["board"] = serde_json::json!("601");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "board must be 205");
}

#[test]
fn mining_allow_rejects_wrong_detector_command() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["detector_command"] = serde_json::json!("just detect-any-board");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "detector command must be just detect-ultra205");
}

#[test]
fn mining_allow_rejects_detector_port_mismatch() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "detector port mismatch");
}

#[test]
fn mining_allow_rejects_failed_board_info() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["board_info_status"] = serde_json::json!("failed");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "board-info must pass");
}

#[test]
fn mining_allow_rejects_package_identity_mismatch() {
    // Arrange
    let (manifest, mut package_manifest) = manifest_with_change(|_json| {});
    package_manifest["source_commit"] = serde_json::json!("stale-source");

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "package identity mismatch");
}

#[test]
fn mining_allow_accepts_only_phase15_surfaces() {
    // Arrange
    let allowed_surface_claims = [
            (
                "bm1366-chip-detect",
                "diagnostic-chip-detect",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/chip-detect",
            ),
            (
                "bm1366-work-result",
                "diagnostic-work-result",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/work-result",
            ),
            (
                "mining-smoke",
                "controlled-no-share",
                "hardware-smoke",
                "scripts/phase15-controlled-mining.sh --manifest allow.json --surface mining-smoke --out-dir evidence/mining-smoke --chip-detect-summary chip.md --work-result-summary work.md",
            ),
            (
                "bounded-soak",
                "unsupported-pending",
                "workflow",
                "scripts/phase15-controlled-mining.sh --manifest allow.json --surface bounded-soak --duration-seconds 120 --out-dir evidence/bounded-soak --chip-detect-summary chip.md --work-result-summary work.md",
            ),
            (
                "parity-redaction",
                "parity-redaction",
                "workflow",
                "rg -n -i secret docs/parity/evidence/phase-15-bm1366-mining-evidence-completion",
            ),
        ];

    for (surface, claim_tier, evidence_class, allowed_command) in allowed_surface_claims {
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!(surface);
            json["claim_tier"] = serde_json::json!(claim_tier);
            json["evidence_class"] = serde_json::json!(evidence_class);
            json["allowed_command"] = serde_json::json!(allowed_command);
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert!(report.passed(), "{surface} should pass: {report:#?}");
    }

    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("voltage-control");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "surface `voltage-control` is not allowed");
}

#[test]
fn mining_allow_rejects_surface_claim_tier_mismatch() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["surface"] = serde_json::json!("bm1366-chip-detect");
        json["claim_tier"] = serde_json::json!("controlled-no-share");
        json["allowed_command"] = serde_json::json!(
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/chip-detect"
            );
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "does not allow claim_tier `controlled-no-share`");
}

#[test]
fn mining_allow_rejects_unapproved_or_unsafe_allowed_command() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["allowed_command"] =
            serde_json::json!("espflash erase-flash --port /dev/cu.usbmodem1101");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "prohibited token `erase-flash`");
    assert_error_contains(&report, "approved mining evidence wrapper");
}

#[test]
fn mining_allow_documents_require_allowed_command_filter() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|_json| {});
    let documents = MiningAllowDocuments {
        manifest,
        package_manifest,
    };
    let filters = MiningAllowFilters {
        maybe_surface: Some("mining-smoke".to_owned()),
        maybe_allowed_command: None,
    };

    // Act
    let report = validate_mining_allow_documents(&documents, &filters);

    // Assert
    assert_error_contains(&report, "allowed command filter is required");
}

#[test]
fn mining_allow_live_pool_smoke_requires_approved_pool_input_category() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["claim_tier"] = serde_json::json!("live-pool-smoke");
        json["allowed_inputs"]["pool_config"] = serde_json::json!("private");
        json["allowed_inputs"]["device_url"] = serde_json::json!("inferred");
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "pool_config");
    assert_error_contains(&report, "device_url");
    assert_error_contains(&report, "--device-url");
}

#[test]
fn mining_allow_bounded_soak_requires_duration_and_safe_stop_contract() {
    // Arrange
    let (manifest, package_manifest) = manifest_with_change(|json| {
        json["claim_tier"] = serde_json::json!("bounded-soak");
        json["evidence_class"] = serde_json::json!("soak");
        json["allowed_inputs"]["duration_seconds"] = serde_json::json!(601);
        json["abort_conditions"] = serde_json::json!(["detector_mismatch"]);
        json["recovery_steps"] = serde_json::json!([]);
        json["post_action_safe_state_markers"] = serde_json::json!(["safe_state: mining=disabled"]);
    });

    // Act
    let report = validate_mining_allow_manifest(&manifest, &package_manifest);

    // Assert
    assert_error_contains(&report, "duration_seconds");
    assert_error_contains(
        &report,
        "abort_conditions must contain `board_info_failure`",
    );
    assert_error_contains(&report, "recovery_steps must not be empty");
    assert_error_contains(
        &report,
        "post_action_safe_state_markers must contain `hardware_control=disabled`",
    );
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
            "surface": "mining-smoke",
            "claim_tier": "controlled-no-share",
            "evidence_class": "hardware-smoke",
            "allowed_command": "scripts/phase15-controlled-mining.sh --manifest allow.json --surface mining-smoke --out-dir evidence/mining-smoke --chip-detect-summary chip.md --work-result-summary work.md",
            "allowed_inputs": {
                "pool_config": "disposable-or-non-secret",
                "device_url": "explicit",
                "duration_seconds": 60
            },
            "abort_conditions": [
                "detector_mismatch",
                "board_info_failure",
                "missing_trusted_wrapper_markers",
                "redaction_uncertainty",
                "unsafe_temperature_or_power",
                "watchdog_unresponsive"
            ],
            "recovery_steps": [
                "just flash board=205 port=/dev/cu.usbmodem1101"
            ],
            "post_action_safe_state_markers": [
                "safe_state: mining=disabled",
                "hardware_control=disabled",
                "work_submission=disabled"
            ],
            "prerequisite_artifacts": [
                "docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/detector.json"
            ],
            "evidence_dir": "docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke",
            "redaction_reviewer": "phase-15-reviewer",
            "checklist_rows": ["STR-007", "STR-008"]
        }"#
        .replace("__PACKAGE_MANIFEST__", package_manifest)
}
