use super::*;

const PHASE16_ROOT: &str =
    "docs/parity/evidence/phase-16-current-commit-release-evidence-completion";
const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";

#[test]
fn release_evidence_rejects_current_git_head_mismatch_with_package_source_commit() {
    // Arrange
    let mut documents = complete_documents();
    documents.current_git_head = "fedcba9876543210fedcba9876543210fedcba98".to_owned();

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(
        &report,
        "current git HEAD does not match package source_commit",
    );
}

#[test]
fn release_evidence_rejects_v2_and_dirty_manifests() {
    // Arrange
    let mut v2_documents = complete_documents();
    v2_documents.manifest.schema_version = 2;
    let mut dirty_documents = complete_documents();
    dirty_documents.manifest.build_identity.source_dirty = true;
    dirty_documents.manifest.build_identity.label = "0123456789ab-dirty-dev".to_owned();

    // Act
    let v2_report = validate_release_evidence(&v2_documents, false);
    let dirty_report = validate_release_evidence(&dirty_documents, false);

    // Assert
    assert_error(&v2_report, "package manifest schema_version must be 3");
    assert_error(
        &dirty_report,
        "dirty package cannot qualify release evidence",
    );
}

#[test]
fn release_evidence_accepts_post_source_evidence_commits_when_explicitly_allowed() {
    // Arrange
    let mut documents = complete_documents();
    documents.current_git_head = "fedcba9876543210fedcba9876543210fedcba98".to_owned();
    documents.allow_post_source_evidence_commits = true;
    documents.source_commit_is_ancestor_of_head = true;
    documents.post_source_changed_paths = vec![
            Utf8PathBuf::from(
                "docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md",
            ),
            Utf8PathBuf::from(
                "docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot.md",
            ),
            Utf8PathBuf::from(
                ".planning/phases/16-current-commit-release-evidence-completion/16-VERIFICATION.md",
            ),
            Utf8PathBuf::from("docs/release/ultra-205.md"),
        ];

    // Act
    let report = validate_release_evidence(&documents, true);

    // Assert
    assert!(report.passed(), "{:?}", report.validation_errors);
}

#[test]
fn release_evidence_rejects_post_source_commits_when_source_is_not_ancestor() {
    // Arrange
    let mut documents = complete_documents();
    documents.current_git_head = "fedcba9876543210fedcba9876543210fedcba98".to_owned();
    documents.allow_post_source_evidence_commits = true;
    documents.source_commit_is_ancestor_of_head = false;
    documents.post_source_changed_paths = vec![Utf8PathBuf::from("docs/release/ultra-205.md")];

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(
        &report,
        "package source_commit is not an ancestor of current git HEAD",
    );
}

#[test]
fn release_evidence_rejects_non_evidence_paths_after_package_source_commit() {
    // Arrange
    let mut documents = complete_documents();
    documents.current_git_head = "fedcba9876543210fedcba9876543210fedcba98".to_owned();
    documents.allow_post_source_evidence_commits = true;
    documents.source_commit_is_ancestor_of_head = true;
    documents.post_source_changed_paths = vec![
        Utf8PathBuf::from("docs/release/ultra-205.md"),
        Utf8PathBuf::from("firmware/bitaxe/src/main.rs"),
    ];

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(
        &report,
        "post-source commits include non-evidence path(s): firmware/bitaxe/src/main.rs",
    );
}

#[test]
fn release_evidence_rejects_flash_evidence_firmware_commit_mismatch() {
    // Arrange
    let mut documents = complete_documents();
    documents
        .maybe_flash_evidence
        .as_mut()
        .expect("flash evidence")
        .firmware_commit = "fedcba9876543210fedcba9876543210fedcba98".to_owned();

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(&report, "flash evidence firmware_commit mismatch");
}

#[test]
fn release_evidence_rejects_observed_firmware_commit_mismatch() {
    // Arrange
    let mut documents = complete_documents();
    let flash_evidence = documents
        .maybe_flash_evidence
        .as_mut()
        .expect("flash evidence");
    flash_evidence.observed_firmware_commit = "0123456789ab".to_owned();
    flash_evidence.observed_firmware_commit.push('x');

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(&report, "observed firmware commit mismatch");
}

#[test]
fn release_evidence_rejects_untrusted_flash_evidence() {
    // Arrange
    let mut documents = complete_documents();
    documents
        .maybe_flash_evidence
        .as_mut()
        .expect("flash evidence")
        .trusted_output = false;

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(&report, "flash evidence is not trusted");
}

