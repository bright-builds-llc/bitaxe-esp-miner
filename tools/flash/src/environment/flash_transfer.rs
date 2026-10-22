use super::*;

pub(super) fn execute_read(
    environment: &LocalFlashEnvironment,
    read: &ManagedFlashRead,
) -> Result<()> {
    validate_esptool(environment, read.program())?;
    if fs::symlink_metadata(read.output().as_std_path()).is_ok() {
        bail!("flash_transfer=blocked reason=output_exists");
    }
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: flash read attempted without a repository session");
    };
    session
        .run_espflash_probe(
            read.program().as_std_path(),
            &read.args(session.port()),
            Duration::from_secs(360),
        )
        .map_err(|error| anyhow::anyhow!(error))?;
    let metadata = fs::symlink_metadata(read.output().as_std_path())?;
    if !metadata.is_file() || metadata.len() != u64::from(read.size()) {
        bail!("flash_transfer=blocked reason=output_size");
    }
    set_private_file_mode(read.output())
}

/// Runs the read into a pipe that `memory_fifo` drains; the output is never a stored file.
pub(super) fn execute_read_to_pipe(
    environment: &LocalFlashEnvironment,
    read: &ManagedFlashRead,
) -> Result<()> {
    validate_esptool(environment, read.program())?;
    let metadata = fs::symlink_metadata(read.output().as_std_path())?;
    if !std::os::unix::fs::FileTypeExt::is_fifo(&metadata.file_type()) {
        bail!("flash_transfer=blocked reason=output_not_a_pipe");
    }
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: flash read attempted without a repository session");
    };
    session
        .run_espflash_probe(
            read.program().as_std_path(),
            &read.args(session.port()),
            Duration::from_secs(360),
        )
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!(error))
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
        bail!("flash_transfer=blocked reason=esptool_contract");
    }
    Ok(())
}
