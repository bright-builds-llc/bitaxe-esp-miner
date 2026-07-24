use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    candidate_digest_from_value, classify_private_capture, Phase36CaptureCandidate,
    PHASE36_CAPTURE_CANDIDATE_SCHEMA,
};
use crate::phase35_evidence::sha256_hex;

const CANDIDATE_KEYS: [&str; 10] = [
    "board_category",
    "candidate_digest",
    "effect_interval",
    "private_capture_digest",
    "runtime_health",
    "runtime_identity",
    "schema_version",
    "sensors",
    "snapshot_join",
    "status",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateInspectionProjection {
    pub category: &'static str,
    pub candidate_digest: String,
    pub private_capture_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateClassificationProjection {
    pub category: &'static str,
    pub candidate_digest: String,
    pub private_capture_digest: String,
    pub sensor_claim_digest: String,
    pub runtime_health_claim_digest: String,
    pub runtime_identity_claim_digest: String,
    pub effect_claim_digest: String,
}

#[derive(Debug, Deserialize)]
struct CandidateIdentity {
    schema_version: String,
    status: String,
    board_category: String,
    private_capture_digest: String,
    candidate_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CaptureFileError {
    #[error("phase36_capture_private_input_invalid")]
    PrivateInputInvalid,
    #[error("phase36_capture_candidate_invalid")]
    CandidateInvalid,
    #[error("phase36_capture_output_invalid")]
    OutputInvalid,
    #[error("phase36_capture_path_alias")]
    PathAlias,
    #[error("phase36_capture_private_input_changed")]
    PrivateInputChanged,
    #[error("phase36_capture_candidate_changed")]
    CandidateChanged,
    #[error("phase36_capture_classification_failed")]
    ClassificationFailed,
    #[error("phase36_capture_output_failed")]
    OutputFailed,
}

pub fn write_candidate_from_private_file(
    private_input: &Utf8Path,
    candidate_output: &Utf8Path,
) -> Result<Phase36CaptureCandidate, CaptureFileError> {
    validate_private_file(private_input)?;
    reject_alias(private_input, candidate_output)?;
    let private_bytes =
        fs::read(private_input).map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    let private_digest = sha256_hex(&private_bytes);
    let candidate = classify_private_capture(&private_bytes)
        .map_err(|_| CaptureFileError::ClassificationFailed)?;
    if sha256_hex(&fs::read(private_input).map_err(|_| CaptureFileError::PrivateInputChanged)?)
        != private_digest
    {
        return Err(CaptureFileError::PrivateInputChanged);
    }
    let bytes =
        serde_json::to_vec_pretty(&candidate).map_err(|_| CaptureFileError::OutputFailed)?;
    write_new_private_file(candidate_output, &bytes)?;
    if sha256_hex(&fs::read(private_input).map_err(|_| CaptureFileError::PrivateInputChanged)?)
        != private_digest
    {
        return Err(CaptureFileError::PrivateInputChanged);
    }
    Ok(candidate)
}

pub fn inspect_candidate_file(
    candidate_input: &Utf8Path,
) -> Result<CandidateInspectionProjection, CaptureFileError> {
    let bytes = fs::read(candidate_input).map_err(|_| CaptureFileError::CandidateInvalid)?;
    inspect_candidate_bytes(&bytes)
}

pub fn classify_candidate_files(
    private_input: &Utf8Path,
    candidate_input: &Utf8Path,
    classification_output: &Utf8Path,
) -> Result<CandidateClassificationProjection, CaptureFileError> {
    validate_private_file(private_input)?;
    reject_alias(private_input, candidate_input)?;
    reject_alias(private_input, classification_output)?;
    reject_alias(candidate_input, classification_output)?;
    let private_bytes =
        fs::read(private_input).map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    let candidate_bytes =
        fs::read(candidate_input).map_err(|_| CaptureFileError::CandidateInvalid)?;
    let private_digest = sha256_hex(&private_bytes);
    let candidate_file_digest = sha256_hex(&candidate_bytes);
    let inspection = inspect_candidate_bytes(&candidate_bytes)?;
    let candidate = classify_private_capture(&private_bytes)
        .map_err(|_| CaptureFileError::ClassificationFailed)?;
    if candidate.private_capture_digest != inspection.private_capture_digest
        || candidate.candidate_digest != inspection.candidate_digest
    {
        return Err(CaptureFileError::CandidateInvalid);
    }
    let projection = CandidateClassificationProjection {
        category: "classification_complete",
        candidate_digest: candidate.candidate_digest,
        private_capture_digest: candidate.private_capture_digest,
        sensor_claim_digest: candidate.sensors.claim_fact_digest,
        runtime_health_claim_digest: candidate.runtime_health.claim_fact_digest,
        runtime_identity_claim_digest: candidate.runtime_identity.claim_fact_digest,
        effect_claim_digest: candidate.effect_interval.claim_fact_digest,
    };
    let output =
        serde_json::to_vec_pretty(&projection).map_err(|_| CaptureFileError::OutputFailed)?;
    write_new_private_file(classification_output, &output)?;
    if sha256_hex(&fs::read(private_input).map_err(|_| CaptureFileError::PrivateInputChanged)?)
        != private_digest
    {
        return Err(CaptureFileError::PrivateInputChanged);
    }
    if sha256_hex(&fs::read(candidate_input).map_err(|_| CaptureFileError::CandidateChanged)?)
        != candidate_file_digest
    {
        return Err(CaptureFileError::CandidateChanged);
    }
    Ok(projection)
}

fn inspect_candidate_bytes(
    bytes: &[u8],
) -> Result<CandidateInspectionProjection, CaptureFileError> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes)
        .map_err(|_| CaptureFileError::CandidateInvalid)?;
    let object = value
        .as_object()
        .ok_or(CaptureFileError::CandidateInvalid)?;
    let keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    if keys != CANDIDATE_KEYS {
        return Err(CaptureFileError::CandidateInvalid);
    }
    let identity = serde_json::from_value::<CandidateIdentity>(value.clone())
        .map_err(|_| CaptureFileError::CandidateInvalid)?;
    if identity.schema_version != PHASE36_CAPTURE_CANDIDATE_SCHEMA
        || identity.status != "eligible"
        || identity.board_category != "205"
        || !valid_digest(&identity.private_capture_digest)
        || !valid_digest(&identity.candidate_digest)
    {
        return Err(CaptureFileError::CandidateInvalid);
    }
    let computed =
        candidate_digest_from_value(&value).map_err(|_| CaptureFileError::CandidateInvalid)?;
    if computed != identity.candidate_digest {
        return Err(CaptureFileError::CandidateInvalid);
    }
    Ok(CandidateInspectionProjection {
        category: "candidate_eligible",
        candidate_digest: identity.candidate_digest,
        private_capture_digest: identity.private_capture_digest,
    })
}

fn validate_private_file(path: &Utf8Path) -> Result<(), CaptureFileError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(CaptureFileError::PrivateInputInvalid);
    }
    Ok(())
}

fn reject_alias(left: &Utf8Path, right: &Utf8Path) -> Result<(), CaptureFileError> {
    if left == right {
        return Err(CaptureFileError::PathAlias);
    }
    Ok(())
}

pub(super) fn write_new_private_file(
    path: &Utf8Path,
    bytes: &[u8],
) -> Result<(), CaptureFileError> {
    let parent = path.parent().ok_or(CaptureFileError::OutputInvalid)?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| CaptureFileError::OutputInvalid)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CaptureFileError::OutputInvalid);
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| CaptureFileError::OutputFailed)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| CaptureFileError::OutputFailed)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
