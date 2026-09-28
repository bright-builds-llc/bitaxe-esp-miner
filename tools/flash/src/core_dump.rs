//! Private development acquisition; task admission precedes every device effect.
use crate::*;

pub(crate) mod clearing;
mod contract;
mod proof;
#[cfg(test)]
mod tests;

#[derive(Debug, Parser)]
pub(crate) struct CoreDumpReadCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long)]
    pub(crate) expected_physical_sha256: String,
    #[arg(long)]
    pub(crate) expected_installed_source: String,
    #[arg(long)]
    pub(crate) expected_installed_elf: String,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) recovery_proof: Utf8PathBuf,
}

pub(crate) fn preflight(command: &CoreDumpReadCommand, clear: bool) -> Result<()> {
    let workspace = detect_workspace_dir()?;
    contract::admit_mode(&fs::read_to_string(workspace.join("TASKS.md"))?, clear)?;
    let source = Command::new("git")
        .current_dir(&workspace)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !source.status.success() {
        bail!("core_dump_source");
    }
    proof::admit(
        command,
        &workspace,
        String::from_utf8(source.stdout)?.trim(),
    )
}

pub(crate) fn run(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    run_mode(command, environment, None)
}

fn run_mode(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
    maybe_clear: Option<&clearing::CoreDumpClearCommand>,
) -> Result<()> {
    contract::admit_mode(
        &environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?,
        maybe_clear.is_some(),
    )?;
    ensure_ultra_205(command.board)?;
    if command.port.trim().is_empty()
        || !contract::hex(&command.expected_physical_sha256, 64)
        || !contract::hex(&command.expected_installed_source, 40)
        || !contract::hex(&command.expected_installed_elf, 64)
    {
        bail!("core_dump=blocked reason=invocation");
    }
    let expected = UsbRuntimeIdentity::new(
        &command.expected_installed_source,
        &command.expected_installed_elf,
    )?;
    let provenance = environment.current_provenance()?;
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != provenance.build_identity().source_commit()
    {
        bail!("core_dump=blocked reason=source_not_clean_and_pushed");
    }
    contract::require_clean_contract(&environment.workspace_path(Utf8Path::new(".")))?;
    proof::admit(
        command,
        &environment.workspace_path(Utf8Path::new(".")),
        provenance.build_identity().source_commit(),
    )?;
    let maybe_archive = maybe_clear
        .map(|clear| clearing::archive(clear, &environment.workspace_path(Utf8Path::new("."))))
        .transpose()?;
    let esptool = environment.prepare_application_exit()?;
    let root = environment.workspace_path(&command.private_root);
    environment.approve_private_evidence_root(&root)?;
    if root
        .components()
        .any(|part| matches!(part, camino::Utf8Component::ParentDir))
    {
        bail!("core_proof_output_path");
    }
    if fs::symlink_metadata(&root).is_ok() {
        bail!("core_dump=blocked reason=private_root_exists");
    }
    let parent = root.parent().context("core_proof_output_parent")?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        bail!("core_proof_output_parent");
    }
    for ancestor in parent.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            bail!("core_proof_output_symlink");
        }
    }
    proof::claim(
        command,
        &environment.workspace_path(Utf8Path::new(".")),
        provenance.build_identity().source_commit(),
        maybe_clear.is_some(),
        &root,
    )?;
    contract::create_root(&root)?;
    let mut rom_admitted = false;
    let mut stage = "session_admission";
    let operation: Result<()> = (|| {
        environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
        stage = "physical_identity";
        require_physical(command, environment)?;
        stage = "rom_admission";
        environment.admit_flash_read()?;
        rom_admitted = true;
        require_physical(command, environment)?;
        stage = "partition_table_read";
        let table = read_partition(
            command,
            environment,
            &root,
            "partition-table.private.bin",
            0x8000,
            0x1000,
        )?;
        stage = "partition_table_validation";
        let (offset, size) = contract::dump_partition(&table)?;
        stage = "dump_read";
        let dump = read_partition(
            command,
            environment,
            &root,
            "core-dump.private.bin",
            offset,
            size,
        )?;
        if let Some(archive) = &maybe_archive {
            stage = "preserved_dump_match";
            if dump != *archive {
                bail!("preserved_dump_drift");
            }
            stage = "dump_erase";
            clearing::erase(command, environment, offset, size)?;
            stage = "dump_erase_readback";
            let cleared = read_partition(
                command,
                environment,
                &root,
                "cleared-core-dump.private.bin",
                offset,
                size,
            )?;
            if cleared.iter().any(|byte| *byte != 0xff) {
                bail!("core_dump_erase_unverified");
            }
        }
        Ok(())
    })();
    let (returned, cleanup) = recover_and_release(
        rom_admitted,
        || {
            require_physical(command, environment)?;
            environment.execute_application_exit(&esptool)?;
            environment
                .observe_installed_runtime()?
                .require_identity(&expected)?;
            Ok(())
        },
        || environment.finish_usb_session(),
    );
    let category = if operation.is_err() {
        "acquisition_failed"
    } else if returned.is_err() {
        "application_return_failed"
    } else if cleanup.is_err() {
        "cleanup_failed"
    } else {
        "complete"
    };
    let result = serde_json::json!({
        "schema_version": if maybe_clear.is_some() { "bitaxe-development-core-dump-clear-v1" } else { "bitaxe-development-core-dump-read-v1" },
        "clearing_complete": maybe_clear.is_some() && operation.is_ok(),
        "terminal_category": category,
        "first_failure_stage": if operation.is_err() { stage } else { category },
        "source_commit": provenance.build_identity().source_commit(),
        "rom_admitted": rom_admitted,
        "acquisition_complete": operation.is_ok(),
        "application_identity_restored": rom_admitted && returned.is_ok(),
        "hardware_baseline_verified": false,
        "cleanup_complete": cleanup.is_ok(),
        "expected_installed_source": command.expected_installed_source,
        "expected_installed_elf": command.expected_installed_elf,
    });
    if write_private_new_bytes(
        &root.join("result.private.json"),
        &serde_json::to_vec_pretty(&result)?,
    )
    .is_err()
    {
        let first = if category == "complete" {
            "result_write_failed"
        } else {
            category
        };
        bail!("core_dump=failed category={first} result_write_failed=true");
    }
    emit_line("core_dump", category)?;
    if category != "complete" {
        bail!("core_dump=failed category={category}");
    }
    Ok(())
}

