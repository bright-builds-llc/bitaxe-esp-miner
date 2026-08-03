use super::*;

#[test]
fn parses_markdown_checklist_rows() {
    // Arrange
    let checklist = CHECKLIST;

    // Act
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Assert
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "WF-001");
    assert_eq!(rows[0].status, "implemented");
    assert_eq!(rows[0].evidence, "pending");
    assert_eq!(rows[0].reference_breadcrumb, "reference/esp-miner");
    assert_eq!(rows[0].rust_owned_target, "tools/xtask/src/main.rs");
}

#[test]
fn json_output_includes_reference_commit() {
    // Arrange
    let rows = parse_checklist(CHECKLIST).expect("checklist should parse");
    let report = ParityReport::new("abc123".to_owned(), rows);

    // Act
    let output = render_report(&report, ReportFormat::Json).expect("json should render");
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("output should be valid json");

    // Assert
    assert_eq!(parsed["reference_commit"], "abc123");
    assert_eq!(parsed["rows"][0]["id"], "WF-001");
}

#[test]
fn release_evidence_validation_paths_resolve_relative_inputs_under_workspace() {
    // Arrange
    let environment = LocalEnvironment {
        workspace_dir: Utf8PathBuf::from("/tmp/bitaxe-workspace"),
    };
    let args = ReleaseEvidenceArgs {
        manifest: Utf8PathBuf::from("docs/evidence/package.json"),
        evidence_root: Utf8PathBuf::from("docs/evidence"),
        maybe_flash_evidence_json: Some(Utf8PathBuf::from("docs/evidence/flash.json")),
        maybe_redaction_review: None,
        require_redaction_passed: false,
        allow_post_source_evidence_commits: false,
    };

    // Act
    let (evidence_root, maybe_flash_evidence_json_path) =
        release_evidence_validation_paths(&args, &environment);

    // Assert
    assert_eq!(
        evidence_root,
        Utf8PathBuf::from("/tmp/bitaxe-workspace/docs/evidence")
    );
    assert_eq!(
        maybe_flash_evidence_json_path,
        Some(Utf8PathBuf::from(
            "/tmp/bitaxe-workspace/docs/evidence/flash.json"
        ))
    );
}

#[test]
fn verified_rows_with_pending_evidence_are_invalid() {
    // Arrange
    let checklist = CHECKLIST.replace("implemented | pending", "verified | pending");
    let rows = parse_checklist(&checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("pending evidence"));
}

#[test]
fn safety_critical_verified_rows_require_hardware_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | verified | unit | Safety-critical. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "PWR-003", "hardware-smoke or hardware-regression");
}

#[test]
fn safety_critical_notes_require_hardware_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-001 | ASIC reset behavior | `reference/esp-miner/main/power/asic_reset.c` | `firmware/bitaxe` | verified | unit | Safety-critical; requires hardware evidence. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "PWR-001", "hardware-smoke or hardware-regression");
}

#[test]
fn safety_critical_self_test_verified_rows_require_hardware_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SELF-001 | Self-test lifecycle | `reference/esp-miner/main/self_test/self_test.c` | `crates/bitaxe-safety`, `firmware/bitaxe` | verified | unit | Self-test hardware requires Ultra 205 hardware smoke before verification. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "SELF-001", "hardware-smoke or hardware-regression");
}

#[test]
fn safety_critical_runtime_input_display_verified_rows_require_hardware_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-003 | Input behavior | `reference/esp-miner/main/input.c` | `firmware/bitaxe` | verified | workflow | Runtime input and runtime display hardware-control rows require hardware-smoke evidence. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "UI-003", "hardware-smoke or hardware-regression");
}

