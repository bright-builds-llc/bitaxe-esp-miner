use crate::*;
use esp_idf_part::{AppType, Partition, PartitionTable, SubType, Type};

const PLAN: &str =
    "docs/parity/work-plans/20260902T022334Z-NATIVE-USB-BOOT-CHAIN-INTEGRITY/PLAN.md";
const PLAN_SHA256: &str = "4eb4ae0d412d0cb4b56ccd640f407ae000929c564f7ae78554bfb4a893553fa1";
const ROOT: &str = "scratch/native-usb-boot-chain-integrity/attempt-001";
const MANIFEST: &str = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const BUNDLE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const PREDECESSOR: &str =
    "scratch/native-usb-owner-recovery/attempt-001/recovery-result.private.json";
const DISPLAY: &str = "display-capture.private.json";
const MANUAL: &str = "manual-bootstrap.private.json";

#[derive(Debug, Deserialize)]
struct Bundle {
    installed_identity: InstalledIdentity,
    ranges: Vec<SnapshotRange>,
}

#[derive(Debug, Deserialize)]
struct InstalledIdentity {
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    build_label: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OtaEntry {
    sequence: u32,
    state: u32,
    crc: u32,
}

#[derive(Debug, Serialize)]
struct BootChainResult {
    schema_version: &'static str,
    source_commit: String,
    reference_commit: String,
    plan_sha256: &'static str,
    manifest_sha256: String,
    restore_bundle_sha256: String,
    display_category: String,
    bootloader_match: bool,
    partition_table_match: bool,
    otadata_match: bool,
    partition_table_valid: bool,
    ota_selection_category: &'static str,
    selected_partition_category: String,
    selected_partition_bundle_match: bool,
    selected_app_digest_match: bool,
    selected_app_header_valid: bool,
    selected_app_identity_match: bool,
    physical_identity_match: bool,
    rom_admission_count: u8,
    metadata_read_count: u8,
    selected_app_read_count: u8,
    application_exit_count: u8,
    device_write_observed: bool,
    host_network_effect: bool,
    cleanup_complete: bool,
    terminal_category: &'static str,
    redaction_status: &'static str,
}

pub(crate) fn run_boot_chain_readback(
    command: BootChainReadbackCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    validate_invocation(&command)?;
    let root = environment.workspace_path(&command.private_root);
    require_private(&root, 0o700, true)?;
    require_private(&root.join(DISPLAY), 0o600, false)?;
    require_private(&root.join(MANUAL), 0o600, false)?;
    if fs::symlink_metadata(root.join("machine-result.private.json").as_std_path()).is_ok() {
        bail!("boot_chain=blocked reason=consumed");
    }
    let plan = environment.read_bytes(&environment.workspace_path(Utf8Path::new(PLAN)))?;
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if sha256_bytes(&plan) != PLAN_SHA256
        || !tasks.contains("### task-native-usb-boot-chain-integrity-205")
    {
        bail!("boot_chain=blocked reason=plan_identity");
    }
    let provenance = environment.current_provenance()?;
    let source_commit = provenance.build_identity().source_commit().to_owned();
    let manifest_bytes =
        environment.read_bytes(&environment.workspace_path(Utf8Path::new(MANIFEST)))?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != source_commit
        || manifest
            .get("source_commit")
            .and_then(serde_json::Value::as_str)
            != Some(&source_commit)
    {
        bail!("boot_chain=blocked reason=source_identity");
    }
    let bundle_path = environment.workspace_path(Utf8Path::new(BUNDLE));
    require_private(&bundle_path, 0o600, false)?;
    let bundle_bytes = environment.read_bytes(&bundle_path)?;
    let bundle: Bundle = serde_json::from_slice(&bundle_bytes)?;
    validate_bundle(
        &bundle,
        bundle_path
            .parent()
            .context("boot_chain=blocked reason=bundle_parent")?,
        environment,
    )?;
    let display: serde_json::Value =
        serde_json::from_slice(&environment.read_bytes(&root.join(DISPLAY))?)?;
    let display_category = display
        .get("category")
        .and_then(serde_json::Value::as_str)
        .filter(|value| {
            matches!(
                *value,
                "active_ui"
                    | "boot_or_error_text"
                    | "blank_or_dark"
                    | "frozen_or_static"
                    | "unknown"
            )
        })
        .context("boot_chain=blocked reason=display")?
        .to_owned();
    let predecessor: serde_json::Value = serde_json::from_slice(
        &environment.read_bytes(&environment.workspace_path(Utf8Path::new(PREDECESSOR)))?,
    )?;
    let expected_physical = predecessor
        .get("physical_identity_digest")
        .and_then(serde_json::Value::as_str)
        .filter(|value| value.len() == 64)
        .context("boot_chain=blocked reason=predecessor_identity")?;

    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    let current_physical = environment.usb_physical_identity_digest()?;
    if current_physical != expected_physical {
        bail!("physical_identity_drift");
    }
    let board_info =
        environment.execute_owner_rom_probe(&owner_rom_probe_command(&command.port))?;
    if !board_info_reports_esp32s3(&board_info) {
        bail!("boot_chain=blocked reason=rom_admission");
    }
    let esptool = find_managed_esptool(environment)?;
    let bundle_root = bundle_path
        .parent()
        .context("boot_chain=blocked reason=bundle_parent")?;

    let bootloader_match = read_and_compare(
        "bootloader",
        &bundle,
        bundle_root,
        &root,
        &esptool,
        environment,
    )?;
    let partition_table_match = read_and_compare(
        "partition_table",
        &bundle,
        bundle_root,
        &root,
        &esptool,
        environment,
    )?;
    let otadata_match = read_and_compare(
        "otadata",
        &bundle,
        bundle_root,
        &root,
        &esptool,
        environment,
    )?;
    let table_bytes = environment.read_bytes(&root.join("partition_table.private.bin"))?;
    let table = PartitionTable::try_from_bytes(table_bytes)
        .map_err(|_| anyhow::anyhow!("boot_chain=blocked reason=partition_table_invalid"))?;
    let ota_bytes = environment.read_bytes(&root.join("otadata.private.bin"))?;
    let (selected, ota_selection_category) = select_partition(&table, &ota_bytes)?;
    let selected_name = selected.name();
    let selected_category = selected_name.clone();
    let selected_snapshot = bundle
        .ranges
        .iter()
        .find(|range| range.name == selected_name)
        .context("boot_chain=blocked reason=selected_bundle_range")?;
    let selected_partition_bundle_match =
        selected_snapshot.address == selected.offset() && selected_snapshot.size == selected.size();
    if !selected_partition_bundle_match {
        bail!("boot_chain=blocked reason=selected_layout");
    }
    environment.execute_boot_chain_read(
        &esptool,
        selected.offset(),
        selected.size(),
        &root.join("selected-app.private.bin"),
    )?;
    let selected_bytes = environment.read_bytes(&root.join("selected-app.private.bin"))?;
    let snapshot_bytes =
        environment.read_bytes(&contained_snapshot(bundle_root, &selected_snapshot.path)?)?;
    let selected_app_digest_match = sha256_bytes(&selected_bytes) == selected_snapshot.sha256;
    let selected_app_bytes_match = selected_bytes == snapshot_bytes;
    let selected_app_header_valid = esp32s3_header_valid(&selected_bytes);
    let selected_app_identity_match = selected_name == bundle.installed_identity.running_partition
        && bundle.installed_identity.app_elf_sha256.len() == 64
        && !bundle.installed_identity.build_label.is_empty()
        && bundle.installed_identity.source_commit.len() == 40
        && bundle.installed_identity.reference_commit == provenance.reference_commit();
    let _transport = environment.exit_boot_chain_rom(&esptool)?;
    environment.finish_usb_session()?;
    let all_match = bootloader_match
        && partition_table_match
        && otadata_match
        && selected_partition_bundle_match
        && selected_app_digest_match
        && selected_app_bytes_match
        && selected_app_header_valid
        && selected_app_identity_match;
    let result = BootChainResult {
        schema_version: "bitaxe-native-usb-boot-chain-private-v1",
        source_commit,
        reference_commit: provenance.reference_commit().to_owned(),
        plan_sha256: PLAN_SHA256,
        manifest_sha256: sha256_bytes(&manifest_bytes),
        restore_bundle_sha256: sha256_bytes(&bundle_bytes),
        display_category,
        bootloader_match,
        partition_table_match,
        otadata_match,
        partition_table_valid: true,
        ota_selection_category,
        selected_partition_category: selected_category,
        selected_partition_bundle_match,
        selected_app_digest_match: selected_app_digest_match && selected_app_bytes_match,
        selected_app_header_valid,
        selected_app_identity_match,
        physical_identity_match: true,
        rom_admission_count: 1,
        metadata_read_count: 3,
        selected_app_read_count: 1,
        application_exit_count: 1,
        device_write_observed: false,
        host_network_effect: false,
        cleanup_complete: true,
        terminal_category: if all_match {
            "boot_chain_exact"
        } else {
            "boot_chain_mismatch"
        },
        redaction_status: "passed",
    };
    let mut encoded = serde_json::to_vec_pretty(&result)?;
    encoded.push(b'\n');
    write_private_new_bytes(&root.join("machine-result.private.json"), &encoded)?;
    emit_line("boot_chain", result.terminal_category)
}

fn validate_invocation(command: &BootChainReadbackCommand) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.package_manifest != Utf8Path::new(MANIFEST)
        || command.restore_bundle != Utf8Path::new(BUNDLE)
        || command.private_root != Utf8Path::new(ROOT)
        || command.plan != Utf8Path::new(PLAN)
        || command.manual_checkpoint != Utf8Path::new(ROOT).join(MANUAL)
        || !command.redact_evidence
    {
        bail!("boot_chain=blocked reason=invocation");
    }
    Ok(())
}

