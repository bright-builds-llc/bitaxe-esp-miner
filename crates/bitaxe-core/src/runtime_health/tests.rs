use super::*;

const TEST_TASK_WATCHDOG_TIMEOUT_MILLIS: u64 = 5_000;

const fn timing(
    current_monotonic_millis: u64,
    publish_interval_millis: u64,
) -> RuntimeHealthTiming {
    RuntimeHealthTiming::new(
        current_monotonic_millis,
        publish_interval_millis,
        TEST_TASK_WATCHDOG_TIMEOUT_MILLIS,
    )
}

fn checkpoint(sequence: u64, observed_at_millis: u64) -> CheckpointObservation {
    CheckpointObservation::new("telemetry", sequence, observed_at_millis)
        .expect("test checkpoint should be valid")
}

#[test]
fn passive_self_test_states_have_exact_serialized_spellings() {
    // Arrange
    let cases = [
        (PassiveSelfTestState::Idle, "idle"),
        (PassiveSelfTestState::Blocked, "blocked"),
        (PassiveSelfTestState::Running, "running"),
        (PassiveSelfTestState::Passed, "passed"),
        (PassiveSelfTestState::Failed, "failed"),
        (PassiveSelfTestState::Canceled, "canceled"),
        (PassiveSelfTestState::Unavailable, "unavailable"),
    ];

    // Act / Assert
    for (state, expected) in cases {
        assert_eq!(state.as_str(), expected);
    }
}

#[test]
fn health_and_watchdog_vocabulary_has_exact_serialized_spellings() {
    // Arrange
    let supervisor = [
        (SupervisorAvailability::Available, "available"),
        (SupervisorAvailability::Unavailable, "unavailable"),
    ];
    let health = [
        (CheckpointHealth::Healthy, "healthy"),
        (CheckpointHealth::Stale, "stale"),
        (CheckpointHealth::Unhealthy, "unhealthy"),
        (CheckpointHealth::Unavailable, "unavailable"),
    ];
    let participation = [
        (TaskWatchdogParticipation::Participating, "participating"),
        (
            TaskWatchdogParticipation::NotParticipating,
            "not_participating",
        ),
        (TaskWatchdogParticipation::Unavailable, "unavailable"),
    ];
    let read_outcomes = [
        (TaskWatchdogReadOutcome::Stable, "stable"),
        (TaskWatchdogReadOutcome::Uninitialized, "uninitialized"),
        (TaskWatchdogReadOutcome::RetryExhausted, "retry_exhausted"),
        (TaskWatchdogReadOutcome::HistoryPoisoned, "history_poisoned"),
    ];

    // Act / Assert
    for (value, expected) in supervisor {
        assert_eq!(value.as_str(), expected);
    }
    for (value, expected) in health {
        assert_eq!(value.as_str(), expected);
    }
    for (value, expected) in participation {
        assert_eq!(value.as_str(), expected);
    }
    for (value, expected) in read_outcomes {
        assert_eq!(value.as_str(), expected);
    }
}

#[test]
fn watchdog_store_read_failures_replace_generic_unproved_reason() {
    // Arrange
    let unavailable = RuntimeHealthSnapshot::fixture_unavailable();

    // Act
    let retry_exhausted = unavailable
        .clone()
        .with_task_watchdog_read_outcome(TaskWatchdogReadOutcome::RetryExhausted);
    let history_poisoned =
        unavailable.with_task_watchdog_read_outcome(TaskWatchdogReadOutcome::HistoryPoisoned);

    // Assert
    assert_eq!(
        retry_exhausted.maybe_task_watchdog_reason(),
        Some("snapshot_retry_exhausted")
    );
    assert_eq!(
        history_poisoned.maybe_task_watchdog_reason(),
        Some("snapshot_history_poisoned")
    );
    assert_eq!(
        retry_exhausted.task_watchdog_participation(),
        TaskWatchdogParticipation::NotParticipating
    );
    assert_eq!(retry_exhausted.maybe_task_watchdog_feed_sequence(), None);
}

