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

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Phase36EffectStatus {
    Completed,
    FailedNoDeviceEffect,
    FailedConfirmedPartialDeviceEffect,
    FailedAfterCompletedDeviceEffect,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase36EffectResult {
    schema_version: String,
    operation: Phase36AllowedOperation,
    status: Phase36EffectStatus,
    failure: Option<Phase36BrokerFailure>,
    package_identity_digest: String,
    factory_image_digest: String,
}

struct ProcessTransactionBoundary {
    attempt_child: Utf8PathBuf,
    candidate_output: Utf8PathBuf,
    wifi_credentials: Utf8PathBuf,
    effect_adapter: Utf8PathBuf,
    handle: AttemptHandle,
    capture_timeout_seconds: u64,
    maybe_ledger: Option<PrivateAppendOnlyLedger>,
    records: Vec<Phase36LedgerRecord>,
    maybe_port: Option<String>,
    maybe_origin: Option<String>,
    interval_start_millis: u64,
    interval_end_millis: u64,
}

impl ProcessTransactionBoundary {
    fn load(
        private_parent: &Utf8Path,
        attempt_handle_file: &Utf8Path,
        candidate_output: &Utf8Path,
        wifi_credentials: &Utf8Path,
        capture_timeout_seconds: u64,
    ) -> Result<Self, Phase36HardwareTransactionError> {
        validate_private_directory(private_parent)?;
        validate_private_file(attempt_handle_file)?;
        if candidate_output.exists() {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        let bytes = fs::read(attempt_handle_file)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        let handle = serde_json::from_slice::<AttemptHandle>(&bytes)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        if handle.schema_version != "phase36-attempt-handle-v2"
            || !valid_child_name(&handle.child_name)
            || !valid_digest(&handle.capability_digest)
            || !valid_commit(&handle.source_commit)
            || !valid_commit(&handle.reference_commit)
            || !valid_digest(&handle.evaluator_identity_digest)
            || handle.target != "xtensa-esp32s3-espidf"
            || handle.board != "205"
            || handle.asic != "BM1366"
            || !valid_digest(&handle.manifest_digest)
            || !valid_digest(&handle.firmware_elf_digest)
            || !valid_digest(&handle.executable_image_digest)
            || !valid_digest(&handle.factory_image_digest)
            || !valid_digest(&handle.package_identity_digest)
        {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        let attempt_child = private_parent.join(&handle.child_name);
        if attempt_child.exists() {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        Ok(Self {
            attempt_child,
            candidate_output: candidate_output.to_owned(),
            wifi_credentials: wifi_credentials.to_owned(),
            effect_adapter: resolve_effect_adapter()?,
            handle,
            capture_timeout_seconds,
            maybe_ledger: None,
            records: Vec::new(),
            maybe_port: None,
            maybe_origin: None,
            interval_start_millis: current_millis()?,
            interval_end_millis: 0,
        })
    }

    fn validate_exact_package(&self) -> Result<(), Phase36HardwareTransactionError> {
        if self.handle.evaluator_identity_digest
            != crate::phase36_evidence::current_phase36_evidence_evaluator_digest()
        {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::AdmissionFailed,
            ));
        }
        for (path, expected_digest) in [
            (&self.handle.manifest_path, &self.handle.manifest_digest),
            (
                &self.handle.firmware_elf_path,
                &self.handle.firmware_elf_digest,
            ),
            (
                &self.handle.executable_image_path,
                &self.handle.executable_image_digest,
            ),
            (
                &self.handle.factory_image_path,
                &self.handle.factory_image_digest,
            ),
        ] {
            if sha256_file(Utf8Path::new(path))?.as_str() != expected_digest {
                return Err(Phase36HardwareTransactionError::EffectFailed(
                    Phase36BrokerFailure::AdmissionFailed,
                ));
            }
        }
        let package_digest = sha256_hex(
            [
                b"phase36-package-identity-v1\\0".as_slice(),
                self.handle.source_commit.as_bytes(),
                b"\\0",
                self.handle.reference_commit.as_bytes(),
                b"\\0",
                self.handle.manifest_digest.as_bytes(),
                b"\\0",
                self.handle.firmware_elf_digest.as_bytes(),
                b"\\0",
                self.handle.executable_image_digest.as_bytes(),
                b"\\0",
                self.handle.factory_image_digest.as_bytes(),
            ]
            .concat(),
        );
        if package_digest != self.handle.package_identity_digest {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::AdmissionFailed,
            ));
        }
        Ok(())
    }

    fn run_detector(&mut self) -> Result<(), Phase36HardwareTransactionError> {
        let output = run_detector_process().map_err(|_| {
            Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::DetectorFailed)
        })?;
        if !output.status.success() {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::DetectorFailed,
            ));
        }
        let stdout = String::from_utf8(output.stdout).map_err(|_| {
            Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::DetectorFailed)
        })?;
        let ports = stdout
            .lines()
            .filter_map(|line| line.strip_prefix("port="))
            .filter(|port| !port.is_empty())
            .collect::<Vec<_>>();
        if ports.len() != 1 {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::DetectorFailed,
            ));
        }
        let port = ports[0].to_owned();
        let physical_identity_digest = sha256_hex(
            [
                b"phase36-physical-device-v1\0".as_slice(),
                port.as_bytes(),
                b"\0",
                &output.stderr,
            ]
            .concat(),
        );
        let facts = DetectorFacts {
            schema_version: "phase36-detector-facts-v1",
            board_category: "205",
            target: "xtensa-esp32s3-espidf",
            asic: "BM1366",
            candidate_count: 1,
            port: &port,
            physical_identity_digest: &physical_identity_digest,
        };
        write_new_private_json(&self.attempt_child.join("detector-facts.json"), &facts)?;
        validate_wifi_credentials(&self.wifi_credentials)?;
        self.maybe_port = Some(port);
        Ok(())
    }

    fn run_effect_classified(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
        let port = match (operation, self.maybe_port.as_deref()) {
            (Phase36AllowedOperation::Cleanup, None) => "unavailable",
            (_, Some(port)) => port,
            _ => {
                return Ok(classify_missing_effect_result(operation));
            }
        };
        let result_path = self
            .attempt_child
            .join(format!("effect-result-{}.json", operation_name(operation)));
        let mut command = Command::new("perl");
        command
            .arg("-e")
            .arg("alarm shift; exec @ARGV")
            .arg(MIN_WALL_CLOCK_TIMEOUT_SECONDS.to_string())
            .arg(&self.effect_adapter)
            .arg(format!("operation={}", operation_name(operation)))
            .arg(format!("board={}", 205))
            .arg(format!("port={port}"))
            .arg(format!("attempt-child={}", self.attempt_child))
            .arg(format!(
                "package-identity-digest={}",
                self.handle.package_identity_digest
            ))
            .arg(format!("manifest-path={}", self.handle.manifest_path))
            .arg(format!("manifest-digest={}", self.handle.manifest_digest))
            .arg(format!(
                "firmware-elf-path={}",
                self.handle.firmware_elf_path
            ))
            .arg(format!(
                "firmware-elf-digest={}",
                self.handle.firmware_elf_digest
            ))
            .arg(format!(
                "executable-image-path={}",
                self.handle.executable_image_path
            ))
            .arg(format!(
                "executable-image-digest={}",
                self.handle.executable_image_digest
            ))
            .arg(format!(
                "factory-image-path={}",
                self.handle.factory_image_path
            ))
            .arg(format!(
                "factory-image-digest={}",
                self.handle.factory_image_digest
            ))
            .arg(format!(
                "capture-timeout-seconds={}",
                self.capture_timeout_seconds
            ))
            .arg(format!(
                "wall-clock-timeout-seconds={MIN_WALL_CLOCK_TIMEOUT_SECONDS}"
            ))
            .arg(format!("result-path={result_path}"))
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        if operation == Phase36AllowedOperation::ExactPackageFlash {
            command.arg(format!("wifi-credentials={}", self.wifi_credentials));
        }
        if matches!(
            operation,
            Phase36AllowedOperation::ReadOnlySystemInfo
                | Phase36AllowedOperation::ReadOnlyWebSocket
                | Phase36AllowedOperation::ReadOnlyRetainedFacts
        ) {
            let origin = self.maybe_origin.as_deref().ok_or(
                Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::CaptureFailed),
            )?;
            command.arg(format!("trusted-origin={origin}"));
        }
        let Ok(output) = command.output() else {
            return Ok(classify_missing_effect_result(operation));
        };
        let Some(result) = read_effect_result(&result_path) else {
            return Ok(classify_missing_effect_result(operation));
        };
        let classified = classify_effect_result(
            result,
            operation,
            output.status.success(),
            &self.handle.package_identity_digest,
            &self.handle.factory_image_digest,
        );
        if operation == Phase36AllowedOperation::PassiveSerialObservation
            && matches!(classified, Phase36OperationResult::Completed { .. })
        {
            let stdout = String::from_utf8(output.stdout).map_err(|_| {
                Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::CaptureFailed)
            })?;
            let origins = stdout
                .lines()
                .filter_map(|line| line.strip_prefix("trusted_origin="))
                .filter(|origin| valid_origin(origin))
                .collect::<Vec<_>>();
            if origins.len() != 1 {
                return Ok(Phase36OperationResult::Failed {
                    failure: Phase36BrokerFailure::CaptureFailed,
                    maybe_partial_device_effect: None,
                });
            }
            self.maybe_origin = Some(origins[0].to_owned());
        }
        Ok(classified)
    }

    fn write_seal(
        &mut self,
        disposition: Phase36HardwareDisposition,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.interval_end_millis = current_millis()?.max(self.interval_start_millis + 1);
        let (status, first_failure, secondary_failure, recovery_disposition) = match disposition {
            Phase36HardwareDisposition::SealedEligible => (
                "sealed_eligible",
                None,
                None,
                Phase36RecoveryDisposition::NotRequired,
            ),
            Phase36HardwareDisposition::SealedNonPromotion {
                first_failure,
                secondary_failure,
                recovery_disposition,
            } => (
                "sealed_non_promotion",
                Some(first_failure),
                secondary_failure,
                recovery_disposition,
            ),
        };
        let mut maybe_candidate_digest = None;
        let mut maybe_private_capture_digest = None;
        if disposition == Phase36HardwareDisposition::SealedEligible {
            let private_capture = self.attempt_child.join("private-capture.json");
            let broker = BrokerCaptureDocument {
                observation_source: CaptureObservationSource::IndependentBrokerLedger,
                capability_digest: self.handle.capability_digest.clone(),
                package_digest: self.handle.package_identity_digest.clone(),
                same_physical_device_observed: true,
                interval_start_millis: self.interval_start_millis,
                interval_end_millis: self.interval_end_millis,
                records: self.records.clone(),
            };
            replace_broker_document(&private_capture, broker)
                .map_err(|_| Phase36HardwareTransactionError::Seal)?;
            let candidate =
                write_candidate_from_private_file(&private_capture, &self.candidate_output)
                    .map_err(|_| Phase36HardwareTransactionError::Seal)?;
            maybe_candidate_digest = Some(candidate.candidate_digest);
            maybe_private_capture_digest = Some(candidate.private_capture_digest);
        }
        let seal = AttemptSeal {
            schema_version: "phase36-attempt-seal-v2",
            status,
            first_failure,
            secondary_failure,
            recovery_disposition,
            capability_digest: &self.handle.capability_digest,
            package_identity_digest: &self.handle.package_identity_digest,
            candidate_digest: maybe_candidate_digest.as_deref(),
            private_capture_digest: maybe_private_capture_digest.as_deref(),
        };
        write_new_private_json(&self.attempt_child.join("seal.json"), &seal)
    }
}

