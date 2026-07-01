//! Compile-time gate for firmware BM1366 diagnostic adapter effects.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/serial.c`
//! - `reference/esp-miner/main/power/asic_reset.c`
//! - parity checklist rows `ASIC-005`, `ASIC-007`, and `ASIC-008`

pub const CHIP_DETECT_DIAGNOSTIC: &str = "chip-detect";
pub const HARDWARE_EVIDENCE_ACK: &str = "ultra205-chip-detect-safe-bench";
pub const WORK_RESULT_DIAGNOSTIC: &str = "work-result";
pub const WORK_RESULT_HARDWARE_EVIDENCE_ACK: &str = "ultra205-work-result-safe-bench";
pub const DEFAULT_FAIL_CLOSED_STATUS_LOG: &str = "asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled";
pub const WORK_RESULT_DIAGNOSTIC_STARTED_LOG: &str =
    "asic_work_result_diagnostic=started mining=disabled";
pub const WORK_RESULT_DIAGNOSTIC_DISPATCHED_LOG: &str = "bm1366_diagnostic_work=dispatched";
pub const WORK_RESULT_DIAGNOSTIC_PARSED_LOG: &str = "bm1366_diagnostic_result=parsed";
pub const WORK_RESULT_DIAGNOSTIC_TIMEOUT_LOG: &str =
    "bm1366_diagnostic_result=timeout fail_closed=true";
pub const WORK_RESULT_DIAGNOSTIC_INVALID_LOG: &str =
    "bm1366_diagnostic_result=invalid fail_closed=true";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicAdapterMode {
    FailClosed,
    ChipDetectOnly,
    WorkResultDiagnostic,
}

impl AsicAdapterMode {
    #[must_use]
    pub fn from_compile_env(
        maybe_diagnostic: Option<&str>,
        maybe_hardware_evidence_ack: Option<&str>,
    ) -> Self {
        if maybe_diagnostic == Some(CHIP_DETECT_DIAGNOSTIC)
            && maybe_hardware_evidence_ack == Some(HARDWARE_EVIDENCE_ACK)
        {
            return Self::ChipDetectOnly;
        }

        if maybe_diagnostic == Some(WORK_RESULT_DIAGNOSTIC)
            && maybe_hardware_evidence_ack == Some(WORK_RESULT_HARDWARE_EVIDENCE_ACK)
        {
            return Self::WorkResultDiagnostic;
        }

        Self::FailClosed
    }
}

#[must_use]
pub const fn default_fail_closed_status_log() -> &'static str {
    DEFAULT_FAIL_CLOSED_STATUS_LOG
}

#[must_use]
pub const fn work_result_diagnostic_started_log() -> &'static str {
    WORK_RESULT_DIAGNOSTIC_STARTED_LOG
}

#[must_use]
pub const fn work_result_diagnostic_dispatched_log() -> &'static str {
    WORK_RESULT_DIAGNOSTIC_DISPATCHED_LOG
}

#[must_use]
pub const fn work_result_diagnostic_parsed_log() -> &'static str {
    WORK_RESULT_DIAGNOSTIC_PARSED_LOG
}

#[must_use]
pub const fn work_result_diagnostic_timeout_log() -> &'static str {
    WORK_RESULT_DIAGNOSTIC_TIMEOUT_LOG
}

#[must_use]
pub const fn work_result_diagnostic_invalid_log() -> &'static str {
    WORK_RESULT_DIAGNOSTIC_INVALID_LOG
}

#[cfg(test)]
mod tests {
    use super::{
        default_fail_closed_status_log, work_result_diagnostic_dispatched_log,
        work_result_diagnostic_invalid_log, work_result_diagnostic_parsed_log,
        work_result_diagnostic_started_log, work_result_diagnostic_timeout_log, AsicAdapterMode,
    };

    #[test]
    fn adapter_gate_default_fail_closed_status_log_is_exact() {
        // Arrange
        let expected = "asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled";

        // Act
        let observed = default_fail_closed_status_log();

        // Assert
        assert_eq!(observed, expected);
    }

