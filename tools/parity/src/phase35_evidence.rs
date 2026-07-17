//! Typed, fail-closed admission for one protected Phase 35 evidence root.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::operator_snapshot_evidence::validate_operator_snapshot_documents;

mod contract;
mod digests;
mod inventory;
mod projection;

pub(crate) use contract::{
    ArtifactFileKind, DetectorRunCapability, EvidenceEpochInput, EvidenceEventCategory,
    EvidenceEventInput, ExactPackageCapability, InventoryArtifact, InventoryEntryInput,
    Phase35EvidenceError, Phase35EvidenceRootInput, RedactedEpochFacts, RootAdmissionFacts,
    ShareablePhase35Projection, ValidatedPhase35Evidence,
};
pub(crate) use digests::{
    detector_run_capability_digest, evidence_epoch_digest, evidence_event_digest,
    exact_package_capability_digest, inventory_contract_digest, phase35_root_contract_digest,
    sha256_hex, storage_confirmation_digest,
};
use digests::{hash_fields, is_lower_hex};
pub(crate) use inventory::load_phase35_evidence_root;
use inventory::validate_inventory;
pub(crate) use projection::validate_projection_value;
use projection::{raw_projection_canaries, validate_projection};

pub(crate) const PHASE35_SCHEMA: &str = "phase35-evidence-v1";
pub(crate) const PHASE35_LIFECYCLE_ID: &str = "35-2026-07-17T17-00-37";
pub(crate) const EVIDENCE_DOCUMENT: &str = "eligible.json";

const INVENTORY_ROLES: [&str; 14] = [
    "package_manifest",
    "executable_image",
    "factory_image",
    "package",
    "runtime_identity",
    "target_lock",
    "detector_capability",
    "no_actuation",
    "boot_a_api",
    "boot_a_websocket",
    "boot_a_retained_log",
    "boot_b_api",
    "boot_b_websocket",
    "boot_b_retained_log",
];

const EVENT_CATEGORIES: [EvidenceEventCategory; 9] = [
    EvidenceEventCategory::RootAdmitted,
    EvidenceEventCategory::BootAObserved,
    EvidenceEventCategory::PatchResponded,
    EvidenceEventCategory::StorageConfirmed,
    EvidenceEventCategory::RebootStarted,
    EvidenceEventCategory::BootBObserved,
    EvidenceEventCategory::NoActuationVerified,
    EvidenceEventCategory::RestorationConfirmed,
    EvidenceEventCategory::CleanupConfirmed,
];

impl ValidatedPhase35Evidence {
    pub(crate) fn root_digest(&self) -> &str {
        &self.root_digest
    }

    pub(crate) fn exact_package(&self) -> &ExactPackageCapability {
        &self.exact_package
    }

    pub(crate) fn detector_run(&self) -> &DetectorRunCapability {
        &self.detector_run
    }

    pub(crate) fn admission_facts(&self) -> &RootAdmissionFacts {
        &self.admission_facts
    }

    pub(crate) fn shareable_projection(
        &self,
    ) -> Result<ShareablePhase35Projection, Phase35EvidenceError> {
        let projection = ShareablePhase35Projection {
            schema: PHASE35_SCHEMA,
            root_digest: self.root_digest.clone(),
            package_capability_digest: self.package_capability_digest.clone(),
            detector_capability_digest: self.detector_capability_digest.clone(),
            run_digest: self.run_digest.clone(),
            board_category: self.board_category.clone(),
            event_count: self.event_count,
            inventory_count: self.inventory_count,
            boot_a: self.boot_a.clone(),
            boot_b: self.boot_b.clone(),
        };
        validate_projection(&projection, &[])?;
        Ok(projection)
    }
}

pub(crate) fn validate_phase35_evidence(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
) -> Result<ValidatedPhase35Evidence, Phase35EvidenceError> {
    validate_static_facts(input)?;
    let package_digest = validate_package_capability(&input.exact_package)?;
    let detector_digest = validate_detector_capability(&input.detector_run)?;
    let inventory_digest = validate_inventory(input, artifacts)?;
    let root_digest = phase35_root_contract_digest(input, &inventory_digest);
    if input.admission_facts.root_contract_digest != root_digest {
        return Err(Phase35EvidenceError::RootContractMismatch);
    }

    let boot_a = validate_epoch(input, &input.boot_a, false)?;
    let boot_b = validate_epoch(input, &input.boot_b, true)?;
    validate_epoch_join(input)?;
    validate_events(input, artifacts)?;

    let validated = ValidatedPhase35Evidence {
        root_digest,
        package_capability_digest: package_digest,
        detector_capability_digest: detector_digest,
        run_digest: input.detector_run.run_id_digest.clone(),
        board_category: input.detector_run.board_category.clone(),
        event_count: input.events.len() as u64,
        inventory_count: input.inventory.len() as u64,
        boot_a,
        boot_b,
        exact_package: input.exact_package.clone(),
        detector_run: input.detector_run.clone(),
        admission_facts: input.admission_facts.clone(),
    };
    let projection = validated.shareable_projection()?;
    let raw_canaries = raw_projection_canaries(input);
    let projection_value = serde_json::to_value(projection)
        .map_err(|_| Phase35EvidenceError::ForbiddenProjectionField)?;
    validate_projection_value(&projection_value, &raw_canaries)?;
    Ok(validated)
}

