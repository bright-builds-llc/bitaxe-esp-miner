use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use bitaxe_api::{BuildProvenance, PlatformFact};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::phase35_evidence::sha256_hex;

const SYS004_VERSION_EVIDENCE_SCHEMA: &str = "sys004-version-evidence-v1";

#[derive(Debug, Deserialize)]
struct AttemptHandle {
    schema_version: String,
    child_name: String,
    capability_digest: String,
    source_commit: String,
    reference_commit: String,
    target: String,
    board: String,
    asic: String,
    manifest_path: String,
    manifest_digest: String,
    firmware_elf_digest: String,
    package_identity_digest: String,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    schema_version: u32,
    semantic_version: String,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    build_identity: ManifestBuildIdentity,
    image_metadata: ImageMetadata,
}

#[derive(Debug, Deserialize)]
struct ManifestBuildIdentity {
    label: String,
    channel: String,
    source_dirty: bool,
    release_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageMetadata {
    board: String,
    asic: String,
    esp_idf_version: String,
    rust_target: String,
}

#[derive(Debug, Deserialize)]
struct PrivateCapture {
    schema_version: String,
    board_category: String,
    substantive: SubstantiveDocuments,
    runtime_identity: RuntimeIdentityDocuments,
    broker: BrokerDocument,
}

#[derive(Debug, Deserialize)]
struct SubstantiveDocuments {
    system_info_document: String,
    websocket_document: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeIdentityDocuments {
    exact_package_document: String,
}

#[derive(Debug, Deserialize)]
struct BrokerDocument {
    capability_digest: String,
    package_digest: String,
    same_physical_device_observed: bool,
}

#[derive(Debug, Deserialize)]
struct ExactPackageDocument {
    schema_version: String,
    source_commit: String,
    reference_commit: String,
    manifest_digest: String,
    firmware_elf_digest: String,
    package_digest: String,
}

#[derive(Debug, Deserialize)]
struct AttemptSeal {
    schema_version: String,
    status: String,
    first_failure: Option<serde_json::Value>,
    secondary_failure: Option<serde_json::Value>,
    capability_digest: String,
    package_identity_digest: String,
    candidate_digest: Option<String>,
    private_capture_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionApiProjection {
    #[serde(rename = "ASICModel")]
    asic_model: String,
    #[serde(rename = "boardVersion")]
    board_version: String,
    version: String,
    #[serde(rename = "semanticVersion")]
    semantic_version: String,
    #[serde(rename = "sourceCommit")]
    source_commit: String,
    #[serde(rename = "referenceCommit")]
    reference_commit: String,
    #[serde(rename = "appElfSha256")]
    app_elf_sha256: String,
    #[serde(rename = "buildTimestampUtc")]
    build_timestamp_utc: String,
    #[serde(rename = "buildChannel")]
    build_channel: String,
    #[serde(rename = "sourceDirty")]
    source_dirty: bool,
    #[serde(rename = "releaseTag")]
    release_tag: Option<String>,
    #[serde(rename = "axeOSVersion")]
    axe_os_version: String,
    #[serde(rename = "idfVersion")]
    idf_version: String,
    #[serde(rename = "platformIdentity")]
    platform_identity: PlatformIdentityProjection,
}

#[derive(Debug, Deserialize)]
struct PlatformIdentityProjection {
    #[serde(rename = "axeOsStaticAsset")]
    axe_os_static_asset: PlatformFact<String>,
    #[serde(rename = "espIdfVersion")]
    esp_idf_version: PlatformFact<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct Sys004VersionEvidence {
    schema_version: &'static str,
    status: &'static str,
    board: String,
    asic: String,
    rust_target: String,
    semantic_version: String,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    build_label: String,
    build_timestamp_utc: String,
    build_channel: String,
    source_dirty: bool,
    release_tag: Option<String>,
    axe_os_version: String,
    esp_idf_version: String,
    manifest_digest: String,
    package_identity_digest: String,
    private_capture_digest: String,
    http_websocket_identical: bool,
    same_physical_device_observed: bool,
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum Sys004VersionEvidenceError {
    #[error("sys004_private_boundary_invalid")]
    PrivateBoundary,
    #[error("sys004_attempt_not_eligible")]
    AttemptNotEligible,
    #[error("sys004_document_invalid")]
    Document,
    #[error("sys004_package_identity_mismatch")]
    PackageIdentity,
    #[error("sys004_live_version_mismatch")]
    LiveVersion,
    #[error("sys004_output_failed")]
    Output,
}

pub(crate) fn project_sys004_version_evidence(
    private_parent: &Utf8Path,
    attempt_handle_file: &Utf8Path,
    package_manifest: &Utf8Path,
    output: &Utf8Path,
) -> Result<Sys004VersionEvidence, Sys004VersionEvidenceError> {
    validate_private_directory(private_parent)?;
    validate_private_file(attempt_handle_file)?;
    if attempt_handle_file
        .parent()
        .map(canonical_utf8)
        .transpose()?
        .as_ref()
        != Some(&canonical_utf8(private_parent)?)
    {
        return Err(Sys004VersionEvidenceError::PrivateBoundary);
    }
    let handle_bytes =
        fs::read(attempt_handle_file).map_err(|_| Sys004VersionEvidenceError::PrivateBoundary)?;
    let handle = serde_json::from_slice::<AttemptHandle>(&handle_bytes)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    validate_handle(&handle)?;

    let canonical_manifest = canonical_utf8(package_manifest)?;
    let canonical_handle_manifest = canonical_utf8(Utf8Path::new(&handle.manifest_path))?;
    if canonical_manifest != canonical_handle_manifest {
        return Err(Sys004VersionEvidenceError::PackageIdentity);
    }
    let manifest_bytes =
        fs::read(&canonical_manifest).map_err(|_| Sys004VersionEvidenceError::Document)?;
    let private_child = private_parent.join(&handle.child_name);
    validate_private_directory(&private_child)?;
    let seal_path = private_child.join("seal.json");
    validate_private_file(&seal_path)?;
    let seal_bytes = fs::read(seal_path).map_err(|_| Sys004VersionEvidenceError::Document)?;
    let seal = serde_json::from_slice::<AttemptSeal>(&seal_bytes)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    require_eligible_attempt_seal(&seal, &handle)?;

    let private_capture_path = private_child.join("private-capture.json");
    validate_private_file(&private_capture_path)?;
    let private_bytes =
        fs::read(private_capture_path).map_err(|_| Sys004VersionEvidenceError::Document)?;

    let evidence = classify_documents(&manifest_bytes, &private_bytes, &seal_bytes, &handle)?;
    write_projection(output, &evidence)?;
    Ok(evidence)
}

pub(crate) fn project_sys004_version_evidence_from_workspace(
    workspace_dir: &Utf8Path,
    private_parent: &Utf8Path,
    attempt_handle_file: &Utf8Path,
    package_manifest: &Utf8Path,
    output: &Utf8Path,
) -> Result<Sys004VersionEvidence, Sys004VersionEvidenceError> {
    project_sys004_version_evidence(
        &workspace_path(workspace_dir, private_parent),
        &workspace_path(workspace_dir, attempt_handle_file),
        &workspace_path(workspace_dir, package_manifest),
        &workspace_path(workspace_dir, output),
    )
}

fn workspace_path(workspace_dir: &Utf8Path, path: &Utf8Path) -> Utf8PathBuf {
    if path.is_absolute() {
        return path.to_owned();
    }
    workspace_dir.join(path)
}

fn require_eligible_attempt_seal(
    seal: &AttemptSeal,
    handle: &AttemptHandle,
) -> Result<(), Sys004VersionEvidenceError> {
    if seal.schema_version != "phase36-attempt-seal-v2"
        || seal.capability_digest != handle.capability_digest
        || seal.package_identity_digest != handle.package_identity_digest
    {
        return Err(Sys004VersionEvidenceError::PackageIdentity);
    }
    if seal.status == "sealed_non_promotion"
        && seal
            .first_failure
            .as_ref()
            .and_then(serde_json::Value::as_str)
            .is_some_and(valid_broker_failure)
        && seal
            .secondary_failure
            .as_ref()
            .is_none_or(|value| value.as_str().is_some_and(valid_broker_failure))
        && seal.candidate_digest.is_none()
        && seal.private_capture_digest.is_none()
    {
        return Err(Sys004VersionEvidenceError::AttemptNotEligible);
    }
    if seal.status != "sealed_eligible"
        || seal.first_failure.is_some()
        || seal.secondary_failure.is_some()
        || seal
            .candidate_digest
            .as_deref()
            .is_none_or(|value| !valid_digest(value))
        || seal
            .private_capture_digest
            .as_deref()
            .is_none_or(|value| !valid_digest(value))
    {
        return Err(Sys004VersionEvidenceError::PackageIdentity);
    }
    Ok(())
}

fn classify_documents(
    manifest_bytes: &[u8],
    private_bytes: &[u8],
    seal_bytes: &[u8],
    handle: &AttemptHandle,
) -> Result<Sys004VersionEvidence, Sys004VersionEvidenceError> {
    let manifest = serde_json::from_slice::<PackageManifest>(manifest_bytes)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    let private = serde_json::from_slice::<PrivateCapture>(private_bytes)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    let seal = serde_json::from_slice::<AttemptSeal>(seal_bytes)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    validate_package_join(
        &manifest,
        manifest_bytes,
        &private,
        private_bytes,
        &seal,
        handle,
    )?;

    let api_json = unique_prefixed_json(
        &private.substantive.system_info_document,
        "system_info_json: ",
    )?;
    let websocket_json = unique_prefixed_json(
        &private.substantive.websocket_document,
        "live_websocket_json: ",
    )?;
    if api_json != websocket_json {
        return Err(Sys004VersionEvidenceError::LiveVersion);
    }
    let live = serde_json::from_value::<VersionApiProjection>(api_json)
        .map_err(|_| Sys004VersionEvidenceError::Document)?;
    validate_live_version(&live, &manifest)?;

    Ok(Sys004VersionEvidence {
        schema_version: SYS004_VERSION_EVIDENCE_SCHEMA,
        status: "verified",
        board: live.board_version,
        asic: live.asic_model,
        rust_target: manifest.image_metadata.rust_target,
        semantic_version: live.semantic_version,
        source_commit: live.source_commit,
        reference_commit: live.reference_commit,
        app_elf_sha256: live.app_elf_sha256,
        build_label: live.version,
        build_timestamp_utc: live.build_timestamp_utc,
        build_channel: live.build_channel,
        source_dirty: live.source_dirty,
        release_tag: live.release_tag,
        axe_os_version: live.axe_os_version,
        esp_idf_version: live.idf_version,
        manifest_digest: handle.manifest_digest.clone(),
        package_identity_digest: handle.package_identity_digest.clone(),
        private_capture_digest: sha256_hex(private_bytes),
        http_websocket_identical: true,
        same_physical_device_observed: true,
    })
}

fn validate_handle(handle: &AttemptHandle) -> Result<(), Sys004VersionEvidenceError> {
    if handle.schema_version != "phase36-attempt-handle-v2"
        || !valid_child_name(&handle.child_name)
        || !valid_digest(&handle.capability_digest)
        || !valid_commit(&handle.source_commit)
        || !valid_commit(&handle.reference_commit)
        || handle.target != "xtensa-esp32s3-espidf"
        || handle.board != "205"
        || handle.asic != "BM1366"
        || !valid_digest(&handle.manifest_digest)
        || !valid_digest(&handle.firmware_elf_digest)
        || !valid_digest(&handle.package_identity_digest)
    {
        return Err(Sys004VersionEvidenceError::PackageIdentity);
    }
    Ok(())
}

fn validate_package_join(
    manifest: &PackageManifest,
    manifest_bytes: &[u8],
    private: &PrivateCapture,
    private_bytes: &[u8],
    seal: &AttemptSeal,
    handle: &AttemptHandle,
) -> Result<(), Sys004VersionEvidenceError> {
    let provenance = BuildProvenance::new(
        &manifest.semantic_version,
        &manifest.source_commit,
        manifest.build_identity.source_dirty,
        manifest.build_identity.release_tag.as_deref(),
        &manifest.reference_commit,
    )
    .map_err(|_| Sys004VersionEvidenceError::PackageIdentity)?;
    let identity = provenance.build_identity();
    let exact = serde_json::from_str::<ExactPackageDocument>(
        &private.runtime_identity.exact_package_document,
    )
    .map_err(|_| Sys004VersionEvidenceError::Document)?;
    if manifest.schema_version != 3
        || manifest.build_identity.label != identity.build_label()
        || manifest.build_identity.channel != identity.build_channel().as_str()
        || manifest.source_commit != handle.source_commit
        || manifest.reference_commit != handle.reference_commit
        || manifest.app_elf_sha256 != handle.firmware_elf_digest
        || manifest.image_metadata.board != "205"
        || manifest.image_metadata.asic != "BM1366"
        || manifest.image_metadata.rust_target != handle.target
        || sha256_hex(manifest_bytes) != handle.manifest_digest
        || private.schema_version != "phase36-private-capture-v1"
        || private.board_category != "205"
        || private.broker.capability_digest != handle.capability_digest
        || private.broker.package_digest != handle.package_identity_digest
        || !private.broker.same_physical_device_observed
        || exact.schema_version != "phase36-runtime-package-v1"
        || exact.source_commit != handle.source_commit
        || exact.reference_commit != handle.reference_commit
        || exact.manifest_digest != handle.manifest_digest
        || exact.firmware_elf_digest != handle.firmware_elf_digest
        || exact.package_digest != handle.package_identity_digest
        || seal.schema_version != "phase36-attempt-seal-v2"
        || seal.status != "sealed_eligible"
        || seal.first_failure.is_some()
        || seal.secondary_failure.is_some()
        || seal.capability_digest != handle.capability_digest
        || seal.package_identity_digest != handle.package_identity_digest
        || seal
            .candidate_digest
            .as_deref()
            .is_none_or(|value| !valid_digest(value))
        || seal.private_capture_digest.as_deref() != Some(sha256_hex(private_bytes).as_str())
    {
        return Err(Sys004VersionEvidenceError::PackageIdentity);
    }
    Ok(())
}

fn validate_live_version(
    live: &VersionApiProjection,
    manifest: &PackageManifest,
) -> Result<(), Sys004VersionEvidenceError> {
    let static_asset = live.platform_identity.axe_os_static_asset.maybe_value();
    let idf = live.platform_identity.esp_idf_version.maybe_value();
    if live.asic_model != "BM1366"
        || live.board_version != "205"
        || live.version != manifest.build_identity.label
        || live.axe_os_version != manifest.build_identity.label
        || static_asset != Some(&manifest.build_identity.label)
        || live.semantic_version != manifest.semantic_version
        || live.source_commit != manifest.source_commit
        || live.reference_commit != manifest.reference_commit
        || live.app_elf_sha256 != manifest.app_elf_sha256
        || live.build_channel != manifest.build_identity.channel
        || live.source_dirty != manifest.build_identity.source_dirty
        || live.release_tag != manifest.build_identity.release_tag
        || live.idf_version != manifest.image_metadata.esp_idf_version
        || idf != Some(&manifest.image_metadata.esp_idf_version)
        || !valid_build_timestamp(&live.build_timestamp_utc)
    {
        return Err(Sys004VersionEvidenceError::LiveVersion);
    }
    Ok(())
}

fn unique_prefixed_json(
    document: &str,
    prefix: &str,
) -> Result<serde_json::Value, Sys004VersionEvidenceError> {
    let values = document
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .collect::<Vec<_>>();
    let [value] = values.as_slice() else {
        return Err(Sys004VersionEvidenceError::Document);
    };
    serde_json::from_str(value).map_err(|_| Sys004VersionEvidenceError::Document)
}

fn validate_private_directory(path: &Utf8Path) -> Result<(), Sys004VersionEvidenceError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| Sys004VersionEvidenceError::PrivateBoundary)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(Sys004VersionEvidenceError::PrivateBoundary);
    }
    Ok(())
}

fn validate_private_file(path: &Utf8Path) -> Result<(), Sys004VersionEvidenceError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| Sys004VersionEvidenceError::PrivateBoundary)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(Sys004VersionEvidenceError::PrivateBoundary);
    }
    Ok(())
}

