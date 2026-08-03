use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use thiserror::Error;

use super::contract::Phase36RecoveryIdentity;
use super::{
    Phase36AllowedOperation, Phase36BrokerFailure, Phase36LedgerError, Phase36LedgerRecord,
    Phase36LedgerState, Phase36LedgerTransition, Phase36RecoveryDisposition,
};

const DETECTOR_PROGRAM: &str = "just";
const DETECTOR_ARGUMENT: &str = "detect-ultra205";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase36HardwareGateStatus {
    DetectorAdmittedCredentialValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36HardwareGateError {
    #[error("phase36_broker_wrong_board")]
    WrongBoard,
    #[error("phase36_broker_detector_failed")]
    DetectorFailed,
    #[error("phase36_broker_wifi_credentials_invalid")]
    WifiCredentialsInvalid,
}

trait Phase36HardwareBoundary {
    fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError>;
    fn validate_wifi_credentials(
        &mut self,
        wifi_credentials: &Path,
    ) -> Result<(), Phase36HardwareGateError>;
}

struct ProcessHardwareBoundary;

impl Phase36HardwareBoundary for ProcessHardwareBoundary {
    fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError> {
        let output =
            run_detector_process().map_err(|_| Phase36HardwareGateError::DetectorFailed)?;
        if !output.status.success() || parse_detector_port(&output.stdout).is_none() {
            return Err(Phase36HardwareGateError::DetectorFailed);
        }
        Ok(())
    }

    fn validate_wifi_credentials(
        &mut self,
        wifi_credentials: &Path,
    ) -> Result<(), Phase36HardwareGateError> {
        let metadata = fs::symlink_metadata(wifi_credentials)
            .map_err(|_| Phase36HardwareGateError::WifiCredentialsInvalid)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.permissions().mode() & 0o777 != 0o600
        {
            return Err(Phase36HardwareGateError::WifiCredentialsInvalid);
        }
        Ok(())
    }
}

pub(super) fn run_detector_process() -> std::io::Result<Output> {
    detector_command(env::var_os("BUILD_WORKSPACE_DIRECTORY").as_deref())
        .stdin(Stdio::null())
        .output()
}

fn detector_command(maybe_workspace: Option<&OsStr>) -> Command {
    let mut command = Command::new(DETECTOR_PROGRAM);
    command.arg(DETECTOR_ARGUMENT);
    if let Some(workspace) = maybe_workspace.filter(|workspace| !workspace.is_empty()) {
        command.current_dir(workspace);
    }
    command
}

pub(super) fn parse_detector_port(stdout: &[u8]) -> Option<String> {
    let stdout = std::str::from_utf8(stdout).ok()?;
    if stdout
        .lines()
        .filter(|line| *line == "usb_session: ready")
        .count()
        != 1
    {
        return None;
    }
    let ports = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("port: "))
        .filter(|port| valid_detector_port(port))
        .collect::<Vec<_>>();
    let [port] = ports.as_slice() else {
        return None;
    };
    Some((*port).to_owned())
}

fn valid_detector_port(port: &str) -> bool {
    if port.is_empty() || port.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return false;
    }
    port.starts_with('/')
        || port.strip_prefix("COM").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
}

pub fn run_phase36_hardware_pre_capture_gate(
    board: u16,
    wifi_credentials: &Path,
) -> Result<Phase36HardwareGateStatus, Phase36HardwareGateError> {
    let mut boundary = ProcessHardwareBoundary;
    run_phase36_hardware_pre_capture_gate_with(&mut boundary, board, wifi_credentials)
}

