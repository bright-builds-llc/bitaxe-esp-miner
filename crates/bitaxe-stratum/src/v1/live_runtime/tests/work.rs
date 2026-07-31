use bitaxe_asic::bm1366::production::ProductionAsicBlocker;

use crate::v1::messages::PoolDifficulty;
use crate::v1::state::{MiningActivityStatus, WorkSubmissionGate};

use super::*;

#[test]
fn notify_without_extranonce_blocks_work() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should be handled safely");

    // Assert
    assert_eq!(event, None);
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("extranonce_missing")
    );
    assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
}

#[test]
fn notify_queues_active_pool_work() {
    // Arrange
    let mut runtime = runtime_with_extranonce();

    // Act
    let event = runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");

    // Assert
    assert_eq!(
        event,
        Some(LiveRuntimeEvent::WorkQueued {
            clean_jobs: false,
            previous_block_changed: false,
        })
    );
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Active);
    assert_eq!(
        runtime.state().mining_activity,
        MiningActivityStatus::Active
    );
    assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Ready);
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("queued work should dispatch");
    assert_eq!(dispatch.work.stratum_job_id, "synthetic-job");
    assert_eq!(dispatch.work.asic_job_id, Bm1366JobId::new(8));
}

#[test]
fn clean_jobs_notify_invalidates_old_generation_before_queueing() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    let old_generation = runtime.production_registry().generation();

    // Act
    let event = runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(true)))
        .expect("clean notify should queue work");

    // Assert
    assert_eq!(
        event,
        Some(LiveRuntimeEvent::WorkQueued {
            clean_jobs: true,
            previous_block_changed: false,
        })
    );
    assert_eq!(
        runtime.production_registry().generation(),
        old_generation.next()
    );
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("fresh work should dispatch");
    assert!(!dispatch.work.clean_jobs);
}

#[test]
fn clean_changed_block_notify_reports_new_block_transition() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("initial notify should queue work");
    let mut changed = notify(true);
    changed.prev_block_hash = "11".repeat(32);

    // Act
    let event = runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(changed))
        .expect("clean changed-block notify should queue replacement work");

    // Assert
    assert_eq!(
        event,
        Some(LiveRuntimeEvent::WorkQueued {
            clean_jobs: true,
            previous_block_changed: true,
        })
    );
}

#[test]
fn changed_block_without_clean_jobs_fails_closed() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("initial notify should queue work");
    let mut inconsistent = notify(false);
    inconsistent.prev_block_hash = "22".repeat(32);

    // Act
    let event = runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(inconsistent))
        .expect("inconsistent notify should be classified without raw data");

    // Assert
    assert_eq!(
        event,
        Some(LiveRuntimeEvent::JobTransitionProtocolInconsistent)
    );
    assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("new_block_without_clean_jobs")
    );
}

#[test]
fn notify_applies_pool_difficulty_and_version_mask_to_work() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: 8.0,
        }))
        .expect("difficulty should apply");
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetVersionMask(VersionMask {
            mask: 0x1fff_e000,
        }))
        .expect("version mask should apply");

    // Act
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("work should dispatch");

    // Assert
    assert_eq!(
        dispatch.work.maybe_pool_difficulty,
        Some(PoolDifficulty { difficulty: 8.0 })
    );
    assert_eq!(
        dispatch.work.maybe_version_mask,
        Some(VersionMask { mask: 0x1fff_e000 })
    );
}

#[test]
fn malformed_notify_propagates_work_builder_error() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    let mut malformed = notify(false);
    malformed.prev_block_hash = "not-hex".to_owned();

    // Act
    let result = runtime.maybe_apply_server_message(StratumV1ServerMessage::Notify(malformed));

    // Assert
    assert!(matches!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "prev_block_hash",
            ..
        })
    ));
}

#[test]
fn regenerate_requires_held_notify() {
    // Arrange
    let mut runtime = runtime_with_extranonce();

    // Act
    let result = runtime.regenerate_work();

    // Assert
    assert_eq!(result, Err(StratumV1Error::MissingField("current_notify")));
}

#[test]
fn regenerate_requires_extranonce_assignment() {
    // Arrange
    let mut runtime = runtime();
    runtime.maybe_current_notify = Some(notify(false));

    // Act
    let result = runtime.regenerate_work();

    // Assert
    assert_eq!(result, Err(StratumV1Error::MissingField("extranonce")));
}

#[test]
fn regenerate_increments_extranonce_and_preserves_pool_context() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: 8.0,
        }))
        .expect("difficulty should apply");
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetVersionMask(VersionMask {
            mask: 0x1fff_e000,
        }))
        .expect("version mask should apply");
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");
    let _initial = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("initial work should dispatch");

    // Act
    let counter = runtime.regenerate_work().expect("work should regenerate");
    let regenerated = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("regenerated work should dispatch");

    // Assert
    assert_eq!(counter, 1);
    assert_eq!(regenerated.work.extranonce2, "01000000");
    assert!(!regenerated.work.clean_jobs);
    assert_eq!(
        regenerated.work.maybe_pool_difficulty,
        Some(PoolDifficulty { difficulty: 8.0 })
    );
    assert_eq!(
        regenerated.work.maybe_version_mask,
        Some(VersionMask { mask: 0x1fff_e000 })
    );
}

