use super::*;

const CHECKLIST: &str = "\
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |\n\
| --- | --- | --- | --- | --- | --- | --- |\n\
| STR-001 | Socket | reference/source.c | crate/src/lib.rs | implemented | pending | Note. |\n";

fn receipt() -> TransitionReceipt {
    TransitionReceipt {
        schema_version: SCHEMA.to_owned(),
        transition_id: "20260802T120000Z-STR-001".to_owned(),
        predecessor_sha256: sha256_hex(CHECKLIST.as_bytes()),
        result_sha256: String::new(),
        row_id: "STR-001".to_owned(),
        reference_commit: "a".repeat(40),
        plan_path: "docs/parity/work-plans/run/PLAN.md".to_owned(),
        plan_sha256: "b".repeat(64),
        result_path: Some("docs/parity/work-plans/run/RESULT.md".to_owned()),
        result_document_sha256: Some("c".repeat(64)),
        migration_ledger_path: None,
        migration_ledger_sha256: None,
        verified_revision_authority_path: None,
        verified_revision_authority_sha256: None,
        before_rust_owned_target: "crate/src/lib.rs".to_owned(),
        after_rust_owned_target: "crate/src/lib.rs".to_owned(),
        before_status: "implemented".to_owned(),
        after_status: "verified".to_owned(),
        before_evidence: "pending".to_owned(),
        after_evidence: "unit".to_owned(),
        before_notes: None,
        after_notes: None,
    }
}

fn valid_chain(label: &str) -> (Utf8PathBuf, String) {
    let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
        "bitaxe-parity-transition-{label}-{}",
        std::process::id()
    )))
    .expect("temporary path must be UTF-8");
    let _ = fs::remove_dir_all(workspace.as_std_path());
    let plan_root = workspace.join("docs/parity/work-plans/run");
    let ledger_root = workspace.join(TRANSITIONS_ROOT);
    fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
    fs::create_dir_all(ledger_root.as_std_path()).expect("ledger root");
    fs::write(plan_root.join("PLAN.md").as_std_path(), "plan\n").expect("plan");
    fs::write(plan_root.join("RESULT.md").as_std_path(), "result\n").expect("result");
    fs::write(ledger_root.join(BASELINE_FILE).as_std_path(), CHECKLIST).expect("baseline");

    let mut receipt = receipt();
    receipt.plan_sha256 = sha256_hex(b"plan\n");
    receipt.result_document_sha256 = Some(sha256_hex(b"result\n"));
    let projected = apply_receipt(CHECKLIST, &receipt).expect("projected checklist");
    receipt.result_sha256 = sha256_hex(projected.as_bytes());
    let receipt_document = serde_json::to_string_pretty(&receipt).expect("transition receipt JSON");
    fs::write(
        ledger_root
            .join(format!("{}.json", receipt.transition_id))
            .as_std_path(),
        receipt_document,
    )
    .expect("receipt");
    (workspace, projected)
}

#[test]
fn receipt_changes_only_the_selected_mutable_cells() {
    // Arrange
    let receipt = receipt();

    // Act
    let projected = apply_receipt(CHECKLIST, &receipt).expect("transition should apply");

    // Assert
    assert!(projected.contains("| verified | unit | Note. |"));
    assert!(projected.contains("| STR-001 | Socket | reference/source.c |"));
}

#[test]
fn receipt_projects_hash_bound_notes_when_requested() {
    // Arrange
    let mut receipt = receipt();
    receipt.before_notes = Some("Note.".to_owned());
    receipt.after_notes = Some("Verified evidence.".to_owned());

    // Act
    let projected = apply_receipt(CHECKLIST, &receipt).expect("transition should apply");

    // Assert
    assert!(projected.contains("| verified | unit | Verified evidence. |"));
    assert!(!projected.contains("| verified | unit | Note. |"));
}

#[test]
fn receipt_rejects_incomplete_notes_binding() {
    // Arrange
    let mut receipt = receipt();
    receipt.after_notes = Some("Verified evidence.".to_owned());

    // Act
    let error =
        apply_receipt(CHECKLIST, &receipt).expect_err("incomplete notes binding must fail closed");

    // Assert
    assert!(error.contains("notes binding is incomplete"));
}

#[test]
fn transition_rejects_status_regression() {
    // Arrange
    let before = "implemented";
    let after = "in-progress";

    // Act
    let error =
        require_policy(before, after, "STR-001", None).expect_err("regression must be rejected");

    // Assert
    assert!(error.contains("monotonically"));
}