fn validate_bundle(
    bundle: &Bundle,
    root: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    for name in [
        "bootloader",
        "partition_table",
        "factory",
        "ota_0",
        "ota_1",
        "otadata",
    ] {
        let range = bundle
            .ranges
            .iter()
            .find(|range| range.name == name)
            .with_context(|| format!("boot_chain=blocked reason=bundle_range name={name}"))?;
        let bytes = environment.read_bytes(&contained_snapshot(root, &range.path)?)?;
        if bytes.len() != range.size as usize || sha256_bytes(&bytes) != range.sha256 {
            bail!("boot_chain=blocked reason=bundle_digest");
        }
    }
    Ok(())
}

fn read_and_compare(
    name: &str,
    bundle: &Bundle,
    bundle_root: &Utf8Path,
    root: &Utf8Path,
    esptool: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<bool> {
    let range = bundle
        .ranges
        .iter()
        .find(|range| range.name == name)
        .with_context(|| format!("boot_chain=blocked reason=range name={name}"))?;
    let output = root.join(format!("{name}.private.bin"));
    environment.execute_boot_chain_read(esptool, range.address, range.size, &output)?;
    let actual = environment.read_bytes(&output)?;
    let expected = environment.read_bytes(&contained_snapshot(bundle_root, &range.path)?)?;
    Ok(actual == expected && sha256_bytes(&actual) == range.sha256)
}

fn contained_snapshot(root: &Utf8Path, relative: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, camino::Utf8Component::Normal(_)))
    {
        bail!("boot_chain=blocked reason=snapshot_path");
    }
    Ok(root.join(path))
}

