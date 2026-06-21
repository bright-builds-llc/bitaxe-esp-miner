/// First Phase 1 board target for boot/log bring-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTarget {
    /// Bitaxe Gamma 601 board.
    Gamma601,
}

impl BoardTarget {
    /// Returns the user-visible board name used in logs and reports.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Gamma601 => "Gamma 601",
        }
    }
}

/// First Phase 1 ASIC target for Gamma 601 bring-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicTarget {
    /// Bitmain BM1370 ASIC.
    Bm1370,
}

impl AsicTarget {
    /// Returns the user-visible ASIC name used in logs and reports.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Bm1370 => "BM1370",
        }
    }
}

/// Phase 1 mining state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MiningState {
    /// Mining is disabled during safe boot/log bring-up.
    #[default]
    Disabled,
}

/// Phase 1 ASIC work-submission state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AsicWorkSubmissionState {
    /// ASIC work submission is disabled during safe boot/log bring-up.
    #[default]
    Disabled,
}

/// Phase 1 hardware-control state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HardwareControlState {
    /// Hardware control is disabled during safe boot/log bring-up.
    #[default]
    Disabled,
}

/// Exact safe-state log line required before any mining or hardware control exists.
pub const PHASE1_SAFE_STATE_LOG_LINE: &str =
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";

/// Safe boot/log state for Phase 1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Phase1SafeState {
    mining: MiningState,
    asic_work_submission: AsicWorkSubmissionState,
    hardware_control: HardwareControlState,
}

impl Phase1SafeState {
    /// Returns the Phase 1 mining state.
    #[must_use]
    pub const fn mining(&self) -> MiningState {
        self.mining
    }

    /// Returns the Phase 1 ASIC work-submission state.
    #[must_use]
    pub const fn asic_work_submission(&self) -> AsicWorkSubmissionState {
        self.asic_work_submission
    }

    /// Returns the Phase 1 hardware-control state.
    #[must_use]
    pub const fn hardware_control(&self) -> HardwareControlState {
        self.hardware_control
    }

    /// Returns the exact Phase 1 safe-state log line.
    #[must_use]
    pub const fn log_line(&self) -> &'static str {
        PHASE1_SAFE_STATE_LOG_LINE
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AsicTarget, AsicWorkSubmissionState, BoardTarget, HardwareControlState, MiningState,
        Phase1SafeState,
    };

    #[test]
    fn gamma_601_display_name_matches_user_visible_board_name() {
        // Arrange
        let board = BoardTarget::Gamma601;

        // Act
        let display_name = board.display_name();

        // Assert
        assert_eq!(display_name, "Gamma 601");
    }

    #[test]
    fn bm1370_display_name_matches_user_visible_asic_name() {
        // Arrange
        let asic = AsicTarget::Bm1370;

        // Act
        let display_name = asic.display_name();

        // Assert
        assert_eq!(display_name, "BM1370");
    }

    #[test]
    fn default_phase_1_state_disables_mining() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let mining_state = safe_state.mining();

        // Assert
        assert_eq!(mining_state, MiningState::Disabled);
    }

    #[test]
    fn default_phase_1_state_disables_asic_work_submission() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let asic_work_submission_state = safe_state.asic_work_submission();

        // Assert
        assert_eq!(
            asic_work_submission_state,
            AsicWorkSubmissionState::Disabled
        );
    }

    #[test]
    fn default_phase_1_state_disables_hardware_control() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let hardware_control_state = safe_state.hardware_control();

        // Assert
        assert_eq!(hardware_control_state, HardwareControlState::Disabled);
    }

    #[test]
    fn default_phase_1_state_emits_safe_state_log_line() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let log_line = safe_state.log_line();

        // Assert
        assert_eq!(
            log_line,
            "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled"
        );
    }
}
