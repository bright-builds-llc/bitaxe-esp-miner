use super::*;
use crate::package_manifest::{sha256_file, tool_versions};
use camino::Utf8Path;
use tempfile::{tempdir, TempDir};

const BUILD_STATUS: &str = "BUILD_USER local\nSTABLE_BITAXE_SOURCE_COMMIT 0123456789abcdef0123456789abcdef01234567\nSTABLE_BITAXE_SOURCE_DIRTY true\nSTABLE_BITAXE_RELEASE_TAG unavailable\nSTABLE_BITAXE_SEMANTIC_VERSION 0.1.0\nSTABLE_BITAXE_REFERENCE_COMMIT abcdef0123456789abcdef0123456789abcdef01\n";
const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const APP_ELF_SHA256: &str = "780d84b20d7ae7e6292919399348bdbf96025270136198083fc8a4da398b5ca9";

#[test]
fn materialize_build_provenance_writes_canonical_stamp_and_sdkconfig() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let status_file = temp_path(&dir, "stable-status.txt");
    let volatile_status_file = temp_path(&dir, "volatile-status.txt");
    let stamp_out = temp_path(&dir, "build-provenance.stamp");
    let sdkconfig_defaults_out = temp_path(&dir, "build-identity.defaults");
    let build_timestamp_out = temp_path(&dir, "build-timestamp-utc.txt");
    write_fixture(&status_file, BUILD_STATUS.as_bytes());
    write_fixture(
        &volatile_status_file,
        b"BUILD_TIMESTAMP 1785057566\nFORMATTED_DATE 2026 Jul 26 06 39 26 Sun\n",
    );
    let args = MaterializeBuildProvenanceArgs {
        status_file,
        volatile_status_file,
        stamp_out: stamp_out.clone(),
        sdkconfig_defaults_out: sdkconfig_defaults_out.clone(),
        build_timestamp_out: build_timestamp_out.clone(),
    };

    // Act
    materialize_build_provenance(&args).expect("materialize provenance");

    // Assert
    let stamp = fs::read_to_string(stamp_out).expect("read stamp");
    let provenance = BuildProvenance::parse_stamp(&stamp).expect("parse stamp");
    assert_eq!(
        provenance.build_identity().build_label(),
        "0123456789ab-dirty-dev"
    );
    assert_eq!(provenance.semantic_version(), "0.1.0");
    assert_eq!(
        provenance.reference_commit(),
        "abcdef0123456789abcdef0123456789abcdef01"
    );
    assert_eq!(
            fs::read_to_string(sdkconfig_defaults_out).expect("read defaults"),
            "CONFIG_APP_PROJECT_VER_FROM_CONFIG=y\nCONFIG_APP_PROJECT_VER=\"0123456789ab-dirty-dev\"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n"
        );
    assert_eq!(
        fs::read_to_string(build_timestamp_out).expect("read build timestamp"),
        "2026-07-26T06:39:26Z\n"
    );
}

#[test]
fn build_timestamp_rejects_missing_duplicate_and_malformed_formatted_dates() {
    // Arrange
    let missing = "BUILD_TIMESTAMP 1785057566\n";
    let duplicate =
        "FORMATTED_DATE 2026 Jul 26 06 39 26 Sun\nFORMATTED_DATE 2026 Jul 26 06 39 27 Sun\n";
    let malformed = "FORMATTED_DATE 2025 Feb 29 06 39 26 Sat\n";

    // Act / Assert
    assert!(build_timestamp_utc(missing).is_err());
    assert!(build_timestamp_utc(duplicate).is_err());
    assert!(build_timestamp_utc(malformed).is_err());
}

#[test]
fn build_timestamp_accepts_a_leap_day() {
    // Arrange
    let volatile_status = "FORMATTED_DATE 2024 Feb 29 23 59 58 Thu\n";

    // Act
    let timestamp = build_timestamp_utc(volatile_status).expect("valid leap-day timestamp");

    // Assert
    assert_eq!(timestamp, "2024-02-29T23:59:58Z");
}

#[test]
fn workspace_status_rejects_unknown_duplicate_and_missing_bitaxe_keys() {
    // Arrange
    let unknown = format!("{BUILD_STATUS}STABLE_BITAXE_BRANCH main\n");
    let duplicate = format!("{BUILD_STATUS}STABLE_BITAXE_SOURCE_DIRTY false\n");
    let missing = BUILD_STATUS.replace("STABLE_BITAXE_SEMANTIC_VERSION 0.1.0\n", "");

    // Act / Assert
    assert!(BuildProvenance::parse_workspace_status(&unknown).is_err());
    assert!(BuildProvenance::parse_workspace_status(&duplicate).is_err());
    assert!(BuildProvenance::parse_workspace_status(&missing).is_err());
}