    #[test]
    fn adapter_gate_missing_diagnostic_env_fails_closed() {
        // Arrange
        let maybe_diagnostic = None;
        let maybe_hardware_evidence_ack = None;

        // Act
        let observed =
            AsicAdapterMode::from_compile_env(maybe_diagnostic, maybe_hardware_evidence_ack);

        // Assert
        assert_eq!(observed, AsicAdapterMode::FailClosed);
    }

    #[test]
    fn adapter_gate_missing_ack_env_fails_closed() {
        // Arrange
        let maybe_diagnostic = Some("chip-detect");
        let maybe_hardware_evidence_ack = None;

        // Act
        let observed =
            AsicAdapterMode::from_compile_env(maybe_diagnostic, maybe_hardware_evidence_ack);

        // Assert
        assert_eq!(observed, AsicAdapterMode::FailClosed);
    }

    #[test]
    fn adapter_gate_wrong_ack_env_fails_closed() {
        // Arrange
        let maybe_diagnostic = Some("chip-detect");
        let maybe_hardware_evidence_ack = Some("wrong-bench");

        // Act
        let observed =
            AsicAdapterMode::from_compile_env(maybe_diagnostic, maybe_hardware_evidence_ack);

        // Assert
        assert_eq!(observed, AsicAdapterMode::FailClosed);
    }

    #[test]
    fn adapter_gate_chip_detect_with_safe_bench_ack_is_chip_detect_only() {
        // Arrange
        let maybe_diagnostic = Some("chip-detect");
        let maybe_hardware_evidence_ack = Some("ultra205-chip-detect-safe-bench");

        // Act
        let observed =
            AsicAdapterMode::from_compile_env(maybe_diagnostic, maybe_hardware_evidence_ack);

        // Assert
        assert_eq!(observed, AsicAdapterMode::ChipDetectOnly);
    }

    #[test]
    fn adapter_gate_work_result_with_safe_bench_ack_is_work_result_diagnostic() {
        // Arrange
        let maybe_diagnostic = Some("work-result");
        let maybe_hardware_evidence_ack = Some("ultra205-work-result-safe-bench");

        // Act
        let observed =
            AsicAdapterMode::from_compile_env(maybe_diagnostic, maybe_hardware_evidence_ack);

        // Assert
        assert_eq!(observed, AsicAdapterMode::WorkResultDiagnostic);
    }

    #[test]
    fn adapter_gate_work_result_missing_or_wrong_ack_fails_closed() {
        // Arrange
        let missing_ack = None;
        let wrong_ack = Some("ultra205-chip-detect-safe-bench");

        // Act
        let missing_observed = AsicAdapterMode::from_compile_env(Some("work-result"), missing_ack);
        let wrong_observed = AsicAdapterMode::from_compile_env(Some("work-result"), wrong_ack);

        // Assert
        assert_eq!(missing_observed, AsicAdapterMode::FailClosed);
        assert_eq!(wrong_observed, AsicAdapterMode::FailClosed);
    }

    #[test]
    fn work_result_diagnostic_log_markers_are_exact() {
        // Arrange
        let expected_started = "asic_work_result_diagnostic=started mining=disabled";
        let expected_dispatched = "bm1366_diagnostic_work=dispatched";
        let expected_parsed = "bm1366_diagnostic_result=parsed";
        let expected_timeout = "bm1366_diagnostic_result=timeout fail_closed=true";
        let expected_invalid = "bm1366_diagnostic_result=invalid fail_closed=true";

        // Act
        let started = work_result_diagnostic_started_log();
        let dispatched = work_result_diagnostic_dispatched_log();
        let parsed = work_result_diagnostic_parsed_log();
        let timeout = work_result_diagnostic_timeout_log();
        let invalid = work_result_diagnostic_invalid_log();

        // Assert
        assert_eq!(started, expected_started);
        assert_eq!(dispatched, expected_dispatched);
        assert_eq!(parsed, expected_parsed);
        assert_eq!(timeout, expected_timeout);
        assert_eq!(invalid, expected_invalid);
    }
}