#[test]
fn transition_accepts_same_status_for_a_metadata_revision() {
    // Arrange
    let status = "implemented";

    // Act
    let result = require_policy(status, status, "STR-001", None);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn receipt_requires_at_least_one_mutable_cell_change() {
    // Arrange
    let mut receipt = receipt();
    receipt.after_status = receipt.before_status.clone();
    receipt.after_evidence = receipt.before_evidence.clone();

    // Act
    let error = require_changed_mutable_cell(&receipt)
        .expect_err("an exact no-op transition must be rejected");

    // Assert
    assert!(error.contains("at least one mutable checklist cell"));
}

#[test]
fn receipt_accepts_same_status_when_the_target_changes() {
    // Arrange
    let mut receipt = receipt();
    receipt.after_status = receipt.before_status.clone();
    receipt.after_evidence = receipt.before_evidence.clone();
    receipt.after_rust_owned_target = "crate/src/new_owner.rs".to_owned();

    // Act
    let result = require_changed_mutable_cell(&receipt);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn transition_rejects_verified_source_rows() {
    // Arrange
    let before = "verified";
    let after = "verified";

    // Act
    let error = require_policy(before, after, "STR-001", None)
        .expect_err("verified source must be terminal");

    // Assert
    assert!(error.contains("forbidden"));
}

#[test]
fn transition_chain_accepts_hash_bound_plan_and_result() {
    // Arrange
    let (workspace, projected) = valid_chain("accepted");

    // Act
    let result = validate_chain(&workspace, CHECKLIST, &projected);

    // Assert
    assert!(result.is_ok());
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

#[test]
fn transition_chain_rejects_result_document_tampering() {
    // Arrange
    let (workspace, projected) = valid_chain("tampered-result");
    fs::write(
        workspace
            .join("docs/parity/work-plans/run/RESULT.md")
            .as_std_path(),
        "tampered\n",
    )
    .expect("tamper result");

    // Act
    let error = validate_chain(&workspace, CHECKLIST, &projected).expect_err("tampering must fail");

    // Assert
    assert!(error.contains("result digest mismatch"));
    fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
}

fn verified_revision_receipt() -> TransitionReceipt {
    TransitionReceipt {
        before_status: "verified".to_owned(),
        after_status: "verified".to_owned(),
        before_evidence: "unit".to_owned(),
        after_evidence: "unit,soak".to_owned(),
        before_notes: Some("Claim. Non-claims: profiles.".to_owned()),
        after_notes: Some("Claim. Non-claims: profiles. Later soak evidence.".to_owned()),
        ..receipt()
    }
}

#[test]
fn a_verified_revision_without_an_owner_authority_is_refused() {
    // Arrange
    let receipt = verified_revision_receipt();

    // Act
    let error = require_receipt_policy(&receipt, None, None).expect_err("no authority");

    // Assert
    assert!(error.contains("out of verified are forbidden"));
}

#[test]
fn a_verified_revision_under_an_owner_authority_naming_the_row_is_accepted() {
    // Arrange
    let receipt = verified_revision_receipt();
    let authority = verified_revision::authority_for(
        "STR-001",
        "soak",
        "Claim. Non-claims: profiles. Later soak evidence.",
    );

    // Act
    let result = require_receipt_policy(&receipt, None, Some(&authority));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn a_receipt_binding_both_authorities_is_refused() {
    // Arrange
    let receipt = verified_revision_receipt();
    let authority = verified_revision::authority_for(
        "STR-001",
        "soak",
        "Claim. Non-claims: profiles. Later soak evidence.",
    );
    let ledger = parse_ledger(
        r#"{"schema_version":"bitaxe-automation-migration-v1","cutover":"typed-automation-deep-refactor","legacy_consumers_accepted":false,"historical_evidence_rewritten":false,"mappings":[{"legacy":"a","replacement":"b","equivalence":"c","parity_rows":["STR-001"]}],"downgraded_rows":[]}"#,
    )
    .expect("fixture ledger");

    // Act
    let error = require_receipt_policy(&receipt, Some(&ledger), Some(&authority))
        .expect_err("two authorities");

    // Assert
    assert!(error.contains("at most one authority"));
}

#[test]
fn a_receipt_cell_that_would_inject_a_table_row_is_refused() {
    // Arrange
    let receipt = TransitionReceipt {
        after_notes: Some("Note.\n| NEW-001 | x | y | z | verified | unit | n |".to_owned()),
        before_notes: Some("Note.".to_owned()),
        ..receipt()
    };

    // Act
    let error = require_single_line_cells(&receipt).expect_err("multi-line cell");

    // Assert
    assert!(error.contains("single-line"));
}
