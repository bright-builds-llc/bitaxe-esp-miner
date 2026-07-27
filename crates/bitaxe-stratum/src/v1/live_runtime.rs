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
    CorrelationOutcome, PoolSessionGeneration, ProductionNonceObservation, ProductionWorkRegistry,
    SubmitIntent,
};
use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus};
use crate::v1::submit_response::{RedactedSubmitRejectReason, SubmitClassification};

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
pub(crate) enum LiveRuntimeAction {
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
pub(crate) enum LiveRuntimeEvent {
    Started,
    Subscribed,
    Authorized,
    WorkQueued,
    WorkInvalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeRequestKind {
    Configure,
    Subscribe,
    Authorize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BridgeObservationOutcome {
    SubmitQueued,
    Blocked { reason: ProductionAsicBlocker },
}

#[derive(Clone, PartialEq)]
pub(crate) struct LiveStratumRuntime {
    config: LiveRuntimeConfig,
    state: MiningRuntimeState,
    production_registry: ProductionWorkRegistry,
    outbound_actions: Vec<LiveRuntimeAction>,
    maybe_extranonce: Option<ExtranonceAssignment>,
    maybe_version_mask: Option<VersionMask>,
    /// Once-shot pending ASIC `SetVersionMask` reload after configure /
    /// `mining.set_version_mask` (upstream `new_stratum_version_rolling_msg`).
    /// Cleared by [`Self::take_pending_version_mask_reload`]. Not value-delta
    /// gated — deterministic pool input may equal init default `0x1fffe000`.
    pending_version_mask_reload: bool,
    maybe_current_notify: Option<MiningNotify>,
    extranonce2_counter: u64,
    next_request_id: u64,
    next_asic_job_id: u8,
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
            .finish()
    }
}

impl LiveStratumRuntime {
    pub(crate) fn new_with_generation(
        config: LiveRuntimeConfig,
        generation: PoolSessionGeneration,
    ) -> Self {
        Self {
            config,
            state: MiningRuntimeState::default(),
            production_registry: ProductionWorkRegistry::new_with_generation(generation),
            outbound_actions: Vec::new(),
            maybe_extranonce: None,
            maybe_version_mask: None,
            pending_version_mask_reload: false,
            maybe_current_notify: None,
            extranonce2_counter: 0,
            next_request_id: 1,
            next_asic_job_id: 0,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &MiningRuntimeState {
        &self.state
    }

    /// Negotiated version-rolling mask stored from configure / set_version_mask.
    /// Counts-only evidence: presence is enough for `mask_applied_to_work` markers.
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

    pub(crate) fn rebase_generation(&mut self, generation: PoolSessionGeneration) {
        self.production_registry.rebase_generation(generation);
    }

    pub fn block_work_submission(&mut self, reason: &'static str) {
        self.state.block_work_submission(reason);
    }

    pub fn start(&mut self) -> LiveRuntimeEvent {
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

    pub(crate) fn apply_matched_response(
        &mut self,
        kind: RuntimeRequestKind,
        response: StratumResponse,
    ) -> Result<Option<LiveRuntimeEvent>, StratumV1Error> {
        match kind {
            RuntimeRequestKind::Configure => {
                if !response.success {
                    self.invalidate_for_authorization_reset();
                    self.state.set_lifecycle(PoolLifecycleStatus::Error);
                    return Ok(Some(LiveRuntimeEvent::WorkInvalidated));
                }
                if let Some(mask) = response.maybe_version_mask {
                    self.store_version_mask_and_raise_reload(mask);
                }
                Ok(None)
            }
            RuntimeRequestKind::Subscribe => {
                if !response.success {
                    self.invalidate_for_authorization_reset();
                    self.state.set_lifecycle(PoolLifecycleStatus::Error);
                    return Ok(Some(LiveRuntimeEvent::WorkInvalidated));
                }
                let Some(extranonce) = response.maybe_extranonce else {
                    return Err(StratumV1Error::MissingField("subscribe_extranonce"));
                };
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
                Ok(Some(LiveRuntimeEvent::Subscribed))
            }
            RuntimeRequestKind::Authorize => {
                if !response.success {
                    self.invalidate_for_authorization_reset();
                    self.state.set_lifecycle(PoolLifecycleStatus::Error);
                    return Ok(Some(LiveRuntimeEvent::WorkInvalidated));
                }
                self.state.set_lifecycle(PoolLifecycleStatus::Authorized);
                Ok(Some(LiveRuntimeEvent::Authorized))
            }
        }
    }

    pub(crate) fn record_submit_classification(&mut self, classification: SubmitClassification) {
        match classification {
            SubmitClassification::Accepted => {
                let difficulty = self
                    .state
                    .maybe_pool_difficulty
                    .map(|difficulty| crate::v1::state::ShareDifficulty::new(difficulty.difficulty))
                    .unwrap_or_else(|| crate::v1::state::ShareDifficulty::new(0.0));
                self.state.record_accepted_share(difficulty);
            }
            SubmitClassification::Rejected { reason } => {
                let reason = match reason {
                    RedactedSubmitRejectReason::PoolRejectedShare => "pool_rejected_share",
                    RedactedSubmitRejectReason::Unknown => "unknown_rejected_share",
                };
                self.state.record_rejected_share(reason);
            }
            SubmitClassification::Timeout
            | SubmitClassification::Reconnect
            | SubmitClassification::Malformed
            | SubmitClassification::NoObservedShare
            | SubmitClassification::Blocked { .. }
            | SubmitClassification::Stopped => {}
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
mod tests;
