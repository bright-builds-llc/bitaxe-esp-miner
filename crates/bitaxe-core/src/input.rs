//! Pure Ultra 205 active-low boot-button classification and routing.
//!
//! Reference breadcrumb: `reference/esp-miner/main/input.c`.

use core::fmt;

/// Firmware sampling cadence for the retained GPIO0 owner.
pub const BUTTON_SAMPLE_MS: u64 = 10;
/// Stable interval required before accepting an electrical edge.
pub const BUTTON_DEBOUNCE_MS: u64 = 30;
/// Exact upstream long-press duration.
pub const BUTTON_LONG_PRESS_MS: u64 = 2_000;

/// One sampled active-low GPIO level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonLevel {
    ReleasedHigh,
    PressedLow,
}

impl ButtonLevel {
    const fn pressed(self) -> bool {
        matches!(self, Self::PressedLow)
    }
}

/// One admitted logical button event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    ShortClick,
    LongPress,
}

/// Current self-test routing context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSelfTestState {
    Inactive,
    Active,
    Unavailable,
}

/// Closed effect vocabulary for one admitted event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRoute {
    AdvanceScreen,
    CancelIdentify,
    ToggleConfigurationAp,
    ResetSelfTest,
    SelfTestResetUnavailable,
    IgnoreShortDuringSelfTest,
}

/// Routes one event without executing firmware effects.
#[must_use]
pub const fn route_button_event(
    event: ButtonEvent,
    self_test: ButtonSelfTestState,
    identify_active: bool,
) -> ButtonRoute {
    match (event, self_test, identify_active) {
        (ButtonEvent::ShortClick, ButtonSelfTestState::Active, _) => {
            ButtonRoute::IgnoreShortDuringSelfTest
        }
        (ButtonEvent::ShortClick, ButtonSelfTestState::Unavailable, _) => {
            ButtonRoute::IgnoreShortDuringSelfTest
        }
        (ButtonEvent::ShortClick, ButtonSelfTestState::Inactive, true) => {
            ButtonRoute::CancelIdentify
        }
        (ButtonEvent::ShortClick, ButtonSelfTestState::Inactive, false) => {
            ButtonRoute::AdvanceScreen
        }
        (ButtonEvent::LongPress, ButtonSelfTestState::Active, _) => ButtonRoute::ResetSelfTest,
        (ButtonEvent::LongPress, ButtonSelfTestState::Unavailable, _) => {
            ButtonRoute::SelfTestResetUnavailable
        }
        (ButtonEvent::LongPress, ButtonSelfTestState::Inactive, _) => {
            ButtonRoute::ToggleConfigurationAp
        }
    }
}

/// Pure Wi-Fi configuration mode selected by the long-press toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationApMode {
    None,
    StationOnly,
    AccessPointOnly,
    StationAndAccessPoint,
}

/// Selects the next Wi-Fi configuration without carrying credentials.
#[must_use]
pub const fn configuration_ap_toggle_mode(
    ap_enabled: bool,
    station_configuration_available: bool,
) -> ConfigurationApMode {
    match (ap_enabled, station_configuration_available) {
        (true, true) => ConfigurationApMode::StationOnly,
        (true, false) => ConfigurationApMode::None,
        (false, true) => ConfigurationApMode::StationAndAccessPoint,
        (false, false) => ConfigurationApMode::AccessPointOnly,
    }
}

/// Closed classifier failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonInputError {
    ClockRegressed,
    DeadlineOverflow,
}

impl fmt::Display for ButtonInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ClockRegressed => "button clock regressed",
            Self::DeadlineOverflow => "button deadline overflow",
        })
    }
}

impl std::error::Error for ButtonInputError {}

/// Retained pure owner of raw, debounced, and held-button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonInput {
    raw_pressed: bool,
    raw_since_ms: u64,
    stable_pressed: bool,
    maybe_pressed_at_ms: Option<u64>,
    long_press_emitted: bool,
    last_update_ms: u64,
}

impl ButtonInput {
    /// Starts from one raw active-low sample without synthesizing an event.
    #[must_use]
    pub const fn new(started_at_ms: u64, level: ButtonLevel) -> Self {
        Self {
            raw_pressed: level.pressed(),
            raw_since_ms: started_at_ms,
            stable_pressed: false,
            maybe_pressed_at_ms: None,
            long_press_emitted: false,
            last_update_ms: started_at_ms,
        }
    }

