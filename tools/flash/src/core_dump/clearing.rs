//! Explicit archive-bound core partition clearing; acquisition never implies erase.
use super::*;

#[derive(Debug, Parser)]
pub(crate) struct CoreDumpClearCommand {
    #[command(flatten)]
    pub(crate) common: CoreDumpReadCommand,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) preserved_dump: Utf8PathBuf,
    #[arg(long)]
    pub(crate) preserved_sha256: String,
}

pub(crate) fn run(
    command: &CoreDumpClearCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    run_mode(&command.common, environment, Some(command))
}

pub(super) fn archive(command: &CoreDumpClearCommand, workspace: &Utf8Path) -> Result<Vec<u8>> {
    let bytes = proof::private_file(&command.preserved_dump, workspace, 0xee000)?;
    if !matches!(bytes.len(), 0x10000 | 0xee000)
        || !contract::hex(&command.preserved_sha256, 64)
        || sha256_bytes(&bytes) != command.preserved_sha256
    {
        bail!("preserved_dump_invalid");
    }
    Ok(bytes)
}

pub(super) fn erase(
    command: &CoreDumpReadCommand,
    environment: &impl FlashEnvironment,
    offset: u32,
    size: u32,
) -> Result<()> {
    require_physical(command, environment)?;
    environment.execute_core_dump_erase(offset, size)?;
    require_physical(command, environment)
}

pub(crate) fn erase_command(port: &str, offset: u32, size: u32) -> CommandSpec {
    CommandSpec::new(
        "espflash",
        [
            "erase-region",
            "--before",
            "no-reset",
            "--after",
            "no-reset",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--skip-update-check",
            &format!("0x{offset:x}"),
            &format!("0x{size:x}"),
        ],
    )
}