fn run_phase36_hardware_pre_capture_gate_with(
    boundary: &mut impl Phase36HardwareBoundary,
    board: u16,
    wifi_credentials: &Path,
) -> Result<Phase36HardwareGateStatus, Phase36HardwareGateError> {
    if board != 205 {
        return Err(Phase36HardwareGateError::WrongBoard);
    }

    boundary.run_detector()?;
    boundary.validate_wifi_credentials(wifi_credentials)?;

    Ok(Phase36HardwareGateStatus::DetectorAdmittedCredentialValidated)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase36HardwareDisposition {
    SealedEligible,
    SealedNonPromotion {
        first_failure: Phase36BrokerFailure,
        secondary_failure: Option<Phase36BrokerFailure>,
        recovery_disposition: Phase36RecoveryDisposition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36HardwareTransactionError {
    #[error("phase36_hardware_effect_failed")]
    EffectFailed(Phase36BrokerFailure),
    #[error("phase36_hardware_ledger_failed")]
    Ledger(#[from] Phase36LedgerError),
    #[error("phase36_hardware_private_attempt_failed")]
    PrivateAttempt,
    #[error("phase36_hardware_recovery_authority_invalid")]
    InvalidRecoveryAuthority,
    #[error("phase36_hardware_seal_failed")]
    Seal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Phase36OperationResult {
    Completed {
        maybe_completed_device_effect: Option<Phase36RecoveryIdentity>,
    },
    Failed {
        failure: Phase36BrokerFailure,
        maybe_partial_device_effect: Option<Phase36RecoveryIdentity>,
    },
}

pub(super) trait Phase36HardwareTransactionBoundary {
    fn prepare_private_attempt(&mut self) -> Result<(), Phase36HardwareTransactionError>;
    fn record(
        &mut self,
        record: &Phase36LedgerRecord,
    ) -> Result<(), Phase36HardwareTransactionError>;
    fn execute(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<(), Phase36HardwareTransactionError>;
    fn recovery_identity(&self)
        -> Result<Phase36RecoveryIdentity, Phase36HardwareTransactionError>;
    fn execute_classified(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
        let result = self.execute(operation);
        match result {
            Ok(()) => {
                let maybe_completed_device_effect =
                    if operation == Phase36AllowedOperation::ExactPackageFlash {
                        Some(self.recovery_identity()?)
                    } else {
                        None
                    };
                Ok(Phase36OperationResult::Completed {
                    maybe_completed_device_effect,
                })
            }
            Err(Phase36HardwareTransactionError::EffectFailed(failure)) => {
                let failure = if failure.valid_for(operation) {
                    failure
                } else {
                    failure_for_operation(operation)
                };
                Ok(Phase36OperationResult::Failed {
                    failure,
                    maybe_partial_device_effect: None,
                })
            }
            Err(error) => Err(error),
        }
    }
    fn seal(
        &mut self,
        disposition: Phase36HardwareDisposition,
    ) -> Result<(), Phase36HardwareTransactionError>;
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

fn append_transition(
    boundary: &mut impl Phase36HardwareTransactionBoundary,
    state: &mut Phase36LedgerState,
    operation: Phase36AllowedOperation,
    transition: Phase36LedgerTransition,
    monotonic_millis: &mut u64,
) -> Result<(), Phase36HardwareTransactionError> {
    *monotonic_millis =
        monotonic_millis
            .checked_add(1)
            .ok_or(Phase36HardwareTransactionError::Ledger(
                Phase36LedgerError::InvalidInterval,
            ))?;
    let record = Phase36LedgerRecord::next(state, operation, transition, *monotonic_millis)?;
    boundary.record(&record)?;
    state.apply(&record)?;
    Ok(())
}

fn execute_operation(
    boundary: &mut impl Phase36HardwareTransactionBoundary,
    state: &mut Phase36LedgerState,
    operation: Phase36AllowedOperation,
    monotonic_millis: &mut u64,
) -> Result<
    (
        Option<Phase36BrokerFailure>,
        Option<Phase36RecoveryIdentity>,
    ),
    Phase36HardwareTransactionError,
> {
    append_transition(
        boundary,
        state,
        operation,
        Phase36LedgerTransition::Authorized,
        monotonic_millis,
    )?;
    append_transition(
        boundary,
        state,
        operation,
        Phase36LedgerTransition::Invoked,
        monotonic_millis,
    )?;
    let (maybe_failure, maybe_device_effect) = match boundary.execute_classified(operation)? {
        Phase36OperationResult::Completed {
            maybe_completed_device_effect,
        } => {
            append_transition(
                boundary,
                state,
                operation,
                Phase36LedgerTransition::Completed,
                monotonic_millis,
            )?;
            (None, maybe_completed_device_effect)
        }
        Phase36OperationResult::Failed {
            failure,
            maybe_partial_device_effect,
        } => {
            if maybe_partial_device_effect.is_some() {
                append_transition(
                    boundary,
                    state,
                    operation,
                    Phase36LedgerTransition::ConfirmedPartialDeviceEffect,
                    monotonic_millis,
                )?;
            }
            append_transition(
                boundary,
                state,
                operation,
                Phase36LedgerTransition::Failed { category: failure },
                monotonic_millis,
            )?;
            (Some(failure), maybe_partial_device_effect)
        }
    };
    append_transition(
        boundary,
        state,
        operation,
        Phase36LedgerTransition::Closed,
        monotonic_millis,
    )?;
    Ok((maybe_failure, maybe_device_effect))
}

pub(super) fn run_phase36_hardware_transaction_with(
    boundary: &mut impl Phase36HardwareTransactionBoundary,
    interval_start_millis: u64,
) -> Result<Phase36HardwareDisposition, Phase36HardwareTransactionError> {
    boundary.prepare_private_attempt()?;
    let mut state = Phase36LedgerState::start(interval_start_millis)?;
    let mut monotonic_millis = interval_start_millis;
    let mut first_failure = None;
    let mut secondary_failure = None;
    let mut maybe_recovery_identity = None;
    let mut cleanup_attempted = false;

    for operation in Phase36AllowedOperation::SUCCESS_ORDER {
        cleanup_attempted |= operation == Phase36AllowedOperation::Cleanup;
        let (maybe_failure, maybe_device_effect) =
            execute_operation(boundary, &mut state, operation, &mut monotonic_millis)?;
        if maybe_device_effect.is_some() {
            maybe_recovery_identity = maybe_device_effect;
        }
        if let Some(failure) = maybe_failure {
            first_failure = Some(failure);
            break;
        }
    }

    if first_failure.is_some() {
        if state.recovery_required() {
            let recovery_matches = match boundary.recovery_identity() {
                Ok(current) => maybe_recovery_identity
                    .as_ref()
                    .is_some_and(|authorized| *authorized == current),
                Err(Phase36HardwareTransactionError::InvalidRecoveryAuthority) => false,
                Err(error) => return Err(error),
            };
            if recovery_matches {
                let (maybe_failure, _) = execute_operation(
                    boundary,
                    &mut state,
                    Phase36AllowedOperation::TypedRecovery,
                    &mut monotonic_millis,
                )?;
                if let Some(failure) = maybe_failure {
                    secondary_failure = Some(failure);
                }
            } else {
                state.reject_recovery_authority()?;
            }
        }
        if !cleanup_attempted {
            let (maybe_failure, _) = execute_operation(
                boundary,
                &mut state,
                Phase36AllowedOperation::Cleanup,
                &mut monotonic_millis,
            )?;
            if let Some(failure) = maybe_failure {
                if secondary_failure.is_none() {
                    secondary_failure = Some(failure);
                }
            }
        }
    }

    let interval = state.seal(monotonic_millis.checked_add(1).ok_or(
        Phase36HardwareTransactionError::Ledger(Phase36LedgerError::InvalidInterval),
    )?)?;
    let disposition = match first_failure {
        None => Phase36HardwareDisposition::SealedEligible,
        Some(failure) => Phase36HardwareDisposition::SealedNonPromotion {
            first_failure: failure,
            secondary_failure: secondary_failure.or(interval.maybe_secondary_failure()),
            recovery_disposition: interval.recovery_disposition(),
        },
    };
    boundary.seal(disposition)?;
    Ok(disposition)
}

#[cfg(test)]
mod tests;
