use crate::*;

const RECOVERY_BUNDLE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const ROOT_PREFIX: &str = "scratch/usb-stability-read";
const FACTORY_ADDRESS: u32 = 0x10000;
const FACTORY_SIZE: u32 = 0x400000;

#[derive(Debug, Deserialize)]
struct StabilityBundle {
    ranges: Vec<StabilityRange>,
}

#[derive(Debug, Deserialize)]
struct StabilityRange {
    name: String,
    address: u32,
    size: u32,
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct StabilityResult {
    schema_version: &'static str,
    source_commit: String,
    restore_bundle_sha256: String,
    adapter: &'static str,
    flash_size: &'static str,
    chunk_bytes: u32,
    pattern: &'static str,
    requested_repetitions: u8,
    completed_repetitions: u8,
    digest_match_count: u8,
    rom_admitted: bool,
    physical_identity_stable: bool,
    application_exit_complete: bool,
    cleanup_complete: bool,
    device_write_observed: bool,
    host_network_effect: bool,
    terminal_category: &'static str,
    redaction_status: &'static str,
}

pub(crate) fn run_usb_stability_read(
    command: UsbStabilityReadCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let root = validate_stability_invocation(&command, environment)?;
    let bundle_path = environment.workspace_path(&command.restore_bundle);
    require_private_stability_path(&bundle_path, 0o600, false)?;
    let bundle_bytes = environment.read_bytes(&bundle_path)?;
    let bundle: StabilityBundle = serde_json::from_slice(&bundle_bytes)?;
    let factory = bundle
        .ranges
        .iter()
        .find(|range| range.name == "factory")
        .context("usb_stability=blocked reason=factory_snapshot_missing")?;
    if factory.address != FACTORY_ADDRESS
        || factory.size != FACTORY_SIZE
        || factory.sha256.len() != 64
    {
        bail!("usb_stability=blocked reason=factory_snapshot_contract");
    }
    let bundle_root = bundle_path
        .parent()
        .context("usb_stability=blocked reason=bundle_parent")?;
    let snapshot_path = contained_stability_path(bundle_root, &factory.path)?;
    require_private_stability_path(&snapshot_path, 0o600, false)?;
    let snapshot = environment.read_bytes(&snapshot_path)?;
    if snapshot.len() != FACTORY_SIZE as usize || sha256_bytes(&snapshot) != factory.sha256 {
        bail!("usb_stability=blocked reason=factory_snapshot_digest");
    }

    create_stability_root(&root, &command.private_root, environment)?;
    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    let initial_identity = environment.usb_physical_identity_digest()?;
    let board_info =
        environment.execute_owner_rom_probe(&owner_rom_probe_command(&command.port))?;
    if !board_info_reports_esp32s3(&board_info) {
        bail!("usb_stability=blocked reason=rom_admission");
    }
    let esptool = find_managed_esptool(environment)?;
    let expected = snapshot
        .get(..command.chunk_bytes as usize)
        .context("usb_stability=blocked reason=chunk_range")?;
    let mut completed_repetitions = 0_u8;
    let mut digest_match_count = 0_u8;
    let mut maybe_failure = None;
    for repetition in 1..=command.repetitions {
        let offset = stability_chunk_offset(command.pattern, command.chunk_bytes, repetition)?;
        let address = FACTORY_ADDRESS
            .checked_add(offset)
            .context("usb_stability=blocked reason=address_overflow")?;
        let output = root.join(format!("chunk-{repetition:03}.private.bin"));
        if environment
            .execute_boot_chain_read(&esptool, address, command.chunk_bytes, &output)
            .is_err()
        {
            maybe_failure = Some("transport_failed");
            break;
        }
        completed_repetitions = completed_repetitions.saturating_add(1);
        let actual = environment.read_bytes(&output)?;
        let start = offset as usize;
        let end = start
            .checked_add(command.chunk_bytes as usize)
            .context("usb_stability=blocked reason=chunk_range")?;
        let expected_chunk = if command.pattern == UsbStabilityPattern::Repeated {
            expected
        } else {
            snapshot
                .get(start..end)
                .context("usb_stability=blocked reason=chunk_range")?
        };
        if actual != expected_chunk {
            maybe_failure = Some("digest_mismatch");
            break;
        }
        digest_match_count = digest_match_count.saturating_add(1);
    }
    let physical_identity_stable = environment
        .current_usb_physical_identity_digest(&command.port)
        .is_ok_and(|identity| identity == initial_identity);
    let application_exit_complete = environment.exit_boot_chain_rom(&esptool).is_ok();
    let cleanup_complete = environment.finish_usb_session().is_ok();
    let terminal_category = match maybe_failure {
        Some(failure) => failure,
        None if !physical_identity_stable => "physical_identity_drift",
        None if !application_exit_complete => "application_exit_failed",
        None if !cleanup_complete => "cleanup_failed",
        None => "complete",
    };
    let provenance = environment.current_provenance()?;
    let result = StabilityResult {
        schema_version: "bitaxe-usb-stability-read-v1",
        source_commit: provenance.build_identity().source_commit().to_owned(),
        restore_bundle_sha256: sha256_bytes(&bundle_bytes),
        adapter: "rom_no_stub",
        flash_size: "16mb",
        chunk_bytes: command.chunk_bytes,
        pattern: stability_pattern_label(command.pattern),
        requested_repetitions: command.repetitions,
        completed_repetitions,
        digest_match_count,
        rom_admitted: true,
        physical_identity_stable,
        application_exit_complete,
        cleanup_complete,
        device_write_observed: false,
        host_network_effect: false,
        terminal_category,
        redaction_status: "passed",
    };
    let mut encoded = serde_json::to_vec_pretty(&result)?;
    encoded.push(b'\n');
    write_private_new_bytes(&root.join("result.private.json"), &encoded)?;
    emit_line("usb_stability_read", terminal_category)?;
    if terminal_category != "complete" {
        bail!("usb_stability=failed category={terminal_category}");
    }
    Ok(())
}

fn validate_stability_invocation(
    command: &UsbStabilityReadCommand,
    environment: &impl FlashEnvironment,
) -> Result<Utf8PathBuf> {
    ensure_ultra_205(command.board)?;
    if command.restore_bundle != Utf8Path::new(RECOVERY_BUNDLE)
        || !matches!(command.chunk_bytes, 16_384 | 65_536 | 262_144)
        || !(1..=20).contains(&command.repetitions)
        || !command.redact_evidence
        || command.private_root.is_absolute()
        || !command.private_root.starts_with(ROOT_PREFIX)
    {
        bail!("usb_stability=blocked reason=invocation");
    }
    if command.pattern == UsbStabilityPattern::Sequential
        && u32::from(command.repetitions)
            .checked_mul(command.chunk_bytes)
            .is_none_or(|bytes| bytes > FACTORY_SIZE)
    {
        bail!("usb_stability=blocked reason=sequential_range");
    }
    let provenance = environment.current_provenance()?;
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != provenance.build_identity().source_commit()
    {
        bail!("usb_stability=blocked reason=source_identity");
    }
    let root = environment.workspace_path(&command.private_root);
    if fs::symlink_metadata(root.as_std_path()).is_ok() {
        bail!("usb_stability=blocked reason=root_exists");
    }
    Ok(root)
}

fn stability_chunk_offset(
    pattern: UsbStabilityPattern,
    chunk_bytes: u32,
    repetition: u8,
) -> Result<u32> {
    match pattern {
        UsbStabilityPattern::Repeated => Ok(0),
        UsbStabilityPattern::Sequential => u32::from(repetition.saturating_sub(1))
            .checked_mul(chunk_bytes)
            .context("usb_stability=blocked reason=chunk_offset"),
    }
}

const fn stability_pattern_label(pattern: UsbStabilityPattern) -> &'static str {
    match pattern {
        UsbStabilityPattern::Repeated => "repeated",
        UsbStabilityPattern::Sequential => "sequential",
    }
}