impl Phase36HardwareTransactionBoundary for ProcessTransactionBoundary {
    fn prepare_private_attempt(&mut self) -> Result<(), Phase36HardwareTransactionError> {
        DirBuilder::new()
            .mode(0o700)
            .create(&self.attempt_child)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        let metadata = fs::symlink_metadata(&self.attempt_child)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        if metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        self.maybe_ledger = Some(
            PrivateAppendOnlyLedger::create(&self.attempt_child.join("effect-ledger.jsonl"))
                .map_err(Phase36HardwareTransactionError::Ledger)?,
        );
        Ok(())
    }

    fn record(
        &mut self,
        record: &Phase36LedgerRecord,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.maybe_ledger
            .as_mut()
            .ok_or(Phase36HardwareTransactionError::PrivateAttempt)?
            .append(record)?;
        self.records.push(record.clone());
        Ok(())
    }

    fn execute(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<(), Phase36HardwareTransactionError> {
        match operation {
            Phase36AllowedOperation::ExactPackageAdmission => self.validate_exact_package(),
            Phase36AllowedOperation::Board205DetectorProbe => self.run_detector(),
            _ => match self.run_effect_classified(operation)? {
                Phase36OperationResult::Completed { .. } => Ok(()),
                Phase36OperationResult::Failed { failure, .. } => {
                    Err(Phase36HardwareTransactionError::EffectFailed(failure))
                }
            },
        }
    }

    fn recovery_identity(
        &self,
    ) -> Result<Phase36RecoveryIdentity, Phase36HardwareTransactionError> {
        Phase36RecoveryIdentity::new(
            self.handle.package_identity_digest.clone(),
            self.handle.factory_image_digest.clone(),
        )
        .map_err(|_| Phase36HardwareTransactionError::InvalidRecoveryAuthority)
    }

    fn execute_classified(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
        match operation {
            Phase36AllowedOperation::ExactPackageAdmission
            | Phase36AllowedOperation::Board205DetectorProbe => {
                let result = self.execute(operation);
                match result {
                    Ok(()) => Ok(Phase36OperationResult::Completed {
                        maybe_completed_device_effect: None,
                    }),
                    Err(Phase36HardwareTransactionError::EffectFailed(failure)) => {
                        Ok(Phase36OperationResult::Failed {
                            failure,
                            maybe_partial_device_effect: None,
                        })
                    }
                    Err(error) => Err(error),
                }
            }
            _ => self.run_effect_classified(operation),
        }
    }

    fn seal(
        &mut self,
        disposition: Phase36HardwareDisposition,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.maybe_ledger
            .as_mut()
            .ok_or(Phase36HardwareTransactionError::PrivateAttempt)?
            .seal()?;
        self.write_seal(disposition)
    }
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

fn operation_name(operation: Phase36AllowedOperation) -> &'static str {
    match operation {
        Phase36AllowedOperation::ExactPackageAdmission => "exact-package-admission",
        Phase36AllowedOperation::Board205DetectorProbe => "board-205-detector-probe",
        Phase36AllowedOperation::ExactPackageFlash => "exact-package-flash",
        Phase36AllowedOperation::PassiveSerialObservation => "passive-serial-observation",
        Phase36AllowedOperation::ReadOnlySystemInfo => "read-only-system-info",
        Phase36AllowedOperation::ReadOnlyWebSocket => "read-only-websocket",
        Phase36AllowedOperation::ReadOnlyRetainedFacts => "read-only-retained-facts",
        Phase36AllowedOperation::TypedRecovery => "typed-recovery",
        Phase36AllowedOperation::Cleanup => "cleanup",
    }
}

fn failure_for_operation(operation: Phase36AllowedOperation) -> Phase36BrokerFailure {
    match operation {
        Phase36AllowedOperation::ExactPackageAdmission => Phase36BrokerFailure::AdmissionFailed,
        Phase36AllowedOperation::Board205DetectorProbe => Phase36BrokerFailure::DetectorFailed,
        Phase36AllowedOperation::ExactPackageFlash => Phase36BrokerFailure::FlashFailed,
        Phase36AllowedOperation::PassiveSerialObservation
        | Phase36AllowedOperation::ReadOnlySystemInfo
        | Phase36AllowedOperation::ReadOnlyWebSocket
        | Phase36AllowedOperation::ReadOnlyRetainedFacts => Phase36BrokerFailure::CaptureFailed,
        Phase36AllowedOperation::TypedRecovery => Phase36BrokerFailure::RecoveryFailed,
        Phase36AllowedOperation::Cleanup => Phase36BrokerFailure::CleanupFailed,
    }
}

fn read_effect_result(path: &Utf8Path) -> Option<Phase36EffectResult> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn classify_missing_effect_result(operation: Phase36AllowedOperation) -> Phase36OperationResult {
    let failure = if operation == Phase36AllowedOperation::ExactPackageFlash {
        Phase36BrokerFailure::InvocationConstructionFailed
    } else {
        failure_for_operation(operation)
    };
    Phase36OperationResult::Failed {
        failure,
        maybe_partial_device_effect: None,
    }
}

fn classify_effect_result(
    result: Phase36EffectResult,
    operation: Phase36AllowedOperation,
    process_succeeded: bool,
    package_identity_digest: &str,
    factory_image_digest: &str,
) -> Phase36OperationResult {
    let identity_matches = result.package_identity_digest == package_identity_digest
        && result.factory_image_digest == factory_image_digest
        && valid_digest(&result.package_identity_digest)
        && valid_digest(&result.factory_image_digest);
    if result.schema_version != "phase36-effect-result-v1"
        || result.operation != operation
        || !identity_matches
    {
        return classify_missing_effect_result(operation);
    }

    match (result.status, result.failure, process_succeeded) {
        (Phase36EffectStatus::Completed, None, true) => {
            let maybe_completed_device_effect =
                if operation == Phase36AllowedOperation::ExactPackageFlash {
                    Phase36RecoveryIdentity::new(
                        result.package_identity_digest,
                        result.factory_image_digest,
                    )
                    .ok()
                } else {
                    None
                };
            Phase36OperationResult::Completed {
                maybe_completed_device_effect,
            }
        }
        (Phase36EffectStatus::FailedNoDeviceEffect, Some(failure), false)
            if failure.valid_for(operation) =>
        {
            Phase36OperationResult::Failed {
                failure,
                maybe_partial_device_effect: None,
            }
        }
        (
            Phase36EffectStatus::FailedConfirmedPartialDeviceEffect
            | Phase36EffectStatus::FailedAfterCompletedDeviceEffect,
            Some(Phase36BrokerFailure::FlashFailed),
            false,
        ) if operation == Phase36AllowedOperation::ExactPackageFlash => {
            let maybe_partial_device_effect = Phase36RecoveryIdentity::new(
                result.package_identity_digest,
                result.factory_image_digest,
            )
            .ok();
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::FlashFailed,
                maybe_partial_device_effect,
            }
        }
        _ => classify_missing_effect_result(operation),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn effect_result(
        status: Phase36EffectStatus,
        failure: Option<Phase36BrokerFailure>,
        package_digest: &str,
        factory_digest: &str,
    ) -> Phase36EffectResult {
        Phase36EffectResult {
            schema_version: "phase36-effect-result-v1".to_owned(),
            operation: Phase36AllowedOperation::ExactPackageFlash,
            status,
            failure,
            package_identity_digest: package_digest.to_owned(),
            factory_image_digest: factory_digest.to_owned(),
        }
    }

    #[test]
    fn closed_partial_flash_result_carries_same_image_recovery_authority() {
        // Arrange
        let package_digest = "a".repeat(64);
        let factory_digest = "b".repeat(64);
        let result = effect_result(
            Phase36EffectStatus::FailedConfirmedPartialDeviceEffect,
            Some(Phase36BrokerFailure::FlashFailed),
            &package_digest,
            &factory_digest,
        );

        // Act
        let classified = classify_effect_result(
            result,
            Phase36AllowedOperation::ExactPackageFlash,
            false,
            &package_digest,
            &factory_digest,
        );

        // Assert
        assert_eq!(
            classified,
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::FlashFailed,
                maybe_partial_device_effect: Some(
                    Phase36RecoveryIdentity::new(package_digest, factory_digest)
                        .expect("fixture identity"),
                ),
            }
        );
    }

    #[test]
    fn mismatched_closed_effect_identity_fails_before_recovery_authority() {
        // Arrange
        let package_digest = "a".repeat(64);
        let factory_digest = "b".repeat(64);
        let result = effect_result(
            Phase36EffectStatus::FailedConfirmedPartialDeviceEffect,
            Some(Phase36BrokerFailure::FlashFailed),
            &"c".repeat(64),
            &factory_digest,
        );

        // Act
        let classified = classify_effect_result(
            result,
            Phase36AllowedOperation::ExactPackageFlash,
            false,
            &package_digest,
            &factory_digest,
        );

        // Assert
        assert_eq!(
            classified,
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::InvocationConstructionFailed,
                maybe_partial_device_effect: None,
            }
        );
    }

    #[test]
    fn successful_exit_without_closed_result_never_claims_device_effect() {
        // Arrange
        let operation = Phase36AllowedOperation::ExactPackageFlash;

        // Act
        let classified = classify_missing_effect_result(operation);

        // Assert
        assert_eq!(
            classified,
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::InvocationConstructionFailed,
                maybe_partial_device_effect: None,
            }
        );
    }
}
