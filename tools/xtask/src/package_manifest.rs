use std::fmt;
use std::fs;
use std::io::{Read, Write};

use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use bitaxe_api::BuildProvenance;

use crate::{
    validate_package_request, PackageEnvironment, PackageRequest, DEFAULT_ELF_NAME,
    EXPECTED_REFERENCE_COMMIT, FACTORY_IMAGE_NAME, RUST_TARGET, UNAVAILABLE,
};

const CANONICAL_PARTITION_TABLE_PATH: &str = "firmware/bitaxe/partitions-ultra205.csv";

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PackageManifestV3 {
    pub(crate) schema_version: u32,
    pub(crate) release_name: String,
    pub(crate) semantic_version: String,
    pub(crate) source_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) app_elf_sha256: String,
    pub(crate) build_identity: ManifestBuildIdentity,
    pub(crate) default_flash_image: String,
    pub(crate) image_metadata: ImageMetadata,
    pub(crate) tool_versions: ToolVersions,
    pub(crate) install_notes: ReleaseNotes,
    pub(crate) license_inventory: String,
    pub(crate) provenance_manifest: String,
    pub(crate) otadata_source: String,
    pub(crate) artifacts: Vec<ReleaseArtifact>,
    #[serde(default)]
    pub(crate) update_segments: Vec<bitaxe_api::update_segments::UpdateSegment>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ManifestBuildIdentity {
    pub(crate) label: String,
    pub(crate) channel: String,
    pub(crate) source_dirty: bool,
    pub(crate) release_tag: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReleaseArtifact {
    pub(crate) kind: ArtifactKind,
    pub(crate) path: String,
    pub(crate) offset: String,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum ArtifactKind {
    #[serde(rename = "bootloader")]
    Bootloader,
    #[serde(rename = "partition_table_binary")]
    PartitionTableBinary,
    #[serde(rename = "firmware_elf")]
    FirmwareElf,
    #[serde(rename = "firmware_ota_image")]
    FirmwareOtaImage,
    #[serde(rename = "www_spiffs_image")]
    WwwSpiffsImage,
    #[serde(rename = "factory_merged_image")]
    FactoryMergedImage,
    #[serde(rename = "partition_table")]
    PartitionTable,
    #[serde(rename = "otadata_initial")]
    OtadataInitial,
    #[serde(rename = "update_only_image")]
    UpdateOnlyImage,
}

impl fmt::Display for ArtifactKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bootloader => formatter.write_str("bootloader"),
            Self::PartitionTableBinary => formatter.write_str("partition_table_binary"),
            Self::FirmwareElf => formatter.write_str("firmware_elf"),
            Self::FirmwareOtaImage => formatter.write_str("firmware_ota_image"),
            Self::WwwSpiffsImage => formatter.write_str("www_spiffs_image"),
            Self::FactoryMergedImage => formatter.write_str("factory_merged_image"),
            Self::PartitionTable => formatter.write_str("partition_table"),
            Self::OtadataInitial => formatter.write_str("otadata_initial"),
            Self::UpdateOnlyImage => formatter.write_str("update_only_image"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImageMetadata {
    pub(crate) board: String,
    pub(crate) device_model: String,
    pub(crate) asic: String,
    pub(crate) esp_idf_version: String,
    pub(crate) rust_target: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReleaseNotes {
    pub(crate) path: String,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ToolVersions {
    pub(crate) cargo: String,
    pub(crate) rustc: String,
    pub(crate) bazel: String,
    pub(crate) espflash: String,
}

pub(crate) fn build_manifest(
    package_request: &PackageRequest,
    environment: &impl PackageEnvironment,
) -> Result<PackageManifestV3> {
    environment.run_reference_guard()?;
    validate_package_request(package_request)?;

    let stamp = fs::read_to_string(package_request.build_provenance_stamp.as_std_path())
        .with_context(|| {
            format!(
                "failed to read build provenance stamp {}",
                package_request.build_provenance_stamp
            )
        })?;
    let provenance =
        BuildProvenance::parse_stamp(&stamp).context("invalid canonical build provenance stamp")?;
    if provenance.reference_commit() != EXPECTED_REFERENCE_COMMIT {
        bail!(
            "reference commit mismatch after guard: expected {EXPECTED_REFERENCE_COMMIT}, found {}",
            provenance.reference_commit()
        );
    }
    let identity = provenance.build_identity();
    if package_request.app_descriptor_version != identity.build_label() {
        bail!(
            "ESP application descriptor version must equal build_label: expected {}, found {}",
            identity.build_label(),
            package_request.app_descriptor_version
        );
    }
    validate_app_elf_sha256(&package_request.app_elf_sha256)?;

    let Some(factory_image) = &package_request.factory_image else {
        bail!("factory image is required for package manifest v3");
    };

    let mut artifacts = vec![
        artifact_entry(
            ArtifactKind::FirmwareElf,
            &package_request.firmware_elf,
            UNAVAILABLE,
            &package_request.manifest,
        )?,
        artifact_entry(
            ArtifactKind::FirmwareOtaImage,
            &package_request.firmware_ota_image,
            "0x10000",
            &package_request.manifest,
        )?,
        artifact_entry(
            ArtifactKind::WwwSpiffsImage,
            &package_request.www_bin,
            "0x410000",
            &package_request.manifest,
        )?,
        artifact_entry(
            ArtifactKind::FactoryMergedImage,
            factory_image,
            "0x0",
            &package_request.manifest,
        )?,
        artifact_entry(
            ArtifactKind::PartitionTable,
            &package_request.partition_table,
            partition_table_offset(&package_request.partition_table),
            &package_request.manifest,
        )?,
        artifact_entry(
            ArtifactKind::OtadataInitial,
            &package_request.otadata_initial,
            "0xf10000",
            &package_request.manifest,
        )?,
    ];

    let bootloader = package_request.out_dir.join("bootloader.bin");
    let binary_table = package_request.out_dir.join("partition-table.bin");
    // Explicit legacy construction remains readable for immutable historical packages.
    let segmented = bootloader.is_file() && binary_table.is_file();
    let mut update_segments = Vec::new();
    if segmented {
        artifacts.push(artifact_entry(
            ArtifactKind::Bootloader,
            &bootloader,
            "0x0",
            &package_request.manifest,
        )?);
        artifacts.push(artifact_entry(
            ArtifactKind::PartitionTableBinary,
            &binary_table,
            "0x8000",
            &package_request.manifest,
        )?);
        for (kind, offset, path) in [
            ("bootloader", 0, bootloader.as_path()),
            ("partition_table_binary", 0x8000, binary_table.as_path()),
            (
                "firmware_ota_image",
                0x10000,
                package_request.firmware_ota_image.as_path(),
            ),
            (
                "www_spiffs_image",
                0x410000,
                package_request.www_bin.as_path(),
            ),
            (
                "otadata_initial",
                0xf10000,
                package_request.otadata_initial.as_path(),
            ),
        ] {
            update_segments.push(bitaxe_api::update_segments::UpdateSegment {
                artifact_kind: kind.to_owned(),
                offset,
                length: u32::try_from(fs::metadata(path)?.len())?,
            });
        }
        bitaxe_api::update_segments::validate_update_segments(&update_segments)?;
    }

    let manifest = PackageManifestV3 {
        schema_version: if segmented { 4 } else { 3 },
        release_name: package_request.release_name.clone(),
        semantic_version: provenance.semantic_version().to_owned(),
        source_commit: identity.source_commit().to_owned(),
        reference_commit: provenance.reference_commit().to_owned(),
        app_elf_sha256: package_request.app_elf_sha256.clone(),
        build_identity: ManifestBuildIdentity {
            label: identity.build_label().to_owned(),
            channel: identity.build_channel().as_str().to_owned(),
            source_dirty: identity.source_dirty(),
            release_tag: identity.maybe_release_tag().map(str::to_owned),
        },
        default_flash_image: manifest_relative_path(
            &package_request.manifest,
            &package_request.default_flash_image,
        ),
        image_metadata: ImageMetadata {
            board: package_request.board.to_string(),
            device_model: "Ultra 205".to_owned(),
            asic: "BM1366".to_owned(),
            esp_idf_version: crate::ESP_IDF_VERSION.to_owned(),
            rust_target: RUST_TARGET.to_owned(),
        },
        tool_versions: tool_versions(environment),
        install_notes: ReleaseNotes {
            path: manifest_relative_path(&package_request.manifest, &package_request.install_notes),
            summary: "Ultra 205 release operator guide".to_owned(),
        },
        license_inventory: manifest_relative_path(
            &package_request.manifest,
            &package_request.license_inventory,
        ),
        provenance_manifest: manifest_relative_path(
            &package_request.manifest,
            &package_request.provenance_manifest,
        ),
        otadata_source: package_request.otadata_source.clone(),
        artifacts,
        update_segments,
    };
    validate_package_manifest_v3(&manifest)?;

    Ok(manifest)
}

pub(crate) fn validate_default_flash_image(default_flash_image: &Utf8Path) -> Result<()> {
    let maybe_file_name = default_flash_image.file_name();
    let Some(file_name) = maybe_file_name else {
        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}");
    };

    if file_name != DEFAULT_ELF_NAME {
        if file_name == FACTORY_IMAGE_NAME {
            bail!(
                "default_flash_image must resolve to {DEFAULT_ELF_NAME}; {FACTORY_IMAGE_NAME} is only an additional artifact"
            );
        }

        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}, not {file_name}");
    }

    Ok(())
}

pub(crate) fn read_manifest_v3(path: &Utf8Path) -> Result<PackageManifestV3> {
    let contents = fs::read_to_string(path.as_std_path())
        .with_context(|| format!("failed to read package manifest {path}"))?;
    serde_json::from_str(&contents).with_context(|| format!("failed to parse manifest v3 {path}"))
}

pub(crate) fn validate_package_manifest_v3(manifest: &PackageManifestV3) -> Result<()> {
    if !matches!(manifest.schema_version, 3 | 4) {
        bail!(
            "package manifest schema_version must be 3, found {}",
            manifest.schema_version
        );
    }

    let provenance = BuildProvenance::new(
        &manifest.semantic_version,
        &manifest.source_commit,
        manifest.build_identity.source_dirty,
        manifest.build_identity.release_tag.as_deref(),
        &manifest.reference_commit,
    )
    .context("package manifest contains invalid build provenance")?;
    let identity = provenance.build_identity();
    if manifest.build_identity.label != identity.build_label()
        || manifest.build_identity.channel != identity.build_channel().as_str()
    {
        bail!("package manifest build_identity contradicts canonical provenance");
    }
    validate_app_elf_sha256(&manifest.app_elf_sha256)?;

    validate_default_flash_image(Utf8Path::new(&manifest.default_flash_image))?;
    require_non_empty("release_name", &manifest.release_name)?;
    require_non_empty("source_commit", &manifest.source_commit)?;
    require_non_empty("reference_commit", &manifest.reference_commit)?;
    require_non_empty("image_metadata.board", &manifest.image_metadata.board)?;
    require_non_empty(
        "image_metadata.device_model",
        &manifest.image_metadata.device_model,
    )?;
    require_non_empty("image_metadata.asic", &manifest.image_metadata.asic)?;
    require_non_empty(
        "image_metadata.esp_idf_version",
        &manifest.image_metadata.esp_idf_version,
    )?;
    require_non_empty(
        "image_metadata.rust_target",
        &manifest.image_metadata.rust_target,
    )?;
    require_non_empty("install_notes.path", &manifest.install_notes.path)?;
    require_non_empty("install_notes.summary", &manifest.install_notes.summary)?;
    require_non_empty("license_inventory", &manifest.license_inventory)?;
    require_non_empty("provenance_manifest", &manifest.provenance_manifest)?;
    require_non_empty("otadata_source", &manifest.otadata_source)?;
    require_non_empty("tool_versions.cargo", &manifest.tool_versions.cargo)?;
    require_non_empty("tool_versions.rustc", &manifest.tool_versions.rustc)?;
    require_non_empty("tool_versions.bazel", &manifest.tool_versions.bazel)?;
    require_non_empty("tool_versions.espflash", &manifest.tool_versions.espflash)?;

    for kind in [
        ArtifactKind::FirmwareElf,
        ArtifactKind::FirmwareOtaImage,
        ArtifactKind::WwwSpiffsImage,
        ArtifactKind::FactoryMergedImage,
        ArtifactKind::PartitionTable,
        ArtifactKind::OtadataInitial,
    ] {
        require_artifact_kind(manifest, kind)?;
    }

    for artifact in &manifest.artifacts {
        validate_sha256(&artifact.kind, &artifact.sha256)?;
        require_non_empty("artifact.path", &artifact.path)?;
        require_non_empty("artifact.offset", &artifact.offset)?;
    }
    validate_firmware_elf_app_sha_relationship(manifest)?;
    if manifest.schema_version == 4 {
        require_artifact_kind(manifest, ArtifactKind::Bootloader)?;
        require_artifact_kind(manifest, ArtifactKind::PartitionTableBinary)?;
        bitaxe_api::update_segments::validate_update_segments(&manifest.update_segments)?;
    }

    require_artifact_offset(manifest, ArtifactKind::FirmwareElf, UNAVAILABLE)?;
    require_artifact_offset(manifest, ArtifactKind::FirmwareOtaImage, "0x10000")?;
    require_artifact_offset(manifest, ArtifactKind::WwwSpiffsImage, "0x410000")?;
    require_artifact_offset(manifest, ArtifactKind::FactoryMergedImage, "0x0")?;
    require_artifact_offset(manifest, ArtifactKind::OtadataInitial, "0xf10000")?;

    let partition_table = require_artifact_kind(manifest, ArtifactKind::PartitionTable)?;
    if partition_table.offset != "0x8000" && partition_table.offset != UNAVAILABLE {
        bail!(
            "partition_table artifact offset must be 0x8000 for a binary table or {UNAVAILABLE} for CSV-only, found {}",
            partition_table.offset
        );
    }

    Ok(())
}

fn require_non_empty(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} must not be empty");
    }

    Ok(())
}

fn require_artifact_kind(
    manifest: &PackageManifestV3,
    kind: ArtifactKind,
) -> Result<&ReleaseArtifact> {
    let mut matches = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind);
    let Some(artifact) = matches.next() else {
        bail!(required_artifact_message(kind));
    };
    if matches.next().is_some() {
        bail!(duplicate_artifact_message(kind));
    }

    Ok(artifact)
}

