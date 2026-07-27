use crate::*;

pub(crate) fn emit_flash_outcome(outcome: &FlashOutcome, expose_operational: bool) -> Result<()> {
    if !expose_operational {
        if outcome.manifest.is_some() {
            emit_line("manifest", operational_console_value("", false))?;
        }
        emit_line("flash_image", operational_console_value("", false))?;
        emit_line("flash_command", operational_console_value("", false))?;
        if outcome.nvs_seed.is_some() {
            emit_line("nvs_seed_status", "provided")?;
            emit_line("nvs_seed_image", operational_console_value("", false))?;
            emit_line("nvs_seed_command", operational_console_value("", false))?;
        }
        return Ok(());
    }
    if let Some(manifest) = &outcome.manifest {
        emit_line("manifest", manifest.as_str())?;
    }
    emit_line("flash_image", outcome.flash_image.as_str())?;
    emit_command("flash_command", &outcome.command)?;
    if let Some(nvs_seed) = &outcome.nvs_seed {
        emit_line("nvs_seed_status", "provided")?;
        emit_line("nvs_seed_image", nvs_seed.image.as_str())?;
        emit_command("nvs_seed_command", &nvs_seed.command)?;
    }
    Ok(())
}

pub(crate) fn emit_operational_command(
    label: &str,
    command: &CommandSpec,
    expose_operational: bool,
) -> Result<()> {
    if expose_operational {
        return emit_command(label, command);
    }
    emit_line(label, operational_console_value("", false))
}

pub(crate) fn operational_console_value(value: &str, expose_operational: bool) -> &str {
    if expose_operational {
        return value;
    }
    PROTECTED_OPERATIONAL
}

pub(crate) fn emit_command(label: &str, command: &CommandSpec) -> Result<()> {
    emit_line(label, &command.display())
}

pub(crate) fn write_receive_only_console(bytes: &[u8]) -> Result<()> {
    let mut stdout = io::stdout().lock();
    write_receive_only_console_to(&mut stdout, bytes)
}

pub(crate) fn write_receive_only_console_to(writer: &mut impl Write, bytes: &[u8]) -> Result<()> {
    writer
        .write_all(bytes)
        .context("failed to write receive-only monitor output")?;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        writer
            .write_all(b"\n")
            .context("failed to frame receive-only monitor output")?;
    }
    writer
        .flush()
        .context("failed to flush receive-only monitor output")
}

pub(crate) fn emit_line(label: &str, value: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    emit_line_to(&mut stdout, label, value)
}

pub(crate) fn emit_line_to(writer: &mut impl Write, label: &str, value: &str) -> Result<()> {
    writeln!(writer, "{label}: {value}").context("failed to write command output")
}
