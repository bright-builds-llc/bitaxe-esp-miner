use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct PendingObservationWake(AtomicBool);

impl PendingObservationWake {
    pub(super) const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub(super) fn mark(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub(super) fn take(&self) -> bool {
        self.0.swap(false, Ordering::AcqRel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marked_wakeup_is_delivered_once() {
        // Arrange
        let wake = PendingObservationWake::new();

        // Act
        wake.mark();

        // Assert
        assert!(wake.take());
        assert!(!wake.take());
    }

    #[test]
    fn multiple_pending_wakeups_coalesce_without_being_lost() {
        // Arrange
        let wake = PendingObservationWake::new();

        // Act
        wake.mark();
        wake.mark();

        // Assert
        assert!(wake.take());
        assert!(!wake.take());
    }
}
