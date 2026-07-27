use super::*;

#[test]
fn phase28_verified_str09_rejects_missing_summary_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | verified | unit,workflow | redaction-review.md redaction_status: passed exact_non_claims accepted share hardware proof. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(
        &errors,
        "STR-09",
        "phase28 verified row missing summary evidence",
    );
}

#[test]
fn phase28_verified_str09_rejects_blocked_safe_prerequisite() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims share_outcome: blocked_safe_prerequisite. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "STR-09", "blocked_safe_prerequisite");
}

#[test]
fn phase28_verified_cfg07_rejects_verified_status() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `scripts/phase23-redacted-operator-evidence.sh` | verified | workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims pool_config: local-owner-supplied. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "CFG-07", "CFG-07 must remain below verified");
}

#[test]
fn phase28_verified_safe10_rejects_without_live_safety_hardware_proof() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SAFE-10 | Production mining prerequisite readiness | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-safety` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims consolidation only. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(
        &errors,
        "SAFE-10",
        "detector-gated live safety hardware proof",
    );
}

#[test]
fn phase28_verified_row_rejects_missing_redaction_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-09 | BM1366 diagnostic and production mode separation | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md exact_non_claims live socket success hardware-regression asic bridge correlation. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "ASIC-09", "phase28 redaction evidence");
}

#[test]
fn phase28_verified_row_rejects_missing_exact_non_claims() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-10 | Pool-derived BM1366 production work registry | `reference/esp-miner/components/stratum/mining.c` | `crates/bitaxe-stratum` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed live socket success hardware-regression asic bridge correlation. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "ASIC-10", "exact_non_claims");
}

#[test]
fn phase28_guard_accepts_conservative_rows() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md redaction-review.md redaction_status: passed exact_non_claims share_outcome: blocked_safe_prerequisite below verified. |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `scripts/phase23-redacted-operator-evidence.sh` | implemented | workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims below verified category labels only. |
| SAFE-10 | Production mining prerequisite readiness | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-safety` | implemented | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md phase-22-claim-ladder-and-safety-preconditions/safety-preconditions.md exact_non_claims below verified. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}
