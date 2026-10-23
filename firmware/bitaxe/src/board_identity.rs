//! The built board compared with the `boardversion` the board stores (ADR-0041).
//!
//! Read once after the settings snapshot loads. The comparison is reported over USB and the
//! API; only a stored value naming another known board withholds hardware control.
use bitaxe_api::{AttestedBoard, MiningSafetyLimits, PlatformAsic, PlatformBoard};
use bitaxe_asic::asic::AsicFamily;
use bitaxe_config::{BoardCatalogEntry, StoredValueKind};
use bitaxe_core::board_identity::{BoardIdentity, KnownBoard, StoredBoard};
use bitaxe_core::{AsicTarget, BoardTarget};
use std::sync::OnceLock;

/// Everything that names the built board, selected together so the facts cannot drift apart.
/// Package admission requires `MARKER` in the application (`tools/flash/src/board_target.rs`).
#[cfg(not(feature = "board-gamma-601"))]
mod built {
    use super::*;
    pub(super) const BOARD: KnownBoard = KnownBoard::Ultra205;
    pub(super) const MARKER: &str = "firmware_board=205";
    pub(super) const TARGET: BoardTarget = BoardTarget::Ultra205;
    pub(super) const ASIC: AsicTarget = AsicTarget::Bm1366;
    pub(super) const ATTESTED: AttestedBoard = AttestedBoard::Ultra205;
    pub(super) const PLATFORM_BOARD: PlatformBoard = PlatformBoard::Ultra205;
    pub(super) const PLATFORM_ASIC: PlatformAsic = PlatformAsic::Bm1366;
    pub(super) const FAMILY: AsicFamily = AsicFamily::Bm1366;
    pub(super) const fn catalog() -> BoardCatalogEntry {
        bitaxe_config::ultra_205_catalog_entry()
    }
}

#[cfg(feature = "board-gamma-601")]
mod built {
    use super::*;
    pub(super) const BOARD: KnownBoard = KnownBoard::Gamma601;
    pub(super) const MARKER: &str = "firmware_board=601";
    pub(super) const TARGET: BoardTarget = BoardTarget::Gamma601;
    pub(super) const ASIC: AsicTarget = AsicTarget::Bm1370;
    pub(super) const ATTESTED: AttestedBoard = AttestedBoard::Gamma601;
    pub(super) const PLATFORM_BOARD: PlatformBoard = PlatformBoard::Gamma601;
    pub(super) const PLATFORM_ASIC: PlatformAsic = PlatformAsic::Bm1370;
    pub(super) const FAMILY: AsicFamily = AsicFamily::Bm1370;
    pub(super) const fn catalog() -> BoardCatalogEntry {
        bitaxe_config::gamma_601_catalog_entry()
    }
}

/// The board this image was built for.
pub(crate) const BUILT: KnownBoard = built::BOARD;
const BUILD_MARKER: &str = built::MARKER;
/// The built board and ASIC for the boot line and the startup screen.
pub(crate) const BOARD_TARGET: BoardTarget = built::TARGET;
pub(crate) const ASIC_TARGET: AsicTarget = built::ASIC;
/// The board the runtime boot attestation names.
pub(crate) const ATTESTED_BOARD: AttestedBoard = built::ATTESTED;
/// The board and ASIC the platform identity reports.
pub(crate) const PLATFORM_BOARD: PlatformBoard = built::PLATFORM_BOARD;
pub(crate) const PLATFORM_ASIC: PlatformAsic = built::PLATFORM_ASIC;
/// The built board's chip family: its job-id step, job interval and result layout.
pub(crate) const ASIC_FAMILY: AsicFamily = built::FAMILY;
/// The ranges the built board's mining-safety facts must hold.
pub(crate) const MINING_SAFETY_LIMITS: MiningSafetyLimits =
    MiningSafetyLimits::for_board(built::catalog());
/// The built board's catalog version, such as "205".
pub(crate) const BOARD_VERSION: &str = built::catalog().board_version();

/// The built board's catalog entry, for the API and the screen.
pub(crate) const fn catalog_entry() -> BoardCatalogEntry {
    built::catalog()
}

/// The chip-address spacing results decode against: the 256 addresses shared over the chain.
pub(crate) fn result_address_interval() -> u16 {
    256 / u16::from(catalog_entry().asic_count())
}

static STORED: OnceLock<StoredBoard> = OnceLock::new();

/// Records what the stored `boardversion` names; later calls keep the first reading.
/// `settings_read` says whether the settings snapshot loaded.
pub(crate) fn initialize(settings_read: bool) {
    let snapshot = crate::settings_adapter::current_settings_snapshot();
    let stored = match snapshot.maybe_stored_value("boardversion") {
        None if !settings_read => StoredBoard::Unreadable,
        None => StoredBoard::Missing,
        Some(stored) => match &stored.value {
            StoredValueKind::String(value) => StoredBoard::from_stored(Some(value)),
            _ => StoredBoard::Other,
        },
    };
    let _ = STORED.set(stored);
    log::info!("{BUILD_MARKER}");
    log::info!("{}", marker());
}

/// What the stored `boardversion` names; `Missing` until read.
pub(crate) fn stored() -> StoredBoard {
    STORED.get().copied().unwrap_or(StoredBoard::Missing)
}

/// How the stored board relates to the built one.
pub(crate) fn identity() -> BoardIdentity {
    BoardIdentity::classify(BUILT, stored())
}

/// Whether hardware control beyond the safe-state actions may run.
pub(crate) fn hardware_control_permitted() -> bool {
    identity().permits_hardware_control()
}

/// The board's own periodic USB record, if its build has one: the Gamma 601 safe boot's.
#[cfg(feature = "board-gamma-601")]
pub(crate) fn maybe_board_runtime_marker() -> Option<String> {
    crate::gamma601::maybe_usb_marker()
}

#[cfg(not(feature = "board-gamma-601"))]
pub(crate) fn maybe_board_runtime_marker() -> Option<String> {
    None
}

/// The periodic USB record.
pub(crate) fn marker() -> String {
    bitaxe_core::board_identity::usb_marker(BUILT, stored())
}
