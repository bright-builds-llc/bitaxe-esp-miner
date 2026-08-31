use super::usb_ownership::route_esptool_to_admitted_rom;
use super::*;

pub(super) fn execute_read_flash(
    environment: &LocalFlashEnvironment,
    command: &ManagedEsptoolReadFlash,
) -> Result<()> {
    validate_managed_esptool(&environment.workspace_dir, command.program())?;
    environment.ensure_bootloader()?;
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: esptool read attempted without a repository session");
    };
    let args = route_esptool_to_admitted_rom(command.args(), session.port())?;
    session
        .run_espflash_probe(
            command.program().as_std_path(),
            &args,
            Duration::from_secs(360),
        )
        .map_err(|error| anyhow::anyhow!(error))?;
    let metadata = fs::symlink_metadata(command.output().as_std_path())?;
    if !metadata.is_file() || metadata.len() != 0x6000 {
        bail!("nvs_readback=blocked reason=output_size");
    }
    set_private_file_mode(command.output())
}

pub(super) fn restore_application_runtime(
    environment: &LocalFlashEnvironment,
) -> Result<ProfileObservationCounts> {
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: runtime restore attempted without a repository session");
    };
    restore_usb_application_runtime(session, environment.espflash_bin.as_std_path())
        .map_err(|error| anyhow::anyhow!(error))
}

fn validate_managed_esptool(workspace: &Utf8Path, program: &Utf8Path) -> Result<()> {
    let canonical_workspace = fs::canonicalize(workspace.as_std_path())?;
    let canonical_program = fs::canonicalize(program.as_std_path())?;
    let relative = canonical_program
        .strip_prefix(&canonical_workspace)
        .context("managed esptool leaves workspace")?;
    let allowed = [
        ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
        ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
    ];
    if !allowed
        .iter()
        .any(|candidate| relative == std::path::Path::new(candidate))
        || fs::symlink_metadata(program.as_std_path())?
            .file_type()
            .is_symlink()
    {
        bail!("managed_esptool=blocked reason=program_contract");
    }
    Ok(())
}
