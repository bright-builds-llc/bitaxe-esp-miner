use super::*;

#[test]
fn release_ota_verified_guard_rejects_filesystem_verified_without_live_static_recovery() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | verified | workflow | Package evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "FS-001", "live recovery/static smoke");
}

#[test]
fn release_ota_verified_guard_rejects_firmware_ota_verified_without_hardware() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | workflow | Firmware OTA compile and package evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "OTA-001", "hardware-smoke or hardware-regression");
}

#[test]
fn release_ota_verified_guard_rejects_otawww_verified_without_interrupted_update_regression() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-002 | AxeOS OTAWWW route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Live static update smoke only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "OTA-002", "interrupted-update");
    assert_validation_error_contains(&errors, "OTA-002", "hardware-regression");
}

#[test]
fn release_ota_verified_guard_rejects_partition_verified_from_package_only_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-001 | Partition layout | `reference/esp-miner/partitions.csv` | `firmware/bitaxe` | verified | workflow | Package evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "REL-001", "release-sensitive");
}

#[test]
fn release_ota_verified_guard_rejects_sdk_config_verified_from_unit_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-002 | SDK config parity | `reference/esp-miner/sdkconfig.defaults` | `firmware/bitaxe` | verified | unit | SDK config fixture evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "REL-002", "release-sensitive");
}

#[test]
fn release_ota_verified_guard_rejects_release_image_verified_without_gate_and_package() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | verified | workflow | Package workflow evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "REL-003", "release-gate");
    assert_validation_error_contains(&errors, "REL-003", "provenance");
    assert_validation_error_contains(&errors, "REL-003", "package workflow");
}

#[test]
fn release_image_verified_requires_rel08_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | verified | workflow | release-gate provenance package workflow evidence is present, but only package output was reviewed. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "REL-003", "rollback");
    assert_validation_error_contains(&errors, "REL-003", "recovery");
    assert_validation_error_contains(&errors, "REL-003", "large erase");
    assert_validation_error_contains(&errors, "REL-003", "failed update");
    assert_validation_error_contains(&errors, "REL-003", "interrupted-update");
}

#[test]
fn firmware_ota_verified_requires_valid_invalid_and_boot_validation() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Ultra 205 route registration and OTA compile evidence only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "OTA-001", "valid OTA");
    assert_validation_error_contains(&errors, "OTA-001", "invalid image rejection");
    assert_validation_error_contains(&errors, "OTA-001", "boot-validation");
}

#[test]
fn filesystem_verified_requires_live_static_recovery_surfaces() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Live recovery and live static smoke passed on Ultra 205. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "FS-001", "/assets/app.css.gz");
    assert_validation_error_contains(&errors, "FS-001", "missing static redirect");
    assert_validation_error_contains(&errors, "FS-001", "/recovery");
}

#[test]
fn release_ota_verified_guard_rejects_blocker_language_that_contains_required_terms() {
    // Arrange
    let cases = [
        (
            "FS-001",
            "SPIFFS/filesystem behavior",
            "hardware-smoke",
            "live static not run; /assets/app.css.gz blocked; missing static redirect pending; /recovery no reachable DEVICE_URL; unverified smoke.",
        ),
        (
            "OTA-001",
            "Firmware OTA route",
            "hardware-smoke",
            "valid OTA not run; invalid image rejection blocked; boot-validation pending.",
        ),
        (
            "OTA-002",
            "AxeOS OTAWWW route",
            "hardware-regression",
            "interrupted-update not run because no reachable DEVICE_URL.",
        ),
        (
            "REL-003",
            "Release image behavior",
            "workflow",
            "release-gate provenance package workflow recorded; rollback not run; recovery blocked; large erase pending; failed update unverified; interrupted-update no reachable DEVICE_URL.",
        ),
    ];

    for (id, surface, evidence, notes) in cases {
        let checklist = format!(
            r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| {id} | {surface} | reference path | rust target | verified | {evidence} | {notes} |
"#
        );
        let rows = parse_checklist(&checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, id, "blocker terms");
    }
}

