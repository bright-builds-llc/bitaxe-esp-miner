use std::env;
use std::fmt;
use std::fs;
use std::process::Command as ProcessCommand;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use bitaxe_api::BuildProvenance;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

mod package_manifest;
mod partition_contract;

use package_manifest::{
    build_manifest, read_manifest_v3, validate_default_flash_image, validate_package_manifest_v3,
    write_manifest,
};

const EXPECTED_REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const UNAVAILABLE: &str = "Unavailable";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_REFERENCE_GUARD: &str = "scripts/verify-reference-clean.sh";
const ESP_IDF_VERSION: &str = "v5.5.4";
const RUST_TARGET: &str = "xtensa-esp32s3-espidf";
const WWW_IMAGE_OFFSET: usize = 0x410000;
const OTADATA_IMAGE_OFFSET: usize = 0xf10000;

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Bitaxe firmware workflow glue.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    #[command(name = "materialize-build-provenance")]
    MaterializeBuildProvenance(MaterializeBuildProvenanceArgs),
    #[command(name = "package-firmware")]
    PackageFirmware(Box<PackageArgs>),
    #[command(name = "validate-package")]
    ValidatePackage(ValidatePackageArgs),
}

#[derive(Debug, Parser)]
struct MaterializeBuildProvenanceArgs {
    #[arg(long = "status-file", value_parser = parse_utf8_path)]
    status_file: Utf8PathBuf,

    #[arg(long = "stamp-out", value_parser = parse_utf8_path)]
    stamp_out: Utf8PathBuf,

    #[arg(long = "sdkconfig-defaults-out", value_parser = parse_utf8_path)]
    sdkconfig_defaults_out: Utf8PathBuf,
}

#[derive(Debug, Parser)]
struct PackageArgs {
    #[arg(long, value_parser = parse_board)]
    board: BoardId,

    #[arg(long = "firmware-elf", value_parser = parse_utf8_path)]
    firmware_elf: Utf8PathBuf,

    #[arg(long = "build-provenance-stamp", value_parser = parse_utf8_path)]
    build_provenance_stamp: Utf8PathBuf,

    #[arg(long = "app-descriptor-version")]
    app_descriptor_version: String,

    #[arg(long = "app-elf-sha256")]
    app_elf_sha256: String,

    #[arg(long = "firmware-ota-image", value_parser = parse_utf8_path)]
    firmware_ota_image: Utf8PathBuf,

    #[arg(long = "www-bin", value_parser = parse_utf8_path)]
    www_bin: Utf8PathBuf,

    #[arg(long = "partition-table", value_parser = parse_utf8_path)]
    partition_table: Utf8PathBuf,

    #[arg(long = "otadata-initial", value_parser = parse_utf8_path)]
    otadata_initial: Utf8PathBuf,

    #[arg(long = "default-flash-image", value_parser = parse_utf8_path)]
    default_flash_image: Utf8PathBuf,

    #[arg(long = "out-dir", value_parser = parse_utf8_path)]
    out_dir: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "factory-image", value_parser = parse_utf8_path)]
    factory_image: Option<Utf8PathBuf>,

    #[arg(long = "release-name")]
    release_name: String,

    #[arg(long = "install-notes", value_parser = parse_utf8_path)]
    install_notes: Utf8PathBuf,

    #[arg(long = "license-inventory", value_parser = parse_utf8_path)]
    license_inventory: Utf8PathBuf,

    #[arg(long = "provenance-manifest", value_parser = parse_utf8_path)]
    provenance_manifest: Utf8PathBuf,

    #[arg(long = "otadata-source", default_value = UNAVAILABLE)]
    otadata_source: String,
}

#[derive(Debug, Parser)]
struct ValidatePackageArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "partition-table", value_parser = parse_utf8_path)]
    partition_table: Utf8PathBuf,
}

#[derive(Debug, Clone)]
struct PackageRequest {
    board: BoardId,
    firmware_elf: Utf8PathBuf,
    build_provenance_stamp: Utf8PathBuf,
    app_descriptor_version: String,
    app_elf_sha256: String,
    firmware_ota_image: Utf8PathBuf,
    www_bin: Utf8PathBuf,
    partition_table: Utf8PathBuf,
    otadata_initial: Utf8PathBuf,
    default_flash_image: Utf8PathBuf,
    out_dir: Utf8PathBuf,
    manifest: Utf8PathBuf,
    factory_image: Option<Utf8PathBuf>,
    release_name: String,
    install_notes: Utf8PathBuf,
    license_inventory: Utf8PathBuf,
    provenance_manifest: Utf8PathBuf,
    otadata_source: String,
}

