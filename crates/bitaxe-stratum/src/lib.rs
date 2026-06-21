/// Phase 1 Stratum status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StratumRuntimeStatus {
    /// Stratum behavior is deferred to Phase 4.
    DeferredUntilPhase4,
}

#[cfg(test)]
mod tests {
    use super::StratumRuntimeStatus;

    #[test]
    fn stratum_runtime_status_defers_active_behavior_until_phase_4() {
        // Arrange
        let status = StratumRuntimeStatus::DeferredUntilPhase4;

        // Act
        let observed = status;

        // Assert
        assert_eq!(observed, StratumRuntimeStatus::DeferredUntilPhase4);
    }
}