fn create_stability_root(
    root: &Utf8Path,
    requested: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    environment.approve_private_evidence_root(requested)?;
    let parent = root
        .parent()
        .context("usb_stability=blocked reason=root_parent")?;
    fs::create_dir_all(parent.as_std_path())?;
    set_private_directory_mode(parent)?;
    fs::create_dir(root.as_std_path())?;
    set_private_directory_mode(root)
}

fn contained_stability_path(root: &Utf8Path, relative: &str) -> Result<Utf8PathBuf> {
    let relative = Utf8Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, camino::Utf8Component::Normal(_)))
    {
        bail!("usb_stability=blocked reason=snapshot_path");
    }
    Ok(root.join(relative))
}

fn require_private_stability_path(path: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("usb_stability=blocked reason=private_path");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("usb_stability=blocked reason=private_mode");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{stability_chunk_offset, UsbStabilityPattern};

    #[test]
    fn sequential_chunks_cover_distinct_offsets() {
        // Arrange / Act / Assert
        assert_eq!(
            stability_chunk_offset(UsbStabilityPattern::Sequential, 262_144, 1)
                .expect("first chunk"),
            0
        );
        assert_eq!(
            stability_chunk_offset(UsbStabilityPattern::Sequential, 262_144, 16)
                .expect("last chunk"),
            3_932_160
        );
        assert_eq!(
            stability_chunk_offset(UsbStabilityPattern::Repeated, 65_536, 20)
                .expect("repeated chunk"),
            0
        );
    }

    #[test]
    fn production_source_has_no_device_write_or_network_surface() {
        // Arrange
        let source = include_str!("usb_stability.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        // Act / Assert
        for forbidden in ["write_flash", "write-bin", "erase_flash", "wifi", "http://"] {
            assert!(!source.contains(forbidden));
        }
    }
}
