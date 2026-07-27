use super::*;

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

#[test]
fn operation_names_and_missing_result_failures_cover_the_closed_operation_set() {
    // Arrange
    let cases = [
        (
            Phase36AllowedOperation::ExactPackageAdmission,
            "exact-package-admission",
            Phase36BrokerFailure::AdmissionFailed,
        ),
        (
            Phase36AllowedOperation::Board205DetectorProbe,
            "board-205-detector-probe",
            Phase36BrokerFailure::DetectorFailed,
        ),
        (
            Phase36AllowedOperation::ExactPackageFlash,
            "exact-package-flash",
            Phase36BrokerFailure::InvocationConstructionFailed,
        ),
        (
            Phase36AllowedOperation::PassiveSerialObservation,
            "passive-serial-observation",
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlySystemInfo,
            "read-only-system-info",
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlyWebSocket,
            "read-only-websocket",
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlyRetainedFacts,
            "read-only-retained-facts",
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::TypedRecovery,
            "typed-recovery",
            Phase36BrokerFailure::RecoveryFailed,
        ),
        (
            Phase36AllowedOperation::Cleanup,
            "cleanup",
            Phase36BrokerFailure::CleanupFailed,
        ),
    ];

    // Act
    let actual = cases.map(|(operation, _, _)| {
        (
            operation_name(operation),
            classify_missing_effect_result(operation),
        )
    });

    // Assert
    for ((_, expected_name, expected_failure), (actual_name, actual_result)) in
        cases.into_iter().zip(actual)
    {
        assert_eq!(actual_name, expected_name);
        assert_eq!(
            actual_result,
            Phase36OperationResult::Failed {
                failure: expected_failure,
                maybe_partial_device_effect: None,
            }
        );
    }
}
