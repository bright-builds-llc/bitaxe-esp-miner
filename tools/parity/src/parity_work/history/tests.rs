use super::*;

fn progress_record(checklist_sha256: &str) -> ProgressRecord {
    ProgressRecord {
        schema_version: HISTORY_SCHEMA.to_owned(),
        recorded_at: "2026-08-02T12:00:00Z".to_owned(),
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        checklist_sha256: checklist_sha256.to_owned(),
        previous_record_sha256: None,
        status_counts: BTreeMap::from([("verified".to_owned(), 1)]),
        total: 1,
        active_total: 1,
        verified_total: 1,
        completion_basis_points: 10_000,
        selected_row: None,
        plan_path: None,
    }
}

fn history_entry(record: ProgressRecord) -> HistoryEntry {
    let line = serde_json::to_string(&record).expect("progress record JSON");
    HistoryEntry { record, line }
}

#[test]
fn readme_block_is_inserted_after_managed_badges() {
    // Arrange
    let readme = format!("# Project\n\n{README_BADGES_END}\n\n## Quickstart\n");

    // Act
    let updated = replace_or_insert_readme_block(&readme, "generated").expect("README insertion");

    // Assert
    assert_eq!(
        updated,
        format!("# Project\n\n{README_BADGES_END}\n\ngenerated\n\n## Quickstart\n")
    );
}

#[test]
fn readme_block_rejects_incomplete_markers() {
    // Arrange
    let readme = format!("# Project\n{README_BEGIN}\n");

    // Act
    let error = replace_or_insert_readme_block(&readme, "replacement")
        .expect_err("incomplete markers must fail");

    // Assert
    assert!(error.to_string().contains("incomplete"));
}

#[test]
fn progress_history_accepts_a_valid_hash_chain() {
    // Arrange
    let first = history_entry(progress_record(&"c".repeat(64)));
    let mut second_record = progress_record(&"d".repeat(64));
    second_record.previous_record_sha256 =
        Some(phase35_evidence::sha256_hex(first.line.as_bytes()));
    let history = vec![first, history_entry(second_record)];

    // Act
    let result = validate_history(&history);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn progress_history_rejects_a_broken_predecessor_hash() {
    // Arrange
    let first = history_entry(progress_record(&"c".repeat(64)));
    let mut second_record = progress_record(&"d".repeat(64));
    second_record.previous_record_sha256 = Some("e".repeat(64));
    let history = vec![first, history_entry(second_record)];

    // Act
    let error = validate_history(&history).expect_err("broken chain must fail");

    // Assert
    assert!(error.to_string().contains("predecessor mismatch"));
}

#[test]
fn progress_history_rejects_duplicate_checklist_snapshots() {
    // Arrange
    let first = history_entry(progress_record(&"c".repeat(64)));
    let mut second_record = progress_record(&"c".repeat(64));
    second_record.previous_record_sha256 =
        Some(phase35_evidence::sha256_hex(first.line.as_bytes()));
    let history = vec![first, history_entry(second_record)];

    // Act
    let error = validate_history(&history).expect_err("duplicate snapshot must fail");

    // Assert
    assert!(error.to_string().contains("duplicate checklist digest"));
}
