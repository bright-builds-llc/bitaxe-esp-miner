//! Exact private capture bundle and commit-redacted candidate derivation.

mod filesystem;
mod hardware;
mod synthetic;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::effects::ValidatedIndependentEffectInterval;
use super::runtime_identity::{
    validate_observed_runtime_identity_documents_with_hardware, ObservedRuntimeIdentityAdmission,
    ValidatedObservedRuntimeIdentity,
};
use super::substance::{
    validate_substantive_snapshot_documents, SubstantiveEvidenceAdmission, SubstantiveSnapshotJoin,
    ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};
use crate::phase35_evidence::sha256_hex;
use crate::phase36_broker::{Phase36LedgerRecord, Phase36LedgerState};

pub const PHASE36_PRIVATE_CAPTURE_SCHEMA: &str = "phase36-private-capture-v1";
pub const PHASE36_CAPTURE_CANDIDATE_SCHEMA: &str = "phase36-capture-candidate-v1";

pub use filesystem::{
    classify_candidate_files, inspect_candidate_file, replace_broker_document,
    write_candidate_from_private_file, CandidateClassificationProjection,
    CandidateInspectionProjection,
};
pub use hardware::{assemble_hardware_capture, HardwareCaptureAssembly};
pub use synthetic::write_synthetic_capture;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase36PrivateCaptureBundle {
    pub schema_version: String,
    pub board_category: String,
    pub substantive: SubstantiveCaptureDocuments,
    pub runtime_identity: RuntimeIdentityCaptureDocuments,
    pub broker: BrokerCaptureDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubstantiveCaptureDocuments {
    pub system_info_document: String,
    pub websocket_document: String,
    pub retained_document: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentityCaptureDocuments {
    pub exact_package_document: String,
    pub request_document: String,
    pub event_ledger_document: String,
    pub private_result_document: String,
    pub public_projection_document: String,
    #[serde(default)]
    pub hardware_observation_document: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureObservationSource {
    IndependentBrokerLedger,
    SupervisorAttestation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BrokerCaptureDocument {
    pub observation_source: CaptureObservationSource,
    pub capability_digest: String,
    pub package_digest: String,
    pub same_physical_device_observed: bool,
    pub interval_start_millis: u64,
    pub interval_end_millis: u64,
    pub records: Vec<Phase36LedgerRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureCandidateStatus {
    Eligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase36CaptureCandidate {
    pub schema_version: &'static str,
    pub status: CaptureCandidateStatus,
    pub board_category: &'static str,
    pub private_capture_digest: String,
    pub candidate_digest: String,
    pub sensors: ValidatedSensorSubstance,
    pub runtime_health: ValidatedRuntimeHealthSubstance,
    pub snapshot_join: SubstantiveSnapshotJoin,
    pub runtime_identity: ValidatedObservedRuntimeIdentity,
    pub effect_interval: ValidatedIndependentEffectInterval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36CaptureError {
    #[error("phase36_capture_document_invalid")]
    DocumentInvalid,
    #[error("phase36_capture_schema_invalid")]
    UnsupportedSchema,
    #[error("phase36_capture_board_invalid")]
    WrongBoard,
    #[error("phase36_capture_substance_invalid")]
    SubstanceInvalid,
    #[error("phase36_capture_substance_insufficient")]
    SubstanceInsufficient,
    #[error("phase36_capture_runtime_identity_invalid")]
    RuntimeIdentityInvalid,
    #[error("phase36_capture_runtime_identity_insufficient")]
    RuntimeIdentityInsufficient,
    #[error("phase36_capture_snapshot_join_invalid")]
    SnapshotJoinMismatch,
    #[error("phase36_capture_effect_source_invalid")]
    EffectSourceInvalid,
    #[error("phase36_capture_effect_ledger_invalid")]
    EffectLedgerInvalid,
    #[error("phase36_capture_effect_interval_failed")]
    EffectIntervalFailed,
    #[error("phase36_capture_package_join_invalid")]
    PackageJoinMismatch,
    #[error("phase36_capture_encoding_failed")]
    EncodingFailed,
}

pub fn classify_private_capture(
    private_bytes: &[u8],
) -> Result<Phase36CaptureCandidate, Phase36CaptureError> {
    let private_capture_digest = sha256_hex(private_bytes);
    let bundle = serde_json::from_slice::<Phase36PrivateCaptureBundle>(private_bytes)
        .map_err(|_| Phase36CaptureError::DocumentInvalid)?;
    validate_bundle_identity(&bundle)?;

    let substance = validate_substantive_snapshot_documents(
        &bundle.substantive.system_info_document,
        &bundle.substantive.websocket_document,
        &bundle.substantive.retained_document,
    )
    .map_err(|_| Phase36CaptureError::SubstanceInvalid)?;
    let SubstantiveEvidenceAdmission::Validated { evidence } = substance else {
        return Err(Phase36CaptureError::SubstanceInsufficient);
    };

    let identity_documents = &bundle.runtime_identity;
    let runtime_identity = validate_observed_runtime_identity_documents_with_hardware(
        &identity_documents.exact_package_document,
        Some(&identity_documents.request_document),
        (!identity_documents.event_ledger_document.is_empty())
            .then_some(identity_documents.event_ledger_document.as_str()),
        Some(&identity_documents.private_result_document),
        Some(&identity_documents.public_projection_document),
        identity_documents.hardware_observation_document.as_deref(),
    )
    .map_err(|_| Phase36CaptureError::RuntimeIdentityInvalid)?;
    let ObservedRuntimeIdentityAdmission::Validated { identity } = runtime_identity else {
        return Err(Phase36CaptureError::RuntimeIdentityInsufficient);
    };

    if evidence.join.operator_boot_session_digest != identity.boot_b_session_digest {
        return Err(Phase36CaptureError::SnapshotJoinMismatch);
    }
    if bundle.broker.observation_source != CaptureObservationSource::IndependentBrokerLedger {
        return Err(Phase36CaptureError::EffectSourceInvalid);
    }
    if !valid_digest(&bundle.broker.capability_digest) {
        return Err(Phase36CaptureError::EffectSourceInvalid);
    }
    if !bundle.broker.same_physical_device_observed || !identity.same_physical_device {
        return Err(Phase36CaptureError::SnapshotJoinMismatch);
    }
    if bundle.broker.package_digest != identity.exact_package.package_digest {
        return Err(Phase36CaptureError::PackageJoinMismatch);
    }

    let effect_interval = validate_broker_interval(&bundle.broker)?;
    let mut candidate = Phase36CaptureCandidate {
        schema_version: PHASE36_CAPTURE_CANDIDATE_SCHEMA,
        status: CaptureCandidateStatus::Eligible,
        board_category: "205",
        private_capture_digest,
        candidate_digest: String::new(),
        sensors: evidence.sensors,
        runtime_health: evidence.runtime_health,
        snapshot_join: evidence.join,
        runtime_identity: *identity,
        effect_interval,
    };
    candidate.candidate_digest = candidate_digest(&candidate)?;
    Ok(candidate)
}

fn validate_bundle_identity(
    bundle: &Phase36PrivateCaptureBundle,
) -> Result<(), Phase36CaptureError> {
    if bundle.schema_version != PHASE36_PRIVATE_CAPTURE_SCHEMA {
        return Err(Phase36CaptureError::UnsupportedSchema);
    }
    if bundle.board_category != "205" {
        return Err(Phase36CaptureError::WrongBoard);
    }
    Ok(())
}

fn validate_broker_interval(
    broker: &BrokerCaptureDocument,
) -> Result<ValidatedIndependentEffectInterval, Phase36CaptureError> {
    let mut state = Phase36LedgerState::start(broker.interval_start_millis)
        .map_err(|_| Phase36CaptureError::EffectLedgerInvalid)?;
    for record in &broker.records {
        state
            .apply(record)
            .map_err(|_| Phase36CaptureError::EffectLedgerInvalid)?;
    }
    let interval = state
        .seal(broker.interval_end_millis)
        .map_err(|_| Phase36CaptureError::EffectLedgerInvalid)?;
    if interval.maybe_first_failure().is_some()
        || interval.maybe_secondary_failure().is_some()
        || interval.effect_count() != 8
    {
        return Err(Phase36CaptureError::EffectIntervalFailed);
    }
    Ok(ValidatedIndependentEffectInterval::from(&interval))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn candidate_digest(candidate: &Phase36CaptureCandidate) -> Result<String, Phase36CaptureError> {
    let value = serde_json::to_value(candidate).map_err(|_| Phase36CaptureError::EncodingFailed)?;
    candidate_digest_from_value(&value)
}

pub(super) fn candidate_digest_from_value(
    value: &serde_json::Value,
) -> Result<String, Phase36CaptureError> {
    let mut facts = value.clone();
    let object = facts
        .as_object_mut()
        .ok_or(Phase36CaptureError::DocumentInvalid)?;
    object.remove("candidate_digest");
    let bytes = serde_json::to_vec(&facts).map_err(|_| Phase36CaptureError::EncodingFailed)?;
    Ok(sha256_hex(&bytes))
}
