use std::env;
use std::fmt;
use std::fs;
use std::process::Command as ProcessCommand;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

mod package_manifest;
mod partition_contract;

use package_manifest::{
    build_manifest, read_manifest_v2, validate_default_flash_image, validate_package_manifest_v2,
    write_manifest,
};

const EXPECTED_REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const UNAVAILABLE: &str = "Unavailable";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_REFERENCE_GUARD: &str = "scripts/verify-reference-clean.sh";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
const DEFAULT_ULTRA205_PARTITION_TABLE: &str = "firmware/bitaxe/partitions-ultra205.csv";
const ESP_IDF_VERSION: &str = "v5.5.4";
const RUST_TARGET: &str = "xtensa-esp32s3-espidf";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Bitaxe firmware workflow glue.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    #[command(name = "package-firmware")]
    PackageFirmware(PackageArgs),
    #[command(name = "validate-package")]
    ValidatePackage(ValidatePackageArgs),
}

#[derive(Debug, Parser)]
struct PackageArgs {
    #[arg(long, value_parser = parse_board)]
    board: BoardId,

    #[arg(long = "firmware-elf", value_parser = parse_utf8_path)]
    firmware_elf: Utf8PathBuf,

    #[arg(long = "default-flash-image", value_parser = parse_utf8_path)]
    default_flash_image: Utf8PathBuf,

    #[arg(long = "out-dir", value_parser = parse_utf8_path)]
    out_dir: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "factory-image", value_parser = parse_utf8_path)]
    factory_image: Option<Utf8PathBuf>,
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
    default_flash_image: Utf8PathBuf,
    out_dir: Utf8PathBuf,
    manifest: Utf8PathBuf,
    factory_image: Option<Utf8PathBuf>,
}

impl From<PackageArgs> for PackageRequest {
    fn from(args: PackageArgs) -> Self {
        Self {
            board: args.board,
            firmware_elf: args.firmware_elf,
            default_flash_image: args.default_flash_image,
            out_dir: args.out_dir,
            manifest: args.manifest,
            factory_image: args.factory_image,
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
    fn firmware_commit(&self) -> Result<String>;
    fn reference_commit(&self) -> Result<String>;
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

    fn firmware_commit(&self) -> Result<String> {
        command_stdout_trimmed(
            ProcessCommand::new("git")
                .current_dir(self.workspace_dir.as_std_path())
                .arg("rev-parse")
                .arg("HEAD"),
            "git rev-parse HEAD",
        )
    }

    fn reference_commit(&self) -> Result<String> {
        let reference_dir = self.workspace_dir.join(DEFAULT_REFERENCE_DIR);
        command_stdout_trimmed(
            ProcessCommand::new("git")
                .arg("-C")
                .arg(reference_dir.as_str())
                .arg("rev-parse")
                .arg("HEAD"),
            "git -C reference/esp-miner rev-parse HEAD",
        )
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
    let environment = LocalPackageEnvironment::detect()?;

    match cli.command {
        CliCommand::PackageFirmware(args) => {
            let request = PackageRequest::from(args);
            run_package_firmware(&request, &environment)?;
        }
        CliCommand::ValidatePackage(args) => {
            run_validate_package(&args)?;
        }
    }

    Ok(())
}

fn run_package_firmware(
    package_request: &PackageRequest,
    environment: &impl PackageEnvironment,
) -> Result<()> {
    let partition_table = detect_workspace_dir()?.join(DEFAULT_ULTRA205_PARTITION_TABLE);
    partition_contract::validate_ultra205_partition_contract(&partition_table)?;

    let manifest = build_manifest(package_request, environment)?;
    fs::create_dir_all(package_request.out_dir.as_std_path()).with_context(|| {
        format!(
            "failed to create output directory {}",
            package_request.out_dir
        )
    })?;
    write_manifest(&package_request.manifest, &manifest)
}

fn run_validate_package(args: &ValidatePackageArgs) -> Result<()> {
    let manifest = read_manifest_v2(&args.manifest)?;
    validate_package_manifest_v2(&manifest)?;
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

fn command_stdout_trimmed(command: &mut ProcessCommand, description: &str) -> Result<String> {
    let output = command
        .output()
        .with_context(|| format!("failed to run {description}"))?;

    if !output.status.success() {
        bail!(
            "{description} failed: {}",
            command_stderr_or_status(&output)
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{description} output was not valid UTF-8"))?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        bail!("{description} output was empty");
    }

    Ok(trimmed.to_owned())
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

    #[test]
    fn manifest_serializes_ultra205_default_elf_and_factory_artifact() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let package_elf = temp_path(&dir, DEFAULT_ELF_NAME);
        let factory_image = temp_path(&dir, FACTORY_IMAGE_NAME);
        write_fixture(&package_elf, b"elf");
        std::fs::write(factory_image.as_std_path(), b"factory").expect("write factory");

        let request = package_request(&dir, package_elf, Some(factory_image));
        let environment = FakePackageEnvironment::clean();

        // Act
        let manifest = build_manifest(&request, &environment).expect("manifest");

        // Assert
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.board, "205");
        assert_eq!(manifest.device_model, "Ultra 205");
        assert_eq!(manifest.asic, "BM1366");
        assert_eq!(manifest.reference_commit, EXPECTED_REFERENCE_COMMIT);
        assert_eq!(manifest.default_flash_image, DEFAULT_ELF_NAME);
        assert!(manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.path == FACTORY_IMAGE_NAME && artifact.offset == "0x0"));
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
            "schema_version": 2,
            "release_name": "bitaxe-ultra205-v1",
            "default_flash_image": "bitaxe-ultra205.elf",
            "image_metadata": {
                "board": "205",
                "device_model": "Ultra 205",
                "asic": "BM1366",
                "esp_idf_version": "v5.5.4",
                "rust_target": "xtensa-esp32s3-espidf"
            },
            "license_inventory": "docs/release/license-inventory.json",
            "provenance_manifest": "docs/release/provenance-manifest.json",
            "artifacts": artifacts
        });
        if include_install_notes {
            manifest["install_notes"] = serde_json::json!({
                "path": "docs/release/ultra-205.md",
                "summary": "Flash with just flash board=205"
            });
        }

        let path = temp_path(dir, "manifest-v2.json");
        std::fs::write(
            path.as_std_path(),
            serde_json::to_string_pretty(&manifest).expect("manifest json"),
        )
        .expect("write manifest");
        path
    }

    fn artifact_json(kind: &str, path: &str, offset: &str) -> serde_json::Value {
        serde_json::json!({
            "kind": kind,
            "path": path,
            "offset": offset,
            "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
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
        PackageRequest {
            board: BoardId::Ultra205,
            firmware_elf: temp_path(dir, DEFAULT_ELF_NAME),
            default_flash_image,
            factory_image,
            manifest: temp_path(dir, "bitaxe-ultra205-package.json"),
            out_dir: temp_dir_path(dir),
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

        fn firmware_commit(&self) -> Result<String> {
            Ok("firmware-commit".to_owned())
        }

        fn reference_commit(&self) -> Result<String> {
            Ok(EXPECTED_REFERENCE_COMMIT.to_owned())
        }

        fn maybe_tool_version(&self, tool: &str) -> Option<String> {
            if !self.tools_available {
                return None;
            }

            Some(format!("{tool} 1.0.0"))
        }
    }
}
