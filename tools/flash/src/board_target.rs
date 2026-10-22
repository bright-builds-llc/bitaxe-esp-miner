//! The board a command targets, and the package names and markers that belong to it (ADR-0041).
//!
//! Only `flash`, `flash-monitor`, `monitor` and `detect` accept the Gamma 601
//! (`parse_flash_board`); every other command parses with `parse_board` and stays Ultra 205
//! only. A package must name the requested board in its manifest and carry that board's marker,
//! and no other board's, in its application bytes. Packages are admitted only when built from
//! the current source, which always writes both.

use crate::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoardId {
    Ultra205,
    Gamma601,
}

impl FromStr for BoardId {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "205" => Ok(Self::Ultra205),
            "601" => Ok(Self::Gamma601),
            other => Err(format!("unsupported board {other}; boards are 205 and 601")),
        }
    }
}

impl fmt::Display for BoardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.registered().label())
    }
}

/// Parses a board for a command that supports the Ultra 205 only.
pub(crate) fn parse_board(value: &str) -> std::result::Result<BoardId, String> {
    match value.parse()? {
        BoardId::Ultra205 => Ok(BoardId::Ultra205),
        BoardId::Gamma601 => {
            Err("board 601 is accepted only by flash, flash-monitor, monitor and detect".to_owned())
        }
    }
}

/// Parses a board for `flash`, `flash-monitor`, `monitor` and `detect`.
pub(crate) fn parse_flash_board(value: &str) -> std::result::Result<BoardId, String> {
    value.parse()
}

impl BoardId {
    /// The registry entry that ROM admission requires for this board.
    pub(crate) const fn registered(self) -> crate::board_identity::RegisteredBoard {
        match self {
            Self::Ultra205 => crate::board_identity::RegisteredBoard::Ultra205,
            Self::Gamma601 => crate::board_identity::RegisteredBoard::Gamma601,
        }
    }

    /// The Bazel target that builds this board's package.
    pub(crate) const fn package_build_target(self) -> &'static str {
        match self {
            Self::Ultra205 => PACKAGE_BUILD_TARGET,
            Self::Gamma601 => "//firmware/bitaxe:firmware_image_gamma601",
        }
    }

    /// The package manifest's path under `bazel-bin`.
    pub(crate) const fn package_manifest_relative_path(self) -> &'static str {
        match self {
            Self::Ultra205 => PACKAGE_MANIFEST_RELATIVE_PATH,
            Self::Gamma601 => "firmware/bitaxe/bitaxe-gamma601-package.json",
        }
    }

    pub(crate) const fn default_elf_name(self) -> &'static str {
        match self {
            Self::Ultra205 => DEFAULT_ELF_NAME,
            Self::Gamma601 => "bitaxe-gamma601.elf",
        }
    }

    pub(crate) const fn factory_image_name(self) -> &'static str {
        match self {
            Self::Ultra205 => FACTORY_IMAGE_NAME,
            Self::Gamma601 => "bitaxe-gamma601-factory.bin",
        }
    }

    /// The manifest's execution profile for a physical install.
    pub(crate) const fn physical_execution_profile(self) -> &'static str {
        match self {
            Self::Ultra205 => "physical-ultra205",
            Self::Gamma601 => "physical-gamma601",
        }
    }

    /// The literal the firmware compiles in for this board (`firmware/bitaxe/src/board_identity.rs`).
    const fn firmware_marker(self) -> &'static [u8] {
        match self {
            Self::Ultra205 => b"firmware_board=205",
            Self::Gamma601 => b"firmware_board=601",
        }
    }
}

/// Requires the manifest to name the requested board.
pub(crate) fn require_manifest_board(board: BoardId, maybe_recorded: Option<&str>) -> Result<()> {
    match maybe_recorded {
        Some(recorded) if recorded == board.registered().label() => Ok(()),
        Some(_) => bail!("identity_admission=blocked reason=manifest_board_mismatch"),
        None => bail!("identity_admission=blocked reason=manifest_board_missing"),
    }
}

/// Requires the application bytes to carry the requested board's marker and no other board's.
pub(crate) fn require_board_bytes(board: BoardId, application: &[u8]) -> Result<()> {
    let carries = |candidate: BoardId| {
        let marker = candidate.firmware_marker();
        application
            .windows(marker.len())
            .any(|window| window == marker)
    };
    let other = match board {
        BoardId::Ultra205 => BoardId::Gamma601,
        BoardId::Gamma601 => BoardId::Ultra205,
    };
    if carries(other) {
        bail!("identity_admission=blocked reason=firmware_board_mismatch");
    }
    if !carries(board) {
        bail!("identity_admission=blocked reason=firmware_board_marker_missing");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
