//! Pure Ultra 205 display configuration and power policy.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/display.c`
//! - `reference/esp-miner/main/screen.c:screen_update_cb`

use core::fmt;

/// Exact upstream panel name used by the Ultra 205.
pub const ULTRA205_DISPLAY_NAME: &str = "SSD1306 (128x32)";
/// Upstream button/activity wake window when the timeout is zero.
pub const DISPLAY_WAKE_WINDOW_MS: u64 = 5_000;
const MILLIS_PER_MINUTE: u64 = 60_000;
const MAX_TIMEOUT_MINUTES: i32 = u16::MAX as i32;

/// Supported physical orientation of the Ultra 205 panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayRotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl DisplayRotation {
    /// Parses the exact upstream rotation vocabulary.
    pub const fn from_degrees(degrees: u16) -> Result<Self, DisplayConfigurationError> {
        match degrees {
            0 => Ok(Self::Rotate0),
            90 => Ok(Self::Rotate90),
            180 => Ok(Self::Rotate180),
            270 => Ok(Self::Rotate270),
            _ => Err(DisplayConfigurationError::UnsupportedRotation),
        }
    }

    /// Returns the persisted degree representation.
    #[must_use]
    pub const fn degrees(self) -> u16 {
        match self {
            Self::Rotate0 => 0,
            Self::Rotate90 => 90,
            Self::Rotate180 => 180,
            Self::Rotate270 => 270,
        }
    }
}

/// Upstream display timeout modes after validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayTimeout {
    AlwaysOn,
    ActivityOnly,
    InactivityMillis(u64),
}

impl DisplayTimeout {
    fn from_minutes(minutes: i32) -> Result<Self, DisplayConfigurationError> {
        match minutes {
            -1 => Ok(Self::AlwaysOn),
            0 => Ok(Self::ActivityOnly),
            1..=MAX_TIMEOUT_MINUTES => {
                let millis = u64::try_from(minutes)
                    .ok()
                    .and_then(|value| value.checked_mul(MILLIS_PER_MINUTE))
                    .ok_or(DisplayConfigurationError::TimeoutOverflow)?;
                Ok(Self::InactivityMillis(millis))
            }
            _ => Err(DisplayConfigurationError::InvalidTimeout),
        }
    }
}

/// Validated Ultra 205 display settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ultra205DisplayConfiguration {
    rotation: DisplayRotation,
    inverted: bool,
    timeout: DisplayTimeout,
}

impl Ultra205DisplayConfiguration {
    /// Validates the exact Ultra 205 panel and upstream settings vocabulary.
    pub fn new(
        panel_name: &str,
        rotation_degrees: u16,
        inverted: bool,
        timeout_minutes: i32,
    ) -> Result<Self, DisplayConfigurationError> {
        if panel_name != ULTRA205_DISPLAY_NAME {
            return Err(DisplayConfigurationError::UnsupportedPanel);
        }
        Ok(Self {
            rotation: DisplayRotation::from_degrees(rotation_degrees)?,
            inverted,
            timeout: DisplayTimeout::from_minutes(timeout_minutes)?,
        })
    }

    #[must_use]
    pub const fn rotation(self) -> DisplayRotation {
        self.rotation
    }

    #[must_use]
    pub const fn inverted(self) -> bool {
        self.inverted
    }

    #[must_use]
    pub const fn timeout(self) -> DisplayTimeout {
        self.timeout
    }
}

/// Closed display configuration and clock failure categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayConfigurationError {
    UnsupportedPanel,
    UnsupportedRotation,
    InvalidTimeout,
    TimeoutOverflow,
    ClockRegressed,
}

impl fmt::Display for DisplayConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPanel => "unsupported display panel",
            Self::UnsupportedRotation => "unsupported display rotation",
            Self::InvalidTimeout => "invalid display timeout",
            Self::TimeoutOverflow => "display timeout overflow",
            Self::ClockRegressed => "display clock regressed",
        })
    }
}

impl std::error::Error for DisplayConfigurationError {}

/// Edge-triggered physical display power command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayPowerCommand {
    TurnOn,
    TurnOff,
}

/// Pure owner of inactivity and current panel-power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPowerPolicy {
    timeout: DisplayTimeout,
    last_activity_ms: u64,
    display_on: bool,
}

impl DisplayPowerPolicy {
    /// Starts from an initialized, visible panel and treats startup as activity.
    #[must_use]
    pub const fn new(configuration: Ultra205DisplayConfiguration, started_at_ms: u64) -> Self {
        Self {
            timeout: configuration.timeout(),
            last_activity_ms: started_at_ms,
            display_on: true,
        }
    }

    /// Records explicit input or equivalent user activity.
    pub fn record_activity(&mut self, now_ms: u64) -> Result<(), DisplayConfigurationError> {
        if now_ms < self.last_activity_ms {
            return Err(DisplayConfigurationError::ClockRegressed);
        }
        self.last_activity_ms = now_ms;
        Ok(())
    }

