use super::*;

#[test]
fn completed_results_authorize_recovery_only_for_exact_package_flash() {
    // Arrange
    let package_digest = "a".repeat(64);
    let factory_digest = "b".repeat(64);
    let flash = effect_result_for(
        Phase36AllowedOperation::ExactPackageFlash,
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &factory_digest,
    );
    let cleanup = effect_result_for(
        Phase36AllowedOperation::Cleanup,
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &factory_digest,
    );

    // Act
    let flash_result = classify_effect_result(
        flash,
        Phase36AllowedOperation::ExactPackageFlash,
        true,
        &package_digest,
        &factory_digest,
    );
    let cleanup_result = classify_effect_result(
        cleanup,
        Phase36AllowedOperation::Cleanup,
        true,
        &package_digest,
        &factory_digest,
    );

    // Assert
    assert_eq!(
        flash_result,
        Phase36OperationResult::Completed {
            maybe_completed_device_effect: Some(
                Phase36RecoveryIdentity::new(package_digest.clone(), factory_digest.clone(),)
                    .expect("fixture recovery identity"),
            ),
        }
    );
    assert_eq!(
        cleanup_result,
        Phase36OperationResult::Completed {
            maybe_completed_device_effect: None,
        }
    );
}

#[test]
fn failed_no_effect_results_preserve_each_operation_specific_failure() {
    // Arrange
    let package_digest = "a".repeat(64);
    let factory_digest = "b".repeat(64);
    let cases = [
        (
            Phase36AllowedOperation::ExactPackageAdmission,
            Phase36BrokerFailure::AuthenticationFailed,
        ),
        (
            Phase36AllowedOperation::Board205DetectorProbe,
            Phase36BrokerFailure::DetectorFailed,
        ),
        (
            Phase36AllowedOperation::ExactPackageFlash,
            Phase36BrokerFailure::ParserFailed,
        ),
        (
            Phase36AllowedOperation::PassiveSerialObservation,
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlySystemInfo,
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlyWebSocket,
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlyRetainedFacts,
            Phase36BrokerFailure::CaptureFailed,
        ),
        (
            Phase36AllowedOperation::TypedRecovery,
            Phase36BrokerFailure::RecoveryFailed,
        ),
        (
            Phase36AllowedOperation::Cleanup,
            Phase36BrokerFailure::CleanupFailed,
        ),
    ];

    // Act / Assert
    for (operation, failure) in cases {
        let result = effect_result_for(
            operation,
            Phase36EffectStatus::FailedNoDeviceEffect,
            Some(failure),
            &package_digest,
            &factory_digest,
        );
        assert_eq!(
            classify_effect_result(result, operation, false, &package_digest, &factory_digest,),
            Phase36OperationResult::Failed {
                failure,
                maybe_partial_device_effect: None,
            }
        );
    }
}

#[test]
fn both_partial_flash_statuses_carry_exact_recovery_authority() {
    // Arrange
    let package_digest = "a".repeat(64);
    let factory_digest = "b".repeat(64);
    let statuses = [
        Phase36EffectStatus::FailedConfirmedPartialDeviceEffect,
        Phase36EffectStatus::FailedAfterCompletedDeviceEffect,
    ];

    // Act
    let results = statuses.map(|status| {
        classify_effect_result(
            effect_result(
                status,
                Some(Phase36BrokerFailure::FlashFailed),
                &package_digest,
                &factory_digest,
            ),
            Phase36AllowedOperation::ExactPackageFlash,
            false,
            &package_digest,
            &factory_digest,
        )
    });

    // Assert
    for result in results {
        assert_eq!(
            result,
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::FlashFailed,
                maybe_partial_device_effect: Some(
                    Phase36RecoveryIdentity::new(package_digest.clone(), factory_digest.clone(),)
                        .expect("fixture recovery identity"),
                ),
            }
        );
    }
}

