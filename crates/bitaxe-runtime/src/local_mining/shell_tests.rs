use super::*;

fn id(raw: u64) -> MiningCampaignLeaseId {
    MiningCampaignLeaseId::new(raw).expect("lease id")
}

#[test]
fn a_held_lease_is_running() {
    // Arrange / Act / Assert
    assert_eq!(
        run_tick(id(2), Some(id(2)), Some(id(1)), false),
        RunTick::Running
    );
}

#[test]
fn a_consumed_lease_with_stopped_hardware_ended() {
    // Arrange / Act / Assert
    assert_eq!(run_tick(id(2), None, Some(id(2)), true), RunTick::Ended);
}

#[test]
fn a_consumed_lease_still_stopping_is_running() {
    // Arrange / Act / Assert
    assert_eq!(run_tick(id(2), None, Some(id(2)), false), RunTick::Running);
}

#[test]
fn an_earlier_consumed_lease_never_ends_a_new_run() {
    // Arrange: the session still shows the previous run's consumption.
    let tick = run_tick(id(2), Some(id(2)), Some(id(1)), true);
    // Act / Assert
    assert_eq!(tick, RunTick::Running);
}

#[test]
fn a_lease_the_session_never_held_was_not_adopted() {
    // Arrange / Act / Assert
    assert_eq!(
        run_tick(id(2), None, Some(id(1)), true),
        RunTick::NotAdopted
    );
}

#[test]
fn run_after_paused_is_a_resume() {
    // Arrange
    let mut edge = ResumeEdge::default();
    edge.observe(MiningOperatorIntent::Paused);
    // Act / Assert
    assert!(edge.observe(MiningOperatorIntent::Run));
}

#[test]
fn run_from_boot_or_after_run_is_no_resume() {
    // Arrange
    let mut edge = ResumeEdge::default();
    // Act
    let first = edge.observe(MiningOperatorIntent::Run);
    let again = edge.observe(MiningOperatorIntent::Run);
    // Assert
    assert!(!first && !again);
}

#[test]
fn a_resume_right_after_a_device_pause_is_seen() {
    // Arrange: the operator resumed before the owner's next tick saw Paused.
    let mut edge = ResumeEdge::default();
    edge.observe(MiningOperatorIntent::Run);
    edge.note_device_pause();
    // Act / Assert
    assert!(edge.observe(MiningOperatorIntent::Run));
}

#[test]
fn a_device_pause_names_its_cause() {
    // Arrange / Act / Assert
    let paused = Some(LocalHoldReason::Paused);
    assert_eq!(
        api_hold(paused, true, false),
        Some(LocalHoldReason::Overheat)
    );
    assert_eq!(
        api_hold(paused, false, true),
        Some(LocalHoldReason::RestartGuardHeld)
    );
}

#[test]
fn operator_pauses_owners_and_links_show_no_local_reason() {
    // Arrange / Act / Assert
    for hold in [
        LocalHoldReason::Paused,
        LocalHoldReason::OtherBootOwner,
        LocalHoldReason::GateLink,
    ] {
        assert_eq!(api_hold(Some(hold), false, false), None, "{hold:?}");
    }
}

#[test]
fn other_holds_are_named_as_they_are() {
    // Arrange / Act / Assert
    assert_eq!(
        api_hold(Some(LocalHoldReason::NetworkUnavailable), false, false),
        Some(LocalHoldReason::NetworkUnavailable)
    );
}
