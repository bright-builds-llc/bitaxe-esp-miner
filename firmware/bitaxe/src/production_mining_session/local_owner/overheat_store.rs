//! Saves and reads what local mining keeps across an overheat (ADR-0038), through the
//! device-owned settings store. The write sets are pure
//! (`bitaxe_runtime::local_mining::persistence`).
use bitaxe_config::NvsWrite;
use bitaxe_runtime::local_mining::persistence::{OverheatBootState, OVERHEAT_STOP_MARKER_KEY};

/// Commits one device-owned write set; the error names what failed, for the log.
pub(super) fn persist(writes: Vec<NvsWrite>) -> Result<(), String> {
    crate::device_settings_store::persist(writes)
}

/// `overheat_mode` and the stop marker as saved at boot. An unreadable marker reads
/// as set, so a failed read keeps an overheat stopped until the operator resumes.
pub(super) fn boot_state() -> OverheatBootState {
    let maybe_marker = match crate::device_settings_store::read_u16(OVERHEAT_STOP_MARKER_KEY) {
        Ok(maybe_marker) => maybe_marker,
        Err(category) => {
            log::warn!("local_mining=overheat_marker_unreadable category={category}");
            Some(1)
        }
    };
    OverheatBootState::read(
        &crate::settings_adapter::current_settings_snapshot(),
        maybe_marker,
    )
}
