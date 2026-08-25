use crate::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(crate) const RESTORE_BUNDLE_RELATIVE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const RESTORE_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const RESTORE_SCHEMA: &str = "bitaxe-stratum-v2-restore-bundle-v1";
const RESTORE_RANGES: [(&str, u32, u32); 8] = [
    ("bootloader", 0x000000, 0x008000),
    ("partition_table", 0x008000, 0x001000),
    ("phy_init", 0x00f000, 0x001000),
    ("factory", 0x010000, 0x400000),
    ("www", 0x410000, 0x300000),
    ("ota_0", 0x710000, 0x400000),
    ("ota_1", 0xb10000, 0x400000),
    ("otadata", 0xf10000, 0x002000),
];

#[derive(Debug, Deserialize)]
struct InstalledIdentity {
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    build_timestamp_utc: String,
    semantic_version: String,
    build_label: String,
    build_channel: String,
    source_dirty: bool,
    release_tag: Option<String>,
    idf_version: String,
    running_partition: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotRange {
    name: String,
    address: u32,
    size: u32,
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
enum RestoreBundle {
    #[serde(rename = "package_v3")]
    Package {
        schema_version: String,
        board: u16,
        installed_identity: InstalledIdentity,
        package_manifest: String,
        package_manifest_sha256: String,
        factory_sha256: String,
        capture_source_commit: String,
        plan_sha256: String,
    },
    #[serde(rename = "flash_snapshot_v1")]
    Snapshot {
        schema_version: String,
        board: u16,
        installed_identity: InstalledIdentity,
        ranges: Vec<SnapshotRange>,
        capture_source_commit: String,
        plan_sha256: String,
    },
}

struct PreparedRestore {
    command: CommandSpec,
    _snapshots: Vec<AdmittedExecutionSnapshot>,
}

fn require_mode(path: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("restore_installed=blocked reason=protected_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("restore_installed=blocked reason=protected_mode");
    }
    Ok(())
}

fn contained(root: &Utf8Path, relative: &str) -> Result<Utf8PathBuf> {
    let relative = Utf8Path::new(relative);
    if relative.as_str().is_empty() || relative.is_absolute() {
        bail!("restore_installed=blocked reason=path_contract");
    }
    let candidate = root.join(relative);
    let canonical_root = fs::canonicalize(root.as_std_path())?;
    let canonical_parent = fs::canonicalize(
        candidate
            .parent()
            .context("restore_installed=blocked reason=path_parent")?
            .as_std_path(),
    )?;
    if !canonical_parent.starts_with(&canonical_root) {
        bail!("restore_installed=blocked reason=path_escape");
    }
    require_mode(&candidate, 0o600, false)?;
    Ok(candidate)
}

fn validate_common(
    schema: &str,
    board: u16,
    identity: &InstalledIdentity,
    capture_source: &str,
    plan_sha256: &str,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let provenance = environment.current_provenance()?;
    if schema != RESTORE_SCHEMA
        || board != 205
        || identity.source_dirty
        || capture_source != provenance.build_identity().source_commit()
        || identity.reference_commit != environment.reference_commit()
        || identity.source_commit.len() != 40
        || identity.app_elf_sha256.len() != 64
        || identity.build_timestamp_utc.len() != 20
        || identity.semantic_version.is_empty()
        || !matches!(identity.build_channel.as_str(), "dev" | "release")
        || (identity.build_channel == "release") != identity.release_tag.is_some()
        || identity.idf_version != "v5.5.4"
        || !matches!(
            identity.running_partition.as_str(),
            "factory" | "ota_0" | "ota_1"
        )
        || plan_sha256.len() != 64
    {
        bail!("restore_installed=blocked reason=identity_contract");
    }
    let plan = environment.read_to_string(Utf8Path::new(RESTORE_PLAN_RELATIVE))?;
    if sha256_bytes(plan.as_bytes()) != plan_sha256 {
        bail!("restore_installed=blocked reason=plan_drift");
    }
    Ok(())
}

fn snapshot_command(
    root: &Utf8Path,
    ranges: &[SnapshotRange],
    port: &str,
    environment: &impl FlashEnvironment,
) -> Result<PreparedRestore> {
    let expected = RESTORE_RANGES;
    if ranges.len() != expected.len() {
        bail!("restore_installed=blocked reason=range_count");
    }
    let mut snapshots = Vec::with_capacity(ranges.len());
    for (range, (name, address, size)) in ranges.iter().zip(expected) {
        if range.name != name
            || range.address != address
            || range.size != size
            || range.sha256.len() != 64
        {
            bail!("restore_installed=blocked reason=range_contract");
        }
        let input = contained(root, &range.path)?;
        let bytes = environment.read_bytes(&input)?;
        if bytes.len() != size as usize || sha256_bytes(&bytes) != range.sha256 {
            bail!("restore_installed=blocked reason=range_digest");
        }
        snapshots.push(environment.create_admitted_execution_snapshot(&bytes)?);
    }
    let esptool = [
        ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
        ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
    ]
    .into_iter()
    .map(Utf8Path::new)
    .map(|path| environment.workspace_path(path))
    .find(|path| path.is_file())
    .context("restore_installed=blocked reason=esptool_missing")?;
    let mut args = vec![
        "--chip".to_owned(),
        "esp32s3".to_owned(),
        "--port".to_owned(),
        port.to_owned(),
        "--before".to_owned(),
        "usb_reset".to_owned(),
        "--after".to_owned(),
        "hard_reset".to_owned(),
        "write_flash".to_owned(),
        "--flash_mode".to_owned(),
        "dio".to_owned(),
        "--flash_size".to_owned(),
        "16MB".to_owned(),
        "--flash_freq".to_owned(),
        "80m".to_owned(),
        "--verify".to_owned(),
    ];
    for ((_, address, _), snapshot) in expected.into_iter().zip(snapshots.iter()) {
        args.push(format!("0x{address:x}"));
        args.push(snapshot.path().as_str().to_owned());
    }
    Ok(PreparedRestore {
        command: CommandSpec::new(esptool.as_str(), args),
        _snapshots: snapshots,
    })
}

fn package_command(
    root: &Utf8Path,
    manifest_relative: &str,
    manifest_sha256: &str,
    factory_sha256: &str,
    identity: &InstalledIdentity,
    port: &str,
    environment: &impl FlashEnvironment,
) -> Result<PreparedRestore> {
    let manifest_path = contained(root, manifest_relative)?;
    let manifest_document = environment.read_to_string(&manifest_path)?;
    if sha256_bytes(manifest_document.as_bytes()) != manifest_sha256 {
        bail!("restore_installed=blocked reason=manifest_digest");
    }
    let manifest: PackageManifest = serde_json::from_str(&manifest_document)?;
    if manifest.schema_version != 3
        || manifest.source_commit != identity.source_commit
        || manifest.reference_commit != identity.reference_commit
        || manifest.app_elf_sha256 != identity.app_elf_sha256
        || manifest.build_identity.label != identity.build_label
        || manifest.build_identity.source_dirty
    {
        bail!("restore_installed=blocked reason=package_identity");
    }
    validate_required_artifact_kinds(&manifest)?;
    let elf = require_artifact(&manifest, "firmware_elf")?;
    let elf_path = resolve_manifest_sibling(&manifest_path, Utf8Path::new(&elf.path))?;
    let elf_bytes = read_validated_artifact(elf, &elf_path, environment)?;
    if sha256_bytes(&elf_bytes) != identity.app_elf_sha256 {
        bail!("restore_installed=blocked reason=elf_identity");
    }
    let ota = require_artifact(&manifest, "firmware_ota_image")?;
    let ota_path = resolve_manifest_sibling(&manifest_path, Utf8Path::new(&ota.path))?;
    let ota_bytes = read_validated_artifact(ota, &ota_path, environment)?;
    let factory = require_artifact(&manifest, "factory_merged_image")?;
    let factory_path =
        resolve_manifest_factory_artifact(&manifest_path, Utf8Path::new(&factory.path))?;
    let factory_bytes = read_validated_artifact(factory, &factory_path, environment)?;
    if sha256_bytes(&factory_bytes) != factory_sha256 {
        bail!("restore_installed=blocked reason=factory_identity");
    }
    let app_sha = decode_lower_hex(&identity.app_elf_sha256)?;
    package_admission::validate_factory_ota_identity(
        &factory_bytes,
        &ota_bytes,
        package_admission::ExpectedApplicationIdentity {
            build_label: &identity.build_label,
            source_commit: &identity.source_commit,
            app_elf_sha256: &app_sha,
        },
    )?;
    let snapshot = environment.create_admitted_execution_snapshot(&factory_bytes)?;
    let command = CommandSpec::new(
        "espflash",
        [
            "write-bin",
            "--no-stub",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
            "--skip-update-check",
            "0x0",
            snapshot.path().as_str(),
        ],
    );
    Ok(PreparedRestore {
        command,
        _snapshots: vec![snapshot],
    })
}

pub(crate) fn run_restore_installed(
    command: &RestoreInstalledCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if !command.redact_evidence || command.restore_bundle != Utf8Path::new(RESTORE_BUNDLE_RELATIVE)
    {
        bail!("restore_installed=blocked reason=invocation_contract");
    }
    let bundle_path = environment.workspace_path(&command.restore_bundle);
    let root = bundle_path
        .parent()
        .context("restore_installed=blocked reason=root_missing")?;
    require_mode(root, 0o700, true)?;
    require_mode(&bundle_path, 0o600, false)?;
    let bundle_document = environment.read_to_string(&bundle_path)?;
    let bundle: RestoreBundle = serde_json::from_str(&bundle_document)?;
    let prepared = match bundle {
        RestoreBundle::Package {
            schema_version,
            board,
            installed_identity,
            package_manifest,
            package_manifest_sha256,
            factory_sha256,
            capture_source_commit,
            plan_sha256,
        } => {
            validate_common(
                &schema_version,
                board,
                &installed_identity,
                &capture_source_commit,
                &plan_sha256,
                environment,
            )?;
            package_command(
                root,
                &package_manifest,
                &package_manifest_sha256,
                &factory_sha256,
                &installed_identity,
                &command.port,
                environment,
            )?
        }
        RestoreBundle::Snapshot {
            schema_version,
            board,
            installed_identity,
            ranges,
            capture_source_commit,
            plan_sha256,
        } => {
            validate_common(
                &schema_version,
                board,
                &installed_identity,
                &capture_source_commit,
                &plan_sha256,
                environment,
            )?;
            snapshot_command(root, &ranges, &command.port, environment)?
        }
    };
    let nvs = prepare_wifi_nvs_seed(
        &command.port,
        &command.wifi_credentials,
        WifiNvsSeedMode::Ordinary,
        environment,
    )?;
    emit_line("restore_installed", PROTECTED_OPERATIONAL)?;
    environment.begin_usb_session(UsbOperation::Flash, &command.port)?;
    environment.execute(&prepared.command)?;
    environment.phase35_stage_readiness_gate("after-restore", &command.port)?;
    environment.execute(&nvs.command)?;
    environment.phase35_stage_readiness_gate("after-restore-nvs", &command.port)?;
    Ok(())
}

#[cfg(test)]
mod restore_contract_tests {
    use super::*;

    #[test]
    fn restore_ranges_exclude_nvs_and_coredump_storage() {
        // Arrange
        let nvs = 0x009000..0x00f000;
        let coredump_start = 0xf12000;

        // Act / Assert
        assert_eq!(RESTORE_RANGES.len(), 8);
        for (_, address, size) in RESTORE_RANGES {
            let end = address + size;
            assert!(address >= nvs.end || end <= nvs.start);
            assert!(end <= coredump_start);
        }
    }
}
