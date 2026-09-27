//! Narrow retained-ROM erase: fresh proof on the current physical lease, no reset.
use super::*;

pub(super) fn erase(environment: &LocalFlashEnvironment, offset: u32, size: u32) -> Result<()> {
    if offset != 0xf12000 || !matches!(size, 0x10000 | 0xee000) {
        bail!("core_dump_erase_range");
    }
    environment.validate_espflash_identity()?;
    let mut slot = environment.usb_session.borrow_mut();
    let session = slot.as_mut().context("core_dump_erase_session_missing")?;
    let before = inspect_usb_profile(session.port())?;
    if before.physical_identity_digest != session.physical_identity_digest() {
        bail!("physical_identity_drift");
    }
    let proof = session.run_espflash_probe(
        environment.espflash_bin.as_std_path(),
        &installed_rom_probe_args(session.port()),
        Duration::from_secs(30),
    )?;
    let mut board_info = proof.stdout;
    board_info.extend_from_slice(&proof.stderr);
    let rom = admit_rom_downloader(inspect_usb_profile(session.port())?, &board_info)?;
    if rom.physical_identity_digest != session.physical_identity_digest() {
        bail!("physical_identity_drift");
    }
    environment.validate_espflash_identity()?;
    let command = crate::core_dump::clearing::erase_command(session.port(), offset, size);
    let result = session.run_espflash(
        environment.espflash_bin.as_std_path(),
        &command.args,
        Duration::from_secs(360),
    );
    let after = inspect_usb_profile(session.port());
    result?;
    if after?.physical_identity_digest != session.physical_identity_digest() {
        bail!("physical_identity_drift");
    }
    Ok(())
}
