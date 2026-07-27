use crate::v1::messages::{PoolDifficulty, StratumResponseError};
use crate::v1::state::{MiningActivityStatus, WorkSubmissionGate};

use super::*;

#[test]
fn credentials_debug_redacts_pool_values() {
    // Arrange
    let credentials = LivePoolCredentials {
        username: "hidden-user".to_owned(),
        password: "hidden-password".to_owned(),
    };

    // Act
    let rendered = format!("{credentials:?}");

    // Assert
    assert!(rendered.contains("pool_credentials_redacted"));
    assert!(!rendered.contains("hidden-user"));
    assert!(!rendered.contains("hidden-password"));
}

#[test]
fn runtime_config_debug_redacts_credentials() {
    // Arrange
    let config = LiveRuntimeConfig {
        model: "ultra".to_owned(),
        version: "205".to_owned(),
        credentials: LivePoolCredentials {
            username: "hidden-user".to_owned(),
            password: "hidden-password".to_owned(),
        },
    };

    // Act
    let rendered = format!("{config:?}");

    // Assert
    assert!(rendered.contains("ultra"));
    assert!(rendered.contains("205"));
    assert!(rendered.contains("redacted"));
    assert!(!rendered.contains("hidden-user"));
    assert!(!rendered.contains("hidden-password"));
}

#[test]
fn runtime_debug_redacts_outbound_and_pool_context() {
    // Arrange
    let mut runtime = runtime();
    let _event = runtime.start();

    // Act
    let rendered = format!("{runtime:?}");

    // Assert
    assert!(rendered.contains("LiveStratumRuntime"));
    assert!(rendered.contains("outbound_actions"));
    assert!(rendered.contains("redacted"));
    assert!(!rendered.contains("synthetic-user"));
    assert!(!rendered.contains("synthetic-secret"));
}

#[test]
fn start_queues_configure_and_subscribe_handshake() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime.start();
    let actions = runtime.drain_actions();

    // Assert
    assert_eq!(event, LiveRuntimeEvent::Started);
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Connecting);
    assert!(matches!(
        actions.as_slice(),
        [
            LiveRuntimeAction::SendClientMessage(
                StratumV1ClientMessage::ConfigureVersionRolling { id, mask }
            ),
            LiveRuntimeAction::SendClientMessage(StratumV1ClientMessage::Subscribe {
                id: subscribe_id,
                user_agent
            })
        ] if id.raw() == 1
            && *mask == 0xffff_ffff
            && subscribe_id.raw() == 2
            && user_agent == "bitaxe/ultra/205"
    ));
}

#[test]
fn set_difficulty_updates_runtime_state() {
    // Arrange
    let mut runtime = runtime();
    let difficulty = PoolDifficulty { difficulty: 42.0 };

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::SetDifficulty(difficulty))
        .expect("difficulty should apply");

    // Assert
    assert_eq!(event, None);
    assert_eq!(runtime.state().maybe_pool_difficulty, Some(difficulty));
}

#[test]
fn set_extranonce_enables_later_work_without_emitting_event() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::SetExtranonce(extranonce()))
        .expect("extranonce should apply");

    // Assert
    assert_eq!(event, None);
    assert_eq!(runtime.maybe_extranonce, Some(extranonce()));
}

#[test]
fn version_mask_reload_is_consumed_once() {
    // Arrange
    let mut runtime = runtime();
    let mask = VersionMask { mask: 0x1fff_e000 };
    runtime
        .apply_server_message(StratumV1ServerMessage::SetVersionMask(mask))
        .expect("version mask should apply");

    // Act
    let first = runtime.take_pending_version_mask_reload();
    let second = runtime.take_pending_version_mask_reload();

    // Assert
    assert_eq!(first, Some(mask));
    assert_eq!(second, None);
}

#[test]
fn informational_server_messages_are_state_neutral() {
    // Arrange
    let messages = [
        StratumV1ServerMessage::ClientShowMessage("hello".to_owned()),
        StratumV1ServerMessage::ClientGetVersion,
        StratumV1ServerMessage::Ping {
            maybe_id: Some(StratumRequestId::new(8)),
        },
    ];

    // Act / Assert
    for message in messages {
        let mut runtime = runtime();
        let before = runtime.state().clone();
        let event = runtime
            .apply_server_message(message)
            .expect("informational message should apply");
        assert_eq!(event, None);
        assert_eq!(runtime.state(), &before);
    }
}

#[test]
fn reconnect_invalidates_work_and_enters_reconnecting_state() {
    // Arrange
    let mut runtime = runtime();
    let generation = runtime.production_registry().generation();

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::ClientReconnect)
        .expect("reconnect should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(
        runtime.production_registry().generation(),
        generation.next()
    );
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Reconnecting);
    assert_eq!(runtime.state().maybe_blocked_reason, Some("pool_reconnect"));
}