#[test]
fn task_watchdog_owner_phases_have_exact_closed_spellings_and_encoding() {
    // Arrange
    let cases = [
        (TaskWatchdogOwnerPhase::Unavailable, "unavailable"),
        (TaskWatchdogOwnerPhase::Subscribing, "subscribing"),
        (TaskWatchdogOwnerPhase::LoopStart, "loop_start"),
        (TaskWatchdogOwnerPhase::WaitingInbox, "waiting_inbox"),
        (TaskWatchdogOwnerPhase::HandlingInbox, "handling_inbox"),
        (
            TaskWatchdogOwnerPhase::HandlingObservation,
            "handling_observation",
        ),
        (
            TaskWatchdogOwnerPhase::HandlingReadiness,
            "handling_readiness",
        ),
        (
            TaskWatchdogOwnerPhase::PublishingCampaignStatus,
            "publishing_campaign_status",
        ),
        (
            TaskWatchdogOwnerPhase::ServicingHashrate,
            "servicing_hashrate",
        ),
        (TaskWatchdogOwnerPhase::Shutdown, "shutdown"),
    ];

    // Act / Assert
    for (phase, expected) in cases {
        assert_eq!(phase.as_str(), expected);
        assert_eq!(TaskWatchdogOwnerPhase::from_u8(phase as u8), phase);
    }
    assert_eq!(
        TaskWatchdogOwnerPhase::from_u8(u8::MAX),
        TaskWatchdogOwnerPhase::Unavailable
    );
}

#[test]
fn runtime_health_snapshot_accepts_independent_owner_phase() {
    // Arrange / Act
    let snapshot = RuntimeHealthSnapshot::fixture_unavailable()
        .with_task_watchdog_owner_phase(TaskWatchdogOwnerPhase::WaitingInbox);

    // Assert
    assert_eq!(
        snapshot.task_watchdog_owner_phase(),
        TaskWatchdogOwnerPhase::WaitingInbox
    );
}

#[test]
fn checkpoint_category_rejects_empty_non_ascii_and_overlong_text() {
    // Arrange
    let overlong = "x".repeat(CHECKPOINT_CATEGORY_MAX_ASCII_BYTES + 1);

    // Act
    let empty = CheckpointCategory::new("");
    let non_ascii = CheckpointCategory::new("télémetry");
    let too_long = CheckpointCategory::new(&overlong);

    // Assert
    assert_eq!(empty, Err(CheckpointObservationError::EmptyCategory));
    assert_eq!(non_ascii, Err(CheckpointObservationError::CategoryNotAscii));
    assert_eq!(too_long, Err(CheckpointObservationError::CategoryTooLong));
}

#[test]
fn checkpoint_transition_rejects_sequence_and_monotonic_time_regression() {
    // Arrange
    let previous = checkpoint(7, 1_000);
    let sequence_regression = checkpoint(6, 1_100);
    let time_regression = checkpoint(8, 999);

    // Act
    let sequence_result = sequence_regression.validate_after(&previous);
    let time_result = time_regression.validate_after(&previous);

    // Assert
    assert_eq!(
        sequence_result,
        Err(CheckpointObservationError::SequenceRegression)
    );
    assert_eq!(
        time_result,
        Err(CheckpointObservationError::MonotonicTimeRegression)
    );
}

#[test]
fn checkpoint_transition_rejects_same_sequence_mutation() {
    // Arrange
    let previous = checkpoint(7, 1_000);
    let changed_timestamp = checkpoint(7, 1_001);

    // Act
    let result = changed_timestamp.validate_after(&previous);

    // Assert
    assert_eq!(result, Err(CheckpointObservationError::SameSequenceChanged));
}

#[test]
fn missing_checkpoint_is_explicitly_unavailable() {
    // Arrange / Act
    let snapshot = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        None,
        timing(10_000, 500),
    );

    // Assert
    assert_eq!(
        snapshot.supervisor_availability(),
        SupervisorAvailability::Unavailable
    );
    assert_eq!(snapshot.checkpoint_health(), CheckpointHealth::Unavailable);
    assert_eq!(snapshot.maybe_checkpoint_sequence(), None);
}

