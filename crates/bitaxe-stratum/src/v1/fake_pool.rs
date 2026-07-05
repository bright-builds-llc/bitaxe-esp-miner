use std::fmt;

use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;
use crate::v1::live_runtime::{LiveRuntimeAction, LiveStratumRuntime};
use crate::v1::messages::{StratumResponse, StratumV1ClientMessage, StratumV1ServerMessage};
use crate::v1::production_work::{CorrelationOutcome, ProductionNonceObservation, SubmitIntent};
use crate::v1::state::{MiningRuntimeState, PoolLifecycleStatus, ShareDifficulty};
use crate::v1::submit_response::{
    classify_submit_response, SubmitClassification, SubmitResponseObservation,
};

#[derive(Clone, PartialEq)]
pub enum FakePoolEvent {
    ExpectClient(StratumV1ClientMessage),
    SendServer(StratumV1ServerMessage),
    Disconnect,
    Timeout,
    MalformedResponse,
    BlockedPrerequisite { reason: &'static str },
    FallbackActivation,
    NoResponse,
    ClassifySubmitResponse(StratumResponse),
    ClassifyStaleSubmitResponse(StratumResponse),
}

impl fmt::Debug for FakePoolEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectClient(_) => formatter
                .debug_struct("FakePoolEvent::ExpectClient")
                .field("client_message", &"redacted")
                .finish(),
            Self::SendServer(_) => formatter
                .debug_struct("FakePoolEvent::SendServer")
                .field("server_message", &"redacted")
                .finish(),
            Self::Disconnect => formatter.write_str("FakePoolEvent::Disconnect"),
            Self::Timeout => formatter.write_str("FakePoolEvent::Timeout"),
            Self::MalformedResponse => formatter.write_str("FakePoolEvent::MalformedResponse"),
            Self::BlockedPrerequisite { reason } => formatter
                .debug_struct("FakePoolEvent::BlockedPrerequisite")
                .field("reason", reason)
                .finish(),
            Self::FallbackActivation => formatter.write_str("FakePoolEvent::FallbackActivation"),
            Self::NoResponse => formatter.write_str("FakePoolEvent::NoResponse"),
            Self::ClassifySubmitResponse(_) => formatter
                .debug_struct("FakePoolEvent::ClassifySubmitResponse")
                .field("pool_response", &"redacted")
                .finish(),
            Self::ClassifyStaleSubmitResponse(_) => formatter
                .debug_struct("FakePoolEvent::ClassifyStaleSubmitResponse")
                .field("pool_response", &"redacted")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FakePoolTranscript {
    pub events: Vec<FakePoolEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FakePoolRuntimeReport {
    pub state: MiningRuntimeState,
    pub classifications: Vec<SubmitClassification>,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedClientKind {
    Subscribe,
    Authorize,
    SubmitShare,
    Other,
}

impl FakePoolTranscript {
    pub fn run(
        &self,
        client_messages: &[StratumV1ClientMessage],
    ) -> Result<MiningRuntimeState, StratumV1Error> {
        let mut state = MiningRuntimeState::default();
        let mut client_index = 0;
        let mut observed_clients = Vec::new();

        for event in &self.events {
            match event {
                FakePoolEvent::ExpectClient(expected) => {
                    let Some(actual) = client_messages.get(client_index) else {
                        return unexpected_client_message();
                    };
                    if actual != expected {
                        return unexpected_client_message();
                    }
                    observed_clients.push((client_id(actual), client_kind(actual)));
                    client_index += 1;
                }
                FakePoolEvent::SendServer(message) => {
                    apply_server_message(&mut state, message, &observed_clients);
                }
                FakePoolEvent::Disconnect => {
                    state.set_lifecycle(PoolLifecycleStatus::Reconnecting);
                }
                FakePoolEvent::Timeout => {
                    state.set_fallback_active(true);
                }
                FakePoolEvent::MalformedResponse
                | FakePoolEvent::BlockedPrerequisite { .. }
                | FakePoolEvent::FallbackActivation
                | FakePoolEvent::NoResponse
                | FakePoolEvent::ClassifySubmitResponse(_)
                | FakePoolEvent::ClassifyStaleSubmitResponse(_) => {}
            }
        }

        if client_index != client_messages.len() {
            return unexpected_client_message();
        }

        Ok(state)
    }

    pub fn run_live_runtime(
        &self,
        runtime: &mut LiveStratumRuntime,
    ) -> Result<FakePoolRuntimeReport, StratumV1Error> {
        let _event = runtime.start();
        let mut pending_actions = runtime.drain_actions();
        let mut classifications = Vec::new();
        let mut maybe_submit_intent: Option<SubmitIntent> = None;
        let submit_request_id = StratumRequestId::new(7);

        for event in &self.events {
            match event {
                FakePoolEvent::ExpectClient(expected) => {
                    expect_live_client_message(expected, &mut pending_actions, runtime)?;
                }
                FakePoolEvent::SendServer(message) => {
                    runtime.apply_server_message(message.clone())?;
                    pending_actions.extend(runtime.drain_actions());
                    if matches!(message, StratumV1ServerMessage::ClientReconnect) {
                        classifications.push(SubmitClassification::Reconnect);
                    }
                }
                FakePoolEvent::Disconnect => {
                    runtime.apply_server_message(StratumV1ServerMessage::ClientReconnect)?;
                    classifications.push(SubmitClassification::Reconnect);
                }
                FakePoolEvent::Timeout | FakePoolEvent::NoResponse => {
                    classifications.push(SubmitClassification::Timeout);
                }
                FakePoolEvent::MalformedResponse => {
                    classifications.push(SubmitClassification::Malformed);
                }
                FakePoolEvent::BlockedPrerequisite { reason } => {
                    runtime.block_work_submission(reason);
                    classifications.push(SubmitClassification::Blocked { reason });
                }
                FakePoolEvent::FallbackActivation => {
                    runtime.activate_fallback();
                }
                FakePoolEvent::ClassifySubmitResponse(response) => {
                    if maybe_submit_intent.is_none() {
                        maybe_submit_intent = Some(correlate_runtime_submit_intent(runtime)?);
                    }
                    let Some(intent) = &maybe_submit_intent else {
                        return unexpected_client_message();
                    };
                    classifications.push(classify_submit_response(
                        intent,
                        submit_request_id,
                        SubmitResponseObservation::Response(response.clone()),
                    ));
                }
                FakePoolEvent::ClassifyStaleSubmitResponse(response) => {
                    let Some(intent) = &maybe_submit_intent else {
                        return unexpected_client_message();
                    };
                    classifications.push(classify_submit_response(
                        intent,
                        submit_request_id,
                        SubmitResponseObservation::StaleGeneration {
                            observed_generation: runtime.production_registry().generation(),
                            response: response.clone(),
                        },
                    ));
                }
            }
        }

        Ok(FakePoolRuntimeReport {
            state: runtime.state().clone(),
            classifications,
            generation: runtime.production_registry().generation().raw(),
        })
    }
}

fn expect_live_client_message(
    expected: &StratumV1ClientMessage,
    pending_actions: &mut Vec<LiveRuntimeAction>,
    runtime: &mut LiveStratumRuntime,
) -> Result<(), StratumV1Error> {
    if pending_actions.is_empty() {
        pending_actions.extend(runtime.drain_actions());
    }
    let Some(action) = pending_actions.first() else {
        return unexpected_client_message();
    };
    let LiveRuntimeAction::SendClientMessage(actual) = action else {
        return unexpected_client_message();
    };
    if actual != expected {
        return unexpected_client_message();
    }
    pending_actions.remove(0);
    Ok(())
}

fn correlate_runtime_submit_intent(
    runtime: &mut LiveStratumRuntime,
) -> Result<SubmitIntent, StratumV1Error> {
    let dispatch = runtime.production_registry_mut().dispatch_next()?;
    let outcome =
        runtime
            .production_registry_mut()
            .correlate_nonce_result(ProductionNonceObservation {
                observed_generation: dispatch.generation,
                result: fake_nonce_result(dispatch.work.asic_job_id),
            });
    let CorrelationOutcome::SubmitIntent(intent) = outcome else {
        return unexpected_client_message();
    };

    Ok(intent)
}

fn fake_nonce_result(job_id: Bm1366JobId) -> Bm1366NonceResult {
    Bm1366NonceResult {
        job_id,
        nonce: 0x1234_5678,
        asic_index: 0,
        core_id: 1,
        small_core_id: 0,
        version_bits: 0x0000_2000,
    }
}

fn apply_server_message(
    state: &mut MiningRuntimeState,
    message: &StratumV1ServerMessage,
    observed_clients: &[(StratumRequestId, ExpectedClientKind)],
) {
    match message {
        StratumV1ServerMessage::SetDifficulty(difficulty) => {
            state.set_pool_difficulty(*difficulty);
        }
        StratumV1ServerMessage::Notify(_) => {
            state.set_lifecycle(PoolLifecycleStatus::Active);
            state.allow_work_submission();
        }
        StratumV1ServerMessage::Response(response) => {
            apply_response(state, response, observed_clients);
        }
        StratumV1ServerMessage::ClientReconnect => {
            state.set_lifecycle(PoolLifecycleStatus::Reconnecting);
        }
        StratumV1ServerMessage::SetExtranonce(_)
        | StratumV1ServerMessage::SetVersionMask(_)
        | StratumV1ServerMessage::ClientShowMessage(_)
        | StratumV1ServerMessage::ClientGetVersion
        | StratumV1ServerMessage::Ping { .. } => {}
    }
}

fn apply_response(
    state: &mut MiningRuntimeState,
    response: &StratumResponse,
    observed_clients: &[(StratumRequestId, ExpectedClientKind)],
) {
    let maybe_kind = response
        .maybe_id
        .and_then(|id| maybe_expected_client_kind(id, observed_clients));

    if !response.success {
        apply_failed_response(state, response, maybe_kind);
        return;
    }

    if response.maybe_extranonce.is_some() {
        state.set_lifecycle(PoolLifecycleStatus::Subscribed);
        return;
    }

    match maybe_kind {
        Some(ExpectedClientKind::Authorize) => state.set_lifecycle(PoolLifecycleStatus::Authorized),
        Some(ExpectedClientKind::SubmitShare) => {
            let difficulty = state
                .maybe_pool_difficulty
                .map(|pool| ShareDifficulty::new(pool.difficulty))
                .unwrap_or_else(|| ShareDifficulty::new(0.0));
            state.record_accepted_share(difficulty);
            state.set_lifecycle(PoolLifecycleStatus::Active);
        }
        Some(ExpectedClientKind::Subscribe | ExpectedClientKind::Other) | None => {}
    }
}

fn apply_failed_response(
    state: &mut MiningRuntimeState,
    response: &StratumResponse,
    maybe_kind: Option<ExpectedClientKind>,
) {
    match maybe_kind {
        Some(ExpectedClientKind::SubmitShare) => {
            let reason = response
                .maybe_error
                .as_ref()
                .map(|error| error.message.as_str())
                .unwrap_or("pool rejected share");
            state.record_rejected_share(reason);
        }
        Some(ExpectedClientKind::Authorize) => {
            state.set_lifecycle(PoolLifecycleStatus::Error);
            state.block_work_submission("authorize_failed");
        }
        Some(ExpectedClientKind::Subscribe) => {
            state.set_lifecycle(PoolLifecycleStatus::Error);
            state.block_work_submission("subscribe_failed");
        }
        Some(ExpectedClientKind::Other) | None => {
            state.block_work_submission("non_submit_response_failed");
        }
    }
}

fn maybe_expected_client_kind(
    id: StratumRequestId,
    observed_clients: &[(StratumRequestId, ExpectedClientKind)],
) -> Option<ExpectedClientKind> {
    observed_clients
        .iter()
        .rev()
        .find(|(observed_id, _)| *observed_id == id)
        .map(|(_, kind)| *kind)
}

fn client_id(client: &StratumV1ClientMessage) -> StratumRequestId {
    match client {
        StratumV1ClientMessage::Subscribe { id, .. }
        | StratumV1ClientMessage::Authorize { id, .. }
        | StratumV1ClientMessage::ConfigureVersionRolling { id, .. }
        | StratumV1ClientMessage::SuggestDifficulty { id, .. }
        | StratumV1ClientMessage::ExtranonceSubscribe { id }
        | StratumV1ClientMessage::Pong { id }
        | StratumV1ClientMessage::SendVersion { id, .. }
        | StratumV1ClientMessage::SubmitShare { id, .. } => *id,
    }
}

fn client_kind(client: &StratumV1ClientMessage) -> ExpectedClientKind {
    match client {
        StratumV1ClientMessage::Subscribe { .. } => ExpectedClientKind::Subscribe,
        StratumV1ClientMessage::Authorize { .. } => ExpectedClientKind::Authorize,
        StratumV1ClientMessage::SubmitShare { .. } => ExpectedClientKind::SubmitShare,
        StratumV1ClientMessage::ConfigureVersionRolling { .. }
        | StratumV1ClientMessage::SuggestDifficulty { .. }
        | StratumV1ClientMessage::ExtranonceSubscribe { .. }
        | StratumV1ClientMessage::Pong { .. }
        | StratumV1ClientMessage::SendVersion { .. } => ExpectedClientKind::Other,
    }
}

fn unexpected_client_message<T>() -> Result<T, StratumV1Error> {
    Err(StratumV1Error::InvalidParams {
        method: "fake_pool.unexpected_client",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc::StratumRequestId;
    use crate::v1::live_runtime::{LivePoolCredentials, LiveRuntimeConfig};
    use crate::v1::messages::{
        ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse, StratumResponseError,
        StratumV1ClientMessage, StratumV1ServerMessage,
    };
    use crate::v1::state::{PoolLifecycleStatus, WorkSubmissionGate};
    use crate::v1::submit_response::{RedactedSubmitRejectReason, SubmitClassification};

    #[test]
    fn fake_pool_accepts_subscribe_authorize_notify_and_submit() {
        // Arrange
        let transcript = accepted_share_transcript();
        let client_messages = accepted_share_clients();

        // Act
        let state = match transcript.run(&client_messages) {
            Ok(state) => state,
            Err(error) => panic!("fake pool transcript failed: {error}"),
        };

        // Assert
        assert_eq!(state.lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(state.counters.accepted, 1);
        assert_eq!(state.counters.rejected, 0);
    }

    #[test]
    fn fake_pool_records_rejected_submit_reason() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![
                FakePoolEvent::ExpectClient(submit_share(7)),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Response(
                    rejected_submit_response(7, "low difficulty"),
                )),
            ],
        };

        // Act
        let state = match transcript.run(&[submit_share(7)]) {
            Ok(state) => state,
            Err(error) => panic!("fake pool transcript failed: {error}"),
        };

        // Assert
        assert_eq!(state.counters.rejected, 1);
        assert_eq!(
            state.counters.rejected_reasons,
            vec!["low difficulty".to_owned()]
        );
    }

    #[test]
    fn fake_pool_authorize_failure_does_not_record_rejected_share() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![
                FakePoolEvent::ExpectClient(authorize()),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Response(
                    rejected_submit_response(2, "authorize denied"),
                )),
            ],
        };

        // Act
        let state = match transcript.run(&[authorize()]) {
            Ok(state) => state,
            Err(error) => panic!("fake pool transcript failed: {error}"),
        };

        // Assert
        assert_eq!(state.counters.rejected, 0);
        assert!(state.counters.rejected_reasons.is_empty());
        assert_eq!(state.lifecycle, PoolLifecycleStatus::Error);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
        assert_eq!(state.maybe_blocked_reason, Some("authorize_failed"));
    }