#[test]
fn schema_operation_and_identity_mismatches_fail_closed() {
    // Arrange
    let package_digest = "a".repeat(64);
    let factory_digest = "b".repeat(64);
    let mut wrong_schema = effect_result(
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &factory_digest,
    );
    wrong_schema.schema_version = "phase36-effect-result-v2".to_owned();
    let wrong_operation = effect_result_for(
        Phase36AllowedOperation::Cleanup,
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &factory_digest,
    );
    let wrong_package = effect_result(
        Phase36EffectStatus::Completed,
        None,
        &"c".repeat(64),
        &factory_digest,
    );
    let wrong_factory = effect_result(
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &"d".repeat(64),
    );
    let invalid_package = effect_result(
        Phase36EffectStatus::Completed,
        None,
        &"g".repeat(64),
        &factory_digest,
    );
    let invalid_factory = effect_result(
        Phase36EffectStatus::Completed,
        None,
        &package_digest,
        &"g".repeat(64),
    );
    let invalid_digest = "g".repeat(64);
    let cases = [
        (
            wrong_schema,
            package_digest.as_str(),
            factory_digest.as_str(),
        ),
        (
            wrong_operation,
            package_digest.as_str(),
            factory_digest.as_str(),
        ),
        (
            wrong_package,
            package_digest.as_str(),
            factory_digest.as_str(),
        ),
        (
            wrong_factory,
            package_digest.as_str(),
            factory_digest.as_str(),
        ),
        (
            invalid_package,
            invalid_digest.as_str(),
            factory_digest.as_str(),
        ),
        (
            invalid_factory,
            package_digest.as_str(),
            invalid_digest.as_str(),
        ),
    ];

    // Act / Assert
    for (result, expected_package, expected_factory) in cases {
        assert_eq!(
            classify_effect_result(
                result,
                Phase36AllowedOperation::ExactPackageFlash,
                true,
                expected_package,
                expected_factory,
            ),
            Phase36OperationResult::Failed {
                failure: Phase36BrokerFailure::InvocationConstructionFailed,
                maybe_partial_device_effect: None,
            }
        );
    }
}

#[test]
fn contradictory_status_failure_and_process_combinations_fail_closed() {
    // Arrange
    let package_digest = "a".repeat(64);
    let factory_digest = "b".repeat(64);
    let cases = [
        (
            effect_result(
                Phase36EffectStatus::Completed,
                Some(Phase36BrokerFailure::FlashFailed),
                &package_digest,
                &factory_digest,
            ),
            true,
        ),
        (
            effect_result(
                Phase36EffectStatus::Completed,
                None,
                &package_digest,
                &factory_digest,
            ),
            false,
        ),
        (
            effect_result(
                Phase36EffectStatus::FailedNoDeviceEffect,
                Some(Phase36BrokerFailure::CleanupFailed),
                &package_digest,
                &factory_digest,
            ),
            false,
        ),
        (
            effect_result(
                Phase36EffectStatus::FailedNoDeviceEffect,
                Some(Phase36BrokerFailure::FlashFailed),
                &package_digest,
                &factory_digest,
            ),
            true,
        ),
        (
            effect_result_for(
                Phase36AllowedOperation::Cleanup,
                Phase36EffectStatus::FailedConfirmedPartialDeviceEffect,
                Some(Phase36BrokerFailure::FlashFailed),
                &package_digest,
                &factory_digest,
            ),
            false,
        ),
    ];

    // Act / Assert
    for (result, process_succeeded) in cases {
        let operation = result.operation;
        assert_eq!(
            classify_effect_result(
                result,
                operation,
                process_succeeded,
                &package_digest,
                &factory_digest,
            ),
            Phase36OperationResult::Failed {
                failure: match classify_missing_effect_result(operation) {
                    Phase36OperationResult::Failed { failure, .. } => failure,
                    Phase36OperationResult::Completed { .. } => {
                        panic!("missing effect result must fail closed")
                    }
                },
                maybe_partial_device_effect: None,
            }
        );
    }
}
