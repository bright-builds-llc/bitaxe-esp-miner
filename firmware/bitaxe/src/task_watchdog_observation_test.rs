use super::*;
use bitaxe_core::runtime_health::{
    TaskWatchdogOwnerSubphase, TaskWatchdogReadOutcome, TaskWatchdogWaitState,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Barrier;
use std::time::Duration;

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
fn preempted_writer_is_serialized_into_one_complete_snapshot() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record(TaskWatchdogObservation::fed(1, 100));
    let publication_started = Barrier::new(2);

    // Act
    let snapshot = std::thread::scope(|scope| {
        scope.spawn(|| {
            let mut state = store.state.lock().expect("fresh state lock");
            state.owner_phase = TaskWatchdogOwnerPhase::HandlingInbox;
            publication_started.wait();
            std::thread::sleep(Duration::from_millis(20));
            TaskWatchdogObservationStore::record_history(
                &mut state.history,
                TaskWatchdogObservation::fed(2, 200),
            );
            state.owner_subphase = TaskWatchdogOwnerSubphase::EffectPollChip;
        });
        publication_started.wait();
        store.coherent_observation()
    });

    // Assert
    assert_eq!(snapshot.read_outcome, TaskWatchdogReadOutcome::Stable);
    assert_eq!(
        snapshot.maybe_latest,
        Some(TaskWatchdogObservation::fed(2, 200))
    );
    assert_eq!(snapshot.owner_phase, TaskWatchdogOwnerPhase::HandlingInbox);
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::EffectPollChip
    );
}

#[test]
fn poisoned_history_fails_closed_without_mixed_atomic_facts() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record_owner_wait(Some(500));
    let poisoned = catch_unwind(AssertUnwindSafe(|| {
        let _state = store.state.lock().expect("fresh state lock");
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
    store.record_owner_progress(
        TaskWatchdogOwnerSubphase::EffectPollChip,
        Some(TaskWatchdogObservation::fed(1, 100)),
    );

    // Act
    let effect = store.coherent_observation();
    store.record_owner_phase(TaskWatchdogOwnerPhase::PublishingCampaignStatus);
    let cleared = store.coherent_observation();

    // Assert
    assert_eq!(effect.owner_phase, TaskWatchdogOwnerPhase::HandlingInbox);
    assert_eq!(
        effect.maybe_latest,
        Some(TaskWatchdogObservation::fed(1, 100))
    );
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

#[test]
fn owner_progress_without_a_new_feed_preserves_history_and_updates_subphase() {
    // Arrange
    let store = TaskWatchdogObservationStore::new();
    store.record(TaskWatchdogObservation::SubscriptionFailed);

    // Act
    store.record_owner_progress(TaskWatchdogOwnerSubphase::SessionEvaluation, None);
    let snapshot = store.coherent_observation();

    // Assert
    assert_eq!(
        snapshot.maybe_latest,
        Some(TaskWatchdogObservation::SubscriptionFailed)
    );
    assert_eq!(
        snapshot.owner_subphase,
        TaskWatchdogOwnerSubphase::SessionEvaluation
    );
}
