use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use thiserror::Error;

use super::{
    Phase36AllowedOperation, Phase36BrokerFailure, Phase36LedgerError, Phase36LedgerRecord,
    Phase36LedgerState, Phase36LedgerTransition,
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
        if !output.status.success() {
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
    Command::new(DETECTOR_PROGRAM)
        .arg(DETECTOR_ARGUMENT)
        .stdin(Stdio::null())
        .output()
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
    #[error("phase36_hardware_seal_failed")]
    Seal,
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
) -> Result<Option<Phase36BrokerFailure>, Phase36HardwareTransactionError> {
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
    let maybe_failure = match boundary.execute(operation) {
        Ok(()) => {
            append_transition(
                boundary,
                state,
                operation,
                Phase36LedgerTransition::Completed,
                monotonic_millis,
            )?;
            None
        }
        Err(Phase36HardwareTransactionError::EffectFailed(failure)) => {
            let normalized_failure = if failure.valid_for(operation) {
                failure
            } else {
                failure_for_operation(operation)
            };
            append_transition(
                boundary,
                state,
                operation,
                Phase36LedgerTransition::Failed {
                    category: normalized_failure,
                },
                monotonic_millis,
            )?;
            Some(normalized_failure)
        }
        Err(error) => return Err(error),
    };
    append_transition(
        boundary,
        state,
        operation,
        Phase36LedgerTransition::Closed,
        monotonic_millis,
    )?;
    Ok(maybe_failure)
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

    for operation in Phase36AllowedOperation::SUCCESS_ORDER {
        let maybe_failure =
            execute_operation(boundary, &mut state, operation, &mut monotonic_millis)?;
        if let Some(failure) = maybe_failure {
            first_failure = Some(failure);
            break;
        }
    }

    if first_failure.is_some() {
        if let Some(failure) = execute_operation(
            boundary,
            &mut state,
            Phase36AllowedOperation::TypedRecovery,
            &mut monotonic_millis,
        )? {
            secondary_failure = Some(failure);
        }
        if let Some(failure) = execute_operation(
            boundary,
            &mut state,
            Phase36AllowedOperation::Cleanup,
            &mut monotonic_millis,
        )? {
            if secondary_failure.is_none() {
                secondary_failure = Some(failure);
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
            secondary_failure: secondary_failure.or(interval.secondary_failure()),
        },
    };
    boundary.seal(disposition)?;
    Ok(disposition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase36_broker::Phase36AllowedOperation;

    #[derive(Default)]
    struct FakeHardwareBoundary {
        detector_calls: usize,
        credential_calls: usize,
        detector_fails: bool,
        credential_fails: bool,
    }

    impl Phase36HardwareBoundary for FakeHardwareBoundary {
        fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError> {
            self.detector_calls += 1;
            if self.detector_fails {
                return Err(Phase36HardwareGateError::DetectorFailed);
            }
            Ok(())
        }

        fn validate_wifi_credentials(
            &mut self,
            _wifi_credentials: &Path,
        ) -> Result<(), Phase36HardwareGateError> {
            self.credential_calls += 1;
            if self.credential_fails {
                return Err(Phase36HardwareGateError::WifiCredentialsInvalid);
            }
            Ok(())
        }
    }

    #[test]
    fn detector_failure_stops_before_credential_access() {
        // Arrange
        let mut boundary = FakeHardwareBoundary {
            detector_fails: true,
            ..FakeHardwareBoundary::default()
        };

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 205, Path::new("opaque"));

        // Assert
        assert_eq!(result, Err(Phase36HardwareGateError::DetectorFailed));
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(boundary.credential_calls, 0);
    }

    #[test]
    fn detector_success_precedes_single_credential_validation() {
        // Arrange
        let mut boundary = FakeHardwareBoundary::default();

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 205, Path::new("opaque"));

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareGateStatus::DetectorAdmittedCredentialValidated)
        );
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(boundary.credential_calls, 1);
    }

    #[test]
    fn wrong_board_stops_before_detector_or_credential_access() {
        // Arrange
        let mut boundary = FakeHardwareBoundary::default();

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 601, Path::new("opaque"));

        // Assert
        assert_eq!(result, Err(Phase36HardwareGateError::WrongBoard));
        assert_eq!(boundary.detector_calls, 0);
        assert_eq!(boundary.credential_calls, 0);
    }

    #[derive(Default)]
    struct FakeTransactionBoundary {
        events: Vec<&'static str>,
        detector_calls: usize,
        fail_operation: Option<Phase36AllowedOperation>,
    }

    impl Phase36HardwareTransactionBoundary for FakeTransactionBoundary {
        fn prepare_private_attempt(&mut self) -> Result<(), Phase36HardwareTransactionError> {
            self.events.push("prepare_private_attempt");
            Ok(())
        }

        fn record(
            &mut self,
            _record: &Phase36LedgerRecord,
        ) -> Result<(), Phase36HardwareTransactionError> {
            Ok(())
        }

        fn execute(
            &mut self,
            operation: Phase36AllowedOperation,
        ) -> Result<(), Phase36HardwareTransactionError> {
            if operation == Phase36AllowedOperation::Board205DetectorProbe {
                self.detector_calls += 1;
            }
            self.events.push(operation.event_name());
            if self.fail_operation == Some(operation) {
                return Err(Phase36HardwareTransactionError::EffectFailed(
                    failure_for_operation(operation),
                ));
            }
            Ok(())
        }

        fn seal(
            &mut self,
            disposition: Phase36HardwareDisposition,
        ) -> Result<(), Phase36HardwareTransactionError> {
            self.events.push(match disposition {
                Phase36HardwareDisposition::SealedEligible => "sealed_eligible",
                Phase36HardwareDisposition::SealedNonPromotion { .. } => "sealed_non_promotion",
            });
            Ok(())
        }
    }

    impl Phase36AllowedOperation {
        fn event_name(self) -> &'static str {
            match self {
                Self::ExactPackageAdmission => "exact_package_admission",
                Self::Board205DetectorProbe => "board_205_detector_probe",
                Self::ExactPackageFlash => "exact_package_flash",
                Self::PassiveSerialObservation => "passive_serial_observation",
                Self::ReadOnlySystemInfo => "read_only_system_info",
                Self::ReadOnlyWebSocket => "read_only_web_socket",
                Self::ReadOnlyRetainedFacts => "read_only_retained_facts",
                Self::TypedRecovery => "typed_recovery",
                Self::Cleanup => "cleanup",
            }
        }
    }

    #[test]
    fn transaction_prepares_private_ledger_before_exactly_one_detector_and_seals_eligible() {
        // Arrange
        let mut boundary = FakeTransactionBoundary::default();

        // Act
        let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

        // Assert
        assert_eq!(result, Ok(Phase36HardwareDisposition::SealedEligible));
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(
            boundary.events,
            [
                "prepare_private_attempt",
                "exact_package_admission",
                "board_205_detector_probe",
                "exact_package_flash",
                "passive_serial_observation",
                "read_only_system_info",
                "read_only_web_socket",
                "read_only_retained_facts",
                "cleanup",
                "sealed_eligible",
            ]
        );
    }

    #[test]
    fn transaction_preserves_earliest_failure_then_recovers_cleans_and_seals_non_promotion() {
        // Arrange
        let mut boundary = FakeTransactionBoundary {
            fail_operation: Some(Phase36AllowedOperation::ReadOnlyWebSocket),
            ..FakeTransactionBoundary::default()
        };

        // Act
        let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareDisposition::SealedNonPromotion {
                first_failure: Phase36BrokerFailure::CaptureFailed,
                secondary_failure: None,
            })
        );
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(
            boundary.events,
            [
                "prepare_private_attempt",
                "exact_package_admission",
                "board_205_detector_probe",
                "exact_package_flash",
                "passive_serial_observation",
                "read_only_system_info",
                "read_only_web_socket",
                "typed_recovery",
                "cleanup",
                "sealed_non_promotion",
            ]
        );
    }
}
