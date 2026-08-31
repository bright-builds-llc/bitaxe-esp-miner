use std::sync::{Mutex, OnceLock};

use bitaxe_api::{
    UsbBootBaseline, UsbBootProfileMarker, UsbBootProfileReason, UsbBootProfileReplay,
    UsbBootTransport,
};

static REPLAY: OnceLock<Mutex<Option<UsbBootProfileReplay>>> = OnceLock::new();

pub(super) fn initialize() {
    REPLAY.get_or_init(|| Mutex::new(None));
}

#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    transport: UsbBootTransport,
    reason: UsbBootProfileReason,
    baseline: UsbBootBaseline,
    firmware_commit: String,
    app_elf_sha256: String,
    boot_ordinal: u64,
    now_ms: u64,
    interval_ms: u64,
) {
    let marker = match UsbBootProfileMarker::new(
        transport,
        reason,
        baseline,
        firmware_commit,
        app_elf_sha256,
        boot_ordinal,
    ) {
        Ok(marker) => marker,
        Err(error) => {
            log::warn!("usb_boot_profile=unavailable reason=invalid_identity error={error}");
            return;
        }
    };
    let replay = UsbBootProfileReplay::new(marker, now_ms, interval_ms);
    let immediate = replay.immediate();
    let cell = REPLAY.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("usb_boot_profile=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = Some(replay);
    drop(maybe_replay);
    log::info!("{immediate}");
}

pub(super) fn emit_due(now_ms: u64) {
    let cell = REPLAY.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        return;
    };
    let Some(replay) = maybe_replay.as_mut() else {
        return;
    };
    if let Some(marker) = replay.maybe_take_due(now_ms) {
        drop(maybe_replay);
        log::info!("{marker}");
    }
}

pub(super) fn next_deadline() -> u64 {
    let cell = REPLAY.get_or_init(|| Mutex::new(None));
    let Ok(maybe_replay) = cell.lock() else {
        return u64::MAX;
    };
    maybe_replay
        .as_ref()
        .map_or(u64::MAX, UsbBootProfileReplay::next_deadline_ms)
}
