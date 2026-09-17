/// Independent facts: hardware restoration cannot substitute for network return.
#[derive(Default)]
pub struct ShareRelease {
    worker_returned: bool,
    restored: bool,
    released: bool,
}
impl ShareRelease {
    pub const fn new() -> Self {
        Self {
            worker_returned: false,
            restored: false,
            released: false,
        }
    }
    pub fn worker_returned(&mut self) {
        self.worker_returned = true;
    }
    pub fn restored(&mut self) {
        self.restored = true;
    }
    pub fn waiting_for_worker(&self) -> bool {
        self.restored && !self.worker_returned
    }
    pub fn can_release(&self) -> bool {
        self.worker_returned && self.restored && !self.released
    }
    pub fn mark_released(&mut self) -> bool {
        if !self.can_release() {
            return false;
        }
        self.released = true;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_hardware_restore_does_not_release_an_opaque_network_job() {
        // Arrange
        let mut release = ShareRelease::new();
        // Act / Assert
        release.restored();
        assert!(!release.can_release());
        assert!(!release.mark_released());
        release.worker_returned();
        assert!(release.can_release());
        assert!(release.mark_released());
        assert!(!release.mark_released());
    }
    #[test]
    fn network_return_does_not_skip_ordered_hardware_restoration() {
        // Arrange
        let mut release = ShareRelease::new();
        // Act / Assert
        release.worker_returned();
        assert!(!release.can_release());
        release.restored();
        assert!(release.mark_released());
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;
    #[test]
    fn settings_mutation_remains_excluded_between_restoration_and_actual_return() {
        // Arrange: the same EffectFence used by native settings transactions.
        let fence = crate::noise::EffectFence::new();
        let mut release = ShareRelease::new();
        assert!(fence.claim_diagnostic());
        // Act / Assert
        release.restored();
        assert!(!release.can_release());
        assert!(fence.maybe_mutation().is_none());
        release.worker_returned();
        assert!(release.can_release());
        assert!(fence.release_diagnostic());
        assert!(release.mark_released());
        assert!(fence.maybe_mutation().is_some());
    }
}
