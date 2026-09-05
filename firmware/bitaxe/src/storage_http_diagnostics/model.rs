use bitaxe_core::usb_diagnostics::{
    StorageHttpError, StorageHttpFailure, StorageHttpOutcome, StorageHttpPhase,
};
use std::sync::atomic::{AtomicU32, Ordering};

pub(super) struct StartupResults {
    first_failure: AtomicU32,
    outcomes: AtomicU32,
}
impl StartupResults {
    pub(super) const fn new() -> Self {
        Self {
            first_failure: AtomicU32::new(0),
            outcomes: AtomicU32::new(0),
        }
    }
    pub(super) fn failure(&self, phase: StorageHttpPhase, error: StorageHttpError) -> bool {
        self.first_failure
            .compare_exchange(
                0,
                u32::from(phase as u8) | (u32::from(error as u8) << 8),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
    pub(super) fn failure_marker(&self) -> Option<String> {
        let word = self.first_failure.load(Ordering::Acquire);
        Some(
            StorageHttpFailure {
                phase: StorageHttpPhase::from_code(word as u8)?,
                error: StorageHttpError::from_code((word >> 8) as u8)?,
            }
            .marker(),
        )
    }
    pub(super) fn filesystem(&self, available: bool) {
        self.outcome(1, 2, available);
    }
    pub(super) fn http(&self, ready: bool) {
        self.outcome(4, 8, ready);
    }
    fn outcome(&self, observed: u32, ready: u32, value: bool) {
        self.outcomes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |flags| {
                Some((flags & !ready) | observed | if value { ready } else { 0 })
            })
            .expect("outcome update always supplies a value");
    }
    pub(super) fn status_marker(&self) -> Option<String> {
        let flags = self.outcomes.load(Ordering::Acquire);
        (flags & 5 == 5).then(|| {
            StorageHttpOutcome {
                spiffs_available: flags & 2 != 0,
                http_ready: flags & 8 != 0,
            }
            .marker()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_failure_survives_later_failure_and_outcome_publication() {
        // Arrange
        let state = StartupResults::new();
        // Act
        assert!(state.failure(
            StorageHttpPhase::SpiffsRegister,
            StorageHttpError::MountFailed
        ));
        assert!(!state.failure(StorageHttpPhase::HttpServer, StorageHttpError::NoMemory));
        state.filesystem(false);
        assert_eq!(state.status_marker(), None);
        state.http(true);
        // Assert
        assert_eq!(state.failure_marker().as_deref(),Some("storage_http_failure schema=v1 phase=spiffs_register error=mount_failed redacted=true"));
        assert_eq!(state.status_marker().as_deref(),Some("storage_http_status schema=v1 spiffs_available=false http_ready=true redacted=true"));
    }
}
