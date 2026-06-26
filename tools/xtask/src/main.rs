use std::env;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::process::Command as ProcessCommand;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand};
use serde::Serialize;
use sha2::{Digest, Sha256};

const EXPECTED_REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const UNAVAILABLE: &str = "Unavailable";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_REFERENCE_GUARD: &str = "scripts/verify-reference-clean.sh";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
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

#[derive(Debug, Serialize)]
struct PackageManifest {
    schema_version: u32,
    board: String,
    device_model: String,
    asic: String,
    firmware_commit: String,
    reference_commit: String,
    esp_idf_version: String,
    rust_target: String,
    tool_versions: ToolVersions,
    default_flash_image: String,
    artifacts: Vec<ManifestArtifact>,
}

#[derive(Debug, Serialize)]
struct ToolVersions {
    cargo: String,
    rustc: String,
    bazel: String,
    espflash: String,
}

#[derive(Debug, Serialize)]
struct ManifestArtifact {
    kind: String,
    path: String,
    offset: String,
    sha256: String,
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
    }

    Ok(())
}

fn run_package_firmware(
    package_request: &PackageRequest,
    environment: &impl PackageEnvironment,
) -> Result<()> {
    let manifest = build_manifest(package_request, environment)?;
    fs::create_dir_all(package_request.out_dir.as_std_path()).with_context(|| {
        format!(
            "failed to create output directory {}",
            package_request.out_dir
        )
    })?;
    write_manifest(&package_request.manifest, &manifest)
}

fn build_manifest(
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
        esp_idf_version: ESP_IDF_VERSION.to_owned(),
        rust_target: RUST_TARGET.to_owned(),
        tool_versions: tool_versions(environment),
        default_flash_image: manifest_relative_path(
            &package_request.manifest,
            &package_request.default_flash_image,
        ),
        artifacts,
    })
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

fn validate_default_flash_image(default_flash_image: &Utf8Path) -> Result<()> {
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

fn sha256_file(path: &Utf8Path) -> Result<String> {
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

fn tool_versions(environment: &impl PackageEnvironment) -> ToolVersions {
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

fn write_manifest(path: &Utf8Path, manifest: &PackageManifest) -> Result<()> {
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

    fn temp_path(dir: &TempDir, file_name: &str) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().join(file_name)).expect("utf8 path")
    }

    fn temp_dir_path(dir: &TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
    }

    fn write_fixture(path: &Utf8Path, bytes: &[u8]) {
        std::fs::write(path.as_std_path(), bytes).expect("write fixture");
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
