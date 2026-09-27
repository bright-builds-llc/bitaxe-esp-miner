//! Private development acquisition; task admission precedes every device effect.
use crate::*;

mod contract;
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
}

pub(crate) fn preflight() -> Result<()> {
    let workspace = detect_workspace_dir()?;
    contract::admit_task(&fs::read_to_string(workspace.join("TASKS.md"))?)
}

pub(crate) fn run(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    contract::admit_task(
        &environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?,
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
    let esptool = environment.prepare_application_exit()?;
    let root = environment.workspace_path(&command.private_root);
    environment.approve_private_evidence_root(&root)?;
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
        read_partition(
            command,
            environment,
            &root,
            "core-dump.private.bin",
            offset,
            size,
        )?;
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
        "schema_version": "bitaxe-development-core-dump-read-v1",
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
    write_private_new_bytes(
        &root.join("result.private.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
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
        || environment.current_usb_physical_identity_digest(&command.port)?
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