pub(crate) fn inventory_artifact_digest(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
    role: &str,
) -> Result<String, Phase35EvidenceError> {
    let entry = input
        .inventory
        .iter()
        .find(|entry| entry.role == role)
        .ok_or(Phase35EvidenceError::InventoryMismatch)?;
    let artifact = artifacts
        .get(&entry.path)
        .ok_or(Phase35EvidenceError::InventoryMismatch)?;
    if artifact.file_kind != ArtifactFileKind::Regular {
        return Err(Phase35EvidenceError::InventoryMismatch);
    }
    Ok(sha256_hex(&artifact.bytes))
}

pub(crate) fn inventory_artifact_equals(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
    role: &str,
    expected: &[u8],
) -> Result<bool, Phase35EvidenceError> {
    let entry = input
        .inventory
        .iter()
        .find(|entry| entry.role == role)
        .ok_or(Phase35EvidenceError::InventoryMismatch)?;
    let artifact = artifacts
        .get(&entry.path)
        .ok_or(Phase35EvidenceError::InventoryMismatch)?;
    Ok(artifact.file_kind == ArtifactFileKind::Regular && artifact.bytes == expected)
}

fn validate_static_facts(input: &Phase35EvidenceRootInput) -> Result<(), Phase35EvidenceError> {
    if input.schema_version != PHASE35_SCHEMA {
        return Err(Phase35EvidenceError::UnsupportedSchema);
    }
    if input.exact_package.manifest_schema != "manifest-v3" {
        return Err(Phase35EvidenceError::ManifestV3Mismatch);
    }
    if input.detector_run.board_category != "205" {
        return Err(Phase35EvidenceError::WrongBoard);
    }
    if input.admission_facts.lifecycle_id != PHASE35_LIFECYCLE_ID
        || !input.admission_facts.lifecycle_verified
    {
        return Err(Phase35EvidenceError::LifecycleMismatch);
    }
    if !input.exact_package.current_head_verified || !input.admission_facts.current_head_rechecked {
        return Err(Phase35EvidenceError::StaleCurrentHead);
    }
    if !input.exact_package.reference_clean
        || !input.admission_facts.reference_cleanliness_rechecked
    {
        return Err(Phase35EvidenceError::DirtyReference);
    }
    if !input.admission_facts.runtime_identity_rechecked {
        return Err(Phase35EvidenceError::RuntimeIdentityMismatch);
    }
    if !input.admission_facts.no_actuation_verified {
        return Err(Phase35EvidenceError::NoActuationFailure);
    }
    if !input.admission_facts.restoration_verified {
        return Err(Phase35EvidenceError::RestorationFailure);
    }
    if !input.admission_facts.cleanup_verified {
        return Err(Phase35EvidenceError::CleanupFailure);
    }
    if !input.admission_facts.inventory_verified {
        return Err(Phase35EvidenceError::InventoryMismatch);
    }
    if !input.admission_facts.chronology_verified {
        return Err(Phase35EvidenceError::ChronologyViolation);
    }
    if !input.admission_facts.redaction_verified {
        return Err(Phase35EvidenceError::ForbiddenProjectionField);
    }
    validate_digest_fields(input)
}

fn validate_digest_fields(input: &Phase35EvidenceRootInput) -> Result<(), Phase35EvidenceError> {
    let digests = [
        &input.exact_package.manifest_digest,
        &input.exact_package.executable_image_digest,
        &input.exact_package.factory_image_digest,
        &input.exact_package.package_digest,
        &input.exact_package.runtime_identity_digest,
        &input.exact_package.capability_digest,
        &input.detector_run.detector_capability_digest,
        &input.detector_run.physical_identity_digest,
        &input.detector_run.run_id_digest,
        &input.detector_run.capability_digest,
        &input.admission_facts.root_contract_digest,
        &input.admission_facts.target_lock_digest,
    ];
    if digests.into_iter().all(|digest| is_lower_hex(digest, 64)) {
        return Ok(());
    }
    Err(Phase35EvidenceError::InvalidDigest)
}