impl From<PackageArgs> for PackageRequest {
    fn from(args: PackageArgs) -> Self {
        Self {
            board: args.board,
            firmware_elf: args.firmware_elf,
            build_provenance_stamp: args.build_provenance_stamp,
            app_descriptor_version: args.app_descriptor_version,
            app_elf_sha256: args.app_elf_sha256,
            firmware_ota_image: args.firmware_ota_image,
            www_bin: args.www_bin,
            partition_table: args.partition_table,
            otadata_initial: args.otadata_initial,
            default_flash_image: args.default_flash_image,
            out_dir: args.out_dir,
            manifest: args.manifest,
            factory_image: args.factory_image,
            release_name: args.release_name,
            install_notes: args.install_notes,
            license_inventory: args.license_inventory,
            provenance_manifest: args.provenance_manifest,
            otadata_source: args.otadata_source,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoardId {
    Ultra205,
}

impl FromStr for BoardId {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "205" => Ok(Self::Ultra205),
            "601" => Err(
                "board 601 is deferred after the Ultra 205 pivot; Phase 1 supports board=205 only"
                    .to_owned(),
            ),
            other => Err(format!(
                "unsupported board {other}; Phase 1 supports board=205 only"
            )),
        }
    }
}

impl fmt::Display for BoardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ultra205 => formatter.write_str("205"),
        }
    }
}

trait PackageEnvironment {
    fn run_reference_guard(&self) -> Result<()>;
    fn maybe_tool_version(&self, tool: &str) -> Option<String>;
}

#[derive(Debug)]
struct LocalPackageEnvironment {
    workspace_dir: Utf8PathBuf,
    reference_guard: Utf8PathBuf,
}

impl LocalPackageEnvironment {
    fn detect() -> Result<Self> {
        let workspace_dir = detect_workspace_dir()?;
        let reference_guard = workspace_dir.join(DEFAULT_REFERENCE_GUARD);

        Ok(Self {
            workspace_dir,
            reference_guard,
        })
    }
}

impl PackageEnvironment for LocalPackageEnvironment {
    fn run_reference_guard(&self) -> Result<()> {
        let output = ProcessCommand::new(self.reference_guard.as_std_path())
            .env("BUILD_WORKSPACE_DIRECTORY", self.workspace_dir.as_str())
            .output()
            .with_context(|| format!("failed to run reference guard {}", self.reference_guard))?;

        if output.status.success() {
            return Ok(());
        }

        bail!(
            "reference guard blocked package manifest generation: {}",
            command_stderr_or_status(&output)
        );
    }

    fn maybe_tool_version(&self, tool: &str) -> Option<String> {
        let output = ProcessCommand::new(tool).arg("--version").output().ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8(output.stdout).ok()?;
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(trimmed.to_owned())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommand::MaterializeBuildProvenance(args) => {
            materialize_build_provenance(&args)?;
        }
        CliCommand::PackageFirmware(args) => {
            let environment = LocalPackageEnvironment::detect()?;
            let request = PackageRequest::from(*args);
            run_package_firmware(&request, &environment)?;
        }
        CliCommand::ValidatePackage(args) => {
            run_validate_package(&args)?;
        }
    }

    Ok(())
}

fn materialize_build_provenance(args: &MaterializeBuildProvenanceArgs) -> Result<()> {
    let status = fs::read_to_string(args.status_file.as_std_path())
        .with_context(|| format!("failed to read workspace status {}", args.status_file))?;
    let provenance = BuildProvenance::parse_workspace_status(&status)
        .context("invalid Bitaxe build provenance")?;

    write_parented_file(&args.stamp_out, &provenance.render_stamp())?;
    write_parented_file(
        &args.sdkconfig_defaults_out,
        &format!(
            "CONFIG_APP_PROJECT_VER_FROM_CONFIG=y\nCONFIG_APP_PROJECT_VER=\"{}\"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n",
            provenance.build_identity().build_label()
        ),
    )
}

fn write_parented_file(path: &Utf8PathBuf, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create output directory {parent}"))?;
    }
    fs::write(path.as_std_path(), contents)
        .with_context(|| format!("failed to write output {path}"))
}

fn run_package_firmware(
    package_request: &PackageRequest,
    environment: &impl PackageEnvironment,
) -> Result<()> {
    partition_contract::validate_ultra205_partition_contract(&package_request.partition_table)?;

    let manifest = build_manifest(package_request, environment)?;
    fs::create_dir_all(package_request.out_dir.as_std_path()).with_context(|| {
        format!(
            "failed to create output directory {}",
            package_request.out_dir
        )
    })?;
    validate_package_manifest_v3(&manifest)?;
    write_manifest(&package_request.manifest, &manifest)?;
    run_validate_package(&ValidatePackageArgs {
        manifest: package_request.manifest.clone(),
        partition_table: package_request.partition_table.clone(),
    })
}

fn run_validate_package(args: &ValidatePackageArgs) -> Result<()> {
    let manifest = read_manifest_v3(&args.manifest)?;
    validate_package_manifest_v3(&manifest)?;
    partition_contract::validate_ultra205_partition_contract(&args.partition_table)
}