#[test]
fn manifest_serializes_ultra205_default_elf_and_release_artifacts() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
    let factory_image = temp_path(&dir, FACTORY_IMAGE_NAME);
    write_fixture(&package_elf, b"elf");

    let request = package_request(&dir, package_elf, Some(factory_image));
    write_factory_fixture(&request);
    let environment = FakePackageEnvironment::clean();

    // Act
    let manifest = build_manifest(&request, &environment).expect("manifest");

    // Assert
    assert_eq!(manifest.schema_version, 3);
    assert_eq!(manifest.image_metadata.board, "205");
    assert_eq!(manifest.image_metadata.device_model, "Ultra 205");
    assert_eq!(manifest.image_metadata.asic, "BM1366");
    assert_eq!(manifest.reference_commit, EXPECTED_REFERENCE_COMMIT);
    assert_eq!(manifest.default_flash_image, DEFAULT_ELF_NAME);
    assert!(manifest
        .artifacts
        .iter()
        .any(
            |artifact| artifact.kind.to_string() == "factory_merged_image"
                && artifact.path == FACTORY_IMAGE_NAME
                && artifact.offset == "0x0"
        ));
    assert!(manifest
        .artifacts
        .iter()
        .any(|artifact| artifact.kind.to_string() == "firmware_ota_image"
            && artifact.path == "esp-miner.bin"
            && artifact.offset == "0x10000"));
    assert!(manifest
        .artifacts
        .iter()
        .any(|artifact| artifact.kind.to_string() == "www_spiffs_image"
            && artifact.path == "www.bin"
            && artifact.offset == "0x410000"));
}

#[test]
fn rejects_factory_bin_as_default_flash_image() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
    let factory_image = temp_path(&dir, FACTORY_IMAGE_NAME);
    write_fixture(&package_elf, b"elf");
    write_fixture(&factory_image, b"factory");
    let request = package_request(&dir, factory_image, None);

    // Act
    let result = validate_default_flash_image(&request.default_flash_image);

    // Assert
    assert!(result.is_err());
}

#[test]
fn checksum_uses_sha256_for_existing_artifact() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let artifact = temp_path(&dir, DEFAULT_ELF_NAME);
    write_fixture(&artifact, b"abc");

    // Act
    let checksum = sha256_file(&artifact).expect("checksum");

    // Assert
    assert_eq!(
        checksum,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn unavailable_non_critical_tool_versions_are_literal_unavailable() {
    // Arrange
    let environment = FakePackageEnvironment::with_unavailable_tools();

    // Act
    let versions = tool_versions(&environment);

    // Assert
    assert_eq!(versions.cargo, UNAVAILABLE);
    assert_eq!(versions.rustc, UNAVAILABLE);
    assert_eq!(versions.bazel, UNAVAILABLE);
    assert_eq!(versions.espflash, UNAVAILABLE);
}

#[test]
fn rejects_deferred_gamma_601_board() {
    // Arrange
    let input = "601";

    // Act
    let result = input.parse::<BoardId>();

    // Assert
    let error = result.expect_err("deferred board");
    assert!(error.contains("deferred"));
}

#[test]
fn accepts_ultra_205_board() {
    // Arrange
    let input = "205";

    // Act
    let result = input.parse::<BoardId>();

    // Assert
    assert_eq!(result.expect("board"), BoardId::Ultra205);
}

#[test]
fn package_manifest_rejects_factory_image_without_static_payloads() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
    let factory_image = temp_path(&dir, FACTORY_IMAGE_NAME);
    write_fixture(&package_elf, b"elf");
    write_fixture(&factory_image, b"factory");
    let request = package_request(&dir, package_elf, Some(factory_image));

    // Act
    let result = validate_package_request(&request);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("factory image"));
    assert!(error.contains("www.bin"));
}

#[test]
fn missing_reference_guard_failure_blocks_manifest() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
    write_fixture(&package_elf, b"elf");
    let request = package_request(&dir, package_elf, None);
    let environment = FakePackageEnvironment::guard_error("reference missing or not initialized");

    // Act
    let result = build_manifest(&request, &environment);

    // Assert
    assert!(format!("{result:#?}").contains("reference missing or not initialized"));
}

#[test]
fn dirty_reference_guard_failure_blocks_manifest() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
    write_fixture(&package_elf, b"elf");
    let request = package_request(&dir, package_elf, None);
    let environment = FakePackageEnvironment::guard_error("reference dirty");

    // Act
    let result = build_manifest(&request, &environment);

    // Assert
    assert!(format!("{result:#?}").contains("reference dirty"));
}