    /// Admits at most one debounced short or long event for this sample.
    pub fn update(
        &mut self,
        now_ms: u64,
        level: ButtonLevel,
    ) -> Result<Option<ButtonEvent>, ButtonInputError> {
        if now_ms < self.last_update_ms {
            return Err(ButtonInputError::ClockRegressed);
        }
        let mut next = *self;
        next.last_update_ms = now_ms;
        let pressed = level.pressed();
        if pressed != next.raw_pressed {
            next.raw_pressed = pressed;
            next.raw_since_ms = now_ms;
        }

        let mut maybe_event = None;
        let raw_age_ms = now_ms
            .checked_sub(next.raw_since_ms)
            .ok_or(ButtonInputError::ClockRegressed)?;
        if next.raw_pressed != next.stable_pressed && raw_age_ms >= BUTTON_DEBOUNCE_MS {
            let admitted_at_ms = next
                .raw_since_ms
                .checked_add(BUTTON_DEBOUNCE_MS)
                .ok_or(ButtonInputError::DeadlineOverflow)?;
            next.stable_pressed = next.raw_pressed;
            if next.stable_pressed {
                next.maybe_pressed_at_ms = Some(admitted_at_ms);
                next.long_press_emitted = false;
            } else {
                if next.maybe_pressed_at_ms.is_some() && !next.long_press_emitted {
                    maybe_event = Some(ButtonEvent::ShortClick);
                }
                next.maybe_pressed_at_ms = None;
                next.long_press_emitted = false;
            }
        }

        if maybe_event.is_none() && next.stable_pressed && !next.long_press_emitted {
            let pressed_at_ms = next
                .maybe_pressed_at_ms
                .ok_or(ButtonInputError::DeadlineOverflow)?;
            let long_press_at_ms = pressed_at_ms
                .checked_add(BUTTON_LONG_PRESS_MS)
                .ok_or(ButtonInputError::DeadlineOverflow)?;
            if now_ms >= long_press_at_ms {
                next.long_press_emitted = true;
                maybe_event = Some(ButtonEvent::LongPress);
            }
        }

        *self = next;
        Ok(maybe_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_low_press_and_release_emit_one_short_click_after_debounce() {
        // Arrange
        let mut input = ButtonInput::new(0, ButtonLevel::ReleasedHigh);

        // Act / Assert
        assert_eq!(input.update(10, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(input.update(39, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(input.update(40, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(input.update(100, ButtonLevel::ReleasedHigh), Ok(None));
        assert_eq!(
            input.update(130, ButtonLevel::ReleasedHigh),
            Ok(Some(ButtonEvent::ShortClick))
        );
        assert_eq!(input.update(140, ButtonLevel::ReleasedHigh), Ok(None));
    }

    #[test]
    fn bounce_never_becomes_a_logical_press() {
        // Arrange
        let mut input = ButtonInput::new(0, ButtonLevel::ReleasedHigh);

        // Act
        let events = [
            (10, ButtonLevel::PressedLow),
            (20, ButtonLevel::ReleasedHigh),
            (30, ButtonLevel::PressedLow),
            (40, ButtonLevel::ReleasedHigh),
            (70, ButtonLevel::ReleasedHigh),
        ]
        .map(|(time, level)| input.update(time, level));

        // Assert
        assert!(events.into_iter().all(|event| event == Ok(None)));
    }

    #[test]
    fn exact_long_boundary_emits_once_and_release_suppresses_short() {
        // Arrange
        let mut input = ButtonInput::new(0, ButtonLevel::ReleasedHigh);
        input.update(10, ButtonLevel::PressedLow).expect("edge");
        input.update(40, ButtonLevel::PressedLow).expect("debounce");

        // Act / Assert
        assert_eq!(input.update(2_039, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(
            input.update(2_040, ButtonLevel::PressedLow),
            Ok(Some(ButtonEvent::LongPress))
        );
        assert_eq!(input.update(3_000, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(input.update(3_010, ButtonLevel::ReleasedHigh), Ok(None));
        assert_eq!(input.update(3_040, ButtonLevel::ReleasedHigh), Ok(None));
    }

    #[test]
    fn regressed_clock_preserves_state_and_high_timestamp_does_not_wrap() {
        // Arrange
        let mut regressed = ButtonInput::new(100, ButtonLevel::ReleasedHigh);
        let regressed_before = regressed;
        let mut overflow = ButtonInput::new(u64::MAX - 20, ButtonLevel::PressedLow);

        // Act / Assert
        assert_eq!(
            regressed.update(99, ButtonLevel::ReleasedHigh),
            Err(ButtonInputError::ClockRegressed)
        );
        assert_eq!(regressed, regressed_before);
        assert_eq!(overflow.update(u64::MAX, ButtonLevel::PressedLow), Ok(None));
        assert_eq!(overflow.last_update_ms, u64::MAX);
        assert!(!overflow.stable_pressed);
    }

    #[test]
    fn every_event_context_has_one_closed_route() {
        // Arrange / Act / Assert
        assert_eq!(
            route_button_event(
                ButtonEvent::ShortClick,
                ButtonSelfTestState::Inactive,
                false
            ),
            ButtonRoute::AdvanceScreen
        );
        assert_eq!(
            route_button_event(ButtonEvent::ShortClick, ButtonSelfTestState::Inactive, true),
            ButtonRoute::CancelIdentify
        );
        assert_eq!(
            route_button_event(ButtonEvent::LongPress, ButtonSelfTestState::Inactive, false),
            ButtonRoute::ToggleConfigurationAp
        );
        assert_eq!(
            route_button_event(ButtonEvent::LongPress, ButtonSelfTestState::Active, false),
            ButtonRoute::ResetSelfTest
        );
        assert_eq!(
            route_button_event(
                ButtonEvent::LongPress,
                ButtonSelfTestState::Unavailable,
                false
            ),
            ButtonRoute::SelfTestResetUnavailable
        );
        assert_eq!(
            route_button_event(ButtonEvent::ShortClick, ButtonSelfTestState::Active, false),
            ButtonRoute::IgnoreShortDuringSelfTest
        );
        assert_eq!(
            route_button_event(
                ButtonEvent::ShortClick,
                ButtonSelfTestState::Unavailable,
                false
            ),
            ButtonRoute::IgnoreShortDuringSelfTest
        );
    }

    #[test]
    fn ap_toggle_mode_never_requires_private_configuration_values() {
        // Arrange / Act / Assert
        assert_eq!(
            configuration_ap_toggle_mode(true, true),
            ConfigurationApMode::StationOnly
        );
        assert_eq!(
            configuration_ap_toggle_mode(true, false),
            ConfigurationApMode::None
        );
        assert_eq!(
            configuration_ap_toggle_mode(false, true),
            ConfigurationApMode::StationAndAccessPoint
        );
        assert_eq!(
            configuration_ap_toggle_mode(false, false),
            ConfigurationApMode::AccessPointOnly
        );
    }
}
