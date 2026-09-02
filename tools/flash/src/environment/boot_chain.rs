use super::*;

pub(super) fn execute_read(
    environment: &LocalFlashEnvironment,
    esptool: &Utf8Path,
    address: u32,
    size: u32,
    output: &Utf8Path,
) -> Result<()> {
    validate_esptool(environment, esptool)?;
    if size == 0 || size > 0x400000 || fs::symlink_metadata(output.as_std_path()).is_ok() {
        bail!("boot_chain=blocked reason=read_contract");
    }
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: boot-chain read attempted without a repository session");
    };
    let args = [
        "--chip".to_owned(),
        "esp32s3".to_owned(),
        "--port".to_owned(),
        session.port().to_owned(),
        "--before".to_owned(),
        "no_reset".to_owned(),
        "--after".to_owned(),
        "no_reset".to_owned(),
        "read_flash".to_owned(),
        format!("0x{address:x}"),
        format!("0x{size:x}"),
        output.as_str().to_owned(),
    ];
    session
        .run_espflash_probe(esptool.as_std_path(), &args, Duration::from_secs(360))
        .map_err(|error| anyhow::anyhow!(error))?;
    let metadata = fs::symlink_metadata(output.as_std_path())?;
    if !metadata.is_file() || metadata.len() != u64::from(size) {
        bail!("boot_chain=blocked reason=read_size");
    }
    set_private_file_mode(output)
}

pub(super) fn exit_rom(
    environment: &LocalFlashEnvironment,
    esptool: &Utf8Path,
) -> Result<UsbProfile> {
    validate_esptool(environment, esptool)?;
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: boot-chain exit attempted without a repository session");
    };
    let observation = run_installed_application(session, esptool.as_std_path())
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