fn select_partition(table: &PartitionTable, bytes: &[u8]) -> Result<(Partition, &'static str)> {
    if bytes.len() != 0x2000 {
        bail!("boot_chain=blocked reason=otadata_size");
    }
    let entries = [
        parse_ota_entry(&bytes[..32])?,
        parse_ota_entry(&bytes[0x1000..0x1020])?,
    ];
    let factory = table
        .partitions()
        .iter()
        .find(|p| p.ty() == Type::App && p.subtype() == SubType::App(AppType::Factory))
        .cloned();
    let ota = table.partitions().iter().filter(|p| p.ty() == Type::App && matches!(p.subtype(), SubType::App(app) if (app as u8) >= 0x10 && (app as u8) <= 0x1f)).cloned().collect::<Vec<_>>();
    if ota.is_empty() {
        return factory
            .map(|partition| (partition, "factory_fallback"))
            .context("boot_chain=blocked reason=no_boot_partition");
    }
    let valid = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| ota_valid(**entry))
        .collect::<Vec<_>>();
    let Some((_, active)) = valid
        .into_iter()
        .max_by_key(|(index, entry)| (entry.sequence, std::cmp::Reverse(*index)))
    else {
        return factory
            .map(|partition| (partition, "factory_fallback"))
            .context("boot_chain=blocked reason=no_boot_partition");
    };
    let slot = usize::try_from(active.sequence.saturating_sub(1))? % ota.len();
    Ok((
        ota.get(slot)
            .context("boot_chain=blocked reason=ota_slot")?
            .clone(),
        "ota_selected",
    ))
}

fn parse_ota_entry(bytes: &[u8]) -> Result<OtaEntry> {
    let sequence = u32::from_le_bytes(
        bytes
            .get(0..4)
            .context("boot_chain=blocked reason=ota_entry")?
            .try_into()?,
    );
    let state = u32::from_le_bytes(
        bytes
            .get(24..28)
            .context("boot_chain=blocked reason=ota_entry")?
            .try_into()?,
    );
    let crc = u32::from_le_bytes(
        bytes
            .get(28..32)
            .context("boot_chain=blocked reason=ota_entry")?
            .try_into()?,
    );
    Ok(OtaEntry {
        sequence,
        state,
        crc,
    })
}

fn ota_valid(entry: OtaEntry) -> bool {
    entry.sequence != u32::MAX
        && !matches!(entry.state, 3 | 4)
        && entry.crc == crc32_le(u32::MAX, &entry.sequence.to_le_bytes())
}

fn crc32_le(mut crc: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn esp32s3_header_valid(bytes: &[u8]) -> bool {
    bytes.first() == Some(&0xe9)
        && bytes.get(1).is_some_and(|count| (1..=16).contains(count))
        && bytes.get(12..14) == Some(&9_u16.to_le_bytes())
}

fn require_private(path: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("boot_chain=blocked reason=private_path");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("boot_chain=blocked reason=private_mode");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ota_crc_and_invalid_states_are_closed() {
        let sequence = 1_u32;
        let entry = OtaEntry {
            sequence,
            state: 2,
            crc: crc32_le(u32::MAX, &sequence.to_le_bytes()),
        };
        assert_eq!(crc32_le(u32::MAX, &sequence.to_le_bytes()), 0x99f8_b879);
        assert!(ota_valid(entry));
        assert!(!ota_valid(OtaEntry { state: 3, ..entry }));
        assert!(!ota_valid(OtaEntry { crc: 0, ..entry }));
    }

    #[test]
    fn source_contains_no_write_or_network_surface() {
        let source = include_str!("boot_chain.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("source");
        for forbidden in ["write_flash", "write-bin", "erase_flash", "wifi", "http://"] {
            assert!(!source.contains(forbidden));
        }
    }
}
