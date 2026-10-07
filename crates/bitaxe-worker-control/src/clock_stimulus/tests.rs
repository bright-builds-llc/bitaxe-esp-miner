use super::*;

fn armed_at(real_now: u64) -> ClockStimulus {
    let mut stimulus = ClockStimulus::new();
    assert!(stimulus.arm(real_now));
    stimulus
}

#[test]
fn idle_stimulus_returns_the_real_clock() {
    // Arrange
    let mut stimulus = ClockStimulus::new();

    // Act
    let sampled = stimulus.sample(10_000, Some(9_000));

    // Assert
    assert_eq!((sampled, stimulus.state_label()), (10_000, "idle"));
}

#[test]
fn armed_stimulus_lands_one_offset_below_the_last_observation_once() {
    // Arrange
    let mut stimulus = armed_at(10_000);

    // Act
    let first = stimulus.sample(10_500, Some(10_400));
    let second = stimulus.sample(10_600, Some(9_400));

    // Assert
    assert_eq!((first, second), (9_400, 10_600));
    assert_eq!(stimulus.state_label(), "consumed");
}

#[test]
fn armed_stimulus_uses_the_real_clock_when_it_is_below_the_last_observation() {
    // Arrange
    let mut stimulus = armed_at(10_000);

    // Act
    let sampled = stimulus.sample(10_100, Some(10_900));

    // Assert
    assert_eq!(sampled, 9_100);
}

#[test]
fn armed_stimulus_expires_two_seconds_after_arming() {
    // Arrange
    let mut last_millisecond = armed_at(10_000);
    let mut boundary = armed_at(10_000);

    // Act
    let inside = last_millisecond.sample(11_999, Some(11_000));
    let outside = boundary.sample(12_000, Some(11_000));

    // Assert
    assert_eq!((inside, outside), (10_000, 12_000));
    assert_eq!(boundary.state_label(), "expired");
}

#[test]
fn stimulus_cannot_be_armed_twice_in_one_boot() {
    // Arrange
    let mut stimulus = armed_at(10_000);
    stimulus.sample(10_001, Some(10_000));

    // Act
    let rearmed = stimulus.arm(20_000);

    // Assert
    assert!(!rearmed);
    assert!(!stimulus.can_prepare());
}

#[test]
fn cancelling_an_armed_stimulus_spends_it_without_a_sample() {
    // Arrange
    let mut stimulus = armed_at(10_000);

    // Act
    stimulus.cancel();
    let sampled = stimulus.sample(10_001, Some(10_000));

    // Assert
    assert_eq!((sampled, stimulus.state_label()), (10_001, "expired"));
}

#[test]
fn a_pending_reply_blocks_another_preparation() {
    // Arrange
    let mut stimulus = ClockStimulus::new();
    stimulus.prepare(7);

    // Act
    let blocked = stimulus.can_prepare();
    let taken = stimulus.take_pending();

    // Assert
    assert!(!blocked);
    assert_eq!(taken, Some(7));
    assert!(stimulus.can_prepare());
}
