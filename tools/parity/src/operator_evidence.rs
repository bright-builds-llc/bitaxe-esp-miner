use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

pub(crate) const REQUIRED_SLOT_FILES: &[&str] = &[
    "package.md",
    "detector.md",
    "board-info.md",
    "command.md",
    "log.md",
    "api.md",
    "websocket.md",
    "share-outcome.md",
    "safe-stop.md",
    "redaction-review.md",
    "conclusion.md",
];

const ALLOWED_SLOT_STATUSES: &[&str] = &[
    "slot_status: passed",
    "slot_status: blocked",
    "slot_status: pending",
    "slot_status: deferred",
];

const FORBIDDEN_SENTINELS: &[&str] = &[
    "stratum+tcp://sentinel-pool.invalid:3333",
    "bc1qsentinelowneraddress.bitaxe",
    "sentinel-password",
    "target=00000000sentinel",
    "extranonce=sentinel-extra",
    "share_payload=sentinel-share",
    "socket_error=sentinel-private-host",
    "device_url=http://192.0.2.55",
    "ip=192.0.2.55",
    "mac=aa:bb:cc:dd:ee:ff",
    "ssid=SentinelWifi",
    "token=sentinel-token",
    "nvs_secret=sentinel-nvs",
    "raw_bm1366_frame=aa55sentinel",
];

const OVERCLAIM_PHRASES: &[&str] = &[
    "phase 23 verifies trusted bm1366 production work",
    "phase 23 verifies live stratum socket success",
    "phase 23 verifies accepted shares",
    "phase 23 verifies rejected shares",
    "phase 23 verifies phase 26 telemetry promotion",
];

