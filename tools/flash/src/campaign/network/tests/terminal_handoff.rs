use super::*;
use crate::campaign::network::{close_serial_input, TerminalCaptureHandoff};

#[test]
fn serial_close_hands_off_analyzer_terminal_before_input_end() {
    // Arrange
    let mut shared = SharedSerialState::default();
    let terminal = TerminalCaptureHandoff {
        terminal_consumed: true,
        pool_config_persisted: true,
        maybe_failure: None,
    };

    // Act
    close_serial_input(&mut shared, Some(terminal));

    // Assert
    assert!(shared.terminal_consumed);
    assert!(shared.terminal_pool_persisted);
    assert!(shared.serial_finished);
    assert_eq!(shared.maybe_failure, None);
}

#[test]
fn contradictory_terminal_handoff_preserves_the_first_failure() {
    // Arrange
    let primary = CampaignTerminalCategory::SafetyStale;
    let mut shared = SharedSerialState {
        terminal_consumed: true,
        terminal_pool_persisted: false,
        maybe_failure: Some(primary),
        ..SharedSerialState::default()
    };
    let terminal = TerminalCaptureHandoff {
        terminal_consumed: true,
        pool_config_persisted: true,
        maybe_failure: None,
    };

    // Act
    close_serial_input(&mut shared, Some(terminal));

    // Assert
    assert!(shared.serial_finished);
    assert!(!shared.terminal_pool_persisted);
    assert_eq!(shared.maybe_failure, Some(primary));
}

#[test]
fn contradictory_terminal_handoff_fails_closed_without_a_prior_failure() {
    // Arrange
    let mut shared = SharedSerialState {
        terminal_consumed: true,
        terminal_pool_persisted: false,
        ..SharedSerialState::default()
    };
    let terminal = TerminalCaptureHandoff {
        terminal_consumed: true,
        pool_config_persisted: true,
        maybe_failure: None,
    };

    // Act
    close_serial_input(&mut shared, Some(terminal));

    // Assert
    assert!(shared.serial_finished);
    assert_eq!(
        shared.maybe_failure,
        Some(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

#[test]
fn contradictory_consumed_reason_is_primary_before_serial_closure() {
    // Arrange
    let mut shared = SharedSerialState::default();
    let terminal = TerminalCaptureHandoff {
        terminal_consumed: false,
        pool_config_persisted: false,
        maybe_failure: Some(CampaignTerminalCategory::TerminalStateUnconfirmed),
    };

    // Act
    close_serial_input(&mut shared, Some(terminal));

    // Assert
    assert!(shared.serial_finished);
    assert!(!shared.terminal_consumed);
    assert_eq!(
        shared.maybe_failure,
        Some(CampaignTerminalCategory::TerminalStateUnconfirmed)
    );
}
