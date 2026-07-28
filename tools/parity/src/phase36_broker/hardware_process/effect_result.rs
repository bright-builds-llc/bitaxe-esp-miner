use super::*;

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
pub(super) struct Phase36EffectResult {
    schema_version: String,
    operation: Phase36AllowedOperation,
    status: Phase36EffectStatus,
    failure: Option<Phase36BrokerFailure>,
    package_identity_digest: String,
    factory_image_digest: String,
}

pub(super) fn operation_name(operation: Phase36AllowedOperation) -> &'static str {
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

pub(super) fn maybe_read_effect_result(path: &Utf8Path) -> Option<Phase36EffectResult> {
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

pub(super) fn classify_missing_effect_result(
    operation: Phase36AllowedOperation,
) -> Phase36OperationResult {
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

pub(super) fn classify_effect_result(
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

#[cfg(test)]
mod tests;
