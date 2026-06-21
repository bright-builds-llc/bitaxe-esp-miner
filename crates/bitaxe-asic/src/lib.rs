/// Phase 1 ASIC status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicRuntimeStatus {
    /// ASIC behavior is deferred to Phase 3.
    DeferredUntilPhase3,
}

#[cfg(test)]
mod tests {
    use super::AsicRuntimeStatus;

    #[test]
    fn asic_runtime_status_defers_active_behavior_until_phase_3() {
        // Arrange
        let status = AsicRuntimeStatus::DeferredUntilPhase3;

        // Act
        let observed = status;

        // Assert
        assert_eq!(observed, AsicRuntimeStatus::DeferredUntilPhase3);
    }
}
