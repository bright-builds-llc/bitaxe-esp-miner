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
    esptool: &Utf8Path,
) -> Result<ProfileObservationCounts> {
    validate_managed_esptool(&environment.workspace_dir, esptool)?;
    environment.ensure_bootloader()?;
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: runtime restore attempted without a repository session");
    };
    let observation = run_installed_application(session, esptool.as_std_path())
        .map_err(|error| anyhow::anyhow!(error))?;
    let mut counts = ProfileObservationCounts::default();
    match observation.transport {
        UsbProfile::WorkerRuntime => counts.same_worker = 1,
        UsbProfile::SerialJtagRuntime => counts.same_serial_jtag = 1,
        UsbProfile::RomDownloader | UsbProfile::Unknown => counts.same_unknown = 1,
    }
    Ok(counts)
}

pub(super) fn execute_rom_exit(
    environment: &LocalFlashEnvironment,
    esptool: &Utf8Path,
    observation_seconds: u64,
) -> Result<RomExitHardwareCapture> {
    validate_managed_esptool(&environment.workspace_dir, esptool)?;
    environment.ensure_bootloader()?;
    let mut session_slot = environment.usb_session.borrow_mut();
    let Some(session) = session_slot.as_mut() else {
        bail!("cleanup_failed: ROM exit attempted without a repository session");
    };
    let output = session
        .run_espflash_probe(
            esptool.as_std_path(),
            &force_download_read_args(session.port()),
            Duration::from_secs(30),
        )
        .map_err(|error| anyhow::anyhow!(error))?;
    let force_download_bit_set = parse_force_download_bit(&output.stdout)?;
    if !force_download_bit_set {
        bail!("rom_exit=blocked reason=force_download_not_set");
    }
    let observation = run_installed_application(session, esptool.as_std_path())
        .map_err(|error| anyhow::anyhow!(error))?;
    let monitor = session
        .observe_receive_only(Duration::from_secs(observation_seconds.min(30)))
        .map_err(|error| anyhow::anyhow!(error))?;
    Ok(RomExitHardwareCapture {
        force_download_bit_set,
        transport: observation.transport,
        reenumerated: observation.reenumerated || monitor.reenumerated,
        monitor,
    })
}

impl RomExitEnvironment for LocalFlashEnvironment {
    fn execute_rom_exit(
        &self,
        esptool: &Utf8Path,
        observation_seconds: u64,
    ) -> Result<RomExitHardwareCapture> {
        execute_rom_exit(self, esptool, observation_seconds)
    }
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