fn validate_package_capability(
    package: &ExactPackageCapability,
) -> Result<String, Phase35EvidenceError> {
    if !is_lower_hex(&package.source_commit, 40) || !is_lower_hex(&package.reference_commit, 40) {
        return Err(Phase35EvidenceError::InvalidPackageCapability);
    }
    let expected = exact_package_capability_digest(package);
    if package.capability_digest != expected {
        return Err(Phase35EvidenceError::InvalidPackageCapability);
    }
    Ok(expected)
}

fn validate_detector_capability(
    detector: &DetectorRunCapability,
) -> Result<String, Phase35EvidenceError> {
    if !detector.board_info_verified || !detector.single_candidate_verified {
        return Err(Phase35EvidenceError::InvalidDetectorCapability);
    }
    let expected = detector_run_capability_digest(detector);
    if detector.capability_digest != expected {
        return Err(Phase35EvidenceError::InvalidDetectorCapability);
    }
    Ok(expected)
}

fn validate_epoch(
    input: &Phase35EvidenceRootInput,
    epoch: &EvidenceEpochInput,
    post_reboot: bool,
) -> Result<RedactedEpochFacts, Phase35EvidenceError> {
    if epoch.started_millis >= epoch.ended_millis {
        return Err(Phase35EvidenceError::ChronologyViolation);
    }
    if post_reboot && epoch.reset_category != "software_cpu" {
        return Err(Phase35EvidenceError::UnapprovedReset);
    }
    if !post_reboot && epoch.reset_category != "setup" {
        return Err(Phase35EvidenceError::UnapprovedReset);
    }
    for digest in [
        &epoch.boot_session_digest,
        &epoch.storage_value_digest,
        &epoch.package_capability_digest,
        &epoch.detector_capability_digest,
        &epoch.root_contract_digest,
        &epoch.target_lock_digest,
        &epoch.run_id_digest,
        &epoch.runtime_identity_digest,
        &epoch.physical_identity_digest,
    ] {
        if !is_lower_hex(digest, 64) {
            return Err(Phase35EvidenceError::InvalidDigest);
        }
    }
    if epoch.package_capability_digest != input.exact_package.capability_digest
        || epoch.detector_capability_digest != input.detector_run.capability_digest
        || epoch.root_contract_digest != input.admission_facts.root_contract_digest
        || epoch.target_lock_digest != input.admission_facts.target_lock_digest
        || epoch.run_id_digest != input.detector_run.run_id_digest
    {
        return Err(Phase35EvidenceError::RootContractMismatch);
    }
    if epoch.runtime_identity_digest != input.exact_package.runtime_identity_digest {
        return Err(Phase35EvidenceError::RuntimeIdentityMismatch);
    }
    if epoch.physical_identity_digest != input.detector_run.physical_identity_digest {
        return Err(Phase35EvidenceError::PhysicalIdentityDrift);
    }

    let snapshot_errors = validate_operator_snapshot_documents(
        &epoch.system_info_document,
        &epoch.websocket_document,
        &epoch.retained_log_document,
    );
    if snapshot_errors
        .iter()
        .any(|error| error.contains("mixed_session"))
    {
        return Err(Phase35EvidenceError::MixedSession);
    }
    if !snapshot_errors.is_empty() {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    }
    let api_identity = parse_snapshot_identity(&epoch.system_info_document, "system_info_json")?;
    let websocket_identity =
        parse_snapshot_identity(&epoch.websocket_document, "live_websocket_json")?;
    if api_identity != websocket_identity
        || api_identity.1 != epoch.storage_revision
        || sha256_hex(api_identity.0.as_bytes()) != epoch.boot_session_digest
    {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    }
    Ok(RedactedEpochFacts {
        session_digest: epoch.boot_session_digest.clone(),
        ordinal: epoch.boot_ordinal,
        revision: epoch.storage_revision,
        duration_millis: epoch.ended_millis - epoch.started_millis,
        setting_digest: epoch.storage_value_digest.clone(),
        coherent_snapshot_count: 3,
    })
}

