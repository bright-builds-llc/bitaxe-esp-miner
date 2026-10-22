//! Board identity from the device itself (ADR-0041).
//!
//! The Ultra 205 and the Gamma 601 both enumerate as the same Espressif USB device, and the
//! detector's physical identity names the host USB port, not the board. Before any effect that
//! needs the ROM downloader, the admitted device's ESP32-S3 base MAC must be registered for the
//! requested board. The local registry keeps only a domain-separated digest of each MAC.

use crate::*;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

/// The registry's file name at the workspace root; it is gitignored and private.
pub(crate) const REGISTRY_FILE: &str = "boards.local.json";
const REGISTRY_SCHEMA: &str = "bitaxe-board-registry-v1";
/// Separates these digests from every other MAC digest the tools compute.
const DIGEST_DOMAIN: &str = "bitaxe-board-base-mac-v1";

/// The boards a registry entry may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub(crate) enum RegisteredBoard {
    #[serde(rename = "205")]
    #[value(name = "205")]
    Ultra205,
    #[serde(rename = "601")]
    #[value(name = "601")]
    Gamma601,
}

impl RegisteredBoard {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ultra205 => "205",
            Self::Gamma601 => "601",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryEntry {
    pub(crate) board: RegisteredBoard,
    pub(crate) base_mac_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryDocument {
    schema: String,
    boards: Vec<RegistryEntry>,
}

/// How a device's digest stands against the registry for one requested board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Admission {
    Admitted,
    Unregistered,
    OtherBoard(RegisteredBoard),
}

/// The single base MAC that `espflash board-info` reports.
pub(crate) fn base_mac(board_info: &[u8]) -> Result<[u8; 6]> {
    let document =
        std::str::from_utf8(board_info).context("board-info output was not valid UTF-8")?;
    let identities = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix("MAC address:").map(str::trim))
        .collect::<Vec<_>>();
    let [identity] = identities.as_slice() else {
        bail!("board-info output did not contain exactly one base MAC identity");
    };
    let octets = identity
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .collect::<Result<Vec<_>, _>>()
        .context("board-info base MAC identity was malformed")?;
    <[u8; 6]>::try_from(octets.as_slice()).context("board-info base MAC identity was malformed")
}

/// The registry's digest of a base MAC; the MAC itself is never stored.
pub(crate) fn base_mac_digest(mac: [u8; 6]) -> String {
    let text = mac.map(|octet| format!("{octet:02x}")).join(":");
    sha256_bytes(format!("{DIGEST_DOMAIN}\n{text}").as_bytes())
}

pub(crate) fn admission(
    entries: &[RegistryEntry],
    requested: RegisteredBoard,
    digest: &str,
) -> Admission {
    match entries.iter().find(|entry| entry.base_mac_sha256 == digest) {
        Some(entry) if entry.board == requested => Admission::Admitted,
        Some(entry) => Admission::OtherBoard(entry.board),
        None => Admission::Unregistered,
    }
}

/// A private file's bytes: a regular mode-0600 file, read through the very file that was
/// checked, so a swap between the check and the read refuses. `None` when it is missing.
fn read_private(path: &Utf8Path, reason: &str) -> Result<Option<Vec<u8>>> {
    let checked = match fs::symlink_metadata(path.as_std_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context(format!("board_identity=blocked reason={reason}")),
    };
    if !checked.is_file() || checked.permissions().mode() & 0o777 != 0o600 {
        bail!("board_identity=blocked reason={reason}");
    }
    let mut file = fs::File::open(path.as_std_path())
        .with_context(|| format!("board_identity=blocked reason={reason}"))?;
    let opened = file.metadata()?;
    if (opened.dev(), opened.ino()) != (checked.dev(), checked.ino()) {
        bail!("board_identity=blocked reason={reason}");
    }
    let mut bytes = Vec::new();
    io::Read::read_to_end(&mut file, &mut bytes)?;
    Ok(Some(bytes))
}

/// The registry's entries; a missing registry has none, so every device is unregistered.
pub(crate) fn load_registry(path: &Utf8Path) -> Result<Vec<RegistryEntry>> {
    let Some(bytes) = read_private(path, "registry_not_private")? else {
        return Ok(Vec::new());
    };
    let document: RegistryDocument = serde_json::from_slice(&bytes)
        .context("board_identity=blocked reason=registry_malformed")?;
    if document.schema != REGISTRY_SCHEMA {
        bail!("board_identity=blocked reason=registry_schema");
    }
    let mut digests = BTreeSet::new();
    for entry in &document.boards {
        let valid = entry.base_mac_sha256.len() == 64
            && entry
                .base_mac_sha256
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
        if !valid || !digests.insert(entry.base_mac_sha256.as_str()) {
            bail!("board_identity=blocked reason=registry_malformed");
        }
    }
    Ok(document.boards)
}

