//! The built board compared with the `boardversion` the board stores (ADR-0041).
//!
//! Read once after the settings snapshot loads. The comparison is reported over USB and the
//! API; only a stored value naming another known board withholds hardware control.
use bitaxe_config::StoredValueKind;
use bitaxe_core::board_identity::{BoardIdentity, KnownBoard, StoredBoard};
use std::sync::OnceLock;

/// The board this image was built for.
pub(crate) const BUILT: KnownBoard = KnownBoard::Ultra205;

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

/// The periodic USB record.
pub(crate) fn marker() -> String {
    bitaxe_core::board_identity::usb_marker(BUILT, stored())
}