fn canonical_utf8(path: &Utf8Path) -> Result<Utf8PathBuf, Sys004VersionEvidenceError> {
    Utf8PathBuf::from_path_buf(
        fs::canonicalize(path).map_err(|_| Sys004VersionEvidenceError::PrivateBoundary)?,
    )
    .map_err(|_| Sys004VersionEvidenceError::PrivateBoundary)
}

fn write_projection(
    output: &Utf8Path,
    evidence: &Sys004VersionEvidence,
) -> Result<(), Sys004VersionEvidenceError> {
    let parent = output.parent().ok_or(Sys004VersionEvidenceError::Output)?;
    fs::create_dir_all(parent).map_err(|_| Sys004VersionEvidenceError::Output)?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| Sys004VersionEvidenceError::Output)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Sys004VersionEvidenceError::Output);
    }
    let bytes =
        serde_json::to_vec_pretty(evidence).map_err(|_| Sys004VersionEvidenceError::Output)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(output)
        .map_err(|_| Sys004VersionEvidenceError::Output)?;
    file.write_all(&bytes)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|_| Sys004VersionEvidenceError::Output)
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

fn valid_broker_failure(value: &str) -> bool {
    matches!(
        value,
        "admission_failed"
            | "capability_failed"
            | "authentication_failed"
            | "detector_failed"
            | "invocation_construction_failed"
            | "parser_failed"
            | "flash_failed"
            | "capture_failed"
            | "recovery_failed"
            | "cleanup_failed"
    )
}

fn valid_build_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            10 => *byte == b'T',
            13 | 16 => *byte == b':',
            19 => *byte == b'Z',
            _ => byte.is_ascii_digit(),
        })
}

#[cfg(test)]
#[path = "sys004_version_evidence/tests.rs"]
mod tests;