fn validate_firmware_elf_app_sha_relationship(manifest: &PackageManifestV3) -> Result<()> {
    let firmware_elf = require_artifact_kind(manifest, ArtifactKind::FirmwareElf)?;
    if firmware_elf.sha256 != manifest.app_elf_sha256 {
        bail!("firmware_elf_app_sha_mismatch");
    }

    Ok(())
}

fn validate_sha256(kind: &ArtifactKind, sha256: &str) -> Result<()> {
    if sha256.len() == 64
        && sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }

    bail!("{kind} sha256 must be a 64 character hex string");
}

fn validate_app_elf_sha256(sha256: &str) -> Result<()> {
    if sha256.len() == 64
        && sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && sha256.bytes().any(|byte| byte != b'0')
    {
        return Ok(());
    }

    bail!("app_elf_sha256 must be a nonzero 64-character lowercase hexadecimal string")
}

fn require_artifact_offset(
    manifest: &PackageManifestV3,
    kind: ArtifactKind,
    expected_offset: &str,
) -> Result<()> {
    let artifact = require_artifact_kind(manifest, kind)?;
    if artifact.offset == expected_offset {
        return Ok(());
    }

    bail!(
        "{kind} artifact must use offset {expected_offset}, found {}",
        artifact.offset
    );
}