fn require_physical(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    if environment.usb_physical_identity_digest()? != command.expected_physical_sha256
        || environment.current_session_physical_identity_digest()?
            != command.expected_physical_sha256
    {
        bail!("core_dump=blocked reason=physical_identity_drift");
    }
    Ok(())
}

fn read_partition(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
    root: &Utf8Path,
    name: &str,
    offset: u32,
    size: u32,
) -> Result<Vec<u8>> {
    require_physical(command, environment)?;
    let output = root.join(name);
    write_private_new_bytes(&output, &[])?;
    let spec = CommandSpec::new(
        "espflash",
        [
            "read-flash",
            "--before",
            "no-reset",
            "--after",
            "no-reset",
            "--chip",
            "esp32s3",
            "--port",
            &command.port,
            "--non-interactive",
            "--skip-update-check",
            &format!("0x{offset:x}"),
            &format!("0x{size:x}"),
            output.as_str(),
        ],
    );
    // Raw tool diagnostics and dump bytes never go to the shared terminal.
    environment.execute_with_output(&spec)?;
    require_physical(command, environment)?;
    let bytes = environment.read_bytes(&output)?;
    if bytes.len() != size as usize {
        bail!("core_dump=failed reason=read_length");
    }
    emit_line("core_dump_read_sha256", &sha256_bytes(&bytes))?;
    Ok(bytes)
}

fn recover_and_release(
    rom_admitted: bool,
    return_application: impl FnOnce() -> Result<()>,
    release: impl FnOnce() -> Result<()>,
) -> (Result<()>, Result<()>) {
    let returned = if rom_admitted {
        return_application()
    } else {
        Ok(())
    };
    let cleanup = release();
    (returned, cleanup)
}