#[test]
fn safety_critical_implemented_rows_do_not_require_hardware_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-safety/src/thermal.rs` | implemented | unit | Pure PID behavior is covered by unit tests; hardware fan and thermal verification remains separate. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn active_safety_control_verified_rows_require_hardware_regression() {
    // Arrange
    let active_ids = [
        "PWR-001", "PWR-002", "PWR-003", "PWR-005", "ASIC-007", "THR-001", "THR-002", "SELF-001",
        "UI-003",
    ];

    for active_id in active_ids {
        let checklist = format!(
            r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| {active_id} | Active safety-control row | `reference/esp-miner/main/safety.c` | `firmware/bitaxe` | verified | hardware-smoke | Active hardware-control behavior cannot be proven by broad smoke evidence. |
"#
        );
        let rows = parse_checklist(&checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            active_id,
            "requires hardware-regression evidence",
        );
    }
}

#[test]
fn active_safety_control_allows_hardware_regression_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | verified | hardware-regression | Active voltage regression passed. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn active_safety_control_allows_read_only_hardware_smoke_rows() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-006 | INA260 power telemetry freshness | `reference/esp-miner/main/power/INA260.c` | `firmware/bitaxe` | verified | hardware-smoke | Read-only INA260 current, bus voltage, and power telemetry freshness observed; no voltage writes claimed. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn asic007_verified_requires_bounded_frequency_transition_hardware_regression() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | hardware-smoke | Frequency transition smoke observed without a bounded frequency-transition hardware-regression artifact. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "ASIC-007", "hardware-regression evidence");
    assert_validation_error_contains(&errors, "ASIC-007", "bounded frequency-transition");
}

#[test]
fn asic007_verified_accepts_bounded_frequency_transition_hardware_regression() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | hardware-regression | Bounded frequency-transition hardware artifact passed on Ultra 205. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn asic_mining_verified_rows_require_hardware_or_soak_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-002 | BM1366 initialization | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | unit,workflow | Pure init and workflow evidence only. |
| ASIC-003 | BM1366 work send | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,golden | Diagnostic work fixture evidence only. |
| ASIC-004 | BM1366 result parsing | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,golden | Result fixture evidence only. |
| ASIC-005 | ASIC serial transport | `reference/esp-miner/components/asic/serial.c` | `firmware/bitaxe` | verified | workflow | Firmware compile evidence only. |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic` | verified | unit | Frequency transition unit evidence only. |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-stratum`, `firmware/bitaxe` | verified | unit,workflow | First live mining loop not observed. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    for row_id in [
        "ASIC-002", "ASIC-003", "ASIC-004", "ASIC-005", "ASIC-007", "STR-006",
    ] {
        assert_validation_error_contains(
            &errors,
            row_id,
            "requires hardware-smoke or soak evidence",
        );
    }
}

#[test]
fn asic_mining_verified_str008_requires_mining_smoke_or_soak_details() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 redaction passed conclusion recorded, but no share or controlled no-share observation. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "STR-008", "requires mining smoke or soak details");
}

#[test]
fn asic_mining_verified_str008_rejects_controlled_no_share_with_missing_live_prerequisites() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 controlled no-share condition redaction passed conclusion recorded; missing live prerequisites kept live smoke below verified. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "STR-008", "blocker terms");
    assert_validation_error_contains(&errors, "STR-008", "requires mining smoke or soak details");
}

#[test]
fn asic_mining_verified_str008_accepts_live_share_metadata() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 accepted share observed redaction passed conclusion recorded. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn asic_mining_verified_str008_accepts_approved_bounded_controlled_no_share_soak() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | soak | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 approved bounded controlled no-share soak redaction passed conclusion recorded. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn asic_mining_verified_rows_reject_blocker_language() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-stratum`, `firmware/bitaxe` | verified | hardware-smoke | Board 205 coordination observed, but live prerequisites missing and pool lifecycle remains below verified. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert_validation_error_contains(&errors, "STR-006", "blocker terms");
}

#[test]
fn asic_mining_verified_str007_workflow_below_verified_remains_allowed() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-007 | Mining smoke and soak criteria | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | implemented | workflow | Criteria documentation only; live smoke remains hardware evidence pending. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}