#[test]
fn deferred_scope_verified_rows_reject_ultra205_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-002 | Deferred Gamma 601 defaults | `reference/esp-miner/config-601.cvs` | `crates/bitaxe-config` | verified | hardware-smoke | Ultra 205 evidence was reused for a non-205 board. |
| ASIC-008 | BM1370 parity | `reference/esp-miner/components/asic/bm1370.c` | `crates/bitaxe-asic` | verified | hardware-smoke | Ultra 205 evidence was reused for BM1370. |
| STR-005 | Stratum v2 protocol | `reference/esp-miner/components/stratum_v2/*.c` | `crates/bitaxe-stratum` | verified | hardware-smoke | Ultra 205 Stratum v1 evidence was reused. |
| BAP-001 | BAP interface initialization | `reference/esp-miner/main/bap/bap.c` | `firmware/bitaxe` | verified | hardware-smoke | Ultra 205 evidence was reused for BAP. |
| V2-FACTORY-001 | all-board factory image matrix | `reference/esp-miner` | `tools/xtask` | verified | hardware-smoke | Ultra 205 evidence was reused for an all-board release matrix. |
| V2-UI-001 | Angular UI rewrite | `reference/esp-miner/main/http_server/axe-os` | `firmware/bitaxe/static/www` | verified | hardware-smoke | Ultra 205 evidence was reused for an Angular rewrite. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "CFG-002", "Ultra 205 evidence");
    assert_validation_error_contains(&errors, "ASIC-008", "Ultra 205 evidence");
    assert_validation_error_contains(&errors, "STR-005", "Ultra 205 evidence");
    assert_validation_error_contains(&errors, "BAP-001", "Ultra 205 evidence");
    assert_validation_error_contains(&errors, "V2-FACTORY-001", "Ultra 205 evidence");
    assert_validation_error_contains(&errors, "V2-UI-001", "Ultra 205 evidence");
}

#[test]
fn release_ota_verified_guard_allows_implemented_package_evidence_below_verified() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | implemented | unit,workflow | Package evidence only; live smoke pending. |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | implemented | workflow | Firmware OTA compile and package evidence only. |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | implemented | workflow | Release-gate and package workflow evidence exist; hardware remains pending. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn phase26_verified_telemetry_row_rejects_missing_summary_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | Phase 26 redaction-review.md redaction_status: passed exact_non_claims no_request_time_fabrication empty_without_parsed_share_outcome. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(
        &errors,
        "API-002",
        "phase26 verified row missing summary evidence",
    );
}

#[test]
fn phase26_verified_row_rejects_blocked_or_pending_language() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-006 | WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed but no reachable DEVICE_URL and blocked proof remain. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "API-006", "phase26 blocked verified row");
}

#[test]
fn phase26_verified_row_rejects_missing_redaction_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md no_request_time_fabrication runtime_projection_marker_only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "STAT-002", "phase26 redaction evidence");
}

#[test]
fn phase26_verified_row_rejects_missing_exact_non_claims() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-006 | WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed projection-backed telemetry closure. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "API-006", "exact_non_claims");
}

#[test]
fn phase26_guard_accepts_conservative_rows_and_evd08_closure() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `firmware/bitaxe` | implemented | unit,api-compare,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed exact_non_claims projection-backed. Accepted shares remain non-claims. |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-api`, `firmware/bitaxe` | implemented | unit,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed no_request_time_fabrication runtime_projection_marker_only. |
| STAT-003 | Scoreboard | `reference/esp-miner/main/tasks/scoreboard.c` | `crates/bitaxe-api` | implemented | unit,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed empty_without_parsed_share_outcome exact_non_claims. |
| EVD-08 | Phase 26 exact telemetry closure | `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md` | `docs/parity/checklist.md`, `tools/parity/src/report/validation/phase26.rs` | verified | workflow | API-11 API-12 API-13 EVD-08 phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed exact_non_claims just parity guard passed. Full active voltage and unbounded stress remain non-claims. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn phase26_scoreboard_accepts_current_v2_live_result() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STAT-003 | Scoreboard | `reference/esp-miner/main/tasks/scoreboard.c` | `crates/bitaxe-api` | verified | unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression | Verified Ultra 205 v2 evidence; see docs/parity/work-plans/20260820T224453Z-STAT-003/RESULT.md and docs/parity/evidence/stat003-scoreboard/summary.md. redaction_status: passed. exact_non_claims: original manifest bytes, arbitrary pools, other boards/ASICs, UART/BAP, pins, and electrical behavior. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}