#[test]
fn exact_age_boundaries_derive_healthy_stale_and_unhealthy() {
    // Arrange
    let latest = checkpoint(8, 1_000);

    // Act
    let healthy = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        None,
        timing(2_500, 500),
    );
    let stale_start = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        None,
        timing(2_501, 500),
    );
    let stale_end = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        None,
        timing(6_000, 500),
    );
    let unhealthy = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        None,
        timing(6_001, 500),
    );

    // Assert
    assert_eq!(healthy.checkpoint_health(), CheckpointHealth::Healthy);
    assert_eq!(stale_start.checkpoint_health(), CheckpointHealth::Stale);
    assert_eq!(stale_end.checkpoint_health(), CheckpointHealth::Stale);
    assert_eq!(unhealthy.checkpoint_health(), CheckpointHealth::Unhealthy);
}

#[test]
fn fixed_sequence_ages_from_healthy_to_stale_to_unhealthy() {
    // Arrange
    let latest = checkpoint(11, 2_000);

    // Act
    let snapshots = [2_100, 3_501, 7_001].map(|now| {
        RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&latest),
            None,
            None,
            timing(now, 500),
        )
    });

    // Assert
    assert_eq!(snapshots[0].checkpoint_health(), CheckpointHealth::Healthy);
    assert_eq!(snapshots[1].checkpoint_health(), CheckpointHealth::Stale);
    assert_eq!(
        snapshots[2].checkpoint_health(),
        CheckpointHealth::Unhealthy
    );
    assert!(snapshots
        .iter()
        .all(|snapshot| snapshot.maybe_checkpoint_sequence() == Some(11)));
}

#[test]
fn recovery_requires_a_sequence_advance() {
    // Arrange
    let previous = checkpoint(11, 2_000);
    let unchanged_sequence = checkpoint(11, 8_000);
    let advanced_sequence = checkpoint(12, 8_000);

    // Act
    let frozen = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&previous),
        None,
        None,
        timing(7_001, 500),
    );
    let synthetic_recovery = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        Some(&previous),
        Some(&unchanged_sequence),
        None,
        None,
        timing(8_001, 500),
    );
    let real_recovery = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        Some(&previous),
        Some(&advanced_sequence),
        None,
        None,
        timing(8_001, 500),
    );

    // Assert
    assert_eq!(frozen.checkpoint_health(), CheckpointHealth::Unhealthy);
    assert_eq!(
        synthetic_recovery.checkpoint_health(),
        CheckpointHealth::Unavailable
    );
    assert_eq!(real_recovery.checkpoint_health(), CheckpointHealth::Healthy);
}

#[test]
fn invalid_time_or_threshold_arithmetic_is_unavailable() {
    // Arrange
    let latest = checkpoint(4, 100);

    // Act
    let time_regression = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        Some(&latest),
        None,
        None,
        timing(99, 500),
    );
    let threshold_overflow = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        Some(&latest),
        None,
        None,
        timing(100, u64::MAX),
    );

    // Assert
    assert_eq!(
        time_regression.checkpoint_health(),
        CheckpointHealth::Unavailable
    );
    assert_eq!(
        threshold_overflow.checkpoint_health(),
        CheckpointHealth::Unavailable
    );
}

#[test]
fn healthy_supervisor_does_not_imply_task_watchdog_participation() {
    // Arrange
    let latest = checkpoint(42, 10_000);

    // Act
    let snapshot = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        Some(&latest),
        None,
        None,
        timing(10_100, 500),
    );

    // Assert
    assert_eq!(
        snapshot.supervisor_availability(),
        SupervisorAvailability::Available
    );
    assert_eq!(snapshot.checkpoint_health(), CheckpointHealth::Healthy);
    assert_eq!(
        snapshot.task_watchdog_participation(),
        TaskWatchdogParticipation::Unavailable
    );
    assert_eq!(snapshot.maybe_task_watchdog_reason(), Some("unproved"));
}

