use std::fmt;
use std::fs;
use std::io::{Read, Write};

use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    validate_package_request, PackageEnvironment, PackageRequest, DEFAULT_ELF_NAME,
    EXPECTED_REFERENCE_COMMIT, FACTORY_IMAGE_NAME, RUST_TARGET, UNAVAILABLE,
};

#[derive(Debug, Serialize)]
pub(crate) struct PackageManifest {
    pub(crate) schema_version: u32,
    pub(crate) board: String,
    pub(crate) device_model: String,
    pub(crate) asic: String,
    pub(crate) firmware_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) esp_idf_version: String,
    pub(crate) rust_target: String,
    pub(crate) tool_versions: ToolVersions,
    pub(crate) default_flash_image: String,
    pub(crate) artifacts: Vec<ManifestArtifact>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PackageManifestV2 {
    pub(crate) schema_version: u32,
    pub(crate) release_name: String,
    pub(crate) default_flash_image: String,
    pub(crate) image_metadata: ImageMetadata,
    pub(crate) install_notes: ReleaseNotes,
    pub(crate) license_inventory: String,
    pub(crate) provenance_manifest: String,
    pub(crate) artifacts: Vec<ReleaseArtifact>,
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

#[derive(Debug, Serialize)]
pub(crate) struct ToolVersions {
    pub(crate) cargo: String,
    pub(crate) rustc: String,
    pub(crate) bazel: String,
    pub(crate) espflash: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ManifestArtifact {
    pub(crate) kind: String,
    pub(crate) path: String,
    pub(crate) offset: String,
    pub(crate) sha256: String,
}

pub(crate) fn build_manifest(
    package_request: &PackageRequest,
    environment: &impl PackageEnvironment,
) -> Result<PackageManifest> {
    environment.run_reference_guard()?;
    validate_package_request(package_request)?;

    let firmware_commit = environment
        .firmware_commit()
        .unwrap_or_else(|_| UNAVAILABLE.to_owned());
    let reference_commit = environment.reference_commit()?;
    if reference_commit != EXPECTED_REFERENCE_COMMIT {
        bail!(
            "reference commit mismatch after guard: expected {EXPECTED_REFERENCE_COMMIT}, found {reference_commit}"
        );
    }

    let mut artifacts = Vec::new();
    artifacts.push(artifact_entry(
        "firmware_elf",
        &package_request.firmware_elf,
        UNAVAILABLE,
        &package_request.manifest,
    )?);

    if let Some(factory_image) = &package_request.factory_image {
        artifacts.push(artifact_entry(
            "factory_image",
            factory_image,
            "0x0",
            &package_request.manifest,
        )?);
    }

    Ok(PackageManifest {
        schema_version: 1,
        board: package_request.board.to_string(),
        device_model: "Ultra 205".to_owned(),
        asic: "BM1366".to_owned(),
        firmware_commit,
        reference_commit,
        esp_idf_version: crate::ESP_IDF_VERSION.to_owned(),
        rust_target: RUST_TARGET.to_owned(),
        tool_versions: tool_versions(environment),
        default_flash_image: manifest_relative_path(
            &package_request.manifest,
            &package_request.default_flash_image,
        ),
        artifacts,
    })
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

pub(crate) fn read_manifest_v2(path: &Utf8Path) -> Result<PackageManifestV2> {
    let contents = fs::read_to_string(path.as_std_path())
        .with_context(|| format!("failed to read package manifest {path}"))?;
    serde_json::from_str(&contents).with_context(|| format!("failed to parse manifest v2 {path}"))
}

pub(crate) fn validate_package_manifest_v2(manifest: &PackageManifestV2) -> Result<()> {
    if manifest.schema_version != 2 {
        bail!(
            "package manifest schema_version must be 2, found {}",
            manifest.schema_version
        );
    }

    validate_default_flash_image(Utf8Path::new(&manifest.default_flash_image))?;
    require_non_empty("release_name", &manifest.release_name)?;
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

    let factory = require_artifact_kind(manifest, ArtifactKind::FactoryMergedImage)?;
    if factory.offset != "0x0" {
        bail!(
            "factory_merged_image artifact must use factory offset 0x0, found {}",
            factory.offset
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
    manifest: &PackageManifestV2,
    kind: ArtifactKind,
) -> Result<&ReleaseArtifact> {
    manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
        .with_context(|| required_artifact_message(kind))
}

fn validate_sha256(kind: &ArtifactKind, sha256: &str) -> Result<()> {
    if sha256.len() == 64
        && sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Ok(());
    }

    bail!("{kind} sha256 must be a 64 character hex string");
}

fn required_artifact_message(kind: ArtifactKind) -> String {
    match kind {
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

fn artifact_entry(
    kind: &str,
    path: &Utf8Path,
    offset: &str,
    manifest_path: &Utf8Path,
) -> Result<ManifestArtifact> {
    Ok(ManifestArtifact {
        kind: kind.to_owned(),
        path: manifest_relative_path(manifest_path, path),
        offset: offset.to_owned(),
        sha256: sha256_file(path)?,
    })
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

pub(crate) fn write_manifest(path: &Utf8Path, manifest: &PackageManifest) -> Result<()> {
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
mod tests {
    use super::*;

    #[test]
    fn package_manifest_v2_requires_release_metadata_and_artifact_kinds() {
        // Arrange
        let artifact_kinds = [
            ArtifactKind::FirmwareElf,
            ArtifactKind::FirmwareOtaImage,
            ArtifactKind::WwwSpiffsImage,
            ArtifactKind::FactoryMergedImage,
            ArtifactKind::PartitionTable,
            ArtifactKind::OtadataInitial,
            ArtifactKind::UpdateOnlyImage,
        ];

        // Act
        let manifest = PackageManifestV2 {
            schema_version: 2,
            release_name: "bitaxe-ultra205-v1".to_owned(),
            default_flash_image: "bitaxe-ultra205.elf".to_owned(),
            image_metadata: ImageMetadata {
                board: "205".to_owned(),
                device_model: "Ultra 205".to_owned(),
                asic: "BM1366".to_owned(),
                esp_idf_version: "v5.5.4".to_owned(),
                rust_target: "xtensa-esp32s3-espidf".to_owned(),
            },
            install_notes: ReleaseNotes {
                path: "docs/release/ultra-205.md".to_owned(),
                summary: "Flash with just flash board=205".to_owned(),
            },
            license_inventory: "docs/release/license-inventory.json".to_owned(),
            provenance_manifest: "docs/release/provenance-manifest.json".to_owned(),
            artifacts: artifact_kinds
                .iter()
                .map(|kind| ReleaseArtifact {
                    kind: *kind,
                    path: format!("{kind}.bin"),
                    offset: "Unavailable".to_owned(),
                    sha256: "0".repeat(64),
                })
                .collect(),
        };

        // Assert
        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.default_flash_image, "bitaxe-ultra205.elf");
        assert!(manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::FirmwareOtaImage));
        assert!(manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::WwwSpiffsImage));
        assert!(manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::FactoryMergedImage));
    }
}
