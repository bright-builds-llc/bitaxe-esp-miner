pub const LIVE_MINING_RUNTIME_MODE: &str = "live-mining-runtime";
pub const LIVE_MINING_RUNTIME_ACK: &str = "ultra205-live-mining-runtime-safe-bench";
pub const PHASE25_LIVE_STRATUM_MODE: &str = "phase25-live-stratum-runtime";
pub const PHASE25_LIVE_STRATUM_ACK: &str = "ultra205-phase25-live-stratum-safe-stop";

macro_rules! bitaxe_mining_evidence_mode_env {
    () => {
        option_env!("BITAXE_MINING_EVIDENCE_MODE")
    };
}

macro_rules! bitaxe_hardware_evidence_ack_env {
    () => {
        option_env!("BITAXE_HARDWARE_EVIDENCE_ACK")
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningEvidenceMode {
    FailClosed,
    LiveMiningRuntime,
    Phase25LiveStratumRuntime,
}

impl MiningEvidenceMode {
    #[must_use]
    pub fn current() -> Self {
        Self::from_compile_env(
            bitaxe_mining_evidence_mode_env!(),
            bitaxe_hardware_evidence_ack_env!(),
        )
    }

    #[must_use]
    pub fn from_compile_env(
        maybe_mode: Option<&'static str>,
        maybe_ack: Option<&'static str>,
    ) -> Self {
        match (maybe_mode, maybe_ack) {
            (Some(LIVE_MINING_RUNTIME_MODE), Some(LIVE_MINING_RUNTIME_ACK)) => {
                Self::LiveMiningRuntime
            }
            (Some(PHASE25_LIVE_STRATUM_MODE), Some(PHASE25_LIVE_STRATUM_ACK)) => {
                Self::Phase25LiveStratumRuntime
            }
            _ => Self::FailClosed,
        }
    }

    #[must_use]
    pub fn is_live_mining_runtime(self) -> bool {
        matches!(self, Self::LiveMiningRuntime)
    }

    #[must_use]
    pub fn is_phase25_live_stratum_runtime(self) -> bool {
        matches!(self, Self::Phase25LiveStratumRuntime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_mining_runtime_requires_mode_and_ack_compile_env_pair() {
        // Arrange
        let missing = MiningEvidenceMode::from_compile_env(None, None);
        let mode_only = MiningEvidenceMode::from_compile_env(Some(LIVE_MINING_RUNTIME_MODE), None);

        // Act
        let live = MiningEvidenceMode::from_compile_env(
            Some(LIVE_MINING_RUNTIME_MODE),
            Some(LIVE_MINING_RUNTIME_ACK),
        );

        // Assert
        assert!(!missing.is_live_mining_runtime());
        assert!(!mode_only.is_live_mining_runtime());
        assert!(live.is_live_mining_runtime());
    }

    #[test]
    fn mismatched_ack_keeps_default_fail_closed_mode() {
        // Arrange
        let wrong_ack = MiningEvidenceMode::from_compile_env(
            Some(LIVE_MINING_RUNTIME_MODE),
            Some("not-the-phase-21-token"),
        );

        // Act
        let live = wrong_ack.is_live_mining_runtime();

        // Assert
        assert_eq!(wrong_ack, MiningEvidenceMode::FailClosed);
        assert!(!live);
    }

    #[test]
    fn phase25_live_stratum_runtime_requires_distinct_mode_and_ack_pair() {
        // Arrange
        let missing = MiningEvidenceMode::from_compile_env(None, None);
        let mode_only = MiningEvidenceMode::from_compile_env(Some(PHASE25_LIVE_STRATUM_MODE), None);
        let wrong_ack = MiningEvidenceMode::from_compile_env(
            Some(PHASE25_LIVE_STRATUM_MODE),
            Some(LIVE_MINING_RUNTIME_ACK),
        );

        // Act
        let phase25 = MiningEvidenceMode::from_compile_env(
            Some(PHASE25_LIVE_STRATUM_MODE),
            Some(PHASE25_LIVE_STRATUM_ACK),
        );

        // Assert
        assert_eq!(missing, MiningEvidenceMode::FailClosed);
        assert_eq!(mode_only, MiningEvidenceMode::FailClosed);
        assert_eq!(wrong_ack, MiningEvidenceMode::FailClosed);
        assert!(phase25.is_phase25_live_stratum_runtime());
        assert!(!phase25.is_live_mining_runtime());
    }
}