#[test]
fn validate_package_accepts_required_manifest_and_partition_table() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_validate_manifest(&dir, true, true, "0x0");
    let partition_table = write_valid_partition_table(&dir);
    let args = ValidatePackageArgs {
        manifest,
        partition_table,
    };

    // Act
    let result = run_validate_package(&args);

    // Assert
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn validate_package_rejects_missing_www_spiffs_image() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_validate_manifest(&dir, false, true, "0x0");
    let partition_table = write_valid_partition_table(&dir);
    let args = ValidatePackageArgs {
        manifest,
        partition_table,
    };

    // Act
    let result = run_validate_package(&args);

    // Assert
    assert!(format!("{result:#?}").contains("www_spiffs_image"));
}

#[test]
fn validate_package_rejects_missing_install_notes() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_validate_manifest(&dir, true, false, "0x0");
    let partition_table = write_valid_partition_table(&dir);
    let args = ValidatePackageArgs {
        manifest,
        partition_table,
    };

    // Act
    let result = run_validate_package(&args);

    // Assert
    assert!(format!("{result:#?}").contains("install_notes"));
}

#[test]
fn validate_package_rejects_factory_image_without_zero_offset() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_validate_manifest(&dir, true, true, "0x10000");
    let partition_table = write_valid_partition_table(&dir);
    let args = ValidatePackageArgs {
        manifest,
        partition_table,
    };

    // Act
    let result = run_validate_package(&args);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("factory"));
    assert!(error.contains("0x0"));
}

fn temp_path(dir: &TempDir, file_name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().join(file_name)).expect("utf8 path")
}

fn temp_dir_path(dir: &TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
}

fn write_fixture(path: &Utf8Path, bytes: &[u8]) {
    std::fs::write(path.as_std_path(), bytes).expect("write fixture");
}

fn write_factory_fixture(request: &PackageRequest) {
    let factory_image = request.factory_image.clone().expect("factory image");
    let www_bin = std::fs::read(request.www_bin.as_std_path()).expect("read www.bin");
    let otadata_initial =
        std::fs::read(request.otadata_initial.as_std_path()).expect("read otadata");
    let www_end = WWW_IMAGE_OFFSET + www_bin.len();
    let otadata_end = OTADATA_IMAGE_OFFSET + otadata_initial.len();
    let mut factory = b"factory".to_vec();
    factory.resize(www_end.max(otadata_end), 0xff);
    factory[WWW_IMAGE_OFFSET..www_end].copy_from_slice(&www_bin);
    factory[OTADATA_IMAGE_OFFSET..otadata_end].copy_from_slice(&otadata_initial);
    std::fs::write(factory_image.as_std_path(), factory).expect("write factory");
}

fn write_validate_manifest(
    dir: &TempDir,
    include_www_spiffs_image: bool,
    include_install_notes: bool,
    factory_offset: &str,
) -> Utf8PathBuf {
    let mut artifacts = vec![
        artifact_json("firmware_elf", "bitaxe-ultra205.elf", "Unavailable"),
        artifact_json("firmware_ota_image", "bitaxe-ultra205.bin", "0x10000"),
        artifact_json(
            "factory_merged_image",
            "bitaxe-ultra205-factory.bin",
            factory_offset,
        ),
        artifact_json("partition_table", "partition-table.bin", "0x8000"),
        artifact_json("otadata_initial", "ota_data_initial.bin", "0xf10000"),
    ];
    if include_www_spiffs_image {
        artifacts.push(artifact_json("www_spiffs_image", "www.bin", "0x410000"));
    }

    let mut manifest = serde_json::json!({
        "schema_version": 3,
        "release_name": "bitaxe-ultra205-v1",
        "semantic_version": "0.1.0",
        "source_commit": SOURCE_COMMIT,
        "reference_commit": EXPECTED_REFERENCE_COMMIT,
        "app_elf_sha256": APP_ELF_SHA256,
        "build_identity": {
            "label": "0123456789ab-dev",
            "channel": "dev",
            "source_dirty": false,
            "release_tag": null
        },
        "default_flash_image": "bitaxe-ultra205.elf",
        "image_metadata": {
            "board": "205",
            "device_model": "Ultra 205",
            "asic": "BM1366",
            "esp_idf_version": "v5.5.4",
            "rust_target": "xtensa-esp32s3-espidf"
        },
        "tool_versions": {
            "cargo": "cargo 1.0.0",
            "rustc": "rustc 1.0.0",
            "bazel": "bazel 1.0.0",
            "espflash": "espflash 1.0.0"
        },
        "license_inventory": "docs/release/license-inventory.json",
        "provenance_manifest": "docs/release/provenance-manifest.json",
        "otadata_source": "generated-erased-flash",
        "artifacts": artifacts
    });
    if include_install_notes {
        manifest["install_notes"] = serde_json::json!({
            "path": "docs/release/ultra-205.md",
            "summary": "Flash with just flash board=205"
        });
    }

    let path = temp_path(dir, "manifest-v3.json");
    std::fs::write(
        path.as_std_path(),
        serde_json::to_string_pretty(&manifest).expect("manifest json"),
    )
    .expect("write manifest");
    path
}

