use super::*;
use bitaxe_core::runtime_health::{
    TaskWatchdogOwnerSubphase, TaskWatchdogReadOutcome, TaskWatchdogWaitState,
};
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn stable_publications_round_trip_as_one_snapshot() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record(TaskWatchdogObservation::fed(1, 100));
    store.record(TaskWatchdogObservation::fed(2, 200));
    store.record_owner_wait(Some(500));

    // Act
    let snapshot = store.coherent_observation();

    // Assert
    assert_eq!(
        snapshot.maybe_previous,
        Some(TaskWatchdogObservation::fed(1, 100))
    );
    assert_eq!(
        snapshot.maybe_latest,
        Some(TaskWatchdogObservation::fed(2, 200))
    );
    assert_eq!(snapshot.read_outcome, TaskWatchdogReadOutcome::Stable);
    assert_eq!(snapshot.owner_phase, TaskWatchdogOwnerPhase::WaitingInbox);
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::Unavailable
    );
    assert_eq!(
        snapshot.owner_wait.state_at(300),
        TaskWatchdogWaitState::WithinDeadline
    );
}

#[test]
fn old_feed_and_new_wait_interleaving_retries_to_one_owner_instant() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record(TaskWatchdogObservation::fed(1, 100));
    store.record_owner_phase(TaskWatchdogOwnerPhase::LoopStart);
    let injected = Cell::new(false);

    // Act
    let snapshot = store.coherent_observation_with(|| {
        if injected.replace(true) {
            return;
        }
        store.record(TaskWatchdogObservation::fed(2, 200));
        store.record_owner_wait(Some(500));
    });

    // Assert
    assert_eq!(
        snapshot.maybe_latest,
        Some(TaskWatchdogObservation::fed(2, 200))
    );
    assert_eq!(snapshot.read_outcome, TaskWatchdogReadOutcome::Stable);
    assert_eq!(snapshot.owner_phase, TaskWatchdogOwnerPhase::WaitingInbox);
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::Unavailable
    );
    assert_eq!(
        snapshot.owner_wait.state_at(300),
        TaskWatchdogWaitState::WithinDeadline
    );
}

#[test]
fn repeated_publication_races_exhaust_to_closed_default() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record(TaskWatchdogObservation::fed(1, 100));
    let publication_count = Cell::new(0_u8);

    // Act
    let snapshot = store.coherent_observation_with(|| {
        publication_count.set(publication_count.get().saturating_add(1));
        store.record_owner_phase(TaskWatchdogOwnerPhase::LoopStart);
    });

    // Assert
    assert_eq!(usize::from(publication_count.get()), COHERENT_READ_ATTEMPTS);
    assert_eq!(snapshot.maybe_previous, None);
    assert_eq!(snapshot.maybe_latest, None);
    assert_eq!(
        snapshot.read_outcome,
        TaskWatchdogReadOutcome::RetryExhausted
    );
    assert_eq!(snapshot.owner_phase, TaskWatchdogOwnerPhase::Unavailable);
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::Unavailable
    );
    assert_eq!(
        snapshot.owner_wait.state_at(0),
        TaskWatchdogWaitState::NotWaiting
    );
}

#[test]
fn poisoned_history_fails_closed_without_mixed_atomic_facts() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record_owner_wait(Some(500));
    let poisoned = catch_unwind(AssertUnwindSafe(|| {
        let _history = store.history.lock().expect("fresh history lock");
        panic!("poison observation history for the fail-closed regression");
    }));
    assert!(poisoned.is_err());

    // Act
    let snapshot = store.coherent_observation();

    // Assert
    assert_eq!(snapshot.maybe_previous, None);
    assert_eq!(snapshot.maybe_latest, None);
    assert_eq!(
        snapshot.read_outcome,
        TaskWatchdogReadOutcome::HistoryPoisoned
    );
    assert_eq!(snapshot.owner_phase, TaskWatchdogOwnerPhase::Unavailable);
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::Unavailable
    );
    assert_eq!(
        snapshot.owner_wait.state_at(0),
        TaskWatchdogWaitState::NotWaiting
    );
}

#[test]
fn untouched_store_is_explicitly_uninitialized() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();

    // Act
    let snapshot = store.coherent_observation();

    // Assert
    assert_eq!(
        snapshot.read_outcome,
        TaskWatchdogReadOutcome::Uninitialized
    );
    assert_eq!(snapshot.maybe_latest, None);
}

#[test]
fn owner_phase_clears_subphase_and_subphase_updates_preserve_phase() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record_owner_phase(TaskWatchdogOwnerPhase::HandlingInbox);
    store.record_owner_subphase(TaskWatchdogOwnerSubphase::EffectPollChip);

    // Act
    let effect = store.coherent_observation();
    store.record_owner_phase(TaskWatchdogOwnerPhase::PublishingCampaignStatus);
    let cleared = store.coherent_observation();

    // Assert
    assert_eq!(effect.owner_phase, TaskWatchdogOwnerPhase::HandlingInbox);
    assert_eq!(
        effect.owner_subphase,
        TaskWatchdogOwnerSubphase::EffectPollChip
    );
    assert_eq!(
        cleared.owner_phase,
        TaskWatchdogOwnerPhase::PublishingCampaignStatus
    );
    assert_eq!(
        cleared.owner_subphase,
        TaskWatchdogOwnerSubphase::Unavailable
    );
}
