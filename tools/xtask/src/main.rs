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

    #[arg(long = "volatile-status-file", value_parser = parse_utf8_path)]
    volatile_status_file: Utf8PathBuf,

    #[arg(long = "stamp-out", value_parser = parse_utf8_path)]
    stamp_out: Utf8PathBuf,

    #[arg(long = "sdkconfig-defaults-out", value_parser = parse_utf8_path)]
    sdkconfig_defaults_out: Utf8PathBuf,

    #[arg(long = "build-timestamp-out", value_parser = parse_utf8_path)]
    build_timestamp_out: Utf8PathBuf,
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
    let volatile_status = fs::read_to_string(args.volatile_status_file.as_std_path())
        .with_context(|| {
            format!(
                "failed to read volatile workspace status {}",
                args.volatile_status_file
            )
        })?;
    let provenance = BuildProvenance::parse_workspace_status(&status)
        .context("invalid Bitaxe build provenance")?;
    let build_timestamp_utc = build_timestamp_utc(&volatile_status)?;

    write_parented_file(&args.stamp_out, &provenance.render_stamp())?;
    write_parented_file(
        &args.sdkconfig_defaults_out,
        &format!(
            "CONFIG_APP_PROJECT_VER_FROM_CONFIG=y\nCONFIG_APP_PROJECT_VER=\"{}\"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n",
            provenance.build_identity().build_label()
        ),
    )?;
    write_parented_file(
        &args.build_timestamp_out,
        &format!("{build_timestamp_utc}\n"),
    )
}

fn build_timestamp_utc(volatile_status: &str) -> Result<String> {
    let mut maybe_formatted_date = None;
    for line in volatile_status.lines() {
        let Some((key, value)) = line.split_once(' ') else {
            continue;
        };
        if key != "FORMATTED_DATE" {
            continue;
        }
        if maybe_formatted_date.replace(value).is_some() {
            bail!("volatile workspace status contains duplicate FORMATTED_DATE");
        }
    }

    let Some(formatted_date) = maybe_formatted_date else {
        bail!("volatile workspace status is missing FORMATTED_DATE");
    };
    let fields: Vec<_> = formatted_date.split_ascii_whitespace().collect();
    let [year, month_name, day, hour, minute, second, _weekday] = fields.as_slice() else {
        bail!("FORMATTED_DATE must contain year month day hour minute second weekday");
    };
    let year = parse_timestamp_field("year", year, 1970, 9999)?;
    let month = month_number(month_name)?;
    let day = parse_timestamp_field("day", day, 1, days_in_month(year, month))?;
    let hour = parse_timestamp_field("hour", hour, 0, 23)?;
    let minute = parse_timestamp_field("minute", minute, 0, 59)?;
    let second = parse_timestamp_field("second", second, 0, 59)?;

    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn parse_timestamp_field(name: &str, value: &str, minimum: u32, maximum: u32) -> Result<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("FORMATTED_DATE {name} must be numeric");
    }
    let parsed = value
        .parse::<u32>()
        .with_context(|| format!("FORMATTED_DATE {name} is out of range"))?;
    if !(minimum..=maximum).contains(&parsed) {
        bail!("FORMATTED_DATE {name} is out of range");
    }
    Ok(parsed)
}

fn month_number(name: &str) -> Result<u32> {
    match name {
        "Jan" => Ok(1),
        "Feb" => Ok(2),
        "Mar" => Ok(3),
        "Apr" => Ok(4),
        "May" => Ok(5),
        "Jun" => Ok(6),
        "Jul" => Ok(7),
        "Aug" => Ok(8),
        "Sep" => Ok(9),
        "Oct" => Ok(10),
        "Nov" => Ok(11),
        "Dec" => Ok(12),
        _ => bail!("FORMATTED_DATE month is invalid"),
    }
}

const fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

const fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
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
mod tests;
