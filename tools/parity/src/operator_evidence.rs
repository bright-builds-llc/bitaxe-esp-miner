use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeMap;
use std::str::FromStr;

mod generation;
mod inventory;
mod profile;
#[cfg(test)]
mod tests;

use inventory::{
    load_operator_evidence_artifacts, validate_artifact_inventory, validate_artifact_redaction,
    OperatorEvidenceArtifact,
};

pub(crate) use generation::{
    complete_operator_evidence, consolidate_phase28_evidence, WorkflowStatus,
};
pub(crate) use profile::{
    EvidenceDisposition, OperatorEvidenceProfile, OperatorEvidenceSlot, ShareOutcome,
};

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

const OVERCLAIM_PHRASES: &[&str] = &[
    "phase 23 verifies trusted bm1366 production work",
    "phase 23 verifies live stratum socket success",
    "phase 23 verifies accepted shares",
    "phase 23 verifies rejected shares",
    "phase 23 verifies phase 26 telemetry promotion",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotStatus {
    Passed,
    Blocked,
    Pending,
    Deferred,
}

impl FromStr for SlotStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "passed" => Ok(Self::Passed),
            "blocked" => Ok(Self::Blocked),
            "pending" => Ok(Self::Pending),
            "deferred" => Ok(Self::Deferred),
            _ => Err(format!("unknown slot status {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RedactionStatus {
    Passed,
    Pending,
    Blocked,
}

impl FromStr for RedactionStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "passed" => Ok(Self::Passed),
            "pending" => Ok(Self::Pending),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!("unknown redaction status {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SafeStopStatus {
    Passed,
    Complete,
    Blocked,
}

impl FromStr for SafeStopStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "passed" => Ok(Self::Passed),
            "complete" => Ok(Self::Complete),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!("unknown safe-stop status {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AsicBridgeStatus {
    Blocked,
    Initialized,
    WorkDispatched,
    ResultCorrelated,
}

impl FromStr for AsicBridgeStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "blocked" => Ok(Self::Blocked),
            "initialized" => Ok(Self::Initialized),
            "work_dispatched" => Ok(Self::WorkDispatched),
            "result_correlated" => Ok(Self::ResultCorrelated),
            _ => Err(format!("unknown ASIC bridge status {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AsicCorrelationStatus {
    Passed,
    Blocked,
}

impl FromStr for AsicCorrelationStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "passed" => Ok(Self::Passed),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!("unknown ASIC correlation status {value:?}")),
        }
    }
}

#[derive(Debug)]
pub(crate) struct OperatorEvidenceDocuments {
    pub(crate) evidence_root: Utf8PathBuf,
    pub(crate) slots: BTreeMap<String, String>,
    artifacts: BTreeMap<Utf8PathBuf, OperatorEvidenceArtifact>,
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

    let root_metadata = std::fs::symlink_metadata(evidence_root.as_std_path())
        .with_context(|| format!("failed to inspect operator evidence root {evidence_root}"))?;
    if root_metadata.file_type().is_symlink() {
        bail!("operator evidence root must not be a symlink: {evidence_root}");
    }

    let mut artifacts = BTreeMap::new();
    load_operator_evidence_artifacts(evidence_root, evidence_root, &mut artifacts)?;
    let mut slots = BTreeMap::new();
    for slot_file in REQUIRED_SLOT_FILES {
        let relative_path = Utf8PathBuf::from(*slot_file);
        let slot_path = evidence_root.join(&relative_path);
        let Some(OperatorEvidenceArtifact::RegularFile(bytes)) = artifacts.get(&relative_path)
        else {
            bail!("failed to read operator evidence slot {slot_path}: required slot is not a regular file");
        };
        let contents = String::from_utf8(bytes.clone())
            .with_context(|| format!("operator evidence slot {slot_path} is not UTF-8"))?;
        slots.insert((*slot_file).to_owned(), contents);
    }

    Ok(OperatorEvidenceDocuments {
        evidence_root: evidence_root.to_owned(),
        slots,
        artifacts,
    })
}

