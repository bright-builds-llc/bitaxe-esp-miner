/// First Phase 1 board target for boot/log bring-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTarget {
    /// Bitaxe Ultra 205 board.
    Ultra205,
}

impl BoardTarget {
    /// Returns the user-visible board name used in logs and reports.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Ultra205 => "Ultra 205",
        }
    }
}

/// First Phase 1 ASIC target for Ultra 205 bring-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicTarget {
    /// Bitmain BM1366 ASIC.
    Bm1366,
}

impl AsicTarget {
    /// Returns the user-visible ASIC name used in logs and reports.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Bm1366 => "BM1366",
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
/// Startup debug screen width in pixels for the Ultra 205 SSD1306 display.
pub const STARTUP_DEBUG_SCREEN_WIDTH_PX: usize = 128;
/// Startup debug screen height in pixels for the Ultra 205 SSD1306 display.
pub const STARTUP_DEBUG_SCREEN_HEIGHT_PX: usize = 32;
/// Selected startup debug font width in pixels.
pub const STARTUP_DEBUG_FONT_WIDTH_PX: usize = 5;
/// Selected startup debug font height in pixels.
pub const STARTUP_DEBUG_FONT_HEIGHT_PX: usize = 7;
/// Vertical stride between startup debug text baselines.
pub const STARTUP_DEBUG_LINE_STRIDE_PX: usize = 8;
/// Number of lines rendered on the startup debug screen.
pub const STARTUP_DEBUG_LINE_COUNT: usize = 4;
/// Maximum ASCII characters that fit on one startup debug line.
pub const STARTUP_DEBUG_MAX_LINE_CHARS: usize =
    STARTUP_DEBUG_SCREEN_WIDTH_PX / STARTUP_DEBUG_FONT_WIDTH_PX;
/// Maximum startup debug lines that fit on the display.
pub const STARTUP_DEBUG_MAX_LINES: usize =
    STARTUP_DEBUG_SCREEN_HEIGHT_PX / STARTUP_DEBUG_LINE_STRIDE_PX;
/// Maximum source commit characters shown on the startup debug screen.
pub const STARTUP_DEBUG_COMMIT_CHARS: usize = 12;

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

    /// Returns the safe-state startup debug screen line.
    #[must_use]
    pub const fn startup_debug_line(&self) -> &'static str {
        "SAFE no mining"
    }
}

/// Four-line startup debug text for the Ultra 205 OLED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupDebugText {
    lines: [String; STARTUP_DEBUG_LINE_COUNT],
}

impl StartupDebugText {
    /// Builds startup debug lines for the current firmware boot target.
    #[must_use]
    pub fn new(
        board: BoardTarget,
        asic: AsicTarget,
        safe_state: Phase1SafeState,
        maybe_firmware_commit: Option<&str>,
    ) -> Self {
        let firmware_commit = startup_debug_commit(maybe_firmware_commit);
        let lines = [
            "Bitaxe Rust".to_owned(),
            format!("{} {}", board.display_name(), asic.display_name()),
            safe_state.startup_debug_line().to_owned(),
            format!("fw {firmware_commit}"),
        ];

        Self { lines }
    }

    /// Returns the startup debug lines in render order.
    #[must_use]
    pub fn lines(&self) -> [&str; STARTUP_DEBUG_LINE_COUNT] {
        [
            self.lines[0].as_str(),
            self.lines[1].as_str(),
            self.lines[2].as_str(),
            self.lines[3].as_str(),
        ]
    }

    /// Returns whether the current line set fits the selected display geometry.
    #[must_use]
    pub fn fits_ultra_205_display(&self) -> bool {
        self.lines.len() <= STARTUP_DEBUG_MAX_LINES
            && self
                .lines
                .iter()
                .all(|line| line.chars().count() <= STARTUP_DEBUG_MAX_LINE_CHARS)
    }
}

fn startup_debug_commit(maybe_firmware_commit: Option<&str>) -> String {
    let Some(firmware_commit) = maybe_firmware_commit else {
        return "Unavailable".to_owned();
    };
    let firmware_commit = firmware_commit.trim();
    if firmware_commit.is_empty() || firmware_commit == "Unavailable" {
        return "Unavailable".to_owned();
    }

    firmware_commit
        .chars()
        .take(STARTUP_DEBUG_COMMIT_CHARS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        AsicTarget, AsicWorkSubmissionState, BoardTarget, HardwareControlState, MiningState,
        Phase1SafeState, StartupDebugText, STARTUP_DEBUG_LINE_COUNT, STARTUP_DEBUG_MAX_LINES,
        STARTUP_DEBUG_MAX_LINE_CHARS,
    };

    #[test]
    fn ultra_205_display_name_matches_user_visible_board_name() {
        // Arrange
        let board = BoardTarget::Ultra205;

        // Act
        let display_name = board.display_name();

        // Assert
        assert_eq!(display_name, "Ultra 205");
    }

    #[test]
    fn bm1366_display_name_matches_user_visible_asic_name() {
        // Arrange
        let asic = AsicTarget::Bm1366;

        // Act
        let display_name = asic.display_name();

        // Assert
        assert_eq!(display_name, "BM1366");
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

    #[test]
    fn startup_debug_text_renders_exact_identity_state_and_commit_lines() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let text = StartupDebugText::new(
            BoardTarget::Ultra205,
            AsicTarget::Bm1366,
            safe_state,
            Some("abcdef123456"),
        );

        // Assert
        assert_eq!(
            text.lines(),
            [
                "Bitaxe Rust",
                "Ultra 205 BM1366",
                "SAFE no mining",
                "fw abcdef123456",
            ]
        );
    }

    #[test]
    fn startup_debug_text_uses_unavailable_when_commit_is_absent() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let text =
            StartupDebugText::new(BoardTarget::Ultra205, AsicTarget::Bm1366, safe_state, None);

        // Assert
        assert_eq!(text.lines()[3], "fw Unavailable");
    }

    #[test]
    fn startup_debug_text_truncates_commit_to_twelve_characters() {
        // Arrange
        let safe_state = Phase1SafeState::default();

        // Act
        let text = StartupDebugText::new(
            BoardTarget::Ultra205,
            AsicTarget::Bm1366,
            safe_state,
            Some("abcdef1234567890"),
        );

        // Assert
        assert_eq!(text.lines()[3], "fw abcdef123456");
    }

    #[test]
    fn startup_debug_text_fits_ultra_205_display_geometry() {
        // Arrange
        let safe_state = Phase1SafeState::default();
        let text = StartupDebugText::new(
            BoardTarget::Ultra205,
            AsicTarget::Bm1366,
            safe_state,
            Some("abcdef123456"),
        );

        // Act
        let lines = text.lines();

        // Assert
        assert_eq!(lines.len(), STARTUP_DEBUG_LINE_COUNT);
        assert!(lines.len() <= STARTUP_DEBUG_MAX_LINES);
        for line in lines {
            assert!(line.chars().count() <= STARTUP_DEBUG_MAX_LINE_CHARS);
        }
        assert!(text.fits_ultra_205_display());
    }
}
