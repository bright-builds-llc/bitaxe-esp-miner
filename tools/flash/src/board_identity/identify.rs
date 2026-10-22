//! `identify-board`: names an attached board from its own records, read-only (ADR-0041).
//!
//! One reset into the ROM downloader yields the chip and base MAC. One read of the NVS
//! partition, with a hard reset back to the installed application, yields the board fields the
//! upstream firmware stores. The partition holds Wi-Fi and pool secrets (`NeverPersistRaw`), so
//! it passes through private pipes only (`memory_fifo`) and is never stored. Only
//! `boardversion`, `asicmodel` and `devicemodel` are kept. With `--register`, the board is
//! registered only when all three name the requested board. Nothing is written to the device.

use super::*;

/// The upstream NVS namespace that holds the board's configuration.
const NVS_NAMESPACE: &str = "main";
const NVS_OFFSET: u32 = 0x9000;
const NVS_SIZE: u32 = 0x6000;
/// Bounds `nvs_tool`'s JSON dump.
const MAX_DUMP_BYTES: usize = 1_048_576;
/// The contract line that enables this command in the bring-up task's active block.
pub(crate) const ENABLE_LINE: &str = "Gamma 601 identification hardware: enabled.";
const TASK: &str = "task-gamma-601-bring-up";

#[derive(Debug, Parser, Clone)]
pub(crate) struct IdentifyBoardCommand {
    /// The board the attached device is expected to be.
    #[arg(long, value_enum)]
    pub(crate) board: RegisteredBoard,
    #[arg(long)]
    pub(crate) port: String,
    /// Register the device's digest when its own records name the expected board.
    #[arg(long)]
    pub(crate) register: bool,
}

/// The board fields the upstream firmware stores, as found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BoardFields {
    pub(crate) boardversion: Option<String>,
    pub(crate) asicmodel: Option<String>,
    pub(crate) devicemodel: Option<String>,
}

/// The fields a stock upstream install stores for each board (`reference/esp-miner/config-*.cvs`).
pub(crate) fn expected_fields(board: RegisteredBoard) -> BoardFields {
    let (version, asic, model) = match board {
        RegisteredBoard::Ultra205 => ("205", "BM1366", "ultra"),
        RegisteredBoard::Gamma601 => ("601", "BM1370", "gamma"),
    };
    BoardFields {
        boardversion: Some(version.to_owned()),
        asicmodel: Some(asic.to_owned()),
        devicemodel: Some(model.to_owned()),
    }
}

#[derive(Debug, Deserialize)]
struct DumpEntry {
    namespace: String,
    key: String,
    data: serde_json::Value,
}

/// Picks the three board fields out of `nvs_tool`'s JSON dump and drops everything else. Its
/// integrity diagnostics are ignored: the fields must still match a board exactly.
pub(crate) fn board_fields(dump: &[u8]) -> Result<BoardFields> {
    if dump.is_empty() || dump.len() > MAX_DUMP_BYTES {
        bail!("board_identify=terminal reason=nvs_dump_size");
    }
    let entries = serde_json::Deserializer::from_slice(dump)
        .into_iter::<Vec<DumpEntry>>()
        .next()
        .and_then(|parsed| parsed.ok())
        .context("board_identify=terminal reason=nvs_dump_invalid")?;
    let value = |key: &str| {
        entries
            .iter()
            .filter(|entry| entry.namespace == NVS_NAMESPACE && entry.key == key)
            .find_map(|entry| entry.data.as_str().map(str::to_owned))
    };
    Ok(BoardFields {
        boardversion: value("boardversion"),
        asicmodel: value("asicmodel"),
        devicemodel: value("devicemodel"),
    })
}

/// What identification concludes: the registration step, or why it stops.
pub(crate) fn decide(
    requested: RegisteredBoard,
    found: &BoardFields,
    standing: Admission,
    register: bool,
) -> Result<&'static str> {
    if found.boardversion.is_none() || found.asicmodel.is_none() || found.devicemodel.is_none() {
        bail!("board_identify=terminal reason=records_missing");
    }
    if *found != expected_fields(requested) {
        bail!(
            "board_identify=terminal reason=records_name_another_board requested={}",
            requested.label()
        );
    }
    match (register, standing) {
        (false, _) => Ok("not_requested"),
        (true, Admission::Admitted) => Ok("already_registered"),
        (true, Admission::Unregistered) => Ok("register"),
        (true, Admission::OtherBoard(registered)) => bail!(
            "board_identity=blocked reason=registered_as_other_board registered={}",
            registered.label()
        ),
    }
}