/// Refuses before the reset into the ROM downloader when no device is registered for
/// `requested`, so a missing registry never strands a board in ROM.
pub(crate) fn require_registered(workspace: &Utf8Path, requested: RegisteredBoard) -> Result<()> {
    let entries = load_registry(&workspace.join(REGISTRY_FILE))?;
    if entries.iter().any(|entry| entry.board == requested) {
        return Ok(());
    }
    bail!(
        "board_identity=blocked reason=none_registered requested={}",
        requested.label()
    )
}

/// Refuses, before any ROM-downloader effect, a device not registered for `requested`.
pub(crate) fn require_board(
    workspace: &Utf8Path,
    requested: RegisteredBoard,
    board_info: &[u8],
) -> Result<()> {
    let digest = base_mac_digest(
        base_mac(board_info).context("board_identity=blocked reason=mac_unreadable")?,
    );
    match admission(
        &load_registry(&workspace.join(REGISTRY_FILE))?,
        requested,
        &digest,
    ) {
        Admission::Admitted => emit_line("board_identity", requested.label()),
        Admission::Unregistered => {
            bail!(
                "board_identity=blocked reason=unregistered requested={}",
                requested.label()
            )
        }
        Admission::OtherBoard(registered) => bail!(
            "board_identity=blocked reason=registered_as_other_board requested={} registered={}",
            requested.label(),
            registered.label()
        ),
    }
}

/// `register-board`: records a board's base-MAC digest from a private `board-info` output.
#[derive(Debug, Parser, Clone)]
pub(crate) struct RegisterBoardCommand {
    #[arg(long, value_enum)]
    pub(crate) board: RegisteredBoard,
    /// A private (mode 0600) file holding that board's `espflash board-info` output.
    #[arg(long = "board-info", value_parser = parse_utf8_path)]
    pub(crate) board_info: Utf8PathBuf,
}

pub(crate) fn run_register(command: &RegisterBoardCommand) -> Result<()> {
    let workspace = detect_workspace_dir()?;
    // Under `bazel run` the working directory is the runfiles tree, so a relative path is the
    // workspace's.
    let board_info = if command.board_info.is_absolute() {
        command.board_info.clone()
    } else {
        workspace.join(&command.board_info)
    };
    let outcome = register(&workspace, command.board, &board_info)?;
    emit_line("board_identity_registered", command.board.label())?;
    emit_line("board_identity_registration", outcome)
}

/// Adds the board's digest, or confirms it; a digest registered for another board refuses.
pub(crate) fn register(
    workspace: &Utf8Path,
    board: RegisteredBoard,
    board_info_path: &Utf8Path,
) -> Result<&'static str> {
    let Some(board_info) = read_private(board_info_path, "board_info_not_private")? else {
        bail!("board_identity=blocked reason=board_info_missing");
    };
    register_digest(workspace, board, base_mac_digest(base_mac(&board_info)?))
}

/// Adds a digest for the board, or confirms it; a digest registered for another board refuses.
pub(crate) fn register_digest(
    workspace: &Utf8Path,
    board: RegisteredBoard,
    digest: String,
) -> Result<&'static str> {
    let path = workspace.join(REGISTRY_FILE);
    let mut entries = load_registry(&path)?;
    match admission(&entries, board, &digest) {
        Admission::Admitted => return Ok("already_registered"),
        Admission::OtherBoard(registered) => {
            bail!(
                "board_identity=blocked reason=registered_as_other_board registered={}",
                registered.label()
            )
        }
        Admission::Unregistered => entries.push(RegistryEntry {
            board,
            base_mac_sha256: digest,
        }),
    }
    let document = RegistryDocument {
        schema: REGISTRY_SCHEMA.to_owned(),
        boards: entries,
    };
    let staging = workspace.join(format!(".{REGISTRY_FILE}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(staging.as_std_path())?;
    file.write_all(&serde_json::to_vec_pretty(&document)?)?;
    file.sync_all()?;
    fs::rename(staging.as_std_path(), path.as_std_path())?;
    Ok("added")
}

pub(crate) mod identify;

#[cfg(test)]
mod tests;
