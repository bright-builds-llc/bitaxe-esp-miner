pub const LIVE_MINING_RUNTIME_MODE: &str = "live-mining-runtime";
pub const LIVE_MINING_RUNTIME_ACK: &str = "ultra205-live-mining-runtime-safe-bench";
pub const PHASE25_LIVE_STRATUM_MODE: &str = "phase25-live-stratum-runtime";
pub const PHASE25_LIVE_STRATUM_ACK: &str = "ultra205-phase25-live-stratum-safe-stop";
pub const PHASE27_LIVE_HARDWARE_BRIDGE_MODE: &str = "phase27-live-hardware-asic-stratum-bridge";
pub const PHASE27_LIVE_HARDWARE_BRIDGE_ACK: &str =
    "ultra205-phase27-live-hardware-bridge-safe-stop";

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
    Phase27LiveHardwareBridge,
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
            (Some(PHASE27_LIVE_HARDWARE_BRIDGE_MODE), Some(PHASE27_LIVE_HARDWARE_BRIDGE_ACK)) => {
                Self::Phase27LiveHardwareBridge
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

    #[must_use]
    pub fn is_phase27_live_hardware_bridge(self) -> bool {
        matches!(self, Self::Phase27LiveHardwareBridge)
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

    #[test]
    fn phase27_live_hardware_bridge_requires_distinct_mode_and_ack_pair() {
        // Arrange
        let missing = MiningEvidenceMode::from_compile_env(None, None);
        let mode_only =
            MiningEvidenceMode::from_compile_env(Some(PHASE27_LIVE_HARDWARE_BRIDGE_MODE), None);
        let wrong_ack = MiningEvidenceMode::from_compile_env(
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_MODE),
            Some(PHASE25_LIVE_STRATUM_ACK),
        );
        let phase25_collision = MiningEvidenceMode::from_compile_env(
            Some(PHASE25_LIVE_STRATUM_MODE),
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_ACK),
        );

        // Act
        let phase27 = MiningEvidenceMode::from_compile_env(
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_MODE),
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_ACK),
        );

        // Assert
        assert_eq!(missing, MiningEvidenceMode::FailClosed);
        assert_eq!(mode_only, MiningEvidenceMode::FailClosed);
        assert_eq!(wrong_ack, MiningEvidenceMode::FailClosed);
        assert_eq!(phase25_collision, MiningEvidenceMode::FailClosed);
        assert!(phase27.is_phase27_live_hardware_bridge());
        assert!(!phase27.is_phase25_live_stratum_runtime());
        assert!(!phase27.is_live_mining_runtime());
    }

    #[test]
    fn only_one_live_bridge_variant_is_active_per_build() {
        // Arrange / Act
        let phase25 = MiningEvidenceMode::from_compile_env(
            Some(PHASE25_LIVE_STRATUM_MODE),
            Some(PHASE25_LIVE_STRATUM_ACK),
        );
        let phase27 = MiningEvidenceMode::from_compile_env(
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_MODE),
            Some(PHASE27_LIVE_HARDWARE_BRIDGE_ACK),
        );

        // Assert
        assert!(phase25.is_phase25_live_stratum_runtime());
        assert!(!phase25.is_phase27_live_hardware_bridge());
        assert!(phase27.is_phase27_live_hardware_bridge());
        assert!(!phase27.is_phase25_live_stratum_runtime());
    }
}
