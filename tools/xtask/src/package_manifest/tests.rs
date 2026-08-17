use super::*;
use crate::{
    BoardId, PackageEnvironment, PackageRequest, DEFAULT_ELF_NAME, EXPECTED_REFERENCE_COMMIT,
    FACTORY_IMAGE_NAME, RUST_TARGET,
};
use camino::Utf8PathBuf;
use tempfile::{tempdir, TempDir};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const APP_ELF_SHA256: &str = "780d84b20d7ae7e6292919399348bdbf96025270136198083fc8a4da398b5ca9";

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
    let request = package_request_fixture(&dir, APP_ELF_SHA256);
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
fn package_manifest_canonicalizes_workspace_partition_table_path() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let mut request = package_request_fixture(&dir, APP_ELF_SHA256);
    let partition_table = dir_path(&dir)
        .join("workspace")
        .join("firmware/bitaxe/partitions-ultra205.csv");
    std::fs::create_dir_all(
        partition_table
            .parent()
            .expect("partition table should have a parent")
            .as_std_path(),
    )
    .expect("create partition-table parent");
    std::fs::write(partition_table.as_std_path(), b"partition-csv")
        .expect("write partition-table fixture");
    request.partition_table = partition_table;

    // Act
    let manifest = build_manifest(&request, &FakePackageEnvironment).expect("manifest");

    // Assert
    assert_artifact(
        &manifest,
        ArtifactKind::PartitionTable,
        "firmware/bitaxe/partitions-ultra205.csv",
        UNAVAILABLE,
    );
}

#[test]
fn build_manifest_rejects_firmware_elf_app_sha_mismatch_before_output() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let request = package_request_fixture(&dir, &"a".repeat(64));
    let environment = FakePackageEnvironment;

    // Act
    let error = build_manifest(&request, &environment)
        .expect_err("producer must reject ELF relationship mismatch");

    // Assert
    assert_eq!(error.to_string(), "firmware_elf_app_sha_mismatch");
    assert!(!request.manifest.exists());
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
fn package_manifest_v3_rejects_firmware_elf_app_sha_mismatch() {
    // Arrange
    let mut manifest = valid_manifest_v3();
    let firmware_elf = manifest
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.kind == ArtifactKind::FirmwareElf)
        .expect("firmware ELF artifact");
    firmware_elf.sha256 = "a".repeat(64);

    // Act
    let error =
        validate_package_manifest_v3(&manifest).expect_err("firmware ELF must bind to app ELF SHA");

    // Assert
    assert_eq!(error.to_string(), "firmware_elf_app_sha_mismatch");
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
        sha256: if kind == ArtifactKind::FirmwareElf {
            APP_ELF_SHA256.to_owned()
        } else {
            "0".repeat(64)
        },
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

fn package_request_fixture(dir: &TempDir, app_elf_sha256: &str) -> PackageRequest {
    let package_elf = write_fixture(dir, DEFAULT_ELF_NAME, b"elf");
    let firmware_ota_image = write_fixture(dir, "esp-miner.bin", b"ota");
    let www_bin = write_fixture(dir, "www.bin", b"www");
    let otadata_initial = write_fixture(dir, "otadata-initial.bin", b"otadata");
    let factory_image = write_factory_fixture(dir, &www_bin, &otadata_initial);
    let partition_table = write_fixture(dir, "partitions-ultra205.csv", b"partition-csv");
    let install_notes = write_fixture(dir, "ultra-205.md", b"install");
    let license_inventory = write_fixture(dir, "license-inventory.md", b"license");
    let provenance_manifest = write_fixture(dir, "provenance-manifest.md", b"provenance");
    PackageRequest {
        board: BoardId::Ultra205,
        firmware_elf: package_elf.clone(),
        build_provenance_stamp: write_provenance_stamp(dir),
        app_descriptor_version: "0123456789ab-dev".to_owned(),
        app_elf_sha256: app_elf_sha256.to_owned(),
        firmware_ota_image,
        www_bin,
        partition_table,
        otadata_initial,
        default_flash_image: package_elf,
        out_dir: dir_path(dir),
        manifest: dir_path(dir).join("bitaxe-ultra205-package.json"),
        factory_image: Some(factory_image),
        release_name: "bitaxe-ultra205".to_owned(),
        install_notes,
        license_inventory,
        provenance_manifest,
        otadata_source: "generated-erased-flash".to_owned(),
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
