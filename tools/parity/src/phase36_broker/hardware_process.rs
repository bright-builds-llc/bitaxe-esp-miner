use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::contract::Phase36RecoveryIdentity;
use super::hardware::{
    run_detector_process, run_phase36_hardware_transaction_with, Phase36HardwareDisposition,
    Phase36HardwareTransactionBoundary, Phase36HardwareTransactionError, Phase36OperationResult,
};
use super::{
    Phase36AllowedOperation, Phase36BrokerFailure, Phase36LedgerRecord, Phase36RecoveryDisposition,
    PrivateAppendOnlyLedger,
};
use crate::phase36_evidence::capture::{
    replace_broker_document, write_candidate_from_private_file, BrokerCaptureDocument,
    CaptureObservationSource,
};

mod effect_result;
mod process_boundary;

use effect_result::{
    classify_effect_result, classify_missing_effect_result, maybe_read_effect_result,
    operation_name,
};
use process_boundary::ProcessTransactionBoundary;

const EFFECT_ADAPTER_PROGRAM: &str = "phase36-hardware-effect";
const MIN_CAPTURE_TIMEOUT_SECONDS: u64 = 360;
const MIN_WALL_CLOCK_TIMEOUT_SECONDS: u64 = 420;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptHandle {
    schema_version: String,
    child_name: String,
    capability_digest: String,
    source_commit: String,
    reference_commit: String,
    evaluator_identity_digest: String,
    target: String,
    board: String,
    asic: String,
    manifest_path: String,
    manifest_digest: String,
    firmware_elf_path: String,
    firmware_elf_digest: String,
    executable_image_path: String,
    executable_image_digest: String,
    factory_image_path: String,
    factory_image_digest: String,
    package_identity_digest: String,
}

#[derive(Debug, Serialize)]
struct DetectorFacts<'a> {
    schema_version: &'static str,
    board_category: &'static str,
    target: &'static str,
    asic: &'static str,
    candidate_count: u8,
    port: &'a str,
    physical_identity_digest: &'a str,
}

#[derive(Debug, Serialize)]
struct AttemptSeal<'a> {
    schema_version: &'static str,
    status: &'a str,
    first_failure: Option<Phase36BrokerFailure>,
    secondary_failure: Option<Phase36BrokerFailure>,
    recovery_disposition: Phase36RecoveryDisposition,
    capability_digest: &'a str,
    package_identity_digest: &'a str,
    candidate_digest: Option<&'a str>,
    private_capture_digest: Option<&'a str>,
}

pub fn run_phase36_hardware_transaction(
    board: u16,
    private_parent: &Utf8Path,
    attempt_handle_file: &Utf8Path,
    candidate_output: &Utf8Path,
    wifi_credentials: &Utf8Path,
    capture_timeout_seconds: u64,
) -> Result<Phase36HardwareDisposition, Phase36HardwareTransactionError> {
    if board != 205 || capture_timeout_seconds < MIN_CAPTURE_TIMEOUT_SECONDS {
        return Err(Phase36HardwareTransactionError::PrivateAttempt);
    }
    let mut boundary = ProcessTransactionBoundary::load(
        private_parent,
        attempt_handle_file,
        candidate_output,
        wifi_credentials,
        capture_timeout_seconds,
    )?;
    let start = boundary.interval_start_millis;
    run_phase36_hardware_transaction_with(&mut boundary, start)
}

fn validate_private_directory(path: &Utf8Path) -> Result<(), Phase36HardwareTransactionError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(Phase36HardwareTransactionError::PrivateAttempt);
    }
    Ok(())
}

fn validate_private_file(path: &Utf8Path) -> Result<(), Phase36HardwareTransactionError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(Phase36HardwareTransactionError::PrivateAttempt);
    }
    Ok(())
}

fn validate_wifi_credentials(path: &Utf8Path) -> Result<(), Phase36HardwareTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::DetectorFailed)
    })?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(Phase36HardwareTransactionError::EffectFailed(
            Phase36BrokerFailure::DetectorFailed,
        ));
    }
    Ok(())
}

fn write_new_private_json<T: Serialize>(
    path: &Utf8Path,
    value: &T,
) -> Result<(), Phase36HardwareTransactionError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|_| Phase36HardwareTransactionError::Seal)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| Phase36HardwareTransactionError::Seal)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| Phase36HardwareTransactionError::Seal)
}

fn valid_child_name(value: &str) -> bool {
    value.len() == 24
        && value.starts_with("attempt-")
        && value[8..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_origin(value: &str) -> bool {
    let maybe_authority = value
        .strip_prefix("http://")
        .or_else(|| value.strip_prefix("https://"));
    let Some(authority) = maybe_authority else {
        return false;
    };
    !authority.is_empty() && !authority.contains('/')
}

fn current_millis() -> Result<u64, Phase36HardwareTransactionError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?
        .as_millis();
    u64::try_from(millis).map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)
}

fn resolve_effect_adapter() -> Result<Utf8PathBuf, Phase36HardwareTransactionError> {
    let mut candidates = Vec::new();
    if let Some(runfiles) = std::env::var_os("RUNFILES_DIR")
        .and_then(|value| Utf8PathBuf::from_path_buf(value.into()).ok())
    {
        candidates.push(runfiles.join("_main/scripts/phase36_hardware_effect"));
    }
    if let Some(workspace) = std::env::var_os("BUILD_WORKSPACE_DIRECTORY")
        .and_then(|value| Utf8PathBuf::from_path_buf(value.into()).ok())
    {
        candidates.push(workspace.join("bazel-bin/scripts/phase36_hardware_effect"));
    }
    for candidate in candidates {
        let Ok(canonical) = fs::canonicalize(&candidate) else {
            continue;
        };
        let Ok(canonical) = Utf8PathBuf::from_path_buf(canonical) else {
            continue;
        };
        let Ok(metadata) = fs::metadata(&canonical) else {
            continue;
        };
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            return Ok(canonical);
        }
    }
    let _ = EFFECT_ADAPTER_PROGRAM;
    Err(Phase36HardwareTransactionError::PrivateAttempt)
}

fn sha256_hex(bytes: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256_file(path: &Utf8Path) -> Result<String, Phase36HardwareTransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::AdmissionFailed)
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Phase36HardwareTransactionError::EffectFailed(
            Phase36BrokerFailure::AdmissionFailed,
        ));
    }
    let bytes = fs::read(path).map_err(|_| {
        Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::AdmissionFailed)
    })?;
    Ok(sha256_hex(bytes))
}
