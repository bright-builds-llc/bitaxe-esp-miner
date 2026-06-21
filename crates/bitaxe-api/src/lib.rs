/// Phase 1 API status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiRuntimeStatus {
    /// API behavior is deferred to Phase 5.
    DeferredUntilPhase5,
}

#[cfg(test)]
mod tests {
    use super::ApiRuntimeStatus;

    #[test]
    fn api_runtime_status_defers_active_behavior_until_phase_5() {
        // Arrange
        let status = ApiRuntimeStatus::DeferredUntilPhase5;

        // Act
        let observed = status;

        // Assert
        assert_eq!(observed, ApiRuntimeStatus::DeferredUntilPhase5);
    }
}