#[test]
fn release_evidence_rejects_evidence_paths_outside_phase_16_root() {
    // Arrange
    let mut documents = complete_documents();
    documents
        .maybe_flash_evidence
        .as_mut()
        .expect("flash evidence")
        .log_path = Utf8PathBuf::from("docs/parity/evidence/phase-15/log");

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(&report, "evidence path is outside Phase 16 root");
}

#[test]
fn release_evidence_require_redaction_passed_rejects_missing_or_pending_review() {
    // Arrange
    let mut documents = complete_documents();
    documents.maybe_redaction_review = None;

    // Act
    let missing_report = validate_release_evidence(&documents, true);

    // Assert
    assert_error(&missing_report, "redaction review is missing");

    // Arrange
    documents.maybe_redaction_review = Some("redaction_status: pending".to_owned());

    // Act
    let pending_report = validate_release_evidence(&documents, true);

    // Assert
    assert_error(&pending_report, "redaction review has not passed");
}

#[test]
fn release_evidence_accepts_valid_flash_evidence_and_redaction_review() {
    // Arrange
    let documents = complete_documents();

    // Act
    let report = validate_release_evidence(&documents, true);
    let output = render_release_evidence_report(&documents, &report);

    // Assert
    assert!(report.passed(), "{output}");
    assert!(output.contains("release_evidence_status: passed"));
    assert!(output.contains("redaction_status: passed"));
}

#[test]
fn release_evidence_rejects_observed_commit_prefix() {
    // Arrange
    let mut documents = complete_documents();
    documents
        .maybe_flash_evidence
        .as_mut()
        .expect("flash evidence")
        .observed_firmware_commit = SOURCE_COMMIT[..12].to_owned();

    // Act
    let report = validate_release_evidence(&documents, false);

    // Assert
    assert_error(&report, "observed firmware commit mismatch");
}

fn complete_documents() -> ReleaseEvidenceDocuments {
    ReleaseEvidenceDocuments {
        manifest: ReleaseEvidenceManifest {
            schema_version: 3,
            semantic_version: "0.1.0".to_owned(),
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: "6".repeat(64),
            build_identity: ReleaseEvidenceBuildIdentity {
                label: "0123456789ab-dev".to_owned(),
                channel: "dev".to_owned(),
                source_dirty: false,
                release_tag: None,
            },
            artifacts: vec![
                ReleaseEvidenceArtifact {
                    path: "esp-miner.bin".to_owned(),
                },
                ReleaseEvidenceArtifact {
                    path: FACTORY_IMAGE_FILE_NAME.to_owned(),
                },
            ],
        },
        current_git_head: SOURCE_COMMIT.to_owned(),
        allow_post_source_evidence_commits: false,
        source_commit_is_ancestor_of_head: false,
        post_source_changed_paths: Vec::new(),
        evidence_root: Utf8PathBuf::from(PHASE16_ROOT),
        maybe_flash_evidence_json_path: Some(Utf8PathBuf::from(format!(
            "{PHASE16_ROOT}/serial-boot/flash-command-evidence.json"
        ))),
        maybe_flash_evidence: Some(ReleaseEvidenceFlashEvidence {
            command_kind: "flash-monitor".to_owned(),
            board: "205".to_owned(),
            firmware_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: REFERENCE_COMMIT.to_owned(),
            manifest_path: Utf8PathBuf::from(
                "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
            ),
            trusted_output: true,
            observed_firmware_commit: SOURCE_COMMIT.to_owned(),
            observed_reference_commit: REFERENCE_COMMIT.to_owned(),
            flash_image_path: Some(Utf8PathBuf::from(
                "bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin",
            )),
            flash_command: None,
            log_path: Utf8PathBuf::from(format!("{PHASE16_ROOT}/serial-boot/flash-monitor.log")),
            monitor_log_path: Utf8PathBuf::from(format!(
                "{PHASE16_ROOT}/serial-boot/flash-monitor.log"
            )),
        }),
        maybe_redaction_review: Some("redaction_status: passed".to_owned()),
    }
}

fn assert_error(report: &ReleaseEvidenceReport, expected: &str) {
    assert!(
        report
            .validation_errors
            .iter()
            .any(|error| error == expected),
        "expected `{expected}` in {:?}",
        report.validation_errors
    );
}