fn validate_package_request(package_request: &PackageRequest) -> Result<()> {
    if package_request.board != BoardId::Ultra205 {
        bail!(
            "unsupported board {}; Phase 1 supports board=205 only",
            package_request.board
        );
    }

    if !package_request.firmware_elf.is_file() {
        bail!(
            "firmware ELF does not exist: {}",
            package_request.firmware_elf
        );
    }

    if !package_request.build_provenance_stamp.is_file() {
        bail!(
            "build provenance stamp does not exist: {}",
            package_request.build_provenance_stamp
        );
    }

    if !package_request.firmware_ota_image.is_file() {
        bail!(
            "firmware OTA image does not exist: {}",
            package_request.firmware_ota_image
        );
    }

    if !package_request.www_bin.is_file() {
        bail!("www.bin does not exist: {}", package_request.www_bin);
    }

    if !package_request.partition_table.is_file() {
        bail!(
            "partition table does not exist: {}",
            package_request.partition_table
        );
    }

    if !package_request.otadata_initial.is_file() {
        bail!(
            "otadata initial image does not exist: {}",
            package_request.otadata_initial
        );
    }

    validate_default_flash_image(&package_request.default_flash_image)?;
    if !package_request.default_flash_image.is_file() {
        bail!(
            "default flash image does not exist: {}",
            package_request.default_flash_image
        );
    }

    if let Some(factory_image) = &package_request.factory_image {
        if !factory_image.is_file() {
            bail!("factory image does not exist: {factory_image}");
        }
        validate_factory_payload(
            factory_image,
            &package_request.www_bin,
            WWW_IMAGE_OFFSET,
            "www.bin",
        )?;
        validate_factory_payload(
            factory_image,
            &package_request.otadata_initial,
            OTADATA_IMAGE_OFFSET,
            "otadata-initial.bin",
        )?;
    } else {
        bail!("factory image is required for package manifest v3");
    }

    if package_request.release_name.trim().is_empty() {
        bail!("release name must not be empty");
    }

    if !package_request.install_notes.is_file() {
        bail!(
            "install notes do not exist: {}",
            package_request.install_notes
        );
    }

    if !package_request.license_inventory.is_file() {
        bail!(
            "license inventory does not exist: {}",
            package_request.license_inventory
        );
    }

    if !package_request.provenance_manifest.is_file() {
        bail!(
            "provenance manifest does not exist: {}",
            package_request.provenance_manifest
        );
    }

    Ok(())
}

fn validate_factory_payload(
    factory_image: &Utf8PathBuf,
    payload_path: &Utf8PathBuf,
    offset: usize,
    label: &str,
) -> Result<()> {
    let factory_bytes = fs::read(factory_image.as_std_path())
        .with_context(|| format!("failed to read factory image {factory_image}"))?;
    let payload_bytes = fs::read(payload_path.as_std_path())
        .with_context(|| format!("failed to read {label} payload {payload_path}"))?;
    let end = offset
        .checked_add(payload_bytes.len())
        .with_context(|| format!("{label} offset overflow"))?;
    if factory_bytes.len() < end {
        bail!(
            "factory image {factory_image} is too small to contain {label} at offset 0x{offset:x}"
        );
    }
    if factory_bytes[offset..end] != payload_bytes {
        bail!("factory image {factory_image} does not contain {label} at offset 0x{offset:x}");
    }

    Ok(())
}

fn parse_board(value: &str) -> std::result::Result<BoardId, String> {
    value.parse()
}

fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    Ok(Utf8PathBuf::from(value))
}

fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = ProcessCommand::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .context("failed to detect workspace directory with git rev-parse --show-toplevel")?;

    if !output.status.success() {
        bail!(
            "failed to detect workspace directory: {}",
            command_stderr_or_status(&output)
        );
    }

    let workspace_dir = String::from_utf8(output.stdout)
        .context("workspace directory output was not valid UTF-8")?;
    let trimmed = workspace_dir.trim();
    if trimmed.is_empty() {
        bail!("workspace directory output was empty");
    }

    Ok(Utf8PathBuf::from(trimmed))
}

fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed_stderr = stderr.trim();
    if !trimmed_stderr.is_empty() {
        return trimmed_stderr.to_owned();
    }

    format!("exit status {}", output.status)
}

#[cfg(test)]
mod tests {
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
        let stamp_out = temp_path(&dir, "build-provenance.stamp");
        let sdkconfig_defaults_out = temp_path(&dir, "build-identity.defaults");
        write_fixture(&status_file, BUILD_STATUS.as_bytes());
        let args = MaterializeBuildProvenanceArgs {
            status_file,
            stamp_out: stamp_out.clone(),
            sdkconfig_defaults_out: sdkconfig_defaults_out.clone(),
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
        let environment =
            FakePackageEnvironment::guard_error("reference missing or not initialized");

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
}
