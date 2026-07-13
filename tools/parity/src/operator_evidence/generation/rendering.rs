use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::str::FromStr;

use camino::Utf8Path;

use super::filesystem::{io_error, sync_directory, write_synced};
use super::{
    ConsolidationOptions, GenerationError, GenerationResult, PromotionFailurePoint, WorkflowStatus,
    MANIFEST_FILE, SUMMARY_FILE,
};
use crate::operator_evidence::{
    load_operator_evidence_documents, validate_operator_evidence_documents, EvidenceDisposition,
    OperatorEvidenceFilters, OperatorEvidenceProfile, OperatorEvidenceSlot, ShareOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase27AsicBridgeStatus {
    Blocked,
    Initialized,
    WorkDispatched,
    ResultCorrelated,
}

impl FromStr for Phase27AsicBridgeStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "blocked" => Ok(Self::Blocked),
            "initialized" => Ok(Self::Initialized),
            "work_dispatched" => Ok(Self::WorkDispatched),
            "result_correlated" => Ok(Self::ResultCorrelated),
            _ => Err(format!("unknown Phase 27 ASIC bridge status {value:?}")),
        }
    }
}

impl Phase27AsicBridgeStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Initialized => "initialized",
            Self::WorkDispatched => "work_dispatched",
            Self::ResultCorrelated => "result_correlated",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase27SafeStopStatus {
    Blocked,
    Complete,
}

impl FromStr for Phase27SafeStopStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "blocked" => Ok(Self::Blocked),
            "complete" => Ok(Self::Complete),
            _ => Err(format!("unknown Phase 27 safe-stop status {value:?}")),
        }
    }
}

impl Phase27SafeStopStatus {
    const fn source_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Complete => "complete",
        }
    }

    const fn normalized_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Complete => "passed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Phase27SourceRecord {
    outcome: ShareOutcome,
    asic_bridge_status: Phase27AsicBridgeStatus,
    safe_stop_status: Phase27SafeStopStatus,
}

impl Phase27SourceRecord {
    fn parse(categories: &BTreeMap<String, String>) -> GenerationResult<Self> {
        let outcome = source_outcome(categories)?;
        let asic_bridge_status = parse_source_category(categories, "asic_bridge_status")?;
        let safe_stop_status = parse_source_category(categories, "safe_stop_status")?;
        Ok(Self {
            outcome,
            asic_bridge_status,
            safe_stop_status,
        })
    }

