use super::*;

const SESSION: &str = "0123456789abcdef0011223344556677";

#[test]
fn historical_profiles_remain_unchanged_when_snapshot_coherence_is_not_required() {
    // Arrange
    let documents = snapshot_documents(true);
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let comparisons = OperatorEvidenceProfile::ALL.map(|profile| {
        let historical = validate_operator_evidence_documents(profile, &documents, &filters);
        let explicitly_disabled = validate_operator_evidence_documents_with_snapshot_coherence(
            profile, &documents, &filters, false,
        );
        (historical, explicitly_disabled)
    });

    // Assert
    for (historical, explicitly_disabled) in comparisons {
        assert_eq!(historical, explicitly_disabled);
    }
}

#[test]
fn operator_snapshot_coherence_is_enforced_only_when_requested() {
    // Arrange
    let coherent_documents = snapshot_documents(true);
    let missing_marker_documents = snapshot_documents(false);
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: false,
    };

    // Act
    let coherent = validate_operator_evidence_documents_with_snapshot_coherence(
        OperatorEvidenceProfile::Phase23,
        &coherent_documents,
        &filters,
        true,
    );
    let missing_marker = validate_operator_evidence_documents_with_snapshot_coherence(
        OperatorEvidenceProfile::Phase23,
        &missing_marker_documents,
        &filters,
        true,
    );

    // Assert
    assert!(!coherent
        .validation_errors
        .iter()
        .any(|error| error.starts_with("operator_snapshot_")));
    assert!(missing_marker
        .validation_errors
        .iter()
        .any(|error| error.contains("operator_snapshot_missing_marker")));
}

fn snapshot_documents(include_second_marker: bool) -> OperatorEvidenceDocuments {
    let mut slots = BTreeMap::new();
    slots.insert(
            "api.md".to_owned(),
            format!(
                "system_info_json: {{\"bootSession\":\"{SESSION}\",\"operatorSnapshotRevision\":4}}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 4\n"
            ),
        );
    slots.insert(
            "websocket.md".to_owned(),
            format!(
                "live_websocket_json: {{\"bootSession\":\"{SESSION}\",\"operatorSnapshotRevision\":5}}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 5\n"
            ),
        );
    let maybe_second_marker = if include_second_marker {
        format!("operator_snapshot session={SESSION} revision=5 redacted=true\n")
    } else {
        String::new()
    };
    slots.insert(
        "log.md".to_owned(),
        format!(
            "operator_snapshot session={SESSION} revision=4 redacted=true\n{maybe_second_marker}"
        ),
    );
    OperatorEvidenceDocuments {
        evidence_root: Utf8PathBuf::from("operator-snapshot-test"),
        slots,
        artifacts: BTreeMap::new(),
    }
}
