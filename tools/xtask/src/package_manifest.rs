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

    let artifacts = vec![
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

    Ok(PackageManifestV3 {
        schema_version: 3,
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

pub(crate) fn read_manifest_v3(path: &Utf8Path) -> Result<PackageManifestV3> {
    let contents = fs::read_to_string(path.as_std_path())
        .with_context(|| format!("failed to read package manifest {path}"))?;
    serde_json::from_str(&contents).with_context(|| format!("failed to parse manifest v3 {path}"))
}

pub(crate) fn validate_package_manifest_v3(manifest: &PackageManifestV3) -> Result<()> {
    if manifest.schema_version != 3 {
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
    Ok(ReleaseArtifact {
        kind,
        path: manifest_relative_path(manifest_path, path),
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
mod tests {
    use super::*;
    use crate::{
        BoardId, PackageEnvironment, PackageRequest, DEFAULT_ELF_NAME, EXPECTED_REFERENCE_COMMIT,
        FACTORY_IMAGE_NAME, RUST_TARGET,
    };
    use camino::Utf8PathBuf;
    use tempfile::{tempdir, TempDir};

    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const APP_ELF_SHA256: &str = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    #[test]
    fn package_manifest_v3_requires_identity_and_release_artifact_kinds() {
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
        let manifest = PackageManifestV3 {
            schema_version: 3,
            release_name: "bitaxe-ultra205-v1".to_owned(),
            semantic_version: "0.1.0".to_owned(),
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: EXPECTED_REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: APP_ELF_SHA256.to_owned(),
            build_identity: ManifestBuildIdentity {
                label: "0123456789ab-dev".to_owned(),
                channel: "dev".to_owned(),
                source_dirty: false,
                release_tag: None,
            },
            default_flash_image: "bitaxe-ultra205.elf".to_owned(),
            image_metadata: ImageMetadata {
                board: "205".to_owned(),
                device_model: "Ultra 205".to_owned(),
                asic: "BM1366".to_owned(),
                esp_idf_version: "v5.5.4".to_owned(),
                rust_target: "xtensa-esp32s3-espidf".to_owned(),
            },
            tool_versions: ToolVersions {
                cargo: "cargo 1.0.0".to_owned(),
                rustc: "rustc 1.0.0".to_owned(),
                bazel: "bazel 1.0.0".to_owned(),
                espflash: "espflash 1.0.0".to_owned(),
            },
            install_notes: ReleaseNotes {
                path: "docs/release/ultra-205.md".to_owned(),
                summary: "Flash with just flash board=205".to_owned(),
            },
            license_inventory: "docs/release/license-inventory.json".to_owned(),
            provenance_manifest: "docs/release/provenance-manifest.json".to_owned(),
            otadata_source: "generated-erased-flash".to_owned(),
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
        assert_eq!(manifest.schema_version, 3);
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

    #[test]
    fn package_manifest_v3_builds_identity_and_release_artifacts_from_real_outputs() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let package_elf = write_fixture(&dir, DEFAULT_ELF_NAME, b"elf");
        let firmware_ota_image = write_fixture(&dir, "esp-miner.bin", b"ota");
        let www_bin = write_fixture(&dir, "www.bin", b"www");
        let otadata_initial = write_fixture(&dir, "otadata-initial.bin", b"otadata");
        let factory_image = write_factory_fixture(&dir, &www_bin, &otadata_initial);
        let partition_table = write_fixture(&dir, "partitions-ultra205.csv", b"partition-csv");
        let install_notes = write_fixture(&dir, "ultra-205.md", b"install");
        let license_inventory = write_fixture(&dir, "license-inventory.md", b"license");
        let provenance_manifest = write_fixture(&dir, "provenance-manifest.md", b"provenance");
        let build_provenance_stamp = write_provenance_stamp(&dir);
        let request = PackageRequest {
            board: BoardId::Ultra205,
            firmware_elf: package_elf.clone(),
            build_provenance_stamp,
            app_descriptor_version: "0123456789ab-dev".to_owned(),
            app_elf_sha256: APP_ELF_SHA256.to_owned(),
            firmware_ota_image: firmware_ota_image.clone(),
            www_bin: www_bin.clone(),
            partition_table: partition_table.clone(),
            otadata_initial: otadata_initial.clone(),
            default_flash_image: package_elf,
            out_dir: dir_path(&dir),
            manifest: dir_path(&dir).join("bitaxe-ultra205-package.json"),
            factory_image: Some(factory_image),
            release_name: "bitaxe-ultra205".to_owned(),
            install_notes,
            license_inventory,
            provenance_manifest,
            otadata_source: "generated-erased-flash".to_owned(),
        };
        let environment = FakePackageEnvironment;

        // Act
        let manifest = build_manifest(&request, &environment).expect("manifest");

        // Assert
        assert_eq!(manifest.schema_version, 3);
        assert_eq!(manifest.release_name, "bitaxe-ultra205");
        assert_eq!(manifest.default_flash_image, DEFAULT_ELF_NAME);
        assert_eq!(manifest.semantic_version, "0.1.0");
        assert_eq!(manifest.source_commit, SOURCE_COMMIT);
        assert_eq!(manifest.reference_commit, EXPECTED_REFERENCE_COMMIT);
        assert_eq!(manifest.app_elf_sha256, APP_ELF_SHA256);
        assert_eq!(manifest.build_identity.label, "0123456789ab-dev");
        assert_eq!(manifest.build_identity.channel, "dev");
        assert!(!manifest.build_identity.source_dirty);
        assert_eq!(manifest.build_identity.release_tag, None);
        assert_eq!(
            manifest.image_metadata.esp_idf_version,
            crate::ESP_IDF_VERSION
        );
        assert_eq!(manifest.image_metadata.rust_target, RUST_TARGET);
        assert_eq!(manifest.install_notes.path, "ultra-205.md");
        assert_eq!(manifest.license_inventory, "license-inventory.md");
        assert_eq!(manifest.provenance_manifest, "provenance-manifest.md");
        assert_eq!(manifest.otadata_source, "generated-erased-flash");
        assert_artifact(
            &manifest,
            ArtifactKind::FirmwareElf,
            DEFAULT_ELF_NAME,
            UNAVAILABLE,
        );
        assert_artifact(
            &manifest,
            ArtifactKind::FirmwareOtaImage,
            "esp-miner.bin",
            "0x10000",
        );
        assert_artifact(
            &manifest,
            ArtifactKind::WwwSpiffsImage,
            "www.bin",
            "0x410000",
        );
        assert_artifact(
            &manifest,
            ArtifactKind::FactoryMergedImage,
            FACTORY_IMAGE_NAME,
            "0x0",
        );
        assert_artifact(
            &manifest,
            ArtifactKind::PartitionTable,
            "partitions-ultra205.csv",
            UNAVAILABLE,
        );
        assert_artifact(
            &manifest,
            ArtifactKind::OtadataInitial,
            "otadata-initial.bin",
            "0xf10000",
        );
    }

    #[test]
    fn package_manifest_v3_rejects_duplicate_ota_artifact() {
        // Arrange
        let mut manifest = valid_manifest_v3();
        let duplicate = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::FirmwareOtaImage)
            .expect("ota artifact")
            .clone();
        manifest.artifacts.push(duplicate);

        // Act
        let result = validate_package_manifest_v3(&manifest);

        // Assert
        assert_eq!(
            result.expect_err("duplicate OTA must fail").to_string(),
            "required artifact kind firmware_ota_image duplicate"
        );
    }

    #[test]
    fn package_manifest_v3_rejects_duplicate_factory_artifact() {
        // Arrange
        let mut manifest = valid_manifest_v3();
        let duplicate = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::FactoryMergedImage)
            .expect("factory artifact")
            .clone();
        manifest.artifacts.push(duplicate);

        // Act
        let result = validate_package_manifest_v3(&manifest);

        // Assert
        assert_eq!(
            result.expect_err("duplicate factory must fail").to_string(),
            "required artifact kind factory_merged_image duplicate"
        );
    }

    #[test]
    fn package_manifest_v3_distinguishes_missing_and_duplicate_artifacts() {
        // Arrange
        let mut missing_manifest = valid_manifest_v3();
        missing_manifest
            .artifacts
            .retain(|artifact| artifact.kind != ArtifactKind::FirmwareOtaImage);
        let mut duplicate_manifest = valid_manifest_v3();
        let duplicate = duplicate_manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::FirmwareOtaImage)
            .expect("ota artifact")
            .clone();
        duplicate_manifest.artifacts.push(duplicate);

        // Act
        let missing = validate_package_manifest_v3(&missing_manifest);
        let duplicate = validate_package_manifest_v3(&duplicate_manifest);

        // Assert
        assert_eq!(
            missing.expect_err("missing OTA must fail").to_string(),
            "required artifact kind firmware_ota_image missing"
        );
        assert_eq!(
            duplicate.expect_err("duplicate OTA must fail").to_string(),
            "required artifact kind firmware_ota_image duplicate"
        );
    }

    fn valid_manifest_v3() -> PackageManifestV3 {
        let artifact = |kind, offset: &str| ReleaseArtifact {
            kind,
            path: format!("{kind}.bin"),
            offset: offset.to_owned(),
            sha256: "0".repeat(64),
        };
        PackageManifestV3 {
            schema_version: 3,
            release_name: "bitaxe-ultra205-v1".to_owned(),
            semantic_version: "0.1.0".to_owned(),
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: EXPECTED_REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: APP_ELF_SHA256.to_owned(),
            build_identity: ManifestBuildIdentity {
                label: "0123456789ab-dev".to_owned(),
                channel: "dev".to_owned(),
                source_dirty: false,
                release_tag: None,
            },
            default_flash_image: DEFAULT_ELF_NAME.to_owned(),
            image_metadata: ImageMetadata {
                board: "205".to_owned(),
                device_model: "Ultra 205".to_owned(),
                asic: "BM1366".to_owned(),
                esp_idf_version: "v5.5.4".to_owned(),
                rust_target: RUST_TARGET.to_owned(),
            },
            tool_versions: ToolVersions {
                cargo: "cargo 1.0.0".to_owned(),
                rustc: "rustc 1.0.0".to_owned(),
                bazel: "bazel 1.0.0".to_owned(),
                espflash: "espflash 1.0.0".to_owned(),
            },
            install_notes: ReleaseNotes {
                path: "ultra-205.md".to_owned(),
                summary: "install".to_owned(),
            },
            license_inventory: "license-inventory.md".to_owned(),
            provenance_manifest: "provenance-manifest.md".to_owned(),
            otadata_source: "generated-erased-flash".to_owned(),
            artifacts: vec![
                artifact(ArtifactKind::FirmwareElf, UNAVAILABLE),
                artifact(ArtifactKind::FirmwareOtaImage, "0x10000"),
                artifact(ArtifactKind::WwwSpiffsImage, "0x410000"),
                artifact(ArtifactKind::FactoryMergedImage, "0x0"),
                artifact(ArtifactKind::PartitionTable, "0x8000"),
                artifact(ArtifactKind::OtadataInitial, "0xf10000"),
            ],
        }
    }

    fn write_fixture(dir: &TempDir, file_name: &str, contents: &[u8]) -> Utf8PathBuf {
        let path = dir_path(dir).join(file_name);
        std::fs::write(path.as_std_path(), contents).expect("write fixture");
        path
    }

    fn write_factory_fixture(
        dir: &TempDir,
        www_bin: &Utf8Path,
        otadata_initial: &Utf8Path,
    ) -> Utf8PathBuf {
        let www = std::fs::read(www_bin.as_std_path()).expect("read www");
        let otadata = std::fs::read(otadata_initial.as_std_path()).expect("read otadata");
        let mut factory = b"factory".to_vec();
        let www_end = crate::WWW_IMAGE_OFFSET + www.len();
        let otadata_end = crate::OTADATA_IMAGE_OFFSET + otadata.len();
        factory.resize(otadata_end.max(www_end), 0xff);
        factory[crate::WWW_IMAGE_OFFSET..www_end].copy_from_slice(&www);
        factory[crate::OTADATA_IMAGE_OFFSET..otadata_end].copy_from_slice(&otadata);
        write_fixture(dir, FACTORY_IMAGE_NAME, &factory)
    }

    fn dir_path(dir: &TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
    }

    fn write_provenance_stamp(dir: &TempDir) -> Utf8PathBuf {
        let path = dir_path(dir).join("build-provenance.stamp");
        let provenance = BuildProvenance::new(
            "0.1.0",
            SOURCE_COMMIT,
            false,
            None::<&str>,
            EXPECTED_REFERENCE_COMMIT,
        )
        .expect("valid provenance");
        std::fs::write(path.as_std_path(), provenance.render_stamp()).expect("write stamp");
        path
    }

    fn assert_artifact(manifest: &PackageManifestV3, kind: ArtifactKind, path: &str, offset: &str) {
        let artifact = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == kind)
            .expect("artifact kind");
        assert_eq!(artifact.path, path);
        assert_eq!(artifact.offset, offset);
        assert_eq!(artifact.sha256.len(), 64);
    }

    #[derive(Debug)]
    struct FakePackageEnvironment;

    impl PackageEnvironment for FakePackageEnvironment {
        fn run_reference_guard(&self) -> Result<()> {
            Ok(())
        }

        fn maybe_tool_version(&self, tool: &str) -> Option<String> {
            Some(format!("{tool} 1.0.0"))
        }
    }
}