pub(crate) fn validate_operator_evidence_documents(
    profile: OperatorEvidenceProfile,
    documents: &OperatorEvidenceDocuments,
    filters: &OperatorEvidenceFilters,
) -> OperatorEvidenceReport {
    let mut validation_errors = Vec::new();

    validate_artifact_inventory(&mut validation_errors, profile, documents);
    validate_required_slots(&mut validation_errors, documents);
    validate_slot_metadata(&mut validation_errors, profile, documents);
    validate_redaction_review(&mut validation_errors, documents, filters);
    validate_blocked_target_slots(&mut validation_errors, documents);
    validate_share_outcome_slot(&mut validation_errors, profile, documents);
    validate_conclusion(&mut validation_errors, profile, documents);
    validate_artifact_redaction(&mut validation_errors, documents);
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
    for slot in OperatorEvidenceSlot::ALL {
        if !documents.slots.contains_key(slot.file_name()) {
            validation_errors.push(format!("missing required slot file {}", slot.file_name()));
        }
    }
}

fn validate_slot_metadata(
    validation_errors: &mut Vec<String>,
    profile: OperatorEvidenceProfile,
    documents: &OperatorEvidenceDocuments,
) {
    let descriptor = profile.descriptor();
    for slot in descriptor.slots() {
        let slot_file = slot.file_name();
        let Some(contents) = documents.slots.get(slot_file) else {
            continue;
        };

        let maybe_slot_status =
            parse_typed_slot_field(validation_errors, slot_file, contents, "slot_status");
        validate_literal_slot_field(
            validation_errors,
            slot_file,
            contents,
            "slot",
            slot.slot_name(),
        );
        validate_literal_slot_field(
            validation_errors,
            slot_file,
            contents,
            "raw_artifacts_committed",
            "no",
        );
        validate_literal_slot_field(validation_errors, slot_file, contents, "board", "205");
        let _: Option<RedactionStatus> =
            parse_typed_slot_field(validation_errors, slot_file, contents, "redaction_status");
        if !contents.contains("exact_non_claims") {
            validation_errors.push(format!("{slot_file} must contain exact_non_claims"));
        }

        validate_profile_field(validation_errors, profile, slot_file, contents);
        validate_disposition(
            validation_errors,
            descriptor,
            slot,
            contents,
            maybe_slot_status,
        );

        if profile == OperatorEvidenceProfile::Phase28 {
            for field in ["source_phase27_root", "consolidation_status"] {
                if let Err(error) = parse_single_field(contents, field) {
                    validation_errors
                        .push(format!("{slot_file} Phase 28 consolidation field {error}"));
                }
            }
        }
    }
}

fn validate_profile_field(
    validation_errors: &mut Vec<String>,
    profile: OperatorEvidenceProfile,
    slot_file: &str,
    contents: &str,
) {
    match parse_single_field(contents, "evidence_profile")
        .and_then(|value| value.parse::<OperatorEvidenceProfile>())
    {
        Ok(value) if value == profile => {}
        Ok(value) => validation_errors.push(format!(
            "{slot_file} evidence_profile {value:?} contradicts selected profile {profile}"
        )),
        Err(error) => validation_errors.push(format!("{slot_file} {error}")),
    }
}

fn validate_disposition(
    validation_errors: &mut Vec<String>,
    descriptor: profile::OperatorEvidenceProfileDescriptor,
    slot: OperatorEvidenceSlot,
    contents: &str,
    maybe_slot_status: Option<SlotStatus>,
) {
    let slot_file = slot.file_name();
    let disposition = match parse_single_field(contents, "evidence_disposition")
        .and_then(|value| value.parse::<EvidenceDisposition>())
    {
        Ok(disposition) => disposition,
        Err(error) => {
            validation_errors.push(format!("{slot_file} {error}"));
            return;
        }
    };

    if !descriptor.allows_disposition(slot, disposition) {
        validation_errors.push(format!(
            "{slot_file} disposition {} is not legal for the selected profile",
            disposition.as_str()
        ));
    }

    if descriptor.requires_observation(slot) && disposition == EvidenceDisposition::CrossLinked {
        validation_errors.push(format!(
            "{slot_file} requires observed evidence; generated or cross-linked provenance cannot satisfy it"
        ));
    }

    if descriptor.generated_provenance_required(disposition) {
        if let Err(error) = parse_single_field(contents, "generated_provenance") {
            validation_errors.push(format!(
                "{slot_file} {error} for disposition {}",
                disposition.as_str()
            ));
        }
    }

    let status_is_consistent = matches!(
        (disposition, maybe_slot_status),
        (
            EvidenceDisposition::Observed | EvidenceDisposition::CrossLinked,
            Some(SlotStatus::Passed),
        ) | (EvidenceDisposition::Blocked, Some(SlotStatus::Blocked))
            | (
                EvidenceDisposition::Deferred,
                Some(SlotStatus::Pending | SlotStatus::Deferred)
            )
    );
    if !status_is_consistent {
        validation_errors.push(format!(
            "{slot_file} slot_status contradicts evidence_disposition {}",
            disposition.as_str()
        ));
    }
}

