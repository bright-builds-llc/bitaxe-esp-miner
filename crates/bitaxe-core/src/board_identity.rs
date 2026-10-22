//! Which board a firmware image was built for, and what the board's own records say (ADR-0041).
//!
//! The firmware compares its built board with the NVS `boardversion` the board stores. The
//! comparison is reported, not authoritative: the flash tool's base-MAC gate decides which
//! package may be written. Only a stored value that names another known board blocks hardware
//! control; a missing or unrecognized value is reported and does not block.

/// A board this project builds firmware for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnownBoard {
    Ultra205,
    Gamma601,
}

impl KnownBoard {
    /// The upstream `boardversion` value for this board (`reference/esp-miner/config-*.cvs`).
    #[must_use]
    pub const fn board_version(self) -> &'static str {
        match self {
            Self::Ultra205 => "205",
            Self::Gamma601 => "601",
        }
    }

    /// The board a stored `boardversion` names, when it is a known one.
    #[must_use]
    pub fn from_board_version(value: &str) -> Option<Self> {
        [Self::Ultra205, Self::Gamma601]
            .into_iter()
            .find(|board| board.board_version() == value)
    }
}

/// What the board's stored `boardversion` names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredBoard {
    Known(KnownBoard),
    /// No `boardversion` is stored.
    Missing,
    /// A value that names no known board; it is never echoed.
    Other,
    /// The settings could not be read, so whether a value is stored is unknown.
    Unreadable,
}

impl StoredBoard {
    /// Classifies the stored `boardversion`, if any.
    #[must_use]
    pub fn from_stored(maybe_stored: Option<&str>) -> Self {
        match maybe_stored {
            None => Self::Missing,
            Some(value) => KnownBoard::from_board_version(value).map_or(Self::Other, Self::Known),
        }
    }

    /// The closed label: a known board's version, `missing`, `other` or `unreadable`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Known(board) => board.board_version(),
            Self::Missing => "missing",
            Self::Other => "other",
            Self::Unreadable => "unreadable",
        }
    }
}

/// How the stored `boardversion` relates to the built board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardIdentity {
    /// The stored value names the built board.
    Match,
    /// The stored value names another known board.
    OtherBoard(KnownBoard),
    /// The value is missing, unreadable or names no known board.
    Unknown,
}

impl BoardIdentity {
    /// Compares the built board with the stored one.
    #[must_use]
    pub fn classify(built: KnownBoard, stored: StoredBoard) -> Self {
        match stored {
            StoredBoard::Known(board) if board == built => Self::Match,
            StoredBoard::Known(board) => Self::OtherBoard(board),
            StoredBoard::Missing | StoredBoard::Other | StoredBoard::Unreadable => Self::Unknown,
        }
    }

    /// The closed label for USB records and the API.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::OtherBoard(_) => "mismatch",
            Self::Unknown => "unknown",
        }
    }

    /// Whether hardware control beyond the safe-state actions may run.
    #[must_use]
    pub const fn permits_hardware_control(self) -> bool {
        !matches!(self, Self::OtherBoard(_))
    }
}

/// The periodic USB record naming the built board, the stored board and their relation.
#[must_use]
pub fn usb_marker(built: KnownBoard, stored: StoredBoard) -> String {
    format!(
        "usb_board_identity schema=v1 built={} stored={} identity={} redacted=true",
        built.board_version(),
        stored.label(),
        BoardIdentity::classify(built, stored).label(),
    )
}

/// Each board's firmware carries `firmware_board=<board_version>` (ADR-0041). Only the firmware
/// may hold a complete marker, so the scanner below never stores one: it keeps the prefix
/// reversed and matches the version separately, and no build's bytes spell another board's
/// marker through this code.
const MARKER_PREFIX_REVERSED: &[u8] = b"=draob_erawmrif";
const MARKER_LENGTH: usize = MARKER_PREFIX_REVERSED.len() + 3;
const BOARDS: [KnownBoard; 2] = [KnownBoard::Ultra205, KnownBoard::Gamma601];

fn is_marker(window: &[u8], board: KnownBoard) -> bool {
    let (prefix, version) = window.split_at(MARKER_PREFIX_REVERSED.len());
    prefix.iter().eq(MARKER_PREFIX_REVERSED.iter().rev())
        && version == board.board_version().as_bytes()
}

/// Finds board markers in an image that arrives in chunks, such as an HTTP firmware update.
#[derive(Debug, Default)]
pub struct FirmwareMarkerScanner {
    /// The last bytes seen, so a marker split across two chunks is still found.
    tail: Vec<u8>,
    seen: [bool; 2],
}

impl FirmwareMarkerScanner {
    pub fn feed(&mut self, chunk: &[u8]) {
        let head = &chunk[..chunk.len().min(MARKER_LENGTH - 1)];
        let mut boundary = std::mem::take(&mut self.tail);
        boundary.extend_from_slice(head);
        for (index, board) in BOARDS.into_iter().enumerate() {
            let found = |bytes: &[u8]| {
                bytes
                    .windows(MARKER_LENGTH)
                    .any(|window| is_marker(window, board))
            };
            self.seen[index] |= found(&boundary) || found(chunk);
        }
        let mut joined = boundary;
        joined.extend_from_slice(&chunk[head.len()..]);
        let keep = joined.len().saturating_sub(MARKER_LENGTH - 1);
        self.tail = joined.split_off(keep);
    }

    /// Whether the image may be installed on `built`: it carries no other known board's marker.
    /// An image with no marker (an upstream release, or one that predates the markers) is
    /// admitted, since nothing in it names another board.
    #[must_use]
    pub fn admits(&self, built: KnownBoard) -> bool {
        BOARDS
            .into_iter()
            .zip(self.seen)
            .all(|(board, seen)| board == built || !seen)
    }

    /// Whether the image carries `board`'s marker.
    #[must_use]
    pub fn carries(&self, board: KnownBoard) -> bool {
        BOARDS
            .into_iter()
            .zip(self.seen)
            .any(|(candidate, seen)| candidate == board && seen)
    }
}

#[cfg(test)]
mod tests;
