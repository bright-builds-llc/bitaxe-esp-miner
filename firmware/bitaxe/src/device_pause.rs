//! Why the device paused mining itself (ADR-0038), shared by the owner thread that
//! sets it and the API, web and screen readers. It lasts until the operator resumes.
//! While a Gate lease or a campaign owns mining, the indication is hidden: the
//! device is mining for that owner, and the pause applies once it ends.
use bitaxe_core::device_pause::DevicePauseReason;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static REASON: AtomicU8 = AtomicU8::new(0);
static HIDDEN: AtomicBool = AtomicBool::new(false);

/// The device paused itself for `reason`.
pub(crate) fn set(reason: DevicePauseReason) {
    REASON.store(reason.code(), Ordering::Release);
}

/// Ends the device's pause; returns the reason it had, if any.
pub(crate) fn take() -> Option<DevicePauseReason> {
    DevicePauseReason::from_code(REASON.swap(0, Ordering::AcqRel))
}

/// The device's pause, whether or not it is shown.
pub(crate) fn held() -> Option<DevicePauseReason> {
    DevicePauseReason::from_code(REASON.load(Ordering::Acquire))
}

/// Hides the indication while another owner mines.
pub(crate) fn set_hidden(hidden: bool) {
    HIDDEN.store(hidden, Ordering::Release);
}

/// The pause to show now: the held reason, unless another owner mines.
pub(crate) fn current() -> Option<DevicePauseReason> {
    if HIDDEN.load(Ordering::Acquire) {
        return None;
    }
    held()
}
