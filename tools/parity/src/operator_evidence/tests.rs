use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
fn every_profile_describes_exactly_the_canonical_eleven_slots() {
    // Arrange
    let profiles = OperatorEvidenceProfile::ALL;

    // Act
    let described_slots = profiles.map(|profile| profile.descriptor().slots());

    // Assert
    for slots in described_slots {
        assert_eq!(slots, OperatorEvidenceSlot::ALL);
    }
}

#[test]
fn explicit_phase23_profile_ignores_misleading_phase28_directory_name() {
    // Arrange
    let evidence_root = create_evidence_root("phase-28-hardware-evidence-and-checklist-promotion");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
}

#[test]
fn rejects_cross_linked_detector_when_phase27_requires_observation() {
    // Arrange
    let evidence_root = create_evidence_root("phase27-cross-linked-detector");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    rewrite_profile(&evidence_root, OperatorEvidenceProfile::Phase27);
    rewrite_slot(&evidence_root, OperatorEvidenceSlot::Detector, |contents| {
        contents.replace(
            "evidence_disposition: observed",
            "evidence_disposition: cross_linked\ngenerated_provenance: source-link",
        )
    });
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase27,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "requires observed evidence");
}

#[test]
fn rejects_accepted_outcome_without_asic_correlation_and_safe_stop_support() {
    assert_unsupported_share_outcome(ShareOutcome::Accepted);
}

#[test]
fn rejects_rejected_outcome_without_asic_correlation_and_safe_stop_support() {
    assert_unsupported_share_outcome(ShareOutcome::Rejected);
}

#[test]
fn accepts_complete_redacted_operator_evidence_root() {
    // Arrange
    let evidence_root = create_evidence_root("complete");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
}

#[test]
fn rejects_missing_share_outcome_slot() {
    // Arrange
    let evidence_root = create_evidence_root("missing-share-outcome");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    std::fs::remove_file(evidence_root.join("share-outcome.md").as_std_path())
        .expect("share outcome slot should be removable");

    // Act
    let result = load_operator_evidence_documents(&evidence_root);

    // Assert
    assert!(result.is_err());
    assert!(result
        .expect_err("missing slot should fail load")
        .to_string()
        .contains("share-outcome.md"));
}

