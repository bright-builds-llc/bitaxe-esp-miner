use bitaxe_core::usb_diagnostics::{
    network_startup_failure_marker, NetworkStartupError, NetworkStartupPhase,
};
use std::sync::atomic::{AtomicU32, Ordering};

pub(super) struct FirstFailure(AtomicU32);
impl FirstFailure {
    pub(super) const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    pub(super) fn record(&self, phase: NetworkStartupPhase, error: NetworkStartupError) -> bool {
        self.0
            .compare_exchange(
                0,
                u32::from(phase as u8) | (u32::from(error as u8) << 8),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
    pub(super) fn marker(&self) -> Option<String> {
        let code = self.0.load(Ordering::Acquire);
        Some(network_startup_failure_marker(
            NetworkStartupPhase::from_code(code as u8)?,
            NetworkStartupError::from_code((code >> 8) as u8)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn later_errors_cannot_replace_or_erase_the_first_failure() {
        // Arrange
        let failure = FirstFailure::new();
        assert_eq!(failure.marker(), None);
        // Act
        assert!(failure.record(
            NetworkStartupPhase::DriverStart,
            NetworkStartupError::NoMemory
        ));
        assert!(!failure.record(
            NetworkStartupPhase::ReconnectSpawn,
            NetworkStartupError::IoError
        ));
        // Assert
        assert_eq!(
            failure.marker().as_deref(),
            Some("wifi_startup_failure schema=v1 phase=driver_start error=no_memory redacted=true")
        );
    }
}