fn required_artifact_message(kind: ArtifactKind) -> String {
    match kind {
        ArtifactKind::Bootloader => "required artifact kind bootloader missing".to_owned(),
        ArtifactKind::PartitionTableBinary => {
            "required artifact kind partition_table_binary missing".to_owned()
        }
        ArtifactKind::FirmwareElf => "required artifact kind firmware_elf missing".to_owned(),
        ArtifactKind::FirmwareOtaImage => {
            "required artifact kind firmware_ota_image missing".to_owned()
        }
        ArtifactKind::WwwSpiffsImage => {
            "required artifact kind www_spiffs_image missing".to_owned()
        }
        ArtifactKind::FactoryMergedImage => {
            "required artifact kind factory_merged_image missing".to_owned()
        }
        ArtifactKind::PartitionTable => "required artifact kind partition_table missing".to_owned(),
        ArtifactKind::OtadataInitial => "required artifact kind otadata_initial missing".to_owned(),
        ArtifactKind::UpdateOnlyImage => {
            "required artifact kind update_only_image missing".to_owned()
        }
    }
}

fn duplicate_artifact_message(kind: ArtifactKind) -> String {
    format!("required artifact kind {kind} duplicate")
}

fn artifact_entry(
    kind: ArtifactKind,
    path: &Utf8Path,
    offset: &str,
    manifest_path: &Utf8Path,
) -> Result<ReleaseArtifact> {
    let artifact_manifest_path =
        if kind == ArtifactKind::PartitionTable && path.ends_with(CANONICAL_PARTITION_TABLE_PATH) {
            CANONICAL_PARTITION_TABLE_PATH.to_owned()
        } else {
            manifest_relative_path(manifest_path, path)
        };
    Ok(ReleaseArtifact {
        kind,
        path: artifact_manifest_path,
        offset: offset.to_owned(),
        sha256: sha256_file(path)?,
    })
}

