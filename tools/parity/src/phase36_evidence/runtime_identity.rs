//! Admission of independently observed runtime identity and exact package correlation.

use bitaxe_device_session::{
    PlatformCategory, PrivateBootB, RequestOutcome, SerialDelivery, SessionRequest, SessionState,
    TerminalCategory, PRIVATE_RESULT_SCHEMA, PUBLIC_PROJECTION_SCHEMA,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::contract::ComponentInsufficiency;
use crate::phase35_evidence::sha256_hex;

mod ledger;

const PACKAGE_SCHEMA: &str = "phase36-runtime-package-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedRuntimeIdentityAdmission {
    Validated {
        identity: Box<ValidatedObservedRuntimeIdentity>,
    },
    Insufficient {
        component_insufficiencies: Vec<ComponentInsufficiency>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedObservedRuntimeIdentity {
    pub observation_source: RuntimeIdentityObservationSource,
    pub same_physical_device: bool,
    pub boot_b_session_digest: String,
    pub boot_b_ordinal: u64,
    pub source_commit_digest: String,
    pub reference_commit_digest: String,
    pub application_elf_digest: String,
    pub exact_package: ExactPackageIdentityJoin,
    pub claim_fact_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeIdentityObservationSource {
    DeviceSessionReplay,
    TerminalResultProjection,
    ExactPackageFlashSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExactPackageIdentityJoin {
    pub manifest_digest: String,
    pub executable_image_digest: String,
    pub factory_image_digest: String,
    pub firmware_elf_digest: String,
    pub package_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RuntimeIdentityEvidenceError {
    #[error("runtime_identity_document_invalid")]
    DocumentInvalid,
    #[error("runtime_identity_replay_mismatch")]
    ReplayMismatch,
    #[error("runtime_identity_public_private_disagreement")]
    PublicPrivateDisagreement,
    #[error("runtime_identity_device_mismatch")]
    PhysicalDeviceMismatch,
    #[error("runtime_identity_build_mismatch")]
    BuildIdentityMismatch,
    #[error("runtime_identity_session_mismatch")]
    BootSessionMismatch,
    #[error("runtime_identity_package_mismatch")]
    ExactPackageMismatch,
    #[error("runtime_identity_ledger_incomplete")]
    MissingLedgerStep,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactPackageDocument {
    schema_version: String,
    source_commit: String,
    reference_commit: String,
    manifest_digest: String,
    executable_image_digest: String,
    factory_image_digest: String,
    firmware_elf_digest: String,
    package_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct HardwareObservationDocument {
    schema_version: String,
    board_category: String,
    target: String,
    asic: String,
    detector_candidate_count: u8,
    same_physical_device: bool,
    physical_identity_digest: String,
    boot_session: String,
    boot_ordinal: u64,
    reset_reason_category: String,
    trusted_origin: String,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PrivateResultDocument {
    schema_version: String,
    terminal_category: TerminalCategory,
    request_outcome: RequestOutcome,
    maybe_secondary_cleanup_failure: bool,
    boot_b: Option<PrivateBootB>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PublicProjectionDocument {
    schema_version: String,
    terminal_category: TerminalCategory,
    platform_category: PlatformCategory,
    board_category: String,
    same_physical_device: bool,
    stable_enumeration: bool,
    reenumerated: bool,
    reader_armed: bool,
    pre_restart_serial_delivery: bool,
    post_restart_serial_delivery: bool,
    serial_delivery: SerialDelivery,
    request_outcome: RequestOutcome,
    request_attempt_count: u8,
    service_loss_observed: bool,
    trusted_origin_preserved: bool,
    application_recovered: bool,
    build_identity_matches: bool,
    boot_session_changed: bool,
    boot_ordinal_advanced_by_one: bool,
    software_reset_observed: bool,
    postcondition_matches: bool,
    cleanup_complete: bool,
    usb_disappearance_count: u16,
    enumeration_change_count: u16,
    serial_byte_count: u64,
    http_observation_count: u16,
    duration_millis: u64,
}

pub fn validate_observed_runtime_identity_documents(
    exact_package_document: &str,
    maybe_request_document: Option<&str>,
    maybe_event_ledger_document: Option<&str>,
    maybe_private_result_document: Option<&str>,
    maybe_public_projection_document: Option<&str>,
) -> Result<ObservedRuntimeIdentityAdmission, RuntimeIdentityEvidenceError> {
    validate_observed_runtime_identity_documents_with_hardware(
        exact_package_document,
        maybe_request_document,
        maybe_event_ledger_document,
        maybe_private_result_document,
        maybe_public_projection_document,
        None,
    )
}

pub fn validate_observed_runtime_identity_documents_with_hardware(
    exact_package_document: &str,
    maybe_request_document: Option<&str>,
    maybe_event_ledger_document: Option<&str>,
    maybe_private_result_document: Option<&str>,
    maybe_public_projection_document: Option<&str>,
    maybe_hardware_observation_document: Option<&str>,
) -> Result<ObservedRuntimeIdentityAdmission, RuntimeIdentityEvidenceError> {
    let package = parse_json::<ExactPackageDocument>(exact_package_document)?;
    validate_package(&package)?;
    if let Some(hardware_document) = maybe_hardware_observation_document {
        return validate_hardware_observation(&package, hardware_document);
    }
    let (Some(request_document), Some(private_document), Some(public_document)) = (
        maybe_request_document,
        maybe_private_result_document,
        maybe_public_projection_document,
    ) else {
        return Ok(ObservedRuntimeIdentityAdmission::Insufficient {
            component_insufficiencies: vec![ComponentInsufficiency::RuntimeIdentityObservation],
        });
    };
    let request = parse_json::<SessionRequest>(request_document)?;
    validate_request_package_join(&request, &package)?;
    let private_result = parse_json::<PrivateResultDocument>(private_document)?;
    let public_projection = parse_json::<PublicProjectionDocument>(public_document)?;

    let observation_source = if let Some(event_ledger) = maybe_event_ledger_document {
        replay_event_ledger(&request, event_ledger, &private_result, &public_projection)?;
        RuntimeIdentityObservationSource::DeviceSessionReplay
    } else {
        validate_terminal_pair(&request, &private_result, &public_projection)?;
        RuntimeIdentityObservationSource::TerminalResultProjection
    };
    let boot_b = validated_boot_b(&request, &private_result, &public_projection, &package)?;
    let exact_package = ExactPackageIdentityJoin {
        manifest_digest: package.manifest_digest,
        executable_image_digest: package.executable_image_digest,
        factory_image_digest: package.factory_image_digest,
        firmware_elf_digest: package.firmware_elf_digest,
        package_digest: package.package_digest,
    };
    let source_commit_digest = sha256_hex(boot_b.source_commit.as_bytes());
    let reference_commit_digest = sha256_hex(boot_b.reference_commit.as_bytes());
    let application_elf_digest = boot_b.app_elf_sha256.clone();
    let boot_b_session_digest = sha256_hex(boot_b.boot_session.as_bytes());
    let digest_input = (
        observation_source,
        true,
        &boot_b_session_digest,
        boot_b.boot_ordinal,
        &source_commit_digest,
        &reference_commit_digest,
        &application_elf_digest,
        &exact_package,
    );
    let claim_fact_digest = digest_serializable(&digest_input)?;

    Ok(ObservedRuntimeIdentityAdmission::Validated {
        identity: Box::new(ValidatedObservedRuntimeIdentity {
            observation_source,
            same_physical_device: true,
            boot_b_session_digest,
            boot_b_ordinal: boot_b.boot_ordinal,
            source_commit_digest,
            reference_commit_digest,
            application_elf_digest,
            exact_package,
            claim_fact_digest,
        }),
    })
}

fn validate_hardware_observation(
    package: &ExactPackageDocument,
    document: &str,
) -> Result<ObservedRuntimeIdentityAdmission, RuntimeIdentityEvidenceError> {
    let observation = parse_json::<HardwareObservationDocument>(document)?;
    if observation.schema_version != "phase36-hardware-runtime-observation-v1"
        || observation.board_category != "205"
        || observation.target != "xtensa-esp32s3-espidf"
        || observation.asic != "BM1366"
        || observation.detector_candidate_count != 1
        || !observation.same_physical_device
        || !is_lower_hex(&observation.physical_identity_digest, 64)
        || !is_lower_hex(&observation.boot_session, 32)
        || observation.boot_ordinal == 0
        || !matches!(
            observation.reset_reason_category.as_str(),
            "power_on" | "software_cpu"
        )
        || !(observation.trusted_origin.starts_with("http://")
            || observation.trusted_origin.starts_with("https://"))
        || observation.source_commit != package.source_commit
        || observation.reference_commit != package.reference_commit
        || observation.app_elf_sha256 != package.firmware_elf_digest
    {
        return Err(RuntimeIdentityEvidenceError::BuildIdentityMismatch);
    }
    let exact_package = ExactPackageIdentityJoin {
        manifest_digest: package.manifest_digest.clone(),
        executable_image_digest: package.executable_image_digest.clone(),
        factory_image_digest: package.factory_image_digest.clone(),
        firmware_elf_digest: package.firmware_elf_digest.clone(),
        package_digest: package.package_digest.clone(),
    };
    let source_commit_digest = sha256_hex(observation.source_commit.as_bytes());
    let reference_commit_digest = sha256_hex(observation.reference_commit.as_bytes());
    let application_elf_digest = observation.app_elf_sha256;
    let boot_b_session_digest = sha256_hex(observation.boot_session.as_bytes());
    let observation_source = RuntimeIdentityObservationSource::ExactPackageFlashSession;
    let claim_fact_digest = digest_serializable(&(
        observation_source,
        true,
        &boot_b_session_digest,
        observation.boot_ordinal,
        &source_commit_digest,
        &reference_commit_digest,
        &application_elf_digest,
        &exact_package,
        &observation.physical_identity_digest,
    ))?;
    Ok(ObservedRuntimeIdentityAdmission::Validated {
        identity: Box::new(ValidatedObservedRuntimeIdentity {
            observation_source,
            same_physical_device: true,
            boot_b_session_digest,
            boot_b_ordinal: observation.boot_ordinal,
            source_commit_digest,
            reference_commit_digest,
            application_elf_digest,
            exact_package,
            claim_fact_digest,
        }),
    })
}

fn replay_event_ledger(
    request: &SessionRequest,
    event_ledger: &str,
    private_result: &PrivateResultDocument,
    public_projection: &PublicProjectionDocument,
) -> Result<(), RuntimeIdentityEvidenceError> {
    let events = ledger::parse_and_validate(event_ledger)?;
    let mut state = SessionState::new(
        request.baseline.clone(),
        request.expected_postcondition.clone(),
        request.trusted_origin.clone(),
    );
    for event in events {
        state.apply(event);
    }
    let replayed_private = serde_json::to_value(state.private_result())
        .map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)?;
    let replayed_public = serde_json::to_value(state.projection())
        .map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)?;
    let supplied_private = serde_json::to_value(private_result)
        .map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)?;
    let supplied_public = serde_json::to_value(public_projection)
        .map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)?;
    if replayed_private != supplied_private || replayed_public != supplied_public {
        return Err(RuntimeIdentityEvidenceError::ReplayMismatch);
    }
    Ok(())
}

fn validate_terminal_pair(
    request: &SessionRequest,
    private_result: &PrivateResultDocument,
    public_projection: &PublicProjectionDocument,
) -> Result<(), RuntimeIdentityEvidenceError> {
    if private_result.schema_version != PRIVATE_RESULT_SCHEMA
        || public_projection.schema_version != PUBLIC_PROJECTION_SCHEMA
        || private_result.terminal_category != public_projection.terminal_category
        || private_result.request_outcome != public_projection.request_outcome
        || private_result.maybe_secondary_cleanup_failure
        || public_projection.platform_category != PlatformCategory::Macos
        || public_projection.board_category != request.board_category
    {
        return Err(RuntimeIdentityEvidenceError::PublicPrivateDisagreement);
    }
    if private_result.terminal_category != TerminalCategory::Ready
        || public_projection.terminal_category != TerminalCategory::Ready
        || public_projection.request_outcome != RequestOutcome::ResponseReceived
        || public_projection.request_attempt_count != 1
        || !public_projection.reader_armed
        || !public_projection.pre_restart_serial_delivery
        || !public_projection.post_restart_serial_delivery
        || !public_projection.service_loss_observed
        || !public_projection.trusted_origin_preserved
        || !public_projection.application_recovered
        || !public_projection.cleanup_complete
        || public_projection.serial_byte_count == 0
        || public_projection.http_observation_count != 1
        || public_projection.duration_millis == 0
    {
        return Err(RuntimeIdentityEvidenceError::MissingLedgerStep);
    }
    if !public_projection.same_physical_device || !public_projection.stable_enumeration {
        return Err(RuntimeIdentityEvidenceError::PhysicalDeviceMismatch);
    }
    if !public_projection.build_identity_matches
        || !public_projection.boot_session_changed
        || !public_projection.boot_ordinal_advanced_by_one
        || !public_projection.software_reset_observed
        || !public_projection.postcondition_matches
    {
        return Err(RuntimeIdentityEvidenceError::BuildIdentityMismatch);
    }
    Ok(())
}

fn validated_boot_b<'a>(
    request: &SessionRequest,
    private_result: &'a PrivateResultDocument,
    public_projection: &PublicProjectionDocument,
    package: &ExactPackageDocument,
) -> Result<&'a PrivateBootB, RuntimeIdentityEvidenceError> {
    validate_terminal_pair(request, private_result, public_projection)?;
    let boot_b = private_result
        .boot_b
        .as_ref()
        .ok_or(RuntimeIdentityEvidenceError::MissingLedgerStep)?;
    let next_ordinal = request
        .baseline
        .boot_ordinal
        .checked_add(1)
        .ok_or(RuntimeIdentityEvidenceError::BootSessionMismatch)?;
    if boot_b.boot_session.is_empty()
        || boot_b.boot_session == request.baseline.boot_session
        || boot_b.boot_ordinal != next_ordinal
        || boot_b.reset_reason_category != "software_cpu"
        || boot_b.trusted_origin != request.trusted_origin
        || boot_b.hostname_sha256 != request.expected_postcondition.hostname_sha256
    {
        return Err(RuntimeIdentityEvidenceError::BootSessionMismatch);
    }
    if boot_b.source_commit != package.source_commit
        || boot_b.reference_commit != package.reference_commit
        || boot_b.app_elf_sha256 != package.firmware_elf_digest
    {
        return Err(RuntimeIdentityEvidenceError::BuildIdentityMismatch);
    }
    Ok(boot_b)
}

fn validate_request_package_join(
    request: &SessionRequest,
    package: &ExactPackageDocument,
) -> Result<(), RuntimeIdentityEvidenceError> {
    if !request.schema_is_valid()
        || request.baseline.source_commit != package.source_commit
        || request.baseline.reference_commit != package.reference_commit
        || request.baseline.app_elf_sha256 != package.firmware_elf_digest
    {
        return Err(RuntimeIdentityEvidenceError::ExactPackageMismatch);
    }
    Ok(())
}

fn validate_package(package: &ExactPackageDocument) -> Result<(), RuntimeIdentityEvidenceError> {
    let commit_valid = [&package.source_commit, &package.reference_commit]
        .into_iter()
        .all(|value| is_lower_hex(value, 40));
    let digests_valid = [
        &package.manifest_digest,
        &package.executable_image_digest,
        &package.factory_image_digest,
        &package.firmware_elf_digest,
        &package.package_digest,
    ]
    .into_iter()
    .all(|value| is_lower_hex(value, 64));
    if package.schema_version != PACKAGE_SCHEMA || !commit_valid || !digests_valid {
        return Err(RuntimeIdentityEvidenceError::DocumentInvalid);
    }
    Ok(())
}

fn parse_json<T: for<'de> Deserialize<'de>>(
    document: &str,
) -> Result<T, RuntimeIdentityEvidenceError> {
    serde_json::from_str(document).map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)
}

fn digest_serializable(value: &impl Serialize) -> Result<String, RuntimeIdentityEvidenceError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)?;
    Ok(sha256_hex(&bytes))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