    fn validate(self) -> GenerationResult<()> {
        match self.outcome {
            ShareOutcome::Accepted | ShareOutcome::Rejected => {
                if self.asic_bridge_status != Phase27AsicBridgeStatus::ResultCorrelated {
                    return Err(GenerationError::InvalidInput(format!(
                        "{} source outcome requires asic_bridge_status: result_correlated",
                        self.outcome.as_str()
                    )));
                }
                if self.safe_stop_status != Phase27SafeStopStatus::Complete {
                    return Err(GenerationError::InvalidInput(format!(
                        "{} source outcome requires safe_stop_status: complete",
                        self.outcome.as_str()
                    )));
                }
            }
            ShareOutcome::BlockedSafePrerequisite => {}
            ShareOutcome::LiveSubmitResponseObserved => {
                return Err(GenerationError::InvalidInput(
                    "Phase 25 live-submit outcome cannot be consolidated as Phase 28 evidence"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }
}

fn parse_source_category<T>(categories: &BTreeMap<String, String>, key: &str) -> GenerationResult<T>
where
    T: FromStr<Err = String>,
{
    let value = categories.get(key).ok_or_else(|| {
        GenerationError::InvalidInput(format!("mandatory source category {key} is missing"))
    })?;
    value.parse().map_err(GenerationError::InvalidInput)
}

pub(super) fn render_completion_slot(
    profile: OperatorEvidenceProfile,
    slot: OperatorEvidenceSlot,
    disposition: EvidenceDisposition,
    workflow_status: WorkflowStatus,
) -> String {
    let slot_status = match disposition {
        EvidenceDisposition::Blocked => "blocked",
        EvidenceDisposition::Deferred => "deferred",
        EvidenceDisposition::Observed | EvidenceDisposition::CrossLinked => "passed",
    };
    let status = match workflow_status {
        WorkflowStatus::Passed => "passed",
        WorkflowStatus::Blocked => "blocked",
        WorkflowStatus::Failed => "failed",
    };
    let mut output = format!(
        "slot: {}\nslot_status: {slot_status}\nevidence_profile: {profile}\nevidence_disposition: {}\ngenerated_provenance: phase29-completion\nboard: 205\nredaction_status: passed\nraw_artifacts_committed: no\nworkflow_status: {status}\nobserved_behavior: no phase-native observation was available for this slot\nsafe_stop_status: blocked\nconclusion: generated non-claim slot\nexact_non_claims:\n- generated evidence does not prove observed hardware behavior\n",
        slot.slot_name(),
        disposition.as_str()
    );
    if matches!(
        slot,
        OperatorEvidenceSlot::Api | OperatorEvidenceSlot::Websocket
    ) {
        output.push_str("target_blocker: stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid.\n");
    }
    if slot == OperatorEvidenceSlot::ShareOutcome {
        output.push_str("share_outcome: blocked_safe_prerequisite\nasic_bridge_status: blocked\n");
    }
    output
}

pub(super) fn read_source_categories(
    source: &Utf8Path,
) -> GenerationResult<BTreeMap<String, String>> {
    let mandatory = [
        "summary.md",
        "share-outcome.md",
        "redaction-review.md",
        "conclusion.md",
    ];
    let allowlist = BTreeSet::from([
        "asic_bridge_status",
        "asic_correlation_status",
        "board",
        "board_info_status",
        "detector_status",
        "raw_artifacts_committed",
        "raw_pool_values_committed",
        "redaction_status",
        "safe_stop_status",
        "share_outcome",
    ]);
    let mut categories = BTreeMap::new();
    for file_name in mandatory {
        let path = source.join(file_name);
        let contents = fs::read_to_string(path.as_std_path()).map_err(|source| {
            io_error(
                format!("failed to read mandatory Phase 27 source {path}"),
                source,
            )
        })?;
        for line in contents.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            if !allowlist.contains(key) {
                continue;
            }
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            if let Some(previous) = categories.insert(key.to_owned(), value.to_owned()) {
                if previous != value {
                    return Err(GenerationError::InvalidInput(format!(
                        "contradictory source category {key}: {previous:?} versus {value:?}"
                    )));
                }
            }
        }
    }
    Ok(categories)
}

pub(super) fn validate_source_categories(
    categories: &BTreeMap<String, String>,
) -> GenerationResult<()> {
    for (key, expected) in [
        ("board", "205"),
        ("redaction_status", "passed"),
        ("raw_artifacts_committed", "no"),
    ] {
        if categories.get(key).map(String::as_str) != Some(expected) {
            return Err(GenerationError::InvalidInput(format!(
                "mandatory source category {key} must be {expected:?}"
            )));
        }
    }

    Phase27SourceRecord::parse(categories)?.validate()
}

pub(super) fn source_outcome(
    categories: &BTreeMap<String, String>,
) -> GenerationResult<ShareOutcome> {
    let value = categories.get("share_outcome").ok_or_else(|| {
        GenerationError::InvalidInput(
            "mandatory source category share_outcome is missing".to_owned(),
        )
    })?;
    value
        .parse()
        .map_err(|message: String| GenerationError::InvalidInput(message))
}

pub(super) fn generate_phase28_staging(
    staging: &Utf8Path,
    relative_source: &Utf8Path,
    source: &Utf8Path,
    categories: &BTreeMap<String, String>,
    options: ConsolidationOptions,
) -> GenerationResult<()> {
    let source_record = Phase27SourceRecord::parse(categories)?;
    let outcome = source_record.outcome;
    for slot in OperatorEvidenceSlot::ALL {
        let source_slot = source.join(slot.file_name());
        let source_exists = source_slot.is_file();
        let (status, disposition, consolidation_status) =
            if slot == OperatorEvidenceSlot::ShareOutcome {
                match outcome {
                    ShareOutcome::Accepted | ShareOutcome::Rejected => {
                        ("passed", EvidenceDisposition::CrossLinked, "cross_linked")
                    }
                    ShareOutcome::LiveSubmitResponseObserved => {
                        unreachable!("Phase 25 outcome was rejected during source validation")
                    }
                    ShareOutcome::BlockedSafePrerequisite => {
                        ("blocked", EvidenceDisposition::Blocked, "blocked")
                    }
                }
            } else if source_exists {
                ("passed", EvidenceDisposition::CrossLinked, "cross_linked")
            } else {
                ("blocked", EvidenceDisposition::Blocked, "blocked")
            };
        let contents = render_phase28_slot(
            slot,
            status,
            disposition,
            consolidation_status,
            relative_source,
            source_exists,
            source_record,
        );
        write_synced(&staging.join(slot.file_name()), &contents)?;
    }
    write_synced(
        &staging.join(SUMMARY_FILE),
        &render_phase28_summary(relative_source, source_record),
    )?;
    write_synced(&staging.join(MANIFEST_FILE), &render_manifest())?;
    if options.maybe_failure == Some(PromotionFailurePoint::BeforeStagingSync) {
        return Err(GenerationError::Injected(
            PromotionFailurePoint::BeforeStagingSync,
        ));
    }
    sync_directory(staging)
}

fn render_phase28_summary(
    relative_source: &Utf8Path,
    source_record: Phase27SourceRecord,
) -> String {
    let mut output = format!(
        "# Phase 28 Evidence Summary\n\nevidence_profile: phase28\ngenerated_provenance: phase29-phase28-consolidation\nboard: 205\nsource_phase27_root: {relative_source}\nshare_outcome: {}\nsafe_stop_status: {}\nredaction_status: passed\nraw_artifacts_committed: no\nraw_pool_values_committed: no\nconsolidation_status: passed\nconclusion: deterministic Phase 28 category-only consolidation\n",
        source_record.outcome.as_str(),
        source_record.safe_stop_status.normalized_str(),
    );
    match source_record.outcome {
        ShareOutcome::Accepted | ShareOutcome::Rejected => {
            output.push_str("asic_correlation_status: passed\n");
        }
        ShareOutcome::BlockedSafePrerequisite => {
            output.push_str("asic_bridge_status: blocked\n");
        }
        ShareOutcome::LiveSubmitResponseObserved => {
            unreachable!("Phase 28 does not support Phase 25 outcomes")
        }
    }
    output.push_str(
        "\n## exact_non_claims\n\n- Raw Phase 27 artifacts and private runtime values are not copied.\n- Phase 30 checklist promotion remains a separate decision.\n",
    );
    output
}

#[allow(clippy::too_many_arguments)]
fn render_phase28_slot(
    slot: OperatorEvidenceSlot,
    status: &str,
    disposition: EvidenceDisposition,
    consolidation_status: &str,
    relative_source: &Utf8Path,
    source_exists: bool,
    source_record: Phase27SourceRecord,
) -> String {
    let outcome = source_record.outcome;
    let source_link = if source_exists {
        format!("{relative_source}/{}", slot.file_name())
    } else {
        "not-available-by-phase27-contract".to_owned()
    };
    let mut output = format!(
        "slot: {}\nslot_status: {status}\nevidence_profile: phase28\nevidence_disposition: {}\ngenerated_provenance: phase29-phase28-consolidation\nboard: 205\nsource_phase27_root: {relative_source}\nsource_link: {source_link}\nconsolidation_status: {consolidation_status}\nredaction_status: passed\nraw_artifacts_committed: no\nraw_pool_values_committed: no\npool_config: not-read\nobserved_behavior: source categories are represented by cross-links only\nsafe_stop_status: {}\nconclusion: deterministic Phase 28 consolidation\nexact_non_claims:\n- accepted or rejected shares require exact ASIC correlation and safe-stop support\n- raw Phase 27 artifacts and private runtime values are not copied\n- Phase 30 checklist promotion remains a separate decision\n",
        slot.slot_name(),
        disposition.as_str(),
        source_record.safe_stop_status.normalized_str(),
    );
    if matches!(
        slot,
        OperatorEvidenceSlot::Api | OperatorEvidenceSlot::Websocket
    ) {
        output.push_str("target_blocker: stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid.\n");
    }
    if slot == OperatorEvidenceSlot::ShareOutcome {
        output.push_str(&format!("share_outcome: {}\n", outcome.as_str()));
        match outcome {
            ShareOutcome::Accepted | ShareOutcome::Rejected => {
                output.push_str("asic_correlation_status: passed\n");
            }
            ShareOutcome::LiveSubmitResponseObserved => {
                unreachable!("Phase 28 does not support Phase 25 outcomes")
            }
            ShareOutcome::BlockedSafePrerequisite => {
                output.push_str(&format!(
                    "asic_bridge_status: blocked\nsource_asic_bridge_status: {}\nsource_safe_stop_status: {}\n",
                    source_record.asic_bridge_status.as_str(),
                    source_record.safe_stop_status.source_str(),
                ));
            }
        }
    }
    if slot == OperatorEvidenceSlot::Conclusion {
        output.push_str("phase28_consolidation_claim: hardware_evidence_consolidation\n");
    }
    output
}

fn render_manifest() -> String {
    let mut output = String::from("generator: bitaxe-parity-phase28-v1\nfiles:\n");
    for slot in OperatorEvidenceSlot::ALL {
        output.push_str("- ");
        output.push_str(slot.file_name());
        output.push('\n');
    }
    output.push_str("- ");
    output.push_str(SUMMARY_FILE);
    output.push('\n');
    output
}

pub(super) fn validate_staging(
    staging: &Utf8Path,
    relative_source: &Utf8Path,
    source_categories: &BTreeMap<String, String>,
) -> GenerationResult<()> {
    let expected_summary = render_phase28_summary(
        relative_source,
        Phase27SourceRecord::parse(source_categories)?,
    );
    let summary_path = staging.join(SUMMARY_FILE);
    let summary = fs::read_to_string(summary_path.as_std_path()).map_err(|source| {
        io_error(
            format!("failed to read generated Phase 28 summary {summary_path}"),
            source,
        )
    })?;
    if summary != expected_summary {
        return Err(GenerationError::Validation(vec![
            "generated Phase 28 summary does not match its typed source categories".to_owned(),
        ]));
    }
    let documents = load_operator_evidence_documents(staging)
        .map_err(|error| GenerationError::InvalidInput(error.to_string()))?;
    let report = validate_operator_evidence_documents(
        OperatorEvidenceProfile::Phase28,
        &documents,
        &OperatorEvidenceFilters {
            require_redaction_passed: true,
        },
    );
    if !report.passed() {
        return Err(GenerationError::Validation(report.validation_errors));
    }
    Ok(())
}
