use super::*;

pub(super) fn verify_runtime_sources() {
    // Arrange
    let publication = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn publish_operator_snapshot",
        "fn collect_operator_snapshot_candidate",
    );
    let collection = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn collect_operator_snapshot_candidate",
        "fn runtime_projection_for_api_views",
    );
    // Act / Assert
    assert_eq!(BOOT_EVIDENCE_SOURCE.matches("esp_random()").count(), 4);
    assert_eq!(
        BOOT_EVIDENCE_SOURCE.matches("static BOOT_SESSION:").count(),
        1
    );
    assert!(BOOT_EVIDENCE_SOURCE.contains("operator_snapshot_boot_session"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("BootSessionId::from_words(boot_session().0)"));

    assert_eq!(
        RUNTIME_SNAPSHOT_SOURCE
            .matches("static OPERATOR_SNAPSHOT_PUBLISHER:")
            .count(),
        1
    );
    assert_eq!(
        PUBLICATION_SOURCE
            .matches("sequence: Mutex<OperatorSnapshotSequence>")
            .count(),
        1
    );
    for body in [
        source_between(
            PUBLICATION_SOURCE,
            "pub fn publish<",
            "pub fn publish_profiled<",
        ),
        source_between(
            PUBLICATION_SOURCE,
            "pub fn publish_profiled<",
            "struct PublicationDepthGuard",
        ),
    ] {
        assert_eq!(body.matches(".next_identity(").count(), 1);
        assert_eq!(body.matches("self.sequence.lock()").count(), 1);
    }
    assert_eq!(
        RUNTIME_SNAPSHOT_SOURCE
            .matches("snapshot.operator_snapshot_identity = operator_snapshot_identity")
            .count(),
        1
    );
    assert!(RUNTIME_SNAPSHOT_SOURCE
        .contains("static OPERATOR_SNAPSHOT_PUBLISHER: OnceLock<OperatorSnapshotPublisher>"));
    for (publication, collector) in [
        (
            source_between(
                RUNTIME_SNAPSHOT_SOURCE,
                "fn publish_operator_snapshot<",
                "fn publish_operator_snapshot_profiled<",
            ),
            "|| collect_operator_snapshot_candidate(drain_sample_marker)",
        ),
        (
            source_between(
                RUNTIME_SNAPSHOT_SOURCE,
                "fn publish_operator_snapshot_profiled<",
                "fn collect_operator_snapshot_candidate(",
            ),
            "|| collect_operator_snapshot_candidate_profiled(drain_sample_marker, timing)",
        ),
    ] {
        let collect_adapter = publication
            .find(collector)
            .expect("unnumbered candidate collection adapter");
        let complete_adapter = publication
            .find("|candidate, identity|")
            .expect("identity completion adapter");
        let retain_adapter = publication
            .find("operator_snapshot_retention::retain_completed_operator_snapshot")
            .expect("retained chronology adapter");
        let issue_adapter = publication
            .find("|publication| issue(publication.output)")
            .expect("external issuance adapter");
        assert!(collect_adapter < complete_adapter);
        assert!(complete_adapter < retain_adapter && retain_adapter < issue_adapter);
    }
    assert!(!collection.contains("OperatorSnapshotIdentity"));
    assert!(!collection.contains("next_identity"));
    assert!(OPERATOR_SNAPSHOT_RETENTION_SOURCE.contains("retain_operator_snapshot_pair"));
    assert_eq!(
        OPERATOR_SNAPSHOT_RETENTION_SOURCE
            .matches("retain_operator_snapshot_pair(")
            .count(),
        1
    );
    assert!(LOG_BUFFER_SOURCE.contains("pub fn retain_operator_snapshot_pair"));
    assert!(!publication.contains("Ok::<(), E>(())"));
    assert_eq!(
        OPERATOR_SNAPSHOT_RETENTION_SOURCE
            .matches("append_runtime_log_line")
            .count(),
        0
    );
    assert!(PUBLICATION_SOURCE.contains("RetentionError, IssueError"));
    assert!(API_IDENTITY_SOURCE.contains("operator_snapshot session={} revision={} redacted=true"));
    assert!(PUBLICATION_SOURCE
        .contains("reverse_collection_completion_publishes_direct_revisions_in_order"));

    for forbidden in [
        "esp_random",
        "SystemTime",
        "firmware_commit",
        "app_elf_sha256",
        "mac_addr",
        "fixture_only",
    ] {
        assert!(
            !publication.contains(forbidden),
            "publication contains forbidden fallback {forbidden}"
        );
    }
}
