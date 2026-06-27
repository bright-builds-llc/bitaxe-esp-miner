//! Compile-time gate for firmware BM1366 diagnostic adapter effects.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/serial.c`
//! - `reference/esp-miner/main/power/asic_reset.c`
//! - parity checklist rows `ASIC-005`, `ASIC-007`, and `ASIC-008`

pub const CHIP_DETECT_DIAGNOSTIC: &str = "chip-detect";
pub const HARDWARE_EVIDENCE_ACK: &str = "ultra205-chip-detect-safe-bench";
pub const DEFAULT_FAIL_CLOSED_STATUS_LOG: &str = "asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicAdapterMode {
    FailClosed,
    ChipDetectOnly,
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

        Self::FailClosed
    }
}

#[must_use]
pub const fn default_fail_closed_status_log() -> &'static str {
    DEFAULT_FAIL_CLOSED_STATUS_LOG
}

#[cfg(test)]
mod tests {
    use super::{default_fail_closed_status_log, AsicAdapterMode};

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
}