#[derive(Debug)]
pub(crate) struct OperatorEvidenceDocuments {
    pub(crate) evidence_root: Utf8PathBuf,
    pub(crate) slots: BTreeMap<String, String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OperatorEvidenceReport {
    pub(crate) validation_errors: Vec<String>,
}

impl OperatorEvidenceReport {
    pub(crate) fn passed(&self) -> bool {
        self.validation_errors.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct OperatorEvidenceFilters {
    pub(crate) require_redaction_passed: bool,
}

pub(crate) fn load_operator_evidence_documents(
    evidence_root: &Utf8Path,
) -> Result<OperatorEvidenceDocuments> {
    if !evidence_root.exists() {
        bail!("operator evidence root does not exist: {evidence_root}");
    }

    if !evidence_root.is_dir() {
        bail!("operator evidence root is not a directory: {evidence_root}");
    }

    let mut slots = BTreeMap::new();
    for slot_file in REQUIRED_SLOT_FILES {
        let slot_path = evidence_root.join(slot_file);
        let contents = std::fs::read_to_string(slot_path.as_std_path())
            .with_context(|| format!("failed to read operator evidence slot {slot_path}"))?;
        slots.insert((*slot_file).to_owned(), contents);
    }

    Ok(OperatorEvidenceDocuments {
        evidence_root: evidence_root.to_owned(),
        slots,
    })
}

pub(crate) fn validate_operator_evidence_documents(
    documents: &OperatorEvidenceDocuments,
    filters: &OperatorEvidenceFilters,
) -> OperatorEvidenceReport {
    let mut validation_errors = Vec::new();

    validate_required_slots(&mut validation_errors, documents);
    validate_slot_metadata(&mut validation_errors, documents);
    validate_redaction_review(&mut validation_errors, documents, filters);
    validate_blocked_target_slots(&mut validation_errors, documents);
    validate_share_outcome_slot(&mut validation_errors, documents);
    validate_conclusion(&mut validation_errors, documents);
    validate_forbidden_sentinels(&mut validation_errors, documents);
    validate_later_phase_overclaims(&mut validation_errors, documents);

    OperatorEvidenceReport { validation_errors }
}

pub(crate) fn render_operator_evidence_report(
    documents: &OperatorEvidenceDocuments,
    report: &OperatorEvidenceReport,
) -> String {
    if report.passed() {
        return format!(
            "operator_evidence_status: passed\nevidence_root: {}\nslots: {}\nredaction_status: passed\n",
            documents.evidence_root,
            REQUIRED_SLOT_FILES.join(",")
        );
    }

    let mut output = String::from("operator_evidence_status: failed\nvalidation_errors:\n");
    for error in &report.validation_errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

fn validate_required_slots(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for slot_file in REQUIRED_SLOT_FILES {
        if !documents.slots.contains_key(*slot_file) {
            validation_errors.push(format!("missing required slot file {slot_file}"));
        }
    }
}

fn validate_slot_metadata(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for slot_file in REQUIRED_SLOT_FILES {
        let Some(contents) = documents.slots.get(*slot_file) else {
            continue;
        };

        if !ALLOWED_SLOT_STATUSES
            .iter()
            .any(|status| contents.contains(status))
        {
            validation_errors.push(format!("{slot_file} must contain a valid slot_status"));
        }

        for required in [
            "raw_artifacts_committed: no",
            "board: 205",
            "redaction_status:",
            "exact_non_claims",
        ] {
            if !contents.contains(required) {
                validation_errors.push(format!("{slot_file} must contain {required}"));
            }
        }
    }
}

fn validate_redaction_review(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
    filters: &OperatorEvidenceFilters,
) {
    let Some(redaction_review) = documents.slots.get("redaction-review.md") else {
        return;
    };

    if filters.require_redaction_passed && !redaction_review.contains("redaction_status: passed") {
        validation_errors
            .push("redaction-review.md must contain redaction_status: passed".to_owned());
    }
}

fn validate_blocked_target_slots(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for slot_file in ["api.md", "websocket.md"] {
        let Some(contents) = documents.slots.get(slot_file) else {
            continue;
        };

        if !contents.contains("slot_status: blocked") {
            continue;
        }

        for required in [
            "stale DEVICE_URL",
            "mDNS",
            "ARP",
            "router state",
            "network scan",
            "unrelated evidence",
        ] {
            if !contents.contains(required) {
                validation_errors.push(format!(
                    "{slot_file} blocked target text must mention {required}"
                ));
            }
        }
    }
}

fn validate_share_outcome_slot(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    let Some(contents) = documents.slots.get("share-outcome.md") else {
        return;
    };

    let is_pending_or_deferred =
        contents.contains("slot_status: pending") || contents.contains("slot_status: deferred");
    if !is_pending_or_deferred {
        return;
    }

    for required in [
        "Phase 25",
        "accepted/rejected share outcomes remain non-claims",
    ] {
        if !contents.contains(required) {
            validation_errors.push(format!("share-outcome.md must contain {required}"));
        }
    }
}

fn validate_conclusion(validation_errors: &mut Vec<String>, documents: &OperatorEvidenceDocuments) {
    let Some(contents) = documents.slots.get("conclusion.md") else {
        return;
    };

    if !contents.contains("phase23_workflow_claim: redacted_operator_evidence_workflow") {
        validation_errors.push(
            "conclusion.md must contain phase23_workflow_claim: redacted_operator_evidence_workflow"
                .to_owned(),
        );
    }
}

fn validate_forbidden_sentinels(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for (slot_file, contents) in &documents.slots {
        for sentinel in FORBIDDEN_SENTINELS {
            if contents.contains(sentinel) {
                validation_errors.push(format!(
                    "{slot_file} contains forbidden sentinel {sentinel}"
                ));
            }
        }
    }
}

fn validate_later_phase_overclaims(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for (slot_file, contents) in &documents.slots {
        let haystack = contents.to_ascii_lowercase();
        for phrase in OVERCLAIM_PHRASES {
            if haystack.contains(phrase) {
                validation_errors.push(format!("{slot_file} contains overclaim phrase {phrase}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

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
        let report = validate_operator_evidence_documents(&documents, &filters);

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
        let report = validate_operator_evidence_documents(&documents, &filters);

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
        let report = validate_operator_evidence_documents(&documents, &filters);

        // Assert
        assert_error_contains(&report, "sentinel-pool.invalid");
        assert_error_contains(&report, "sentinel-password");
        assert_error_contains(&report, "sentinel-extra");
        assert_error_contains(&report, "sentinel-share");
        assert_error_contains(&report, "raw_bm1366_frame");
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
        let report = validate_operator_evidence_documents(&documents, &filters);

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
        let report = validate_operator_evidence_documents(&documents, &filters);

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
        let report = validate_operator_evidence_documents(&documents, &filters);

        // Assert
        assert_error_contains(&report, "phase23_workflow_claim");
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
        let report = validate_operator_evidence_documents(&documents, &filters);

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
                    contents
                        .push_str("phase23_workflow_claim: redacted_operator_evidence_workflow\n");
                }
                contents.push_str(overrides.extra_conclusion);
            }
            std::fs::write(evidence_root.join(slot_file).as_std_path(), contents)
                .expect("slot should be written");
        }
    }

    fn base_slot(slot_name: &str) -> String {
        format!(
            "slot: {slot_name}\nslot_status: passed\nboard: 205\nredaction_status: passed\nraw_artifacts_committed: no\npool_config: local-owner-supplied\nexact_non_claims:\n- trusted BM1366 production work remains a non-claim\n"
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
}