#[test]
fn rejects_redaction_review_without_passed_status_when_required() {
    // Arrange
    let evidence_root = create_evidence_root("redaction-blocked");
    write_complete_slots(
        &evidence_root,
        SlotOverrides {
            redaction_review_status: "redaction_status: pending",
            ..SlotOverrides::default()
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "redaction_status: passed");
}

#[test]
fn rejects_forbidden_synthetic_secret_and_runtime_sentinels() {
    // Arrange
    let evidence_root = create_evidence_root("forbidden-sentinels");
    write_complete_slots(
        &evidence_root,
        SlotOverrides {
            extra_conclusion: "stratum+tcp://sentinel-pool.invalid:3333\nsentinel-password\ntarget=00000000sentinel\nextranonce=sentinel-extra\nshare_payload=sentinel-share\nsocket_error=sentinel-private-host\ndevice_url=http://192.0.2.55\nip=192.0.2.55\nmac=aa:bb:cc:dd:ee:ff\nssid=SentinelWifi\ntoken=sentinel-token\nnvs_secret=sentinel-nvs\nraw_bm1366_frame=aa55sentinel\n",
            ..SlotOverrides::default()
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(
        &report,
        "forbidden redaction sentinel or private runtime value",
    );
    assert!(!format!("{report:#?}").contains("sentinel-password"));
}

#[test]
fn scans_phase27_manifest_and_nested_runtime_artifacts() {
    // Arrange
    let evidence_root = create_evidence_root("phase27-recursive-redaction");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    rewrite_profile(&evidence_root, OperatorEvidenceProfile::Phase27);
    std::fs::write(
        evidence_root.join("summary.md").as_std_path(),
        "summary_status: blocked\n",
    )
    .expect("summary should write");
    std::fs::write(
        evidence_root.join("mining-allow.json").as_std_path(),
        r#"{"private":"sentinel-password"}"#,
    )
    .expect("manifest should write");
    std::fs::create_dir_all(
        evidence_root
            .join("live-capture-runtime/pool-input-bridge")
            .as_std_path(),
    )
    .expect("nested runtime directory should write");
    std::fs::write(
        evidence_root
            .join("live-capture-runtime/pool-input-bridge/runtime.log")
            .as_std_path(),
        "device_url=http://device.local\n",
    )
    .expect("runtime log should write");
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase27,
        &documents,
        &filters,
    );

    // Assert
    let redaction_errors = report
        .validation_errors
        .iter()
        .filter(|error| error.contains("forbidden redaction sentinel or private runtime value"))
        .count();
    assert_eq!(redaction_errors, 2, "{report:#?}");
    assert!(!format!("{report:#?}").contains("device.local"));
}

#[test]
fn rejects_unknown_root_artifact() {
    // Arrange
    let evidence_root = create_evidence_root("unknown-root-artifact");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    std::fs::write(
        evidence_root.join("operator-note.txt").as_std_path(),
        "note",
    )
    .expect("unknown artifact should write");
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "unknown artifact operator-note.txt");
}

#[cfg(unix)]
#[test]
fn rejects_symlink_artifacts_before_scanning() {
    use std::os::unix::fs::symlink;

    // Arrange
    let evidence_root = create_evidence_root("symlink-artifact");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    symlink(
        evidence_root.join("conclusion.md").as_std_path(),
        evidence_root.join("summary.md").as_std_path(),
    )
    .expect("symlink should write");

    // Act
    let result = load_operator_evidence_documents(&evidence_root);

    // Assert
    assert!(result
        .expect_err("symlink artifact should fail")
        .to_string()
        .contains("must not contain symlinks"));
}

#[test]
fn accepts_blocked_api_and_websocket_with_target_provenance_rejections() {
    // Arrange
    let evidence_root = create_evidence_root("blocked-targets");
    write_complete_slots(&evidence_root, SlotOverrides::default());
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
}

#[test]
fn rejects_pending_share_outcome_without_phase25_nonclaim() {
    // Arrange
    let evidence_root = create_evidence_root("share-nonclaim");
    write_complete_slots(
        &evidence_root,
        SlotOverrides {
            share_outcome_extra: "owner: future phase\n",
            omit_share_nonclaim: true,
            ..SlotOverrides::default()
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "Phase 25");
    assert_error_contains(
        &report,
        "accepted/rejected share outcomes remain non-claims",
    );
}

#[test]
fn rejects_conclusion_without_phase23_workflow_claim() {
    // Arrange
    let evidence_root = create_evidence_root("missing-claim");
    write_complete_slots(
        &evidence_root,
        SlotOverrides {
            omit_workflow_claim: true,
            ..SlotOverrides::default()
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "phase23_workflow_claim");
}

#[test]
fn accepts_phase28_consolidation_root_with_cross_linked_slots() {
    // Arrange
    let evidence_root = create_evidence_root("phase28-consolidation");
    write_phase28_consolidation_slots(&evidence_root);
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase28,
        &documents,
        &filters,
    );

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
}

#[test]
fn rejects_later_phase_overclaim_language() {
    // Arrange
    let evidence_root = create_evidence_root("overclaim");
    write_complete_slots(
        &evidence_root,
        SlotOverrides {
            extra_conclusion: "Phase 23 verifies trusted BM1366 production work.\nPhase 23 verifies live Stratum socket success.\nPhase 23 verifies accepted shares.\nPhase 23 verifies rejected shares.\nPhase 23 verifies Phase 26 telemetry promotion.\n",
            ..SlotOverrides::default()
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase23,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "trusted bm1366 production work");
    assert_error_contains(&report, "live stratum socket success");
    assert_error_contains(&report, "accepted shares");
    assert_error_contains(&report, "rejected shares");
    assert_error_contains(&report, "phase 26 telemetry promotion");
}

#[derive(Clone, Copy)]
struct SlotOverrides {
    redaction_review_status: &'static str,
    extra_conclusion: &'static str,
    share_outcome_extra: &'static str,
    omit_share_nonclaim: bool,
    omit_workflow_claim: bool,
}

impl Default for SlotOverrides {
    fn default() -> Self {
        Self {
            redaction_review_status: "redaction_status: passed",
            extra_conclusion: "",
            share_outcome_extra: "",
            omit_share_nonclaim: false,
            omit_workflow_claim: false,
        }
    }
}

fn create_evidence_root(test_name: &str) -> Utf8PathBuf {
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("operator-evidence-{test_name}-{unique_id}"));
    std::fs::create_dir_all(&root).expect("temp evidence root should be created");
    Utf8PathBuf::from_path_buf(root).expect("temp path should be UTF-8")
}

fn write_complete_slots(evidence_root: &Utf8Path, overrides: SlotOverrides) {
    for slot_file in REQUIRED_SLOT_FILES {
        let slot_name = slot_file.trim_end_matches(".md");
        let mut contents = base_slot(slot_name);
        if *slot_file == "api.md" || *slot_file == "websocket.md" {
            contents.push_str(
                "target_blocker: stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid.\n",
            );
        }
        if *slot_file == "share-outcome.md" {
            contents = contents.replace("slot_status: passed", "slot_status: pending");
            contents = contents.replace(
                "evidence_disposition: observed",
                "evidence_disposition: deferred\ngenerated_provenance: phase23-non-claim",
            );
            if !overrides.omit_share_nonclaim {
                contents.push_str(
                    "owner: Phase 25\naccepted/rejected share outcomes remain non-claims\n",
                );
            }
            contents.push_str(overrides.share_outcome_extra);
        }
        if *slot_file == "redaction-review.md" {
            contents = contents.replace(
                "redaction_status: passed",
                overrides.redaction_review_status,
            );
        }
        if *slot_file == "conclusion.md" {
            if !overrides.omit_workflow_claim {
                contents.push_str("phase23_workflow_claim: redacted_operator_evidence_workflow\n");
            }
            contents.push_str(overrides.extra_conclusion);
        }
        std::fs::write(evidence_root.join(slot_file).as_std_path(), contents)
            .expect("slot should be written");
    }
}

fn base_slot(slot_name: &str) -> String {
    format!(
        "slot: {slot_name}\nslot_status: passed\nevidence_profile: phase23\nevidence_disposition: observed\nboard: 205\nredaction_status: passed\nraw_artifacts_committed: no\npool_config: local-owner-supplied\nexact_non_claims:\n- trusted BM1366 production work remains a non-claim\n"
    )
}

fn assert_error_contains(report: &OperatorEvidenceReport, expected: &str) {
    assert!(
        report
            .validation_errors
            .iter()
            .any(|error| error.contains(expected)),
        "expected validation error containing {expected:?}, got {report:#?}"
    );
}

fn assert_unsupported_share_outcome(outcome: ShareOutcome) {
    // Arrange
    let evidence_root = create_evidence_root(outcome.as_str());
    write_complete_slots(&evidence_root, SlotOverrides::default());
    rewrite_profile(&evidence_root, OperatorEvidenceProfile::Phase27);
    rewrite_slot(
        &evidence_root,
        OperatorEvidenceSlot::ShareOutcome,
        |contents| {
            contents
                .replace("slot_status: pending", "slot_status: passed")
                .replace(
                    "evidence_disposition: deferred",
                    "evidence_disposition: observed",
                )
                .replace("generated_provenance: phase23-non-claim\n", "")
                + &format!("share_outcome: {}\n", outcome.as_str())
        },
    );
    let documents = load_operator_evidence_documents(&evidence_root).expect("root should load");
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: true,
    };

    // Act
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase27,
        &documents,
        &filters,
    );

    // Assert
    assert_error_contains(&report, "asic_bridge_status: result_correlated");
    assert_error_contains(&report, "safe_stop_status: complete");
}