/// Whether the bring-up task is active and carries this command's enable line in its block.
pub(crate) fn contract_enabled(tasks: &str) -> bool {
    let mut section = "";
    let mut in_block = false;
    let mut enabled = false;
    for line in tasks.lines() {
        if line.starts_with("## ") {
            section = line;
            in_block = false;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            in_block = section == "## Active" && heading.split_whitespace().next() == Some(TASK);
        }
        if in_block && line.trim() == ENABLE_LINE {
            enabled = true;
        }
    }
    enabled
}

/// The flash size `board-info` reports, when it is one of the sizes these boards ship with.
pub(crate) fn flash_size(board_info: &[u8]) -> &'static str {
    let text = String::from_utf8_lossy(board_info);
    let reported = text
        .lines()
        .find_map(|line| line.trim().strip_prefix("Flash size:").map(str::trim));
    match reported {
        Some("16MB") => "16MB",
        Some("8MB") => "8MB",
        Some("4MB") => "4MB",
        Some(_) => "other",
        None => "missing",
    }
}

/// A label for a field: its value when it is one of the known board values, else a category.
fn field_label(value: Option<&String>) -> &str {
    const KNOWN: [&str; 6] = ["205", "601", "BM1366", "BM1370", "ultra", "gamma"];
    match value {
        Some(text) => KNOWN
            .iter()
            .find(|known| **known == text)
            .copied()
            .unwrap_or("other"),
        None => "missing",
    }
}

pub(crate) fn run_identify(
    command: &IdentifyBoardCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if !contract_enabled(&tasks) {
        bail!("board_identify=blocked reason=contract_not_enabled");
    }
    let workspace = environment.workspace_path(Utf8Path::new("."));
    let scratch = workspace.join("scratch");
    // Everything that can refuse without the device does so before the reset into ROM.
    let entries = load_registry(&workspace.join(REGISTRY_FILE))?;
    let python = crate::nvs_readback::validate_managed_nvs_python(environment)?;
    let nvs_tool = crate::nvs_readback::validate_managed_tool(
        environment,
        crate::nvs_readback::MANAGED_NVS_TOOL,
    )?;
    let esptool = crate::nvs_readback::find_managed_esptool(environment)?;
    let pipe = crate::memory_fifo::PrivateFifo::create(&scratch, "nvs-read.fifo")?;
    let read = ManagedFlashRead::new(
        esptool,
        NVS_OFFSET,
        NVS_SIZE,
        pipe.path().to_owned(),
        FlashReadDisposition::ReturnToApplication,
    )?;

    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    let board_info = environment.admit_rom_for_identification()?;
    let digest = base_mac_digest(base_mac(&board_info)?);
    emit_line("board_identify_flash_size", flash_size(&board_info))?;
    let standing = admission(&entries, command.board, &digest);
    if let Admission::OtherBoard(registered) = standing {
        bail!(
            "board_identity=blocked reason=registered_as_other_board requested={} registered={}",
            command.board.label(),
            registered.label()
        );
    }
    let image = crate::memory_fifo::read_while(&pipe, NVS_SIZE as usize, || {
        environment.execute_flash_read_to_pipe(&read)
    })?;
    environment.finish_usb_session()?;
    if image.len() != NVS_SIZE as usize {
        bail!("board_identify=terminal reason=nvs_read_size");
    }

    let dump = crate::nvs_readback::run_nvs_dump_from_memory(&python, &nvs_tool, image, &scratch)?;
    let found = board_fields(&dump)?;
    emit_line(
        "board_identify_boardversion",
        field_label(found.boardversion.as_ref()),
    )?;
    emit_line(
        "board_identify_asicmodel",
        field_label(found.asicmodel.as_ref()),
    )?;
    emit_line(
        "board_identify_devicemodel",
        field_label(found.devicemodel.as_ref()),
    )?;
    let outcome = decide(command.board, &found, standing, command.register)?;
    emit_line("board_identify_matches", "true")?;
    let registration = if outcome == "register" {
        register_digest(&workspace, command.board, digest)?
    } else {
        outcome
    };
    emit_line("board_identify_registration", registration)
}

#[cfg(test)]
mod tests;
