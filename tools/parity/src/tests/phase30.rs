use super::*;

fn phase30_verified_row(requirement_id: &str, promotion_terms: &str) -> ChecklistRow {
    ChecklistRow {
        id: requirement_id.to_owned(),
        surface: "Phase 30 exact promotion claim".to_owned(),
        reference_breadcrumb: "reference/esp-miner/main/system.c".to_owned(),
        rust_owned_target: "tools/parity/src/main.rs".to_owned(),
        rust_owned_target_markdown: "`tools/parity/src/main.rs`".to_owned(),
        status: "verified".to_owned(),
        evidence: "workflow,hardware-smoke,hardware-regression".to_owned(),
        notes: format!(
            "phase-28-hardware-evidence-and-checklist-promotion/summary.md \
             redaction-review.md exact_non_claims \
             accepted share hardware proof asic bridge correlation {promotion_terms}"
        ),
    }
}

fn phase30_complete_promotion_artifact(requirement_id: &str) -> String {
    let row_proof = match requirement_id {
        "STR-09" => {
            "STR-09.live_submit_response_classified: true\n\
             STR-09.asic_correlation: passed\n\
             STR-09.safe_stop_status: complete"
        }
        "CFG-07" => {
            "CFG-07.runtime_credentials_input: local-owner-supplied\n\
             CFG-07.live_mining_credentials_consumed: true\n\
             CFG-07.committed_credential_values: none\n\
             CFG-07.safe_stop_status: complete"
        }
        "ASIC-11" => {
            "ASIC-11.asic_result_to_active_work: correlated\n\
             ASIC-11.submit_intent_from_correlated_result: true\n\
             ASIC-11.safe_stop_status: complete"
        }
        _ => panic!("unsupported Phase 30 requirement fixture: {requirement_id}"),
    };

    format!(
        "phase30_disposition: promoted\n\
         new_evidence_input: explicit\n\
         archived_lineage_verification: gaps_found\n\
         eligible_share_outcome: accepted\n\
         hardware_accessed: true\n\
         credentials_accessed: false\n\
         raw_artifacts_committed: no\n\
         current_source_gate: passed\n\
         detector_gate: passed\n\
         same_chain_gate: passed\n\
         provenance_gate: passed\n\
         redaction_status: passed\n\
         {row_proof}\n"
    )
}

#[test]
fn phase30_current_artifact_accepts_cfg07_only() {
    // Arrange
    let rows = [
        phase30_verified_row("CFG-07", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH),
        phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH),
    ];
    let artifact = parse_phase30_promotion_artifact(include_str!(
        "../../../../docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md"
    ))
    .expect("committed Phase 30 conclusion should parse");

    // Act
    let errors = validate_rows_with_phase30_artifact(
        &rows,
        &Phase30PromotionArtifactState::Available(artifact),
    );

    // Assert
    assert!(!errors.iter().any(|error| error.id == "CFG-07"));
    assert_validation_error_contains(&errors, "STR-09", "STR-09.live_submit_response_classified");
}

#[test]
fn phase30_report_accepts_cfg07_against_current_promotion_artifact() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `tools/automation/src/cfg07-evidence.ts` | verified | unit,workflow,hardware-smoke,hardware-regression | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction_status: passed exact_non_claims runtime credentials same-chain hardware proof docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md |
"#;
    let environment = FakeEnvironment::with_documents(
        checklist,
        include_str!(
            "../../../../docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md"
        ),
    );
    let request = ReportRequest {
        checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
        format: ReportFormat::Text,
        fail_on_invalid_verified: true,
    };

    // Act
    let result = run_report(&request, &environment);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn phase30_verified_rows_accept_matching_structured_artifacts() {
    // Arrange
    let cases = ["STR-09", "CFG-07", "ASIC-11"];

    // Act
    let results = cases.map(|requirement_id| {
        let artifact =
            parse_phase30_promotion_artifact(&phase30_complete_promotion_artifact(requirement_id))
                .expect("complete Phase 30 promotion artifact should parse");
        let errors = validate_rows_with_phase30_artifact(
            &[phase30_verified_row(
                requirement_id,
                DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH,
            )],
            &Phase30PromotionArtifactState::Available(artifact),
        );
        (requirement_id, errors)
    });

    // Assert
    for (requirement_id, errors) in results {
        assert!(
            errors.is_empty(),
            "expected structured artifact for {requirement_id} to pass, got {errors:#?}"
        );
    }
}

#[test]
fn phase30_verified_row_rejects_missing_artifact() {
    // Arrange
    let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
    let artifact = Phase30PromotionArtifactState::Unavailable(
        "structured Phase 30 evidence artifact is missing".to_owned(),
    );

    // Act
    let errors = validate_rows_with_phase30_artifact(&[row], &artifact);

    // Assert
    assert_validation_error_contains(&errors, "STR-09", "artifact is missing");
}

#[test]
fn phase30_verified_row_rejects_malformed_artifact_value() {
    // Arrange
    let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
    let malformed = phase30_complete_promotion_artifact("STR-09")
        .replace("detector_gate: passed", "detector_gate: maybe");
    let artifact = parse_phase30_promotion_artifact(&malformed)
        .expect_err("invalid closed value must fail parsing");

    // Act
    let errors = validate_rows_with_phase30_artifact(
        &[row],
        &Phase30PromotionArtifactState::Malformed(artifact),
    );

    // Assert
    assert_validation_error_contains(&errors, "STR-09", "detector_gate");
}

#[test]
fn phase30_verified_row_rejects_mismatched_artifact_bundle() {
    // Arrange
    let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
    let artifact = parse_phase30_promotion_artifact(&phase30_complete_promotion_artifact("CFG-07"))
        .expect("complete CFG-07 fixture should parse");

    // Act
    let errors = validate_rows_with_phase30_artifact(
        &[row],
        &Phase30PromotionArtifactState::Available(artifact),
    );

    // Assert
    assert_validation_error_contains(&errors, "STR-09", "STR-09.live_submit_response_classified");
}

#[test]
fn phase30_committed_conservative_rows_remain_valid() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `tools/automation/src/cli.ts` | implemented | workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
| ASIC-11 | BM1366 result correlation before submit intent | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}