#[test]
fn direct_subscribe_response_queues_authorization() {
    // Arrange
    let mut runtime = runtime();
    let mut subscribe = response(true);
    subscribe.maybe_extranonce = Some(extranonce());

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(subscribe))
        .expect("subscribe response should apply");
    let actions = runtime.drain_actions();

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::Subscribed));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Subscribed);
    assert!(matches!(
        actions.as_slice(),
        [LiveRuntimeAction::SendClientMessage(StratumV1ClientMessage::Authorize {
            id,
            username,
            password
        })] if id.raw() == 1
            && username == "synthetic-user"
            && password == "synthetic-secret"
    ));
}

#[test]
fn direct_configure_response_stores_version_mask() {
    // Arrange
    let mut runtime = runtime();
    let mut configure = response(true);
    configure.maybe_version_mask = Some(VersionMask { mask: 0x1fff_e000 });

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(configure))
        .expect("configure response should apply");

    // Assert
    assert_eq!(event, None);
    assert_eq!(
        runtime.take_pending_version_mask_reload(),
        Some(VersionMask { mask: 0x1fff_e000 })
    );
}

#[test]
fn direct_authorize_response_enters_authorized_state() {
    // Arrange
    let mut runtime = runtime();
    let mut authorize = response(true);
    authorize.maybe_id = Some(StratumRequestId::new(3));

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(authorize))
        .expect("authorize response should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::Authorized));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Authorized);
}

#[test]
fn direct_rejected_response_invalidates_authorization() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(response(false)))
        .expect("rejected response should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Error);
    assert_eq!(
        runtime.state().maybe_blocked_reason,
        Some("authorization_reset")
    );
}

#[test]
fn unrelated_success_response_is_state_neutral() {
    // Arrange
    let mut runtime = runtime();
    let mut unrelated = response(true);
    unrelated.maybe_id = Some(StratumRequestId::new(99));

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(unrelated))
        .expect("unrelated response should apply");

    // Assert
    assert_eq!(event, None);
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Disconnected);
}

#[test]
fn matched_configure_success_raises_mask_reload() {
    // Arrange
    let mut runtime = runtime();
    let mut configure = response(true);
    configure.maybe_version_mask = Some(VersionMask { mask: 0x1fff_e000 });

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Configure, configure)
        .expect("configure response should apply");

    // Assert
    assert_eq!(event, None);
    assert_eq!(
        runtime.take_pending_version_mask_reload(),
        Some(VersionMask { mask: 0x1fff_e000 })
    );
}

#[test]
fn matched_configure_rejection_fails_closed() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Configure, response(false))
        .expect("configure rejection should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Error);
}

#[test]
fn matched_subscribe_success_queues_authorization() {
    // Arrange
    let mut runtime = runtime();
    let mut subscribe = response(true);
    subscribe.maybe_extranonce = Some(extranonce());

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Subscribe, subscribe)
        .expect("subscribe response should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::Subscribed));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Subscribed);
    assert!(matches!(
        runtime.drain_actions().as_slice(),
        [LiveRuntimeAction::SendClientMessage(
            StratumV1ClientMessage::Authorize { .. }
        )]
    ));
}

#[test]
fn matched_subscribe_requires_extranonce_assignment() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let result = runtime.apply_matched_response(RuntimeRequestKind::Subscribe, response(true));

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::MissingField("subscribe_extranonce"))
    );
}

#[test]
fn matched_subscribe_rejection_fails_closed() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Subscribe, response(false))
        .expect("subscribe rejection should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Error);
}

#[test]
fn matched_authorize_success_enters_authorized_state() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Authorize, response(true))
        .expect("authorize response should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::Authorized));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Authorized);
}

#[test]
fn matched_authorize_rejection_fails_closed() {
    // Arrange
    let mut runtime = runtime();

    // Act
    let event = runtime
        .apply_matched_response(RuntimeRequestKind::Authorize, response(false))
        .expect("authorize rejection should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Error);
}

#[test]
fn explicit_submission_block_sets_safe_state() {
    // Arrange
    let mut runtime = runtime();

    // Act
    runtime.block_work_submission("test_block");

    // Assert
    assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
    assert_eq!(
        runtime.state().mining_activity,
        MiningActivityStatus::SafeBlocked
    );
    assert_eq!(runtime.state().maybe_blocked_reason, Some("test_block"));
}

#[test]
fn runtime_generation_can_be_rebased() {
    // Arrange
    let mut runtime = runtime();
    let rebased = PoolSessionGeneration::initial().next();

    // Act
    runtime.rebase_generation(rebased);

    // Assert
    assert_eq!(runtime.production_registry().generation(), rebased);
}

#[test]
fn response_error_shape_does_not_affect_runtime_rejection_policy() {
    // Arrange
    let mut runtime = runtime();
    let mut rejected = response(false);
    rejected.maybe_error = Some(StratumResponseError {
        maybe_code: Some(21),
        message: "raw pool text".to_owned(),
    });

    // Act
    let event = runtime
        .apply_server_message(StratumV1ServerMessage::Response(rejected))
        .expect("rejection should apply");

    // Assert
    assert_eq!(event, Some(LiveRuntimeEvent::WorkInvalidated));
    assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Error);
}
