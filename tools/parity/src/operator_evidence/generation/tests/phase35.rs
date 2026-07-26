use std::fs;

use camino::{Utf8Path, Utf8PathBuf};

use super::super::*;
use super::support::{create_workspace, snapshot};
use crate::phase35_evidence::tests::EligibleFixture;
use crate::phase35_promotion::{
    evaluate_phase35_promotion, ChecklistSnapshot, Phase35LiveRechecks, PHASE35_PROMOTABLE_ROWS,
};

const PHASE35_CHECKLIST: &str = include_str!("../../../../../../docs/parity/checklist.md");

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase35_publication_atomically_admits_one_complete_redacted_generation() {
    // Arrange
    let workspace = phase35_publication_workspace("success");
    let documents = phase35_documents();
    let before_rows = checklist_rows(PHASE35_CHECKLIST);

    // Act
    publish_phase35_generation(
        &workspace,
        Utf8Path::new("staging"),
        Utf8Path::new("destination"),
        Utf8Path::new("checklist.md"),
        &documents,
        Phase35PublicationOptions::default(),
    )
    .expect("eligible Phase 35 generation should publish");

    // Assert
    let admitted = snapshot(&workspace.join("destination"));
    assert!(admitted.contains("\"admitted\":true"));
    assert!(admitted.contains("\"decision\": \"promote\""));
    assert!(!workspace.join("destination/checklist.md").exists());
    assert!(admitted.contains("\"checklist_sha256\""));
    let projected = fs::read_to_string(workspace.join("checklist.md").as_std_path())
        .expect("published checklist should read");
    let after_rows = checklist_rows(&projected);
    for (row_id, before) in before_rows {
        let after = after_rows
            .get(&row_id)
            .expect("every checklist row must remain present");
        if PHASE35_PROMOTABLE_ROWS.contains(&row_id.as_str()) {
            assert_ne!(after, &before);
            assert!(after.contains("| verified | hardware-smoke |"));
        } else {
            assert_eq!(after, &before);
        }
    }
    for raw_canary in [
        "synthetic stable physical identity",
        "synthetic run identifier",
        "synthetic persisted setting",
        "0123456789abcdef0011223344556677",
        "fedcba9876543210ffeeddccbbaa9988",
    ] {
        assert!(!admitted.contains(raw_canary));
        assert!(!projected.contains(raw_canary));
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase35_publication_failures_preserve_previous_generation_and_checklist() {
    for failure_point in [
        Phase35PublicationFailurePoint::BeforeValidation,
        Phase35PublicationFailurePoint::AfterValidationBeforeExchange,
        Phase35PublicationFailurePoint::DuringExchange,
    ] {
        // Arrange
        let workspace = phase35_publication_workspace(&format!("{failure_point:?}"));
        let documents = phase35_documents();
        let destination_before = snapshot(&workspace.join("destination"));
        let checklist_before = fs::read_to_string(workspace.join("checklist.md").as_std_path())
            .expect("checklist should read");

        // Act
        let error = publish_phase35_generation(
            &workspace,
            Utf8Path::new("staging"),
            Utf8Path::new("destination"),
            Utf8Path::new("checklist.md"),
            &documents,
            Phase35PublicationOptions {
                maybe_failure: Some(failure_point),
            },
        )
        .expect_err("injected publication failure should fail");

        // Assert
        assert!(matches!(error, GenerationError::Phase35Injected(point) if point == failure_point));
        assert_eq!(snapshot(&workspace.join("destination")), destination_before);
        assert_eq!(
            fs::read_to_string(workspace.join("checklist.md").as_std_path())
                .expect("checklist should read"),
            checklist_before
        );
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase35_publication_rejects_fingerprint_drift_before_exchange() {
    // Arrange
    let workspace = phase35_publication_workspace("fingerprint-drift");
    let mut documents = phase35_documents();
    documents.expected_checklist_fingerprint = "0".repeat(64);
    let destination_before = snapshot(&workspace.join("destination"));

    // Act
    let error = publish_phase35_generation(
        &workspace,
        Utf8Path::new("staging"),
        Utf8Path::new("destination"),
        Utf8Path::new("checklist.md"),
        &documents,
        Phase35PublicationOptions::default(),
    )
    .expect_err("fingerprint drift should fail");

    // Assert
    assert!(matches!(error, GenerationError::Validation(_)));
    assert_eq!(snapshot(&workspace.join("destination")), destination_before);
    assert_eq!(
        fs::read_to_string(workspace.join("checklist.md").as_std_path())
            .expect("checklist should read"),
        PHASE35_CHECKLIST
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn phase35_documents() -> Phase35GenerationDocuments {
    let evidence = EligibleFixture::new()
        .validate()
        .expect("synthetic evidence should validate");
    let live = Phase35LiveRechecks::matching(&evidence);
    let checklist = ChecklistSnapshot::capture(PHASE35_CHECKLIST.to_owned(), live)
        .expect("checklist should parse");
    let matrix = evaluate_phase35_promotion(&evidence, &checklist)
        .expect("synthetic evidence should promote");
    let projection = evidence
        .shareable_projection()
        .expect("projection should remain redacted");
    Phase35GenerationDocuments {
        projection_json: serde_json::to_string_pretty(&projection)
            .expect("projection should serialize"),
        matrix_json: serde_json::to_string_pretty(&matrix).expect("matrix should serialize"),
        projected_checklist: matrix.projected_checklist.clone(),
        expected_checklist_fingerprint: matrix.checklist_fingerprint_before.clone(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn phase35_publication_workspace(name: &str) -> Utf8PathBuf {
    let workspace = create_workspace(&format!("phase35-{name}"));
    fs::write(
        workspace.join("checklist.md").as_std_path(),
        PHASE35_CHECKLIST,
    )
    .expect("checklist should write");
    fs::create_dir_all(workspace.join("destination").as_std_path())
        .expect("destination should exist");
    fs::write(
        workspace.join("destination/marker").as_std_path(),
        "previous-generation",
    )
    .expect("previous marker should write");
    workspace
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn checklist_rows(checklist: &str) -> std::collections::BTreeMap<String, String> {
    checklist
        .lines()
        .filter(|line| line.starts_with("| "))
        .filter_map(|line| {
            let row_id = line.split('|').nth(1)?.trim();
            if matches!(row_id, "ID" | "---") {
                return None;
            }
            Some((row_id.to_owned(), line.to_owned()))
        })
        .collect()
}
