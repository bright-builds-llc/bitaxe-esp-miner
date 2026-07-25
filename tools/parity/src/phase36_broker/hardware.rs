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
            secondary_failure: secondary_failure.or(interval.secondary_failure()),
            recovery_disposition: interval.recovery_disposition(),
        },
    };
    boundary.seal(disposition)?;
    Ok(disposition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase36_broker::Phase36AllowedOperation;

    fn recovery_identity(seed: char) -> Phase36RecoveryIdentity {
        let digest = std::iter::repeat_n(seed, 64).collect::<String>();
        Phase36RecoveryIdentity::new(digest.clone(), digest)
            .expect("fixture recovery identity should be valid")
    }

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
        secondary_fail_operation: Option<Phase36AllowedOperation>,
        fail_category: Option<Phase36BrokerFailure>,
        confirmed_partial_flash: bool,
        mismatch_recovery_identity: bool,
        recovery_calls: usize,
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
            if operation == Phase36AllowedOperation::TypedRecovery {
                self.recovery_calls += 1;
            }
            if self.fail_operation == Some(operation) {
                return Err(Phase36HardwareTransactionError::EffectFailed(
                    self.fail_category
                        .unwrap_or_else(|| failure_for_operation(operation)),
                ));
            }
            if self.secondary_fail_operation == Some(operation) {
                return Err(Phase36HardwareTransactionError::EffectFailed(
                    failure_for_operation(operation),
                ));
            }
            Ok(())
        }

        fn recovery_identity(
            &self,
        ) -> Result<Phase36RecoveryIdentity, Phase36HardwareTransactionError> {
            Ok(if self.mismatch_recovery_identity {
                recovery_identity('b')
            } else {
                recovery_identity('a')
            })
        }

        fn execute_classified(
            &mut self,
            operation: Phase36AllowedOperation,
        ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
            let result = self.execute(operation);
            match result {
                Ok(()) => Ok(Phase36OperationResult::Completed {
                    maybe_completed_device_effect: if operation
                        == Phase36AllowedOperation::ExactPackageFlash
                    {
                        Some(recovery_identity('a'))
                    } else {
                        None
                    },
                }),
                Err(Phase36HardwareTransactionError::EffectFailed(failure)) => {
                    Ok(Phase36OperationResult::Failed {
                        failure,
                        maybe_partial_device_effect: if self.confirmed_partial_flash
                            && operation == Phase36AllowedOperation::ExactPackageFlash
                        {
                            Some(recovery_identity('a'))
                        } else {
                            None
                        },
                    })
                }
                Err(error) => Err(error),
            }
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
    fn completed_flash_authorizes_one_recovery_for_each_later_capture_failure() {
        // Arrange
        let cases = [
            Phase36AllowedOperation::PassiveSerialObservation,
            Phase36AllowedOperation::ReadOnlySystemInfo,
            Phase36AllowedOperation::ReadOnlyWebSocket,
            Phase36AllowedOperation::ReadOnlyRetainedFacts,
        ];

        // Act and Assert
        for operation in cases {
            let mut boundary = FakeTransactionBoundary {
                fail_operation: Some(operation),
                ..FakeTransactionBoundary::default()
            };
            let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

            assert_eq!(
                result,
                Ok(Phase36HardwareDisposition::SealedNonPromotion {
                    first_failure: Phase36BrokerFailure::CaptureFailed,
                    secondary_failure: None,
                    recovery_disposition: Phase36RecoveryDisposition::AttemptedSucceeded,
                })
            );
            assert_eq!(boundary.detector_calls, 1);
            assert_eq!(boundary.recovery_calls, 1);
            assert_eq!(
                boundary
                    .events
                    .iter()
                    .filter(|event| **event == "cleanup")
                    .count(),
                1
            );
        }
    }

    #[test]
    fn transaction_never_recovers_after_pre_effect_failure() {
        // Arrange
        let cases = [
            (
                Phase36AllowedOperation::ExactPackageAdmission,
                Phase36BrokerFailure::AdmissionFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::ExactPackageAdmission,
                Phase36BrokerFailure::CapabilityFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::ExactPackageAdmission,
                Phase36BrokerFailure::AuthenticationFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::Board205DetectorProbe,
                Phase36BrokerFailure::DetectorFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::ExactPackageFlash,
                Phase36BrokerFailure::InvocationConstructionFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::ExactPackageFlash,
                Phase36BrokerFailure::ParserFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::ExactPackageFlash,
                Phase36BrokerFailure::FlashFailed,
                Phase36RecoveryDisposition::NotAuthorized,
            ),
            (
                Phase36AllowedOperation::Cleanup,
                Phase36BrokerFailure::CleanupFailed,
                Phase36RecoveryDisposition::NotRequired,
            ),
        ];

        // Act and Assert
        for (operation, failure, recovery_disposition) in cases {
            let mut boundary = FakeTransactionBoundary {
                fail_operation: Some(operation),
                fail_category: Some(failure),
                ..FakeTransactionBoundary::default()
            };
            let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

            assert_eq!(
                result,
                Ok(Phase36HardwareDisposition::SealedNonPromotion {
                    first_failure: failure,
                    secondary_failure: None,
                    recovery_disposition,
                })
            );
            assert_eq!(boundary.recovery_calls, 0);
            assert!(!boundary.events.contains(&"typed_recovery"));
            assert_eq!(
                boundary
                    .events
                    .iter()
                    .filter(|event| **event == "cleanup")
                    .count(),
                1
            );
        }
    }

    #[test]
    fn confirmed_partial_flash_recovers_once_with_same_identity() {
        // Arrange
        let mut boundary = FakeTransactionBoundary {
            fail_operation: Some(Phase36AllowedOperation::ExactPackageFlash),
            confirmed_partial_flash: true,
            ..FakeTransactionBoundary::default()
        };

        // Act
        let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareDisposition::SealedNonPromotion {
                first_failure: Phase36BrokerFailure::FlashFailed,
                secondary_failure: None,
                recovery_disposition: Phase36RecoveryDisposition::AttemptedSucceeded,
            })
        );
        assert_eq!(boundary.recovery_calls, 1);
    }

    #[test]
    fn mismatched_recovery_identity_fails_closed_before_recovery_invocation() {
        // Arrange
        let mut boundary = FakeTransactionBoundary {
            fail_operation: Some(Phase36AllowedOperation::ExactPackageFlash),
            confirmed_partial_flash: true,
            mismatch_recovery_identity: true,
            ..FakeTransactionBoundary::default()
        };

        // Act
        let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareDisposition::SealedNonPromotion {
                first_failure: Phase36BrokerFailure::FlashFailed,
                secondary_failure: None,
                recovery_disposition: Phase36RecoveryDisposition::NotAuthorized,
            })
        );
        assert_eq!(boundary.recovery_calls, 0);
        assert!(!boundary.events.contains(&"typed_recovery"));
        assert_eq!(
            boundary
                .events
                .iter()
                .filter(|event| **event == "cleanup")
                .count(),
            1
        );
    }

    #[test]
    fn recovery_failure_remains_secondary_to_confirmed_partial_flash_failure() {
        // Arrange
        let mut boundary = FakeTransactionBoundary {
            fail_operation: Some(Phase36AllowedOperation::ExactPackageFlash),
            secondary_fail_operation: Some(Phase36AllowedOperation::TypedRecovery),
            confirmed_partial_flash: true,
            ..FakeTransactionBoundary::default()
        };

        // Act
        let result = run_phase36_hardware_transaction_with(&mut boundary, 1_000);

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareDisposition::SealedNonPromotion {
                first_failure: Phase36BrokerFailure::FlashFailed,
                secondary_failure: Some(Phase36BrokerFailure::RecoveryFailed),
                recovery_disposition: Phase36RecoveryDisposition::AttemptedFailed,
            })
        );
        assert_eq!(boundary.recovery_calls, 1);
        assert_eq!(
            boundary
                .events
                .iter()
                .filter(|event| **event == "cleanup")
                .count(),
            1
        );
    }
}