fn parse_typed_slot_field<T>(
    validation_errors: &mut Vec<String>,
    slot_file: &str,
    contents: &str,
    field: &str,
) -> Option<T>
where
    T: FromStr<Err = String>,
{
    match parse_single_field(contents, field).and_then(T::from_str) {
        Ok(value) => Some(value),
        Err(error) => {
            validation_errors.push(format!("{slot_file} {error}"));
            None
        }
    }
}

fn validate_literal_slot_field(
    validation_errors: &mut Vec<String>,
    slot_file: &str,
    contents: &str,
    field: &str,
    expected: &str,
) {
    match parse_single_field(contents, field) {
        Ok(value) if value == expected => {}
        Ok(value) => validation_errors.push(format!(
            "{slot_file} {field} {value:?} contradicts required value {expected:?}"
        )),
        Err(error) => validation_errors.push(format!("{slot_file} {error}")),
    }
}

fn parse_single_field<'a>(contents: &'a str, field: &str) -> Result<&'a str, String> {
    let prefix = format!("{field}:");
    let values = contents
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::trim)
        .collect::<Vec<_>>();

    match values.as_slice() {
        [value] if !value.is_empty() => Ok(value),
        [] => Err(format!("must contain exactly one {field}: value")),
        _ => Err(format!("must contain exactly one {field}: value")),
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

    let redaction_status = parse_single_field(redaction_review, "redaction_status")
        .and_then(RedactionStatus::from_str);
    if filters.require_redaction_passed && redaction_status != Ok(RedactionStatus::Passed) {
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

        if parse_single_field(contents, "slot_status").and_then(SlotStatus::from_str)
            != Ok(SlotStatus::Blocked)
        {
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
    profile: OperatorEvidenceProfile,
    documents: &OperatorEvidenceDocuments,
) {
    let Some(contents) = documents.slots.get("share-outcome.md") else {
        return;
    };

    let outcome_required = matches!(
        profile,
        OperatorEvidenceProfile::Phase27 | OperatorEvidenceProfile::Phase28
    );
    let outcome_present = field_occurrence_count(contents, "share_outcome") > 0;
    if outcome_required || outcome_present {
        match parse_single_field(contents, "share_outcome").and_then(ShareOutcome::from_str) {
            Ok(outcome) => {
                validate_share_outcome_support(validation_errors, profile, outcome, contents)
            }
            Err(error) => validation_errors.push(format!("share-outcome.md {error}")),
        }
    }

    let is_pending_or_deferred = matches!(
        parse_single_field(contents, "slot_status").and_then(SlotStatus::from_str),
        Ok(SlotStatus::Pending | SlotStatus::Deferred)
    );
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

fn field_occurrence_count(contents: &str, field: &str) -> usize {
    let prefix = format!("{field}:");
    contents
        .lines()
        .filter(|line| line.trim().starts_with(&prefix))
        .count()
}

fn validate_share_outcome_support(
    validation_errors: &mut Vec<String>,
    profile: OperatorEvidenceProfile,
    outcome: ShareOutcome,
    contents: &str,
) {
    if !profile.descriptor().supports_share_outcome(outcome) {
        validation_errors.push(format!(
            "share-outcome.md outcome {} is not supported by {profile}",
            outcome.as_str()
        ));
        return;
    }

    match outcome {
        ShareOutcome::Accepted | ShareOutcome::Rejected => {
            let support_is_valid = if profile == OperatorEvidenceProfile::Phase28 {
                let correlation: Option<AsicCorrelationStatus> = parse_typed_slot_field(
                    validation_errors,
                    "share-outcome.md",
                    contents,
                    "asic_correlation_status",
                );
                let safe_stop: Option<SafeStopStatus> = parse_typed_slot_field(
                    validation_errors,
                    "share-outcome.md",
                    contents,
                    "safe_stop_status",
                );
                correlation == Some(AsicCorrelationStatus::Passed)
                    && safe_stop == Some(SafeStopStatus::Passed)
            } else {
                let asic_bridge: Option<AsicBridgeStatus> = parse_typed_slot_field(
                    validation_errors,
                    "share-outcome.md",
                    contents,
                    "asic_bridge_status",
                );
                let safe_stop: Option<SafeStopStatus> = parse_typed_slot_field(
                    validation_errors,
                    "share-outcome.md",
                    contents,
                    "safe_stop_status",
                );
                asic_bridge == Some(AsicBridgeStatus::ResultCorrelated)
                    && safe_stop == Some(SafeStopStatus::Complete)
            };
            if !support_is_valid {
                let required = if profile == OperatorEvidenceProfile::Phase28 {
                    "asic_correlation_status: passed and safe_stop_status: passed"
                } else {
                    "asic_bridge_status: result_correlated and safe_stop_status: complete"
                };
                validation_errors.push(format!(
                    "share-outcome.md {} requires {required}",
                    outcome.as_str()
                ));
            }
        }
        ShareOutcome::LiveSubmitResponseObserved => {
            let safe_stop: Option<SafeStopStatus> = parse_typed_slot_field(
                validation_errors,
                "share-outcome.md",
                contents,
                "safe_stop_status",
            );
            if safe_stop != Some(SafeStopStatus::Complete) {
                validation_errors.push(
                    "share-outcome.md live_submit_response_observed requires safe_stop_status: complete"
                        .to_owned(),
                );
            }
        }
        ShareOutcome::BlockedSafePrerequisite => {
            let safe_stop: Option<SafeStopStatus> = parse_typed_slot_field(
                validation_errors,
                "share-outcome.md",
                contents,
                "safe_stop_status",
            );
            let supported = match profile {
                OperatorEvidenceProfile::Phase25 => matches!(
                    safe_stop,
                    Some(SafeStopStatus::Complete | SafeStopStatus::Blocked)
                ),
                OperatorEvidenceProfile::Phase27 => {
                    let asic_bridge: Option<AsicBridgeStatus> = parse_typed_slot_field(
                        validation_errors,
                        "share-outcome.md",
                        contents,
                        "asic_bridge_status",
                    );
                    asic_bridge.is_some()
                        && matches!(
                            safe_stop,
                            Some(SafeStopStatus::Complete | SafeStopStatus::Blocked)
                        )
                }
                OperatorEvidenceProfile::Phase28 => {
                    let asic_bridge: Option<AsicBridgeStatus> = parse_typed_slot_field(
                        validation_errors,
                        "share-outcome.md",
                        contents,
                        "asic_bridge_status",
                    );
                    asic_bridge == Some(AsicBridgeStatus::Blocked)
                        && matches!(
                            safe_stop,
                            Some(SafeStopStatus::Passed | SafeStopStatus::Blocked)
                        )
                }
                OperatorEvidenceProfile::Phase23 => false,
            };
            if !supported {
                validation_errors.push(format!(
                    "share-outcome.md blocked_safe_prerequisite lacks {profile} support categories"
                ));
            }
        }
    }
}

fn validate_conclusion(
    validation_errors: &mut Vec<String>,
    profile: OperatorEvidenceProfile,
    documents: &OperatorEvidenceDocuments,
) {
    let Some(contents) = documents.slots.get("conclusion.md") else {
        return;
    };

    if profile == OperatorEvidenceProfile::Phase28 {
        if !contents.contains("phase28_consolidation_claim: hardware_evidence_consolidation") {
            validation_errors.push(
                "conclusion.md must contain phase28_consolidation_claim: hardware_evidence_consolidation"
                    .to_owned(),
            );
        }
        return;
    }

    if profile == OperatorEvidenceProfile::Phase23
        && !contents.contains("phase23_workflow_claim: redacted_operator_evidence_workflow")
    {
        validation_errors.push(
            "conclusion.md must contain phase23_workflow_claim: redacted_operator_evidence_workflow"
                .to_owned(),
        );
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