#[test]
fn session_replacement_clears_regeneration_context() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");

    // Act
    runtime.invalidate_for_session_replacement();

    // Assert
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("session_replacement")
    );
    assert_eq!(
        runtime.regenerate_work(),
        Err(StratumV1Error::MissingField("current_notify"))
    );
}

#[test]
fn authorization_reset_clears_regeneration_context() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");

    // Act
    runtime.invalidate_for_authorization_reset();

    // Assert
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("authorization_reset")
    );
    assert_eq!(
        runtime.regenerate_work(),
        Err(StratumV1Error::MissingField("current_notify"))
    );
}

#[test]
fn clean_jobs_invalidation_clears_regeneration_context() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");

    // Act
    runtime.invalidate_for_clean_jobs();

    // Assert
    assert_eq!(runtime.state().maybe_blocked_reason, Some("clean_jobs"));
    assert_eq!(
        runtime.regenerate_work(),
        Err(StratumV1Error::MissingField("current_notify"))
    );
}

#[test]
fn uncorrelated_bridge_observation_blocks_submission() {
    // Arrange
    let mut runtime = runtime();
    let observation = ProductionNonceObservation {
        observed_generation: runtime.production_registry().generation(),
        result: nonce_result(Bm1366JobId::new(8)),
    };

    // Act
    let outcome = runtime
        .apply_bridge_observation(observation)
        .expect("observation should classify");

    // Assert
    assert_eq!(
        outcome,
        BridgeObservationOutcome::Blocked {
            reason: ProductionAsicBlocker::JobUncorrelated
        }
    );
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("production_job_uncorrelated")
    );
}

#[test]
fn correlated_bridge_observation_queues_submit_action() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("work should dispatch");
    let observation = ProductionNonceObservation {
        observed_generation: dispatch.generation,
        result: nonce_result(dispatch.work.asic_job_id),
    };

    // Act
    let outcome = runtime
        .apply_bridge_observation(observation)
        .expect("observation should queue submit");
    let actions = runtime.drain_actions();

    // Assert
    assert_eq!(outcome, BridgeObservationOutcome::SubmitQueued);
    assert!(matches!(
        actions.as_slice(),
        [LiveRuntimeAction::SendSubmitShare {
            request_id,
            message: StratumV1ClientMessage::SubmitShare {
                username,
                job_id,
                ..
            },
            ..
        }] if request_id.raw() == 1
            && username == "synthetic-user"
            && job_id == "synthetic-job"
    ));
}

#[test]
fn below_target_candidate_is_counted_without_blocking_or_submitting() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: f64::MAX,
        }))
        .expect("high fixture difficulty should apply");
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("work should dispatch");
    let observation = ProductionNonceObservation {
        observed_generation: dispatch.generation,
        result: nonce_result(dispatch.work.asic_job_id),
    };

    // Act
    let outcome = runtime
        .apply_bridge_observation(observation)
        .expect("candidate should classify");

    // Assert
    assert_eq!(
        outcome,
        BridgeObservationOutcome::Ignored {
            reason: NonSubmitReason::BelowPoolTarget
        }
    );
    assert_eq!(runtime.state().counters.below_pool_target, 1);
    assert_eq!(runtime.state().counters.qualified_candidates, 0);
    assert!(runtime.drain_actions().is_empty());
    assert!(runtime.state().maybe_blocked_reason.is_none());
}

#[test]
fn submit_action_debug_redacts_share_context() {
    // Arrange
    let mut runtime = runtime_with_extranonce();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
        .expect("notify should queue work");
    let dispatch = runtime
        .production_registry_mut()
        .dispatch_next()
        .expect("work should dispatch");
    runtime
        .apply_bridge_observation(ProductionNonceObservation {
            observed_generation: dispatch.generation,
            result: nonce_result(dispatch.work.asic_job_id),
        })
        .expect("observation should queue submit");
    let action = runtime
        .drain_actions()
        .pop()
        .expect("submit action should exist");

    // Act
    let rendered = format!("{action:?}");

    // Assert
    assert!(rendered.contains("SendSubmitShare"));
    assert!(rendered.contains("client_message"));
    assert!(!rendered.contains("synthetic-user"));
    assert!(!rendered.contains("synthetic-job"));
    assert!(!rendered.contains("12345678"));
}

#[test]
fn client_action_debug_redacts_message() {
    // Arrange
    let mut runtime = runtime();
    let _event = runtime.start();
    let action = runtime.drain_actions().remove(0);

    // Act
    let rendered = format!("{action:?}");

    // Assert
    assert!(rendered.contains("SendClientMessage"));
    assert!(rendered.contains("redacted"));
    assert!(!rendered.contains("mining.configure"));
}