    #[test]
    fn pool_lifecycle_disconnect_reconnects() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![FakePoolEvent::Disconnect],
        };

        // Act
        let state = match transcript.run(&[]) {
            Ok(state) => state,
            Err(error) => panic!("fake pool disconnect failed: {error}"),
        };

        // Assert
        assert_eq!(state.lifecycle, PoolLifecycleStatus::Reconnecting);
    }

    #[test]
    fn pool_lifecycle_timeout_activates_fallback_pool() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![FakePoolEvent::Timeout],
        };

        // Act
        let state = match transcript.run(&[]) {
            Ok(state) => state,
            Err(error) => panic!("fake pool timeout failed: {error}"),
        };

        // Assert
        assert!(state.fallback_active);
        assert_eq!(state.lifecycle, PoolLifecycleStatus::FallbackActive);
    }

    #[test]
    fn fake_pool_rejects_unexpected_client_message() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![FakePoolEvent::ExpectClient(subscribe())],
        };

        // Act
        let result = transcript.run(&[authorize()]);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn fake_pool_phase25_live_runtime_sequences_subscribe_authorize_difficulty_and_notify() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: phase25_ready_events(false),
        };
        let mut runtime = live_runtime();

        // Act
        let report = transcript
            .run_live_runtime(&mut runtime)
            .expect("phase25 live runtime transcript should run");

        // Assert
        assert_eq!(report.state.lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(report.state.work_submission, WorkSubmissionGate::Ready);
        assert_eq!(
            report.state.maybe_pool_difficulty,
            Some(PoolDifficulty { difficulty: 42.0 })
        );
        assert!(report.classifications.is_empty());
    }

    #[test]
    fn fake_pool_phase25_classifies_accepted_and_rejected_only_in_deterministic_scope() {
        // Arrange
        let mut events = phase25_ready_events(false);
        events.push(FakePoolEvent::ClassifySubmitResponse(success_response(7)));
        events.push(FakePoolEvent::ClassifySubmitResponse(
            rejected_submit_response(7, "low difficulty"),
        ));
        let transcript = FakePoolTranscript { events };
        let mut runtime = live_runtime();

        // Act
        let report = transcript
            .run_live_runtime(&mut runtime)
            .expect("phase25 classification transcript should run");

        // Assert
        assert_eq!(
            report.classifications,
            vec![
                SubmitClassification::Accepted,
                SubmitClassification::Rejected {
                    reason: RedactedSubmitRejectReason::PoolRejectedShare
                }
            ]
        );
    }

    #[test]
    fn fake_pool_phase25_clean_jobs_and_reconnect_block_stale_submit_classification() {
        // Arrange
        let mut events = phase25_ready_events(false);
        events.push(FakePoolEvent::ClassifySubmitResponse(success_response(7)));
        events.push(FakePoolEvent::SendServer(StratumV1ServerMessage::Notify(
            notify_with_clean_jobs(true),
        )));
        events.push(FakePoolEvent::ClassifyStaleSubmitResponse(
            success_response(7),
        ));
        events.push(FakePoolEvent::SendServer(
            StratumV1ServerMessage::ClientReconnect,
        ));
        let transcript = FakePoolTranscript { events };
        let mut runtime = live_runtime();

        // Act
        let report = transcript
            .run_live_runtime(&mut runtime)
            .expect("phase25 clean-jobs transcript should run");

        // Assert
        assert_eq!(report.generation, 2);
        assert!(report
            .classifications
            .contains(&SubmitClassification::Blocked {
                reason: "stale_generation"
            }));
        assert!(report
            .classifications
            .contains(&SubmitClassification::Reconnect));
    }

    #[test]
    fn fake_pool_phase25_fail_closed_paths_are_redaction_safe_non_accepted_outcomes() {
        // Arrange
        let transcript = FakePoolTranscript {
            events: vec![
                FakePoolEvent::BlockedPrerequisite {
                    reason: "precondition_blocked",
                },
                FakePoolEvent::FallbackActivation,
                FakePoolEvent::Timeout,
                FakePoolEvent::MalformedResponse,
                FakePoolEvent::Disconnect,
                FakePoolEvent::NoResponse,
            ],
        };
        let mut runtime = live_runtime();

        // Act
        let report = transcript
            .run_live_runtime(&mut runtime)
            .expect("phase25 fail-closed transcript should run");
        let rendered = format!("{:?}", transcript.events);

        // Assert
        assert!(report.state.fallback_active);
        assert!(report
            .classifications
            .contains(&SubmitClassification::Blocked {
                reason: "precondition_blocked"
            }));
        assert!(report
            .classifications
            .contains(&SubmitClassification::Timeout));
        assert!(report
            .classifications
            .contains(&SubmitClassification::Malformed));
        assert!(report
            .classifications
            .contains(&SubmitClassification::Reconnect));
        assert!(!report
            .classifications
            .contains(&SubmitClassification::Accepted));
        assert!(!rendered.contains("low difficulty"));
        assert!(!rendered.contains("00000000"));
    }

    fn accepted_share_transcript() -> FakePoolTranscript {
        FakePoolTranscript {
            events: vec![
                FakePoolEvent::ExpectClient(subscribe()),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Response(subscribe_response(1))),
                FakePoolEvent::ExpectClient(authorize()),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Response(success_response(2))),
                FakePoolEvent::SendServer(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
                    difficulty: 42.0,
                })),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Notify(notify())),
                FakePoolEvent::ExpectClient(submit_share(3)),
                FakePoolEvent::SendServer(StratumV1ServerMessage::Response(success_response(3))),
            ],
        }
    }

    fn accepted_share_clients() -> Vec<StratumV1ClientMessage> {
        vec![subscribe(), authorize(), submit_share(3)]
    }

    fn phase25_ready_events(clean_jobs: bool) -> Vec<FakePoolEvent> {
        vec![
            FakePoolEvent::ExpectClient(subscribe()),
            FakePoolEvent::SendServer(StratumV1ServerMessage::Response(subscribe_response(1))),
            FakePoolEvent::ExpectClient(authorize()),
            FakePoolEvent::SendServer(StratumV1ServerMessage::Response(success_response(2))),
            FakePoolEvent::SendServer(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
                difficulty: 42.0,
            })),
            FakePoolEvent::SendServer(StratumV1ServerMessage::Notify(notify_with_clean_jobs(
                clean_jobs,
            ))),
        ]
    }

    fn live_runtime() -> LiveStratumRuntime {
        LiveStratumRuntime::new(LiveRuntimeConfig {
            model: "ultra".to_owned(),
            version: "205".to_owned(),
            credentials: LivePoolCredentials {
                username: "synthetic-user".to_owned(),
                password: "x".to_owned(),
            },
        })
    }

    fn subscribe() -> StratumV1ClientMessage {
        StratumV1ClientMessage::subscribe(StratumRequestId::new(1), "ultra", "205")
    }

    fn authorize() -> StratumV1ClientMessage {
        StratumV1ClientMessage::authorize(StratumRequestId::new(2), "synthetic-user", "x")
    }

    fn submit_share(id: u64) -> StratumV1ClientMessage {
        StratumV1ClientMessage::submit_share(
            StratumRequestId::new(id),
            "synthetic-user",
            "job",
            "00000000",
            0x6470_25b5,
            0x1234_5678,
            0,
        )
    }

    fn subscribe_response(id: u64) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: true,
            maybe_error: None,
            maybe_extranonce: Some(ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 8,
            }),
            maybe_version_mask: None,
        }
    }

    fn success_response(id: u64) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: true,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: None,
        }
    }

    fn rejected_submit_response(id: u64, reason: &str) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: false,
            maybe_error: Some(StratumResponseError {
                maybe_code: Some(21),
                message: reason.to_owned(),
            }),
            maybe_extranonce: None,
            maybe_version_mask: None,
        }
    }

    fn notify() -> MiningNotify {
        notify_with_clean_jobs(true)
    }

    fn notify_with_clean_jobs(clean_jobs: bool) -> MiningNotify {
        MiningNotify {
            job_id: "job".to_owned(),
            prev_block_hash: "00".repeat(32),
            coinbase_1: "ffffffff".to_owned(),
            coinbase_2: "ffffffff".to_owned(),
            merkle_branches: Vec::new(),
            version: 0x2000_0004,
            nbits: 0x1705_ae3a,
            ntime: 0x6470_25b5,
            clean_jobs,
        }
    }
}
