use super::*;

pub(super) fn exit_rom(
    environment: &LocalFlashEnvironment,
    esptool: &Utf8Path,
) -> Result<UsbProfile> {
    validate_esptool(environment, esptool)?;
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: boot-chain exit attempted without a repository session");
    };
    environment.validate_espflash_identity()?;
    let observation = run_installed_application(
        session,
        esptool.as_std_path(),
        environment.espflash_bin.as_std_path(),
    )
    .map_err(|error| anyhow::anyhow!(error))?;
    Ok(observation.transport)
}

fn validate_esptool(environment: &LocalFlashEnvironment, esptool: &Utf8Path) -> Result<()> {
    let workspace = fs::canonicalize(environment.workspace_dir.as_std_path())?;
    let program = fs::canonicalize(esptool.as_std_path())?;
    let relative = program.strip_prefix(&workspace)?;
    let allowed = [
        ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
        ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
    ];
    if !allowed
        .iter()
        .any(|candidate| relative == std::path::Path::new(candidate))
        || fs::symlink_metadata(esptool.as_std_path())?
            .file_type()
            .is_symlink()
    {
        bail!("boot_chain=blocked reason=esptool_contract");
    }
    Ok(())
}
