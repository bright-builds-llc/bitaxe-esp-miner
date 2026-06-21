use bitaxe_core::{Phase1SafeState, PHASE1_SAFE_STATE_LOG_LINE};

/// Returns the exact Phase 1 safe-state log line.
#[must_use]
pub const fn phase1_safe_state_log_line() -> &'static str {
    PHASE1_SAFE_STATE_LOG_LINE
}

/// Asserts that a log line matches the exact Phase 1 safe-state contract.
#[track_caller]
pub fn assert_phase1_safe_state_log_line(actual: &str) {
    assert_eq!(actual, PHASE1_SAFE_STATE_LOG_LINE);
}

/// Asserts that a core safe-state value emits the exact Phase 1 log line.
#[track_caller]
pub fn assert_phase1_safe_state(safe_state: &Phase1SafeState) {
    assert_phase1_safe_state_log_line(safe_state.log_line());
}

#[cfg(test)]
mod tests {
    use bitaxe_core::{Phase1SafeState, PHASE1_SAFE_STATE_LOG_LINE};

    use super::{
        assert_phase1_safe_state, assert_phase1_safe_state_log_line, phase1_safe_state_log_line,
    };

    #[test]
    fn exposes_exact_phase_1_safe_state_log_line() {
        // Arrange
        let expected = PHASE1_SAFE_STATE_LOG_LINE;

        // Act
        let log_line = phase1_safe_state_log_line();

        // Assert
        assert_eq!(log_line, expected);
    }

    #[test]
    fn asserts_exact_phase_1_safe_state_log_line() {
        // Arrange
        let log_line =
            "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";

        // Act
        let result = std::panic::catch_unwind(|| assert_phase1_safe_state_log_line(log_line));

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn asserts_phase_1_safe_state_from_core_contract() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let result = std::panic::catch_unwind(|| assert_phase1_safe_state(&safe_state));

        // Assert
        assert!(result.is_ok());
    }
}
