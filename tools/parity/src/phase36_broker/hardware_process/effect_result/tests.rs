use super::*;

fn effect_result(
    status: Phase36EffectStatus,
    failure: Option<Phase36BrokerFailure>,
    package_digest: &str,
    factory_digest: &str,
) -> Phase36EffectResult {
    effect_result_for(
        Phase36AllowedOperation::ExactPackageFlash,
        status,
        failure,
        package_digest,
        factory_digest,
    )
}

fn effect_result_for(
    operation: Phase36AllowedOperation,
    status: Phase36EffectStatus,
    failure: Option<Phase36BrokerFailure>,
    package_digest: &str,
    factory_digest: &str,
) -> Phase36EffectResult {
    Phase36EffectResult {
        schema_version: "phase36-effect-result-v1".to_owned(),
        operation,
        status,
        failure,
        package_identity_digest: package_digest.to_owned(),
        factory_image_digest: factory_digest.to_owned(),
    }
}

mod classification;
mod filesystem;
mod mapping;