fn artifact_json(kind: &str, path: &str, offset: &str) -> serde_json::Value {
    let sha256 = if kind == "firmware_elf" {
        APP_ELF_SHA256
    } else {
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    };
    serde_json::json!({
        "kind": kind,
        "path": path,
        "offset": offset,
        "sha256": sha256
    })
}

fn write_valid_partition_table(dir: &TempDir) -> Utf8PathBuf {
    let path = temp_path(dir, "partitions-ultra205.csv");
    std::fs::write(
        path.as_std_path(),
        "# Name, Type, SubType, Offset, Size, Flags\n\
             nvs, data, nvs, 0x9000, 0x6000\n\
             phy_init, data, phy, 0xf000, 0x1000\n\
             factory, app, factory, 0x10000, 4M\n\
             www, data, spiffs, 0x410000, 3M\n\
             ota_0, app, ota_0, 0x710000, 4M\n\
             ota_1, app, ota_1, 0xb10000, 4M\n\
             otadata, data, ota, 0xf10000, 8k\n\
             coredump, data, coredump, , 64K\n",
    )
    .expect("write partition table");
    path
}

fn package_request(
    dir: &TempDir,
    default_flash_image: Utf8PathBuf,
    factory_image: Option<Utf8PathBuf>,
) -> PackageRequest {
    let firmware_ota_image = temp_path(dir, "esp-miner.bin");
    let www_bin = temp_path(dir, "www.bin");
    let partition_table = temp_path(dir, "partitions-ultra205.csv");
    let otadata_initial = temp_path(dir, "otadata-initial.bin");
    let install_notes = temp_path(dir, "ultra-205.md");
    let license_inventory = temp_path(dir, "license-inventory.md");
    let provenance_manifest = temp_path(dir, "provenance-manifest.md");
    let build_provenance_stamp = temp_path(dir, "build-provenance.stamp");
    let provenance = BuildProvenance::new(
        "0.1.0",
        SOURCE_COMMIT,
        false,
        None::<&str>,
        EXPECTED_REFERENCE_COMMIT,
    )
    .expect("valid provenance");
    write_fixture(
        &build_provenance_stamp,
        provenance.render_stamp().as_bytes(),
    );
    for path in [
        &firmware_ota_image,
        &www_bin,
        &partition_table,
        &otadata_initial,
        &install_notes,
        &license_inventory,
        &provenance_manifest,
    ] {
        if !path.is_file() {
            write_fixture(path, path.as_str().as_bytes());
        }
    }

    PackageRequest {
        board: BoardId::Ultra205,
        firmware_elf: temp_path(dir, DEFAULT_ELF_NAME),
        build_provenance_stamp,
        app_descriptor_version: "0123456789ab-dev".to_owned(),
        app_elf_sha256: APP_ELF_SHA256.to_owned(),
        firmware_ota_image,
        www_bin,
        partition_table,
        otadata_initial,
        default_flash_image,
        factory_image,
        manifest: temp_path(dir, "bitaxe-ultra205-package.json"),
        out_dir: temp_dir_path(dir),
        release_name: "bitaxe-ultra205".to_owned(),
        install_notes,
        license_inventory,
        provenance_manifest,
        otadata_source: "generated-erased-flash".to_owned(),
    }
}

#[derive(Debug)]
struct FakePackageEnvironment {
    maybe_guard_error: Option<String>,
    tools_available: bool,
}

impl FakePackageEnvironment {
    fn clean() -> Self {
        Self {
            maybe_guard_error: None,
            tools_available: true,
        }
    }

    fn with_unavailable_tools() -> Self {
        Self {
            maybe_guard_error: None,
            tools_available: false,
        }
    }

    fn guard_error(message: &str) -> Self {
        Self {
            maybe_guard_error: Some(message.to_owned()),
            tools_available: true,
        }
    }
}

impl PackageEnvironment for FakePackageEnvironment {
    fn run_reference_guard(&self) -> Result<()> {
        if let Some(error) = &self.maybe_guard_error {
            bail!("{error}");
        }

        Ok(())
    }

    fn maybe_tool_version(&self, tool: &str) -> Option<String> {
        if !self.tools_available {
            return None;
        }

        Some(format!("{tool} 1.0.0"))
    }
}