    /// Returns an on/off command only when the physical state must change.
    pub fn command_at(
        &mut self,
        now_ms: u64,
        priority_visible: bool,
    ) -> Result<Option<DisplayPowerCommand>, DisplayConfigurationError> {
        let elapsed_ms = now_ms
            .checked_sub(self.last_activity_ms)
            .ok_or(DisplayConfigurationError::ClockRegressed)?;
        let desired_on = match self.timeout {
            DisplayTimeout::AlwaysOn => true,
            DisplayTimeout::ActivityOnly => priority_visible || elapsed_ms < DISPLAY_WAKE_WINDOW_MS,
            DisplayTimeout::InactivityMillis(timeout_ms) => {
                priority_visible || elapsed_ms < timeout_ms
            }
        };
        if desired_on == self.display_on {
            return Ok(None);
        }
        self.display_on = desired_on;
        Ok(Some(if desired_on {
            DisplayPowerCommand::TurnOn
        } else {
            DisplayPowerCommand::TurnOff
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration(timeout_minutes: i32) -> Ultra205DisplayConfiguration {
        Ultra205DisplayConfiguration::new(ULTRA205_DISPLAY_NAME, 0, false, timeout_minutes)
            .expect("fixture display configuration")
    }

    #[test]
    fn exact_panel_and_every_rotation_are_admitted() {
        // Arrange / Act
        let rotations = [0, 90, 180, 270].map(|degrees| {
            Ultra205DisplayConfiguration::new(ULTRA205_DISPLAY_NAME, degrees, true, -1)
                .expect("supported rotation")
        });

        // Assert
        assert_eq!(
            rotations.map(|item| item.rotation().degrees()),
            [0, 90, 180, 270]
        );
        assert!(rotations
            .into_iter()
            .all(Ultra205DisplayConfiguration::inverted));
    }

    #[test]
    fn alternate_panel_rotation_and_timeout_fail_closed() {
        // Arrange / Act / Assert
        assert_eq!(
            Ultra205DisplayConfiguration::new("SSD1306 (128x64)", 0, false, -1),
            Err(DisplayConfigurationError::UnsupportedPanel)
        );
        assert_eq!(
            Ultra205DisplayConfiguration::new(ULTRA205_DISPLAY_NAME, 45, false, -1),
            Err(DisplayConfigurationError::UnsupportedRotation)
        );
        assert_eq!(
            Ultra205DisplayConfiguration::new(ULTRA205_DISPLAY_NAME, 0, false, -2),
            Err(DisplayConfigurationError::InvalidTimeout)
        );
        assert_eq!(
            Ultra205DisplayConfiguration::new(ULTRA205_DISPLAY_NAME, 0, false, 65_536),
            Err(DisplayConfigurationError::InvalidTimeout)
        );
    }

    #[test]
    fn always_on_policy_never_emits_a_power_edge() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(-1), 10);

        // Act / Assert
        assert_eq!(policy.command_at(u64::MAX, false), Ok(None));
    }

    #[test]
    fn zero_timeout_turns_off_at_exact_wake_boundary_and_only_once() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(0), 100);

        // Act / Assert
        assert_eq!(policy.command_at(5_099, false), Ok(None));
        assert_eq!(
            policy.command_at(5_100, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );
        assert_eq!(policy.command_at(5_101, false), Ok(None));
    }

    #[test]
    fn priority_visibility_overrides_timeout_without_rewriting_activity() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(0), 0);
        assert_eq!(
            policy.command_at(DISPLAY_WAKE_WINDOW_MS, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );

        // Act / Assert
        assert_eq!(
            policy.command_at(DISPLAY_WAKE_WINDOW_MS, true),
            Ok(Some(DisplayPowerCommand::TurnOn))
        );
        assert_eq!(
            policy.command_at(DISPLAY_WAKE_WINDOW_MS + 1, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );
    }

    #[test]
    fn positive_timeout_uses_exact_minute_boundary() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(2), 500);

        // Act / Assert
        assert_eq!(policy.command_at(120_499, false), Ok(None));
        assert_eq!(
            policy.command_at(120_500, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );
    }

    #[test]
    fn maximum_timeout_uses_a_bounded_exact_boundary() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(65_535), 0);

        // Act / Assert
        assert_eq!(policy.command_at(3_932_099_999, false), Ok(None));
        assert_eq!(
            policy.command_at(3_932_100_000, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );
    }

    #[test]
    fn activity_restarts_the_timeout_and_can_wake_the_panel() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(0), 0);
        assert_eq!(
            policy.command_at(DISPLAY_WAKE_WINDOW_MS, false),
            Ok(Some(DisplayPowerCommand::TurnOff))
        );

        // Act
        policy
            .record_activity(DISPLAY_WAKE_WINDOW_MS)
            .expect("monotonic activity");

        // Assert
        assert_eq!(
            policy.command_at(DISPLAY_WAKE_WINDOW_MS, false),
            Ok(Some(DisplayPowerCommand::TurnOn))
        );
    }

    #[test]
    fn regressed_clock_preserves_policy_state() {
        // Arrange
        let mut policy = DisplayPowerPolicy::new(configuration(0), 100);

        // Act / Assert
        assert_eq!(
            policy.record_activity(99),
            Err(DisplayConfigurationError::ClockRegressed)
        );
        assert_eq!(
            policy.command_at(99, false),
            Err(DisplayConfigurationError::ClockRegressed)
        );
        assert_eq!(policy.command_at(100, false), Ok(None));
    }
}