fn validate_epoch_join(input: &Phase35EvidenceRootInput) -> Result<(), Phase35EvidenceError> {
    let expected = input
        .boot_a
        .boot_ordinal
        .checked_add(1)
        .ok_or(Phase35EvidenceError::BootOrdinalMismatch)?;
    if input.boot_b.boot_ordinal != expected {
        return Err(Phase35EvidenceError::BootOrdinalMismatch);
    }
    if input.boot_a.ended_millis >= input.boot_b.started_millis {
        return Err(Phase35EvidenceError::ChronologyViolation);
    }
    if input.boot_a.boot_session_digest == input.boot_b.boot_session_digest {
        return Err(Phase35EvidenceError::MixedSession);
    }
    if input.boot_a.physical_identity_digest != input.boot_b.physical_identity_digest {
        return Err(Phase35EvidenceError::PhysicalIdentityDrift);
    }
    if input.boot_a.storage_value_digest != input.boot_b.storage_value_digest {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    }
    Ok(())
}

fn validate_events(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
) -> Result<(), Phase35EvidenceError> {
    if input.events.len() < EVENT_CATEGORIES.len() {
        return Err(Phase35EvidenceError::MissingCheckpoint);
    }
    if input.events.len() > EVENT_CATEGORIES.len() {
        return Err(Phase35EvidenceError::DuplicateCheckpoint);
    }
    let mut seen = BTreeSet::new();
    let mut predecessor = input.admission_facts.root_contract_digest.clone();
    let mut previous_millis = None;
    for (index, event) in input.events.iter().enumerate() {
        if event.category != EVENT_CATEGORIES[index] || !seen.insert(event.category) {
            return Err(Phase35EvidenceError::DuplicateCheckpoint);
        }
        if event.sequence != index as u64 + 1 {
            return Err(Phase35EvidenceError::ChronologyViolation);
        }
        if previous_millis.is_some_and(|previous| previous >= event.monotonic_millis) {
            return Err(Phase35EvidenceError::ChronologyViolation);
        }
        if event.predecessor_event_digest != predecessor {
            return Err(Phase35EvidenceError::PredecessorViolation);
        }
        if !is_lower_hex(&event.payload_digest, 64) {
            return Err(Phase35EvidenceError::InvalidDigest);
        }
        let expected_payload = expected_event_payload(input, artifacts, event.category);
        if event.payload_digest != expected_payload {
            return Err(Phase35EvidenceError::PredecessorViolation);
        }
        predecessor = evidence_event_digest(event);
        previous_millis = Some(event.monotonic_millis);
    }
    Ok(())
}

fn expected_event_payload(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
    category: EvidenceEventCategory,
) -> String {
    match category {
        EvidenceEventCategory::RootAdmitted => input.admission_facts.root_contract_digest.clone(),
        EvidenceEventCategory::BootAObserved => evidence_epoch_digest(&input.boot_a),
        EvidenceEventCategory::PatchResponded => input.boot_a.storage_value_digest.clone(),
        EvidenceEventCategory::StorageConfirmed => storage_confirmation_digest(&input.boot_a),
        EvidenceEventCategory::RebootStarted => hash_fields(
            "phase35-reboot-v1",
            &[
                &input.boot_a.boot_ordinal.to_string(),
                &input.boot_b.boot_ordinal.to_string(),
                &input.boot_b.reset_category,
            ],
        ),
        EvidenceEventCategory::BootBObserved => evidence_epoch_digest(&input.boot_b),
        EvidenceEventCategory::NoActuationVerified => artifacts
            .iter()
            .find_map(|(path, artifact)| {
                input
                    .inventory
                    .iter()
                    .any(|entry| entry.role == "no_actuation" && entry.path == *path)
                    .then(|| sha256_hex(&artifact.bytes))
            })
            .unwrap_or_default(),
        EvidenceEventCategory::RestorationConfirmed => {
            hash_fields("phase35-restoration-v1", &["true"])
        }
        EvidenceEventCategory::CleanupConfirmed => hash_fields("phase35-cleanup-v1", &["true"]),
    }
}

fn parse_snapshot_identity(
    document: &str,
    field: &str,
) -> Result<(String, u64), Phase35EvidenceError> {
    let prefix = format!("{field}:");
    let maybe_json = document
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim));
    let Some(json) = maybe_json else {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    };
    let value: Value =
        serde_json::from_str(json).map_err(|_| Phase35EvidenceError::ValueRevisionMismatch)?;
    let Some(session) = value.get("bootSession").and_then(Value::as_str) else {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    };
    let Some(revision) = value
        .get("operatorSnapshotRevision")
        .and_then(Value::as_u64)
    else {
        return Err(Phase35EvidenceError::ValueRevisionMismatch);
    };
    Ok((session.to_owned(), revision))
}

#[cfg(test)]
pub(crate) mod tests;