fn rewrite_profile(evidence_root: &Utf8Path, profile: OperatorEvidenceProfile) {
    for slot in OperatorEvidenceSlot::ALL {
        rewrite_slot(evidence_root, slot, |contents| {
            contents.replace(
                "evidence_profile: phase23",
                &format!("evidence_profile: {profile}"),
            )
        });
    }
}

fn rewrite_slot(
    evidence_root: &Utf8Path,
    slot: OperatorEvidenceSlot,
    transform: impl FnOnce(String) -> String,
) {
    let path = evidence_root.join(slot.file_name());
    let contents = std::fs::read_to_string(path.as_std_path()).expect("slot should read");
    std::fs::write(path.as_std_path(), transform(contents)).expect("slot should rewrite");
}

fn write_phase28_consolidation_slots(evidence_root: &Utf8Path) {
    for slot_file in REQUIRED_SLOT_FILES {
        let slot_name = slot_file.trim_end_matches(".md");
        let mut contents = format!(
            "slot: {slot_name}\nslot_status: passed\nevidence_profile: phase28\nevidence_disposition: cross_linked\ngenerated_provenance: phase27-category-cross-link\nboard: 205\nsource_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/\nconsolidation_status: cross_linked\nredaction_status: passed\nraw_artifacts_committed: no\npool_config: local-owner-supplied\nexact_non_claims:\n- accepted/rejected shares remain non-claims\n"
        );
        if *slot_file == "share-outcome.md" {
            contents = contents.replace("slot_status: passed", "slot_status: blocked");
            contents = contents.replace(
                "evidence_disposition: cross_linked",
                "evidence_disposition: blocked",
            );
            contents.push_str(
                "share_outcome: blocked_safe_prerequisite\nasic_bridge_status: blocked\nsafe_stop_status: blocked\n",
            );
        }
        if *slot_file == "api.md" || *slot_file == "websocket.md" {
            contents = contents.replace("slot_status: passed", "slot_status: blocked");
            contents = contents.replace(
                "evidence_disposition: cross_linked",
                "evidence_disposition: blocked",
            );
            contents.push_str(
                "target_blocker: stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid.\n",
            );
        }
        if *slot_file == "conclusion.md" {
            contents.push_str("phase28_consolidation_claim: hardware_evidence_consolidation\n");
        }
        std::fs::write(evidence_root.join(slot_file).as_std_path(), contents)
            .expect("slot should be written");
    }
}
