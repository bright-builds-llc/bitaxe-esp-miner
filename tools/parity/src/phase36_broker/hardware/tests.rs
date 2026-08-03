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
fn detector_port_parser_accepts_the_canonical_flash_output() {
    // Arrange
    let output = b"espflash_version: 4.5.0\nport: /dev/cu.usbmodem1101\nusb_session: ready\n";

    // Act
    let port = parse_detector_port(output);

    // Assert
    assert_eq!(port.as_deref(), Some("/dev/cu.usbmodem1101"));
}

#[test]
fn detector_port_parser_rejects_noncanonical_and_ambiguous_output() {
    // Arrange
    let invalid_outputs: [&[u8]; 7] = [
        b"port=/dev/cu.usbmodem1101\nusb_session: ready\n",
        b"port: /dev/one\nport: /dev/two\nusb_session: ready\n",
        b"port: /dev/cu.usbmodem1101\n",
        b"port: \nusb_session: ready\n",
        b"port: relative-device\nusb_session: ready\n",
        b"port: COM\nusb_session: ready\n",
        b"port: /dev/cu.usbmodem1101\nusb_session: ready\n\xff",
    ];

    // Act / Assert
    for output in invalid_outputs {
        assert_eq!(parse_detector_port(output), None);
    }
}

#[test]
fn detector_process_resolves_justfile_from_the_bazel_workspace() {
    // Arrange
    let workspace = OsStr::new("/qualified/workspace");

    // Act
    let command = detector_command(Some(workspace));

    // Assert
    assert_eq!(command.get_program(), OsStr::new("just"));
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [OsStr::new("detect-ultra205")]
    );
    assert_eq!(command.get_current_dir(), Some(Path::new(workspace)));
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
