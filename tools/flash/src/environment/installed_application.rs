use super::*;

pub(super) fn prepare(environment: &LocalFlashEnvironment) -> Result<Utf8PathBuf> {
    let program = find_managed_esptool(environment)?;
    let workspace = fs::canonicalize(environment.workspace_dir.as_std_path())?;
    let canonical = fs::canonicalize(program.as_std_path())?;
    let relative = canonical.strip_prefix(workspace)?;
    if ![
        ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
        ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
    ]
    .iter()
    .any(|allowed| relative == std::path::Path::new(allowed))
    {
        bail!("application_exit=blocked reason=managed_tool_contract");
    }
    Ok(program)
}

pub(super) fn begin_session(
    environment: &LocalFlashEnvironment,
    port: &str,
    root: &Utf8Path,
) -> Result<()> {
    if environment.usb_session.borrow().is_some() || root.try_exists()? {
        bail!("start_installed=blocked reason=session_already_exists");
    }
    let session = UsbSession::acquire(UsbOperation::Recover, port, root.as_std_path())?;
    *environment.usb_session.borrow_mut() = Some(session);
    let mut slot = environment.usb_session.borrow_mut();
    let session = slot.as_mut().context("start_installed=session_missing")?;
    let version = session.run_espflash_probe(
        environment.espflash_bin.as_std_path(),
        &["--version".to_owned()],
        Duration::from_secs(10),
    )?;
    if version.stdout != format!("{}\n", environment.espflash_version).as_bytes() {
        bail!("start_installed=blocked reason=espflash_version_mismatch");
    }
    Ok(())
}

pub(super) fn execute_exit(
    environment: &LocalFlashEnvironment,
    esptool: &Utf8Path,
) -> Result<InstalledApplicationExit> {
    if prepare(environment)? != esptool {
        bail!("application_exit=blocked reason=managed_tool_changed");
    }
    let mut slot = environment.usb_session.borrow_mut();
    let session = slot.as_mut().context("application_exit=session_missing")?;
    let before = inspect_usb_profile(session.port())?;
    if before.physical_identity_digest != session.physical_identity_digest() {
        bail!("physical_identity_drift");
    }
    if !matches!(
        before.profile,
        UsbProfile::SerialJtagRuntime | UsbProfile::RomDownloader
    ) {
        bail!("rom_admission_failed");
    }
    let args = installed_rom_probe_args(session.port());
    let output = session.run_espflash_probe(
        environment.espflash_bin.as_std_path(),
        &args,
        Duration::from_secs(30),
    )?;
    let mut board_info = output.stdout;
    board_info.extend_from_slice(&output.stderr);
    let after = inspect_usb_profile(session.port())?;
    let rom = admit_rom_downloader(after, &board_info)?;
    if rom.physical_identity_digest != session.physical_identity_digest() {
        bail!("physical_identity_drift");
    }
    environment.validate_espflash_identity()?;
    let application = run_installed_application(
        session,
        esptool.as_std_path(),
        environment.espflash_bin.as_std_path(),
    )?;
    let force_download_bit_set = application.force_download_bit_set;
    Ok(InstalledApplicationExit {
        force_download_bit_set,
        transport: application.transport,
        reenumerated: application.reenumerated,
    })
}

pub(super) fn observe(environment: &LocalFlashEnvironment) -> Result<UsbRebootLoopObservation> {
    let mut slot = environment.usb_session.borrow_mut();
    let session = slot.as_mut().context("start_installed=session_missing")?;
    session
        .observe_installed_worker()
        .map_err(anyhow::Error::from)
}