#[test]
fn watchdog_feed_after_old_two_second_boundary_remains_fresh() {
    // Arrange
    let latest = TaskWatchdogObservation::fed(7, 1_000);

    // Act
    let fresh = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        Some(latest),
        timing(3_001, 500),
    );

    // Assert
    assert_eq!(
        fresh.task_watchdog_participation(),
        TaskWatchdogParticipation::Participating
    );
    assert_eq!(fresh.maybe_task_watchdog_reason(), Some("feed_fresh"));
    assert_eq!(fresh.maybe_task_watchdog_feed_sequence(), Some(7));
    assert_eq!(fresh.maybe_task_watchdog_feed_age_millis(), Some(2_001));
}

#[test]
fn watchdog_feed_accepts_exact_configured_timeout() {
    // Arrange
    let latest = TaskWatchdogObservation::fed(7, 1_000);

    // Act
    let fresh = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        Some(latest),
        timing(6_000, 500),
    );

    // Assert
    assert_eq!(
        fresh.task_watchdog_participation(),
        TaskWatchdogParticipation::Participating
    );
    assert_eq!(fresh.maybe_task_watchdog_reason(), Some("feed_fresh"));
    assert_eq!(fresh.maybe_task_watchdog_feed_age_millis(), Some(5_000));
}

#[test]
fn watchdog_feed_is_stale_after_configured_timeout() {
    // Arrange
    let latest = TaskWatchdogObservation::fed(7, 1_000);

    // Act
    let stale = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        Some(latest),
        timing(6_001, 500),
    );

    // Assert
    assert_eq!(
        stale.task_watchdog_participation(),
        TaskWatchdogParticipation::NotParticipating
    );
    assert_eq!(stale.maybe_task_watchdog_reason(), Some("feed_stale"));
    assert_eq!(stale.maybe_task_watchdog_feed_age_millis(), Some(5_001));
}

#[test]
fn watchdog_sequence_regression_is_rejected() {
    // Arrange
    let previous = TaskWatchdogObservation::fed(8, 1_000);
    let latest = TaskWatchdogObservation::fed(7, 1_100);

    // Act
    let snapshot = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        Some(previous),
        Some(latest),
        timing(1_200, 500),
    );

    // Assert
    assert_eq!(
        snapshot.task_watchdog_participation(),
        TaskWatchdogParticipation::NotParticipating
    );
    assert_eq!(
        snapshot.maybe_task_watchdog_reason(),
        Some("invalid_observation")
    );
    assert_eq!(snapshot.maybe_task_watchdog_feed_sequence(), None);
}

#[test]
fn post_observation_time_sampling_closes_the_concurrent_feed_race() {
    // Arrange
    let copied_feed = TaskWatchdogObservation::fed(9, 1_001);

    // Act
    let stale_caller_time = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        Some(copied_feed),
        timing(1_000, 500),
    );
    let post_copy_time = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Unavailable,
        None,
        None,
        None,
        Some(copied_feed),
        timing(1_001, 500),
    );

    // Assert
    assert_eq!(
        stale_caller_time.maybe_task_watchdog_reason(),
        Some("invalid_observation")
    );
    assert_eq!(
        post_copy_time.task_watchdog_participation(),
        TaskWatchdogParticipation::Participating
    );
    assert_eq!(
        post_copy_time.maybe_task_watchdog_reason(),
        Some("feed_fresh")
    );
    assert_eq!(
        post_copy_time.maybe_task_watchdog_feed_age_millis(),
        Some(0)
    );
}

#[test]
fn watchdog_failures_have_closed_nonparticipating_reasons() {
    // Arrange
    let cases = [
        (
            TaskWatchdogObservation::SubscriptionFailed,
            "subscription_failed",
        ),
        (TaskWatchdogObservation::FeedFailed, "feed_failed"),
        (
            TaskWatchdogObservation::UnsubscriptionFailed,
            "unsubscription_failed",
        ),
        (TaskWatchdogObservation::Unsubscribed, "unsubscribed"),
    ];

    // Act / Assert
    for (observation, reason) in cases {
        let snapshot = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            None,
            None,
            Some(observation),
            timing(1_000, 500),
        );
        assert_eq!(
            snapshot.task_watchdog_participation(),
            TaskWatchdogParticipation::NotParticipating
        );
        assert_eq!(snapshot.maybe_task_watchdog_reason(), Some(reason));
    }
}
