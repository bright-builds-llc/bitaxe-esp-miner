//! Pure Stratum v1 live runtime state machine.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/stratum_socket.c`
//! - `reference/esp-miner/main/tasks/protocol_coordinator.c`
//! - Parity checklist rows `STR-008`, `STR-009`, `STR-011`, and `SAFE-012`

use std::fmt;

use bitaxe_asic::bm1366::work::Bm1366JobId;

use bitaxe_asic::bm1366::production::ProductionAsicBlocker;

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;
use crate::v1::messages::{
    ExtranonceAssignment, MiningNotify, StratumResponse, StratumV1ClientMessage,
    StratumV1ServerMessage, VersionMask,
};
use crate::v1::mining::MiningWorkBuilder;
use crate::v1::production_work::{
    CorrelationOutcome, ProductionNonceObservation, ProductionWorkRegistry, SubmitIntent,
};
use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus};

#[derive(Clone, PartialEq, Eq)]
pub struct LivePoolCredentials {
    pub username: String,
    pub password: String,
}

impl fmt::Debug for LivePoolCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LivePoolCredentials")
            .field("redaction", &"pool_credentials_redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct LiveRuntimeConfig {
    pub model: String,
    pub version: String,
    pub credentials: LivePoolCredentials,
}

impl fmt::Debug for LiveRuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveRuntimeConfig")
            .field("model", &self.model)
            .field("version", &self.version)
            .field("credentials", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum LiveRuntimeAction {
    SendClientMessage(StratumV1ClientMessage),
    SendSubmitShare {
        intent: SubmitIntent,
        request_id: StratumRequestId,
        message: StratumV1ClientMessage,
    },
}

impl fmt::Debug for LiveRuntimeAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SendClientMessage(_) => formatter
                .debug_struct("LiveRuntimeAction::SendClientMessage")
                .field("client_message", &"redacted")
                .finish(),
            Self::SendSubmitShare {
                request_id, intent, ..
            } => formatter
                .debug_struct("LiveRuntimeAction::SendSubmitShare")
                .field("request_id", request_id)
                .field("intent", intent)
                .field("client_message", &"redacted")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveRuntimeEvent {
    Started,
    Subscribed,
    Authorized,
    WorkQueued,
    WorkInvalidated,
    SafeStopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeObservationOutcome {
    SubmitQueued,
    Blocked { reason: ProductionAsicBlocker },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SafeStopReason {
    label: &'static str,
}

impl SafeStopReason {
    #[must_use]
    pub const fn new(label: &'static str) -> Self {
        Self { label }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.label
    }
}

impl fmt::Debug for SafeStopReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SafeStopReason")
            .field("label", &self.label)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeStopPostconditions {
    pub reason: SafeStopReason,
    pub socket_stopped: bool,
    pub active_work_invalidated: bool,
    pub mining_disabled: bool,
    pub hardware_control_disabled: bool,
    pub work_submission_blocked: bool,
    pub post_stop_snapshot_required: bool,
}

#[derive(Clone, PartialEq)]
pub struct LiveStratumRuntime {
    config: LiveRuntimeConfig,
    state: MiningRuntimeState,
    production_registry: ProductionWorkRegistry,
    outbound_actions: Vec<LiveRuntimeAction>,
    maybe_extranonce: Option<ExtranonceAssignment>,
    maybe_version_mask: Option<VersionMask>,
    /// Once-shot pending ASIC `SetVersionMask` reload after configure /
    /// `mining.set_version_mask` (upstream `new_stratum_version_rolling_msg`).
    /// Cleared by [`Self::take_pending_version_mask_reload`]. Not value-delta
    /// gated — fake-pool mask may equal init default `0x1fffe000`.
    pending_version_mask_reload: bool,
    maybe_current_notify: Option<MiningNotify>,
    extranonce2_counter: u64,
    next_request_id: u64,
    next_asic_job_id: u8,
    stopped: bool,
}

impl fmt::Debug for LiveStratumRuntime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveStratumRuntime")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("production_registry", &self.production_registry)
            .field("outbound_actions", &"redacted")
            .field("extranonce", &"redacted")
            .field("version_mask", &self.maybe_version_mask)
            .field("next_request_id", &self.next_request_id)
            .field("next_asic_job_id", &self.next_asic_job_id)
            .field("stopped", &self.stopped)
            .finish()
    }
}

impl LiveStratumRuntime {
    #[must_use]
    pub fn new(config: LiveRuntimeConfig) -> Self {
        Self {
            config,
            state: MiningRuntimeState::default(),
            production_registry: ProductionWorkRegistry::new(),
            outbound_actions: Vec::new(),
            maybe_extranonce: None,
            maybe_version_mask: None,
            pending_version_mask_reload: false,
            maybe_current_notify: None,
            extranonce2_counter: 0,
            next_request_id: 1,
            next_asic_job_id: 0,
            stopped: false,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &MiningRuntimeState {
        &self.state
    }

    /// Negotiated version-rolling mask stored from configure / set_version_mask.
    /// Counts-only evidence: presence is enough for `mask_applied_to_work` markers.
    #[must_use]
    pub const fn maybe_version_mask(&self) -> Option<VersionMask> {
        self.maybe_version_mask
    }

    /// Take the once-shot pending ASIC version-mask reload, if any.
    ///
    /// Returns the stored mask when configure / `set_version_mask` raised the
    /// pending bit. Clears the bit so firmware flushes at most once per store
    /// (upstream `new_stratum_version_rolling_msg` clear after TX).
    pub fn take_pending_version_mask_reload(&mut self) -> Option<VersionMask> {
        if !self.pending_version_mask_reload {
            return None;
        }
        self.pending_version_mask_reload = false;
        self.maybe_version_mask
    }

    fn store_version_mask_and_raise_reload(&mut self, mask: VersionMask) {
        self.maybe_version_mask = Some(mask);
        self.pending_version_mask_reload = true;
    }

    #[must_use]
    pub const fn production_registry(&self) -> &ProductionWorkRegistry {
        &self.production_registry
    }

    #[must_use]
    pub fn production_registry_mut(&mut self) -> &mut ProductionWorkRegistry {
        &mut self.production_registry
    }

    pub fn activate_fallback(&mut self) {
        self.state.set_fallback_active(true);
    }

    pub fn block_work_submission(&mut self, reason: &'static str) {
        self.state.block_work_submission(reason);
    }

    pub fn start(&mut self) -> LiveRuntimeEvent {
        if self.stopped {
            return LiveRuntimeEvent::SafeStopped;
        }

        self.state.set_lifecycle(PoolLifecycleStatus::Connecting);
        let configure_id = self.next_request_id();
        self.outbound_actions
            .push(LiveRuntimeAction::SendClientMessage(
                StratumV1ClientMessage::ConfigureVersionRolling {
                    id: configure_id,
                    mask: 0xffff_ffff,
                },
            ));
        let subscribe_id = self.next_request_id();
        self.outbound_actions
            .push(LiveRuntimeAction::SendClientMessage(
                StratumV1ClientMessage::subscribe(
                    subscribe_id,
                    &self.config.model,
                    &self.config.version,
                ),
            ));
        LiveRuntimeEvent::Started
    }

    pub fn apply_server_message(
        &mut self,
        message: StratumV1ServerMessage,
    ) -> Result<Option<LiveRuntimeEvent>, StratumV1Error> {
        if self.stopped {
            return Ok(None);
        }

        match message {
            StratumV1ServerMessage::Response(response) => self.apply_response(response),
            StratumV1ServerMessage::SetDifficulty(difficulty) => {
                self.state.set_pool_difficulty(difficulty);
                Ok(None)
            }
            StratumV1ServerMessage::SetExtranonce(extranonce) => {
                self.maybe_extranonce = Some(extranonce);
                Ok(None)
            }
            StratumV1ServerMessage::SetVersionMask(mask) => {
                self.store_version_mask_and_raise_reload(mask);
                Ok(None)
            }
            StratumV1ServerMessage::Notify(notify) => self.apply_notify(notify),
            StratumV1ServerMessage::ClientReconnect => {
                self.invalidate_for_reconnect();
                self.state.set_lifecycle(PoolLifecycleStatus::Reconnecting);
                Ok(Some(LiveRuntimeEvent::WorkInvalidated))
            }
            StratumV1ServerMessage::ClientShowMessage(_)
            | StratumV1ServerMessage::ClientGetVersion
            | StratumV1ServerMessage::Ping { .. } => Ok(None),
        }
    }

    pub fn invalidate_for_clean_jobs(&mut self) {
        self.production_registry.invalidate_for_clean_jobs();
        self.state.block_work_submission("clean_jobs");
        self.reset_regeneration_context();
    }

    pub fn invalidate_for_reconnect(&mut self) {
        self.production_registry.invalidate_for_reconnect();
        self.state.block_work_submission("pool_reconnect");
        self.reset_regeneration_context();
    }

    pub fn invalidate_for_authorization_reset(&mut self) {
        self.production_registry
            .invalidate_for_authorization_reset();
        self.state.block_work_submission("authorization_reset");
        self.reset_regeneration_context();
    }

    pub fn invalidate_for_session_replacement(&mut self) {
        self.production_registry
            .invalidate_for_session_replacement();
        self.state.block_work_submission("session_replacement");
        self.reset_regeneration_context();
    }

    /// Drop the held notify and restart the extranonce2 counter.
    ///
    /// Session invalidation must prevent stale-session work regeneration;
    /// the counter restarts at zero for the next fresh notify.
    fn reset_regeneration_context(&mut self) {
        self.maybe_current_notify = None;
        self.extranonce2_counter = 0;
    }

    /// Regenerate held pool work with a fresh extranonce2 and enqueue it.
    ///
    /// Returns the new counter value for redaction-safe telemetry markers.
    /// Errors as a no-op when no notify is held — regeneration never
    /// fabricates work.
    ///
    /// Reference: reference/esp-miner/main/tasks/create_jobs_task.c:183-186
    /// (cadence timeout regenerates the held work with extranonce_2++).
    pub fn regenerate_work(&mut self) -> Result<u64, StratumV1Error> {
        let Some(notify) = self.maybe_current_notify.clone() else {
            return Err(StratumV1Error::MissingField("current_notify"));
        };
        let Some(extranonce) = self.maybe_extranonce.clone() else {
            return Err(StratumV1Error::MissingField("extranonce"));
        };

        self.extranonce2_counter += 1;
        let mut builder = MiningWorkBuilder::new(notify, extranonce)
            .with_extranonce2_value(self.extranonce2_counter);
        if let Some(pool_difficulty) = self.state.maybe_pool_difficulty {
            builder = builder.with_pool_difficulty(pool_difficulty);
        }
        if let Some(version_mask) = self.maybe_version_mask {
            builder = builder.with_version_mask(version_mask);
        }

        let mut work = builder.build(self.next_asic_job_id())?;
        work.clean_jobs = false;
        self.production_registry.enqueue_pool_work(work)?;
        Ok(self.extranonce2_counter)
    }

    pub fn safe_stop(&mut self, reason: &'static str) -> SafeStopPostconditions {
        self.stopped = true;
        self.outbound_actions.clear();
        self.invalidate_for_session_replacement();
        self.state.set_lifecycle(PoolLifecycleStatus::Disconnected);
        self.state
            .set_mining_activity(MiningActivityStatus::SafeBlocked);
        SafeStopPostconditions {
            reason: SafeStopReason::new(reason),
            socket_stopped: true,
            active_work_invalidated: true,
            mining_disabled: true,
            hardware_control_disabled: true,
            work_submission_blocked: true,
            post_stop_snapshot_required: true,
        }
    }

    #[must_use]
    pub fn drain_actions(&mut self) -> Vec<LiveRuntimeAction> {
        std::mem::take(&mut self.outbound_actions)
    }

    /// Feed a firmware-stamped nonce observation through production correlation.
    pub fn apply_bridge_observation(
        &mut self,
        observation: ProductionNonceObservation,
    ) -> Result<BridgeObservationOutcome, StratumV1Error> {
        match self.production_registry.correlate_nonce_result(observation) {
            CorrelationOutcome::SubmitIntent(intent) => {
                let request_id = self.next_request_id();
                self.queue_submit_share(intent, request_id)?;
                Ok(BridgeObservationOutcome::SubmitQueued)
            }
            CorrelationOutcome::Blocked { reason } => {
                self.block_work_submission(reason.as_str());
                Ok(BridgeObservationOutcome::Blocked { reason })
            }
        }
    }

    fn queue_submit_share(
        &mut self,
        intent: SubmitIntent,
        request_id: StratumRequestId,
    ) -> Result<(), StratumV1Error> {
        let message = intent
            .submission()
            .to_client_message(request_id, &self.config.credentials.username);
        self.outbound_actions
            .push(LiveRuntimeAction::SendSubmitShare {
                intent,
                request_id,
                message,
            });
        Ok(())
    }

    fn apply_response(
        &mut self,
        response: StratumResponse,
    ) -> Result<Option<LiveRuntimeEvent>, StratumV1Error> {
        if let Some(extranonce) = response.maybe_extranonce {
            self.maybe_extranonce = Some(extranonce);
            self.state.set_lifecycle(PoolLifecycleStatus::Subscribed);
            let id = self.next_request_id();
            self.outbound_actions
                .push(LiveRuntimeAction::SendClientMessage(
                    StratumV1ClientMessage::authorize(
                        id,
                        &self.config.credentials.username,
                        &self.config.credentials.password,
                    ),
                ));
            return Ok(Some(LiveRuntimeEvent::Subscribed));
        }

        if let Some(mask) = response.maybe_version_mask {
            self.store_version_mask_and_raise_reload(mask);
            return Ok(None);
        }

        if response.maybe_id == Some(StratumRequestId::new(3)) && response.success {
            self.state.set_lifecycle(PoolLifecycleStatus::Authorized);
            return Ok(Some(LiveRuntimeEvent::Authorized));
        }

        if !response.success {
            self.invalidate_for_authorization_reset();
            self.state.set_lifecycle(PoolLifecycleStatus::Error);
            return Ok(Some(LiveRuntimeEvent::WorkInvalidated));
        }

        Ok(None)
    }

    fn apply_notify(
        &mut self,
        notify: crate::v1::messages::MiningNotify,
    ) -> Result<Option<LiveRuntimeEvent>, StratumV1Error> {
        let clean_jobs = notify.clean_jobs;
        if clean_jobs {
            self.invalidate_for_clean_jobs();
        }

        let Some(extranonce) = self.maybe_extranonce.clone() else {
            self.state.block_work_submission("extranonce_missing");
            return Ok(None);
        };

        let stored_notify = notify.clone();
        let mut builder = MiningWorkBuilder::new(notify, extranonce);
        if let Some(pool_difficulty) = self.state.maybe_pool_difficulty {
            builder = builder.with_pool_difficulty(pool_difficulty);
        }
        if let Some(version_mask) = self.maybe_version_mask {
            builder = builder.with_version_mask(version_mask);
        }

        let mut work = builder.build(self.next_asic_job_id())?;
        if clean_jobs {
            work.clean_jobs = false;
        }
        self.production_registry.enqueue_pool_work(work)?;
        // Hold the notify for cadence regeneration; the counter restarts
        // whenever fresh pool work arrives.
        // Reference: reference/esp-miner/main/tasks/create_jobs_task.c:132
        self.maybe_current_notify = Some(stored_notify);
        self.extranonce2_counter = 0;
        self.state.allow_work_submission();
        self.state.set_lifecycle(PoolLifecycleStatus::Active);
        self.state.set_mining_activity(MiningActivityStatus::Active);
        Ok(Some(LiveRuntimeEvent::WorkQueued))
    }

    fn next_request_id(&mut self) -> StratumRequestId {
        let id = StratumRequestId::new(self.next_request_id);
        self.next_request_id += 1;
        id
    }

    fn next_asic_job_id(&mut self) -> Bm1366JobId {
        // Upstream `BM1366_send_work` pre-increments before assign (`bm1366.c:313-314`).
        self.next_asic_job_id = self.next_asic_job_id.wrapping_add(8) % 128;
        Bm1366JobId::new(self.next_asic_job_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::messages::{
        ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse,
        StratumV1ClientMessage, StratumV1ServerMessage,
    };
    use crate::v1::state::{MiningActivityStatus, PoolLifecycleStatus, WorkSubmissionGate};

    #[test]
    fn live_runtime_start_queues_subscribe_and_redacts_sensitive_debug() {
        // Arrange
        let mut runtime = LiveStratumRuntime::new(config());

        // Act
        let event = runtime.start();
        let actions = runtime.drain_actions();
        let rendered = format!("{runtime:?}");

        // Assert
        assert_eq!(event, LiveRuntimeEvent::Started);
        assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Connecting);
        assert!(matches!(
            actions.as_slice(),
            [
                LiveRuntimeAction::SendClientMessage(StratumV1ClientMessage::ConfigureVersionRolling {
                    id: configure_id,
                    mask,
                }),
                LiveRuntimeAction::SendClientMessage(StratumV1ClientMessage::Subscribe {
                    id: subscribe_id,
                    user_agent,
                }),
            ] if configure_id.raw() == 1
                && *mask == 0xffff_ffff
                && subscribe_id.raw() == 2
                && user_agent == "bitaxe/ultra/205"
        ));
        assert!(!rendered.contains("synthetic-user"));
        assert!(!rendered.contains("synthetic-secret"));
        assert!(!rendered.contains("4de05269"));
        assert!(!rendered.contains("00000000"));
        assert!(!rendered.contains("12345678"));
    }

    #[test]
    fn live_runtime_subscribe_authorize_and_difficulty_progress_lifecycle() {
        // Arrange
        let mut runtime = started_runtime();

        // Act
        runtime
            .apply_server_message(StratumV1ServerMessage::Response(subscribe_response(2)))
            .expect("subscribe response should be accepted");
        let authorize_actions = runtime.drain_actions();
        runtime
            .apply_server_message(StratumV1ServerMessage::Response(success_response(3)))
            .expect("authorize response should be accepted");
        runtime
            .apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
                difficulty: 42.0,
            }))
            .expect("difficulty should be accepted");

        // Assert
        assert!(matches!(
            authorize_actions.as_slice(),
            [LiveRuntimeAction::SendClientMessage(StratumV1ClientMessage::Authorize {
                id,
                username,
                password,
            })] if id.raw() == 3 && username == "synthetic-user" && password == "synthetic-secret"
        ));
        assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Authorized);
        assert_eq!(
            runtime.state().maybe_pool_difficulty,
            Some(PoolDifficulty { difficulty: 42.0 })
        );
    }

    #[test]
    fn live_runtime_first_pool_job_id_matches_upstream_pre_increment_stride() {
        // Arrange
        let mut runtime = authorized_runtime();

        // Act
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should build production work");

        // Assert — upstream `BM1366_send_work` assigns id after `(id + 8) % 128` (`bm1366.c:313-314`).
        assert!(runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(8)));
        assert!(!runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(0)));
    }

    #[test]
    fn configure_mask_store_raises_pending_version_mask_reload_even_when_mask_equals_init_default()
    {
        // Arrange — fake-pool / init default mask (D-05: no value-delta gate).
        let mut runtime = started_runtime();
        let init_default_mask = VersionMask { mask: 0x1fff_e000 };

        // Act
        runtime
            .apply_server_message(StratumV1ServerMessage::Response(StratumResponse {
                maybe_id: Some(StratumRequestId::new(1)),
                success: true,
                maybe_error: None,
                maybe_extranonce: None,
                maybe_version_mask: Some(init_default_mask),
            }))
            .expect("configure response should store mask");
        let taken = runtime.take_pending_version_mask_reload();
        let taken_again = runtime.take_pending_version_mask_reload();

        // Assert
        assert_eq!(runtime.maybe_version_mask(), Some(init_default_mask));
        assert_eq!(taken, Some(init_default_mask));
        assert_eq!(taken_again, None);
    }

    #[test]
    fn set_version_mask_raises_pending_version_mask_reload() {
        // Arrange
        let mut runtime = authorized_runtime();
        let mask = VersionMask { mask: 0x1fff_e000 };

        // Act
        runtime
            .apply_server_message(StratumV1ServerMessage::SetVersionMask(mask))
            .expect("set_version_mask should store mask");

        // Assert
        assert_eq!(runtime.take_pending_version_mask_reload(), Some(mask));
        assert_eq!(runtime.take_pending_version_mask_reload(), None);
    }

    #[test]
    fn live_runtime_notify_queues_production_work_and_allows_submission() {
        // Arrange
        let mut runtime = authorized_runtime();

        // Act
        let maybe_event = runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should build production work");

        // Assert
        assert_eq!(maybe_event, Some(LiveRuntimeEvent::WorkQueued));
        assert_eq!(runtime.state().lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Ready);
        assert_eq!(
            runtime.state().mining_activity,
            MiningActivityStatus::Active
        );
        assert!(runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(8)));
    }

    #[test]
    fn live_runtime_invalidation_paths_advance_generation_and_block_stale_submission() {
        // Arrange
        let mut runtime = authorized_runtime();
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should build production work");

        // Act
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(true)))
            .expect("clean jobs notify should replace production work");
        let after_clean_jobs = runtime.production_registry().generation().raw();
        let clean_jobs_contains_current = runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(16));
        let clean_jobs_contains_stale = runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(8));
        runtime
            .apply_server_message(StratumV1ServerMessage::ClientReconnect)
            .expect("reconnect should invalidate production work");
        let after_reconnect = runtime.production_registry().generation().raw();
        runtime.invalidate_for_authorization_reset();
        let after_authorization_reset = runtime.production_registry().generation().raw();
        runtime.invalidate_for_session_replacement();
        let after_session_replacement = runtime.production_registry().generation().raw();

        // Assert
        assert_eq!(after_clean_jobs, 1);
        assert_eq!(after_reconnect, 2);
        assert_eq!(after_authorization_reset, 3);
        assert_eq!(after_session_replacement, 4);
        assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
        assert!(clean_jobs_contains_current);
        assert!(!clean_jobs_contains_stale);
        assert!(!runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(8)));
    }

    #[test]
    fn live_runtime_safe_stop_sets_postconditions_and_freezes_later_messages() {
        // Arrange
        let mut runtime = authorized_runtime();
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should build production work");

        // Act
        let postconditions = runtime.safe_stop("operator_cancelled");
        let lifecycle_after_stop = runtime.state().lifecycle;
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("post-stop message should be ignored");

        // Assert
        assert_eq!(postconditions.reason.as_str(), "operator_cancelled");
        assert!(postconditions.socket_stopped);
        assert!(postconditions.active_work_invalidated);
        assert!(postconditions.mining_disabled);
        assert!(postconditions.hardware_control_disabled);
        assert!(postconditions.work_submission_blocked);
        assert!(postconditions.post_stop_snapshot_required);
        assert_eq!(runtime.state().lifecycle, lifecycle_after_stop);
        assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
        assert!(!runtime
            .production_registry()
            .valid_jobs()
            .contains(Bm1366JobId::new(8)));
    }

    fn started_runtime() -> LiveStratumRuntime {
        let mut runtime = LiveStratumRuntime::new(config());
        runtime.start();
        let _actions = runtime.drain_actions();
        runtime
    }

    fn authorized_runtime() -> LiveStratumRuntime {
        let mut runtime = started_runtime();
        runtime
            .apply_server_message(StratumV1ServerMessage::Response(subscribe_response(2)))
            .expect("subscribe response should be accepted");
        let _actions = runtime.drain_actions();
        runtime
            .apply_server_message(StratumV1ServerMessage::Response(success_response(3)))
            .expect("authorize response should be accepted");
        runtime
            .apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
                difficulty: 42.0,
            }))
            .expect("difficulty should be accepted");
        runtime
    }

    fn config() -> LiveRuntimeConfig {
        LiveRuntimeConfig {
            model: "ultra".to_owned(),
            version: "205".to_owned(),
            credentials: LivePoolCredentials {
                username: "synthetic-user".to_owned(),
                password: "synthetic-secret".to_owned(),
            },
        }
    }

    fn subscribe_response(id: u64) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: true,
            maybe_error: None,
            maybe_extranonce: Some(ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
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

    fn notify(clean_jobs: bool) -> MiningNotify {
        MiningNotify {
            job_id: "job".to_owned(),
            prev_block_hash: "00".repeat(32),
            coinbase_1: "0200000001".to_owned(),
            coinbase_2: "ffffffff".to_owned(),
            merkle_branches: Vec::new(),
            version: 0x2000_0004,
            nbits: 0x1705_ae3a,
            ntime: 0x6470_25b5,
            clean_jobs,
        }
    }

    #[test]
    fn apply_bridge_observation_queues_submit_share_for_correlated_nonce() {
        // Arrange
        use crate::v1::production_work::ProductionNonceObservation;
        use bitaxe_asic::bm1366::result::Bm1366NonceResult;

        let mut runtime = authorized_runtime();
        let notify = notify(false);
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify.clone()))
            .expect("notify should enqueue work");
        let dispatch = runtime
            .production_registry_mut()
            .dispatch_next()
            .expect("dispatch should succeed");
        let observation = ProductionNonceObservation {
            observed_generation: dispatch.generation,
            result: Bm1366NonceResult {
                job_id: dispatch.work.asic_job_id,
                nonce: 0x1234_5678,
                asic_index: 0,
                core_id: 0,
                small_core_id: 0,
                version_bits: 0,
            },
        };

        // Act
        let outcome = runtime
            .apply_bridge_observation(observation)
            .expect("observation should apply");
        let actions = runtime.drain_actions();

        // Assert
        assert_eq!(outcome, BridgeObservationOutcome::SubmitQueued);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            LiveRuntimeAction::SendSubmitShare { .. }
        ));
        let rendered = format!("{runtime:?}");
        assert!(!rendered.contains("synthetic-secret"));
    }

    #[test]
    fn apply_bridge_observation_blocked_does_not_queue_submit() {
        // Arrange
        use crate::v1::production_work::ProductionNonceObservation;
        use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};

        let mut runtime = authorized_runtime();
        let observation = ProductionNonceObservation {
            observed_generation: runtime.production_registry().generation().next(),
            result: Bm1366NonceResult {
                job_id: Bm1366JobId::new(0x20),
                nonce: 1,
                asic_index: 0,
                core_id: 0,
                small_core_id: 0,
                version_bits: 0,
            },
        };

        // Act
        let outcome = runtime
            .apply_bridge_observation(observation)
            .expect("blocked observation should return outcome");
        let actions = runtime.drain_actions();

        // Assert
        assert!(matches!(outcome, BridgeObservationOutcome::Blocked { .. }));
        assert!(actions.is_empty());
    }

    #[test]
    fn regenerate_work_uses_fresh_extranonce2_counter_sequence() {
        // Arrange
        let mut runtime = authorized_runtime();
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should enqueue work");
        let first = runtime
            .production_registry_mut()
            .dispatch_next()
            .expect("first dispatch should succeed");

        // Act
        let first_counter = runtime
            .regenerate_work()
            .expect("first regeneration should enqueue");
        let second = runtime
            .production_registry_mut()
            .dispatch_next()
            .expect("second dispatch should succeed");
        let second_counter = runtime
            .regenerate_work()
            .expect("second regeneration should enqueue");
        let third = runtime
            .production_registry_mut()
            .dispatch_next()
            .expect("third dispatch should succeed");

        // Assert: counter sequence 0,1,2 rendered with the fixture-tested
        // little-endian extranonce2 encoding (upstream mining.c parity).
        assert_eq!(first.work.extranonce2, "00000000");
        assert_eq!(first_counter, 1);
        assert_eq!(second.work.extranonce2, "01000000");
        assert_eq!(second_counter, 2);
        assert_eq!(third.work.extranonce2, "02000000");
    }

    #[test]
    fn regenerate_work_without_held_notify_is_a_no_op() {
        // Arrange
        let mut runtime = authorized_runtime();

        // Act
        let result = runtime.regenerate_work();

        // Assert: regeneration never fabricates work.
        assert!(result.is_err());
        assert!(matches!(
            runtime.production_registry_mut().dispatch_next(),
            Err(StratumV1Error::QueueEmpty)
        ));
    }

    fn assert_invalidation_clears_regeneration_context(invalidate: fn(&mut LiveStratumRuntime)) {
        // Arrange: held notify with an advanced extranonce2 counter.
        let mut runtime = authorized_runtime();
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("notify should enqueue work");
        let advanced_counter = runtime
            .regenerate_work()
            .expect("regeneration should enqueue");
        assert_eq!(advanced_counter, 1);

        // Act
        invalidate(&mut runtime);
        let regenerate_after_invalidation = runtime.regenerate_work();
        runtime
            .apply_server_message(StratumV1ServerMessage::Notify(notify(false)))
            .expect("fresh notify should enqueue work");
        let fresh = runtime
            .production_registry_mut()
            .dispatch_next()
            .expect("fresh dispatch should succeed");

        // Assert: no-op until a fresh notify; fresh work restarts at extranonce2=0.
        assert!(regenerate_after_invalidation.is_err());
        assert_eq!(fresh.work.extranonce2, "00000000");
    }

    #[test]
    fn clean_jobs_invalidation_clears_regeneration_context() {
        assert_invalidation_clears_regeneration_context(
            LiveStratumRuntime::invalidate_for_clean_jobs,
        );
    }

    #[test]
    fn reconnect_invalidation_clears_regeneration_context() {
        assert_invalidation_clears_regeneration_context(
            LiveStratumRuntime::invalidate_for_reconnect,
        );
    }

    #[test]
    fn authorization_reset_invalidation_clears_regeneration_context() {
        assert_invalidation_clears_regeneration_context(
            LiveStratumRuntime::invalidate_for_authorization_reset,
        );
    }

    #[test]
    fn session_replacement_invalidation_clears_regeneration_context() {
        assert_invalidation_clears_regeneration_context(
            LiveStratumRuntime::invalidate_for_session_replacement,
        );
    }
}