fn partition_table_offset(path: &Utf8Path) -> &'static str {
    let Some(file_name) = path.file_name() else {
        return UNAVAILABLE;
    };

    if file_name.ends_with(".bin") {
        return "0x8000";
    }

    UNAVAILABLE
}

fn manifest_relative_path(manifest_path: &Utf8Path, artifact_path: &Utf8Path) -> String {
    let maybe_manifest_dir = manifest_path.parent();
    if let Some(manifest_dir) = maybe_manifest_dir {
        if artifact_path.parent() == Some(manifest_dir) {
            if let Some(file_name) = artifact_path.file_name() {
                return file_name.to_owned();
            }
        }
    }

    artifact_path.as_str().to_owned()
}

pub(crate) fn sha256_file(path: &Utf8Path) -> Result<String> {
    let mut file = fs::File::open(path.as_std_path())
        .with_context(|| format!("failed to open artifact for checksum: {path}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let byte_count = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read artifact for checksum: {path}"))?;
        if byte_count == 0 {
            break;
        }
        hasher.update(&buffer[..byte_count]);
    }

    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        encoded.push_str(&format!("{byte:02x}"));
    }

    Ok(encoded)
}

pub(crate) fn tool_versions(environment: &impl PackageEnvironment) -> ToolVersions {
    ToolVersions {
        cargo: environment
            .maybe_tool_version("cargo")
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        rustc: environment
            .maybe_tool_version("rustc")
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        bazel: environment
            .maybe_tool_version("bazel")
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        espflash: environment
            .maybe_tool_version("espflash")
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
    }
}

pub(crate) fn write_manifest(path: &Utf8Path, manifest: &PackageManifestV3) -> Result<()> {
    let mut json =
        serde_json::to_string_pretty(manifest).context("failed to serialize package manifest")?;
    json.push('\n');

    let maybe_parent = path.parent();
    if let Some(parent) = maybe_parent {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create manifest directory {parent}"))?;
    }

    let mut file = fs::File::create(path.as_std_path())
        .with_context(|| format!("failed to create package manifest {path}"))?;
    file.write_all(json.as_bytes())
        .with_context(|| format!("failed to write package manifest {path}"))
}

#[cfg(test)]
mod tests;
