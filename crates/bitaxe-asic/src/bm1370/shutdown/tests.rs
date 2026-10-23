use super::*;
use crate::bm1370::init::CORE_COUNT;

#[test]
fn the_shutdown_ramps_down_in_grid_steps_to_the_reset_frequency() {
    // Act
    let commands = safe_shutdown_commands(525, 1, CORE_COUNT).expect("shutdown");
    // Assert
    let ramp_writes = commands
        .iter()
        .filter(|command| matches!(command, Bm1370Command::SetFrequency(_)))
        .count();
    assert_eq!(ramp_writes, 76);
}

#[test]
fn the_shutdown_ends_on_the_nonce_space_for_50_mhz() {
    // Act
    let commands = safe_shutdown_commands(525, 1, CORE_COUNT).expect("shutdown");
    // Assert: the fixture's 50 MHz nonce space.
    assert_eq!(
        commands.last(),
        Some(&Bm1370Command::SetNonceSpace(8_388_340))
    );
}

#[test]
fn the_shutdown_actions_wait_for_the_last_frame_to_leave() {
    // Act
    let actions = safe_shutdown_command_actions(525, 1, CORE_COUNT).expect("actions");
    // Assert
    assert_eq!(actions.last(), Some(&Bm1366AdapterAction::WAIT_TX_DONE));
}

#[test]
fn every_ramp_step_is_a_frame_then_its_delay() {
    // Act
    let actions = safe_shutdown_command_actions(525, 1, CORE_COUNT).expect("actions");
    // Assert: 76 steps of frame and delay, the nonce frame, then the wait.
    let frames = actions
        .iter()
        .filter(|action| matches!(action, Bm1366AdapterAction::WriteFrame(_)))
        .count();
    let delays = actions
        .iter()
        .filter(|action| matches!(action, Bm1366AdapterAction::DelayMs(100)))
        .count();
    assert_eq!((frames, delays, actions.len()), (77, 76, 154));
}
