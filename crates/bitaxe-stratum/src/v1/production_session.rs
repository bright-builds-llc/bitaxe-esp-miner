//! Deep, software-complete Production Mining Session.
//!
//! Callers feed typed events through one interface. The implementation owns
//! recovery, Stratum V1 progression, framing, work correlation, submit
//! classification, bridge cadence, and ordered safe stop.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use bitaxe_asic::bm1366::{command::VersionMask, production::Bm1366ProductionCommand};

use crate::jsonrpc::StratumRequestId;
use crate::v1::bridge_orchestration::{BridgeOrchestrator, BridgeStep};
use crate::v1::line_framer::StratumLineFramer;
pub use crate::v1::live_runtime::{LivePoolCredentials, LiveRuntimeConfig};
use crate::v1::live_runtime::{
    LiveRuntimeAction, LiveRuntimeEvent, LiveStratumRuntime, RuntimeRequestKind,
};
use crate::v1::messages::{
    parse_server_message, StratumResponse, StratumV1ClientMessage, StratumV1ServerMessage,
};
use crate::v1::production_work::{PoolSessionGeneration, ProductionNonceObservation, SubmitIntent};
use crate::v1::recovery_policy::{RecoveryAction, RecoveryPolicy};
use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus};
use crate::v1::submit_response::{classify_submit_response, SubmitResponseObservation};
use crate::StratumV1Error;

pub use crate::v1::recovery_policy::{
    ProductionPool, ProductionPoolAvailability, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionNotificationOutcome, ProductionSessionPhase, ProductionSessionWakeup,
    CONNECTION_ATTEMPTS_PER_POOL, CONNECTION_RETRY_DELAY_MS, PRIMARY_INITIAL_PROBE_DELAY_MS,
    PRIMARY_RECURRING_PROBE_DELAY_MS, RECOVERY_PROBE_DELAY_MS,
};

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolEndpoint {
    pub host: String,
    pub port: u16,
}

impl fmt::Debug for ProductionPoolEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolEndpoint")
            .field("endpoint", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolConfiguration {
    pub endpoint: ProductionPoolEndpoint,
    pub runtime: LiveRuntimeConfig,
}

impl fmt::Debug for ProductionPoolConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolConfiguration")
            .field("endpoint", &"redacted")
            .field("runtime", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolSet {
    pub primary: Option<ProductionPoolConfiguration>,
    pub fallback: Option<ProductionPoolConfiguration>,
    pub prefer_fallback: bool,
}

impl fmt::Debug for ProductionPoolSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolSet")
            .field("primary_configured", &self.primary.is_some())
            .field("fallback_configured", &self.fallback.is_some())
            .field("prefer_fallback", &self.prefer_fallback)
            .finish()
    }
}

impl ProductionPoolSet {
    fn availability(&self) -> ProductionPoolAvailability {
        ProductionPoolAvailability {
            primary_configured: self.primary.is_some(),
            fallback_configured: self.fallback.is_some(),
            prefer_fallback: self.prefer_fallback,
        }
    }

    fn configuration(&self, pool: ProductionPool) -> Option<&ProductionPoolConfiguration> {
        match pool {
            ProductionPool::Primary => self.primary.as_ref(),
            ProductionPool::Fallback => self.fallback.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionSessionSnapshot {
    pub phase: ProductionSessionPhase,
    pub maybe_blocker: Option<ProductionSessionBlocker>,
    pub maybe_active_pool: Option<ProductionPool>,
    pub generation: PoolSessionGeneration,
    pub mining: MiningRuntimeState,
}

#[derive(Clone, PartialEq)]
pub enum ProductionSessionEvent {
    Wake {
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        now_ms: u64,
    },
    PoolConfigurationLoaded(Option<Box<ProductionPoolSet>>),
    TransportConnected {
        pool: ProductionPool,
        now_ms: u64,
    },
    TransportConnectFailed {
        pool: ProductionPool,
        now_ms: u64,
    },
    TransportBytes {
        pool: ProductionPool,
        bytes: Vec<u8>,
        now_ms: u64,
    },
    TransportClosed {
        pool: ProductionPool,
        now_ms: u64,
    },
    AsicResult {
        observation: ProductionNonceObservation,
        now_ms: u64,
    },
    AsicPollTimedOut {
        now_ms: u64,
    },
    EffectFailed {
        maybe_pool: Option<ProductionPool>,
        reason: &'static str,
        now_ms: u64,
    },
}

impl fmt::Debug for ProductionSessionEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportBytes {
                pool,
                bytes,
                now_ms,
            } => formatter
                .debug_struct("ProductionSessionEvent::TransportBytes")
                .field("pool", pool)
                .field("byte_count", &bytes.len())
                .field("now_ms", now_ms)
                .finish(),
            Self::PoolConfigurationLoaded(maybe_pools) => formatter
                .debug_tuple("ProductionSessionEvent::PoolConfigurationLoaded")
                .field(
                    &maybe_pools
                        .as_ref()
                        .map(|_| "configured_redacted")
                        .unwrap_or("unavailable"),
                )
                .finish(),
            other => formatter.write_str(match other {
                Self::Wake { .. } => "ProductionSessionEvent::Wake",
                Self::TransportConnected { .. } => "ProductionSessionEvent::TransportConnected",
                Self::TransportConnectFailed { .. } => {
                    "ProductionSessionEvent::TransportConnectFailed"
                }
                Self::TransportClosed { .. } => "ProductionSessionEvent::TransportClosed",
                Self::AsicResult { .. } => "ProductionSessionEvent::AsicResult(redacted)",
                Self::AsicPollTimedOut { .. } => "ProductionSessionEvent::AsicPollTimedOut",
                Self::EffectFailed { .. } => "ProductionSessionEvent::EffectFailed",
                Self::PoolConfigurationLoaded(_) | Self::TransportBytes { .. } => unreachable!(),
            }),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ProductionSessionEffect {
    ReadPoolConfiguration,
    ConnectPool(ProductionPool),
    WritePoolLine { pool: ProductionPool, line: String },
    ApplyVersionMask(VersionMask),
    DispatchAsic(Bm1366ProductionCommand),
    PollAsic { slice_ms: u32 },
    BlockSubmissions,
    InvalidateWorkAndSubmissions,
    StopAsicInteraction,
    ClosePoolConnection(ProductionPool),
    Publish(ProductionSessionSnapshot),
}

impl fmt::Debug for ProductionSessionEffect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WritePoolLine { pool, .. } => formatter
                .debug_struct("ProductionSessionEffect::WritePoolLine")
                .field("pool", pool)
                .field("line", &"redacted")
                .finish(),
            Self::DispatchAsic(_) => {
                formatter.write_str("ProductionSessionEffect::DispatchAsic(redacted)")
            }
            other => match other {
                Self::ReadPoolConfiguration => {
                    formatter.write_str("ProductionSessionEffect::ReadPoolConfiguration")
                }
                Self::ConnectPool(pool) => formatter
                    .debug_tuple("ProductionSessionEffect::ConnectPool")
                    .field(pool)
                    .finish(),
                Self::ApplyVersionMask(_) => {
                    formatter.write_str("ProductionSessionEffect::ApplyVersionMask(redacted)")
                }
                Self::PollAsic { slice_ms } => formatter
                    .debug_struct("ProductionSessionEffect::PollAsic")
                    .field("slice_ms", slice_ms)
                    .finish(),
                Self::BlockSubmissions => {
                    formatter.write_str("ProductionSessionEffect::BlockSubmissions")
                }
                Self::InvalidateWorkAndSubmissions => {
                    formatter.write_str("ProductionSessionEffect::InvalidateWorkAndSubmissions")
                }
                Self::StopAsicInteraction => {
                    formatter.write_str("ProductionSessionEffect::StopAsicInteraction")
                }
                Self::ClosePoolConnection(pool) => formatter
                    .debug_tuple("ProductionSessionEffect::ClosePoolConnection")
                    .field(pool)
                    .finish(),
                Self::Publish(snapshot) => formatter
                    .debug_tuple("ProductionSessionEffect::Publish")
                    .field(snapshot)
                    .finish(),
                Self::WritePoolLine { .. } | Self::DispatchAsic(_) => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingRequestKind {
    Runtime(RuntimeRequestKind),
    Submit,
}

#[derive(Clone)]
struct PendingSubmit {
    intent: SubmitIntent,
}

struct PoolRuntime {
    runtime: LiveStratumRuntime,
    framer: StratumLineFramer,
    requests: HashMap<StratumRequestId, PendingRequestKind>,
    submits: HashMap<StratumRequestId, PendingSubmit>,
}

impl PoolRuntime {
    fn new(runtime: LiveStratumRuntime) -> Self {
        Self {
            runtime,
            framer: StratumLineFramer::default(),
            requests: HashMap::new(),
            submits: HashMap::new(),
        }
    }
}

pub struct ProductionMiningSession {
    recovery: RecoveryPolicy,
    maybe_pool_set: Option<ProductionPoolSet>,
    primary: Option<PoolRuntime>,
    fallback: Option<PoolRuntime>,
    bridge: BridgeOrchestrator,
    bridge_epoch: Instant,
    generation_cursor: PoolSessionGeneration,
    last_readiness: ProductionReadiness,
    maybe_last_snapshot: Option<ProductionSessionSnapshot>,
}

impl Default for ProductionMiningSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductionMiningSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            recovery: RecoveryPolicy::new(),
            maybe_pool_set: None,
            primary: None,
            fallback: None,
            bridge: BridgeOrchestrator::new(2_000),
            bridge_epoch: Instant::now(),
            generation_cursor: PoolSessionGeneration::initial(),
            last_readiness: ProductionReadiness {
                operator_intent: crate::v1::state::MiningOperatorIntent::Run,
                network_ready: false,
                stratum_v1_supported: false,
                safety_prerequisites_fresh: false,
                production_asic_ready: false,
                actuation_qualified: false,
            },
            maybe_last_snapshot: None,
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> ProductionSessionSnapshot {
        let projection = self.recovery.projection();
        let mut mining = self
            .runtime_for_projection(projection.maybe_active_pool)
            .map(|session| session.runtime.state().clone())
            .unwrap_or_default();
        mining.set_operator_intent(self.last_readiness.operator_intent);
        mining.set_fallback_active(matches!(
            projection.maybe_active_pool,
            Some(ProductionPool::Fallback)
        ));

        if let Some(blocker) = projection.maybe_blocker {
            mining.block_work_submission(blocker.label());
            mining.set_mining_activity(if blocker == ProductionSessionBlocker::OperatorPaused {
                MiningActivityStatus::Paused
            } else {
                MiningActivityStatus::SafeBlocked
            });
        }
        match projection.phase {
            ProductionSessionPhase::RecoveryPaused => {
                mining.set_lifecycle(PoolLifecycleStatus::RecoveryPaused);
                mining.set_mining_activity(MiningActivityStatus::SafeBlocked);
            }
            ProductionSessionPhase::WaitingForReadiness
            | ProductionSessionPhase::SafeStopping
            | ProductionSessionPhase::Shutdown
                if projection.maybe_active_pool.is_none() =>
            {
                mining.set_lifecycle(PoolLifecycleStatus::Disconnected);
            }
            ProductionSessionPhase::ConnectingPrimary
            | ProductionSessionPhase::ConnectingFallback
                if projection.maybe_active_pool.is_none() =>
            {
                if self
                    .runtime_for_projection(projection.maybe_active_pool)
                    .is_none()
                {
                    mining.set_lifecycle(PoolLifecycleStatus::Connecting);
                }
                mining.block_work_submission("production_session_negotiating");
                mining.set_mining_activity(MiningActivityStatus::SafeBlocked);
            }
            _ => {}
        }

        ProductionSessionSnapshot {
            phase: projection.phase,
            maybe_blocker: projection.maybe_blocker,
            maybe_active_pool: projection.maybe_active_pool,
            generation: self.snapshot_generation(projection.maybe_active_pool),
            mining,
        }
    }

    pub fn handle(
        &mut self,
        event: ProductionSessionEvent,
    ) -> Result<Vec<ProductionSessionEffect>, StratumV1Error> {
        let mut effects = Vec::new();
        match event {
            ProductionSessionEvent::Wake {
                wakeup,
                readiness,
                now_ms,
            } => {
                self.last_readiness = readiness;
                let recovery_actions = self.recovery.on_wakeup(wakeup, readiness, now_ms);
                self.apply_recovery_actions(recovery_actions, &mut effects)?;
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::PoolConfigurationLoaded(maybe_pool_set) => {
                let maybe_pool_set = maybe_pool_set.map(|pool_set| *pool_set);
                let availability = maybe_pool_set.as_ref().map_or(
                    ProductionPoolAvailability {
                        primary_configured: false,
                        fallback_configured: false,
                        prefer_fallback: false,
                    },
                    ProductionPoolSet::availability,
                );
                self.maybe_pool_set = maybe_pool_set;
                let recovery_actions = self.recovery.on_pool_configuration(availability);
                self.apply_recovery_actions(recovery_actions, &mut effects)?;
            }
            ProductionSessionEvent::TransportConnected { pool, now_ms } => {
                self.start_pool_runtime(pool, &mut effects)?;
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::TransportConnectFailed { pool, now_ms } => {
                let actions = self.recovery.on_connection_result(pool, false, now_ms);
                self.apply_recovery_actions(actions, &mut effects)?;
            }
            ProductionSessionEvent::TransportBytes {
                pool,
                bytes,
                now_ms,
            } => {
                self.apply_transport_bytes(pool, &bytes, now_ms, &mut effects)?;
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::TransportClosed { pool, now_ms } => {
                self.handle_transport_failure(pool, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::AsicResult {
                observation,
                now_ms,
            } => {
                if let Some(active_pool) = self.recovery.projection().maybe_active_pool {
                    if let Some(pool_runtime) = self.pool_runtime_mut(active_pool) {
                        let _outcome =
                            pool_runtime.runtime.apply_bridge_observation(observation)?;
                    }
                    self.drain_runtime_actions(active_pool, &mut effects)?;
                    self.bridge.note_result_received();
                    self.drive_bridge(now_ms, &mut effects)?;
                }
            }
            ProductionSessionEvent::AsicPollTimedOut { now_ms } => {
                let _streak = self.bridge.note_poll_timeout();
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::EffectFailed {
                maybe_pool,
                reason: _,
                now_ms,
            } => {
                if let Some(pool) = maybe_pool {
                    self.handle_transport_failure(pool, now_ms, &mut effects)?;
                } else {
                    let mut readiness = self.last_readiness;
                    readiness.production_asic_ready = false;
                    self.last_readiness = readiness;
                    let actions = self.recovery.on_wakeup(None, readiness, now_ms);
                    self.apply_recovery_actions(actions, &mut effects)?;
                }
            }
        }
        self.publish_if_changed(&mut effects);
        Ok(effects)
    }

    fn start_pool_runtime(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_config = self
            .maybe_pool_set
            .as_ref()
            .and_then(|set| set.configuration(pool))
            .cloned();
        let Some(config) = maybe_config else {
            return Ok(());
        };

        let generation = self.allocate_generation();
        let mut pool_runtime = PoolRuntime::new(LiveStratumRuntime::new_with_generation(
            config.runtime,
            generation,
        ));
        let _started = pool_runtime.runtime.start();
        self.set_pool_runtime(pool, Some(pool_runtime));
        self.drain_runtime_actions(pool, effects)
    }

    fn apply_transport_bytes(
        &mut self,
        pool: ProductionPool,
        bytes: &[u8],
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let lines = {
            let Some(pool_runtime) = self.pool_runtime_mut(pool) else {
                return Ok(());
            };
            match pool_runtime.framer.push(bytes) {
                Ok(lines) => lines,
                Err(_) => {
                    self.handle_transport_failure(pool, now_ms, effects)?;
                    return Ok(());
                }
            }
        };

        for line in lines {
            let message = match parse_server_message(&line) {
                Ok(message) => message,
                Err(_) => {
                    self.handle_transport_failure(pool, now_ms, effects)?;
                    return Ok(());
                }
            };
            self.apply_server_message(pool, message, now_ms, effects)?;
        }
        Ok(())
    }

    fn apply_server_message(
        &mut self,
        pool: ProductionPool,
        message: StratumV1ServerMessage,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        if let StratumV1ServerMessage::Response(response) = message {
            return self.apply_response(pool, response, now_ms, effects);
        }

        let generation_before = self
            .pool_runtime(pool)
            .map(|runtime| runtime.runtime.production_registry().generation());
        let maybe_event = {
            let Some(pool_runtime) = self.pool_runtime_mut(pool) else {
                return Ok(());
            };
            pool_runtime.runtime.apply_server_message(message)?
        };
        let generation_after = self
            .pool_runtime(pool)
            .map(|runtime| runtime.runtime.production_registry().generation());
        if generation_before != generation_after {
            self.rebase_runtime_generation(pool);
            self.clear_pending_submits(pool);
        }
        match maybe_event {
            Some(LiveRuntimeEvent::WorkQueued) => self.bridge.note_work_queued(),
            Some(LiveRuntimeEvent::WorkInvalidated) => {
                self.handle_transport_failure(pool, now_ms, effects)?;
                return Ok(());
            }
            Some(
                LiveRuntimeEvent::Started
                | LiveRuntimeEvent::Subscribed
                | LiveRuntimeEvent::Authorized,
            )
            | None => {}
        }
        self.drain_runtime_actions(pool, effects)
    }

    fn apply_response(
        &mut self,
        pool: ProductionPool,
        response: StratumResponse,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(request_id) = response.maybe_id else {
            return Ok(());
        };
        let maybe_kind = self
            .pool_runtime_mut(pool)
            .and_then(|runtime| runtime.requests.remove(&request_id));
        let Some(kind) = maybe_kind else {
            return Ok(());
        };

        match kind {
            PendingRequestKind::Submit => {
                let maybe_pending = self
                    .pool_runtime_mut(pool)
                    .and_then(|runtime| runtime.submits.remove(&request_id));
                let Some(pending) = maybe_pending else {
                    return Ok(());
                };
                let current_generation = self
                    .pool_runtime(pool)
                    .map(|runtime| runtime.runtime.production_registry().generation());
                if current_generation != Some(pending.intent.generation) {
                    return Ok(());
                }
                let classification = classify_submit_response(
                    &pending.intent,
                    request_id,
                    SubmitResponseObservation::Response(response),
                );
                if let Some(runtime) = self.pool_runtime_mut(pool) {
                    runtime.runtime.record_submit_classification(classification);
                }
            }
            PendingRequestKind::Runtime(kind) => {
                let maybe_event = {
                    let Some(pool_runtime) = self.pool_runtime_mut(pool) else {
                        return Ok(());
                    };
                    pool_runtime
                        .runtime
                        .apply_matched_response(kind, response)?
                };
                if maybe_event == Some(LiveRuntimeEvent::WorkInvalidated) {
                    self.handle_transport_failure(pool, now_ms, effects)?;
                    return Ok(());
                }
                if maybe_event == Some(LiveRuntimeEvent::Authorized) {
                    self.replace_fallback_after_primary_probe(pool, effects);
                    let actions = self.recovery.on_connection_result(pool, true, now_ms);
                    self.apply_recovery_actions(actions, effects)?;
                    self.bridge.note_listener_armed();
                }
            }
        }
        self.drain_runtime_actions(pool, effects)
    }

    fn replace_fallback_after_primary_probe(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) {
        if pool != ProductionPool::Primary
            || self.recovery.projection().maybe_active_pool != Some(ProductionPool::Fallback)
        {
            return;
        }
        effects.push(ProductionSessionEffect::BlockSubmissions);
        let replacement_generation = self.allocate_generation();
        if let Some(fallback) = self.fallback.as_mut() {
            fallback.runtime.invalidate_for_session_replacement();
            fallback.runtime.rebase_generation(replacement_generation);
            fallback.requests.clear();
            fallback.submits.clear();
            fallback.framer.clear();
        }
        effects.push(ProductionSessionEffect::InvalidateWorkAndSubmissions);
        effects.push(ProductionSessionEffect::StopAsicInteraction);
        if self.fallback.take().is_some() {
            effects.push(ProductionSessionEffect::ClosePoolConnection(
                ProductionPool::Fallback,
            ));
        }
        self.bridge.invalidate_session();
    }

    fn handle_transport_failure(
        &mut self,
        pool: ProductionPool,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_active_pool = self.recovery.projection().maybe_active_pool;
        if maybe_active_pool == Some(pool) {
            let actions = self.recovery.on_connection_lost(now_ms);
            return self.apply_recovery_actions(actions, effects);
        }
        if maybe_active_pool.is_some() {
            if let Some(mut runtime) = self.take_pool_runtime(pool) {
                runtime.runtime.invalidate_for_session_replacement();
                runtime.requests.clear();
                runtime.submits.clear();
                runtime.framer.clear();
                effects.push(ProductionSessionEffect::ClosePoolConnection(pool));
            }
        } else {
            self.invalidate_pool_runtime(pool, effects);
        }
        let actions = self.recovery.on_connection_result(pool, false, now_ms);
        self.apply_recovery_actions(actions, effects)
    }

    fn invalidate_pool_runtime(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) {
        let Some(mut runtime) = self.take_pool_runtime(pool) else {
            return;
        };
        let replacement_generation = self.allocate_generation();
        runtime.runtime.invalidate_for_session_replacement();
        runtime.runtime.rebase_generation(replacement_generation);
        runtime.requests.clear();
        runtime.submits.clear();
        runtime.framer.clear();
        self.bridge.invalidate_session();
        effects.push(ProductionSessionEffect::BlockSubmissions);
        effects.push(ProductionSessionEffect::InvalidateWorkAndSubmissions);
        effects.push(ProductionSessionEffect::StopAsicInteraction);
        effects.push(ProductionSessionEffect::ClosePoolConnection(pool));
    }

    fn apply_recovery_actions(
        &mut self,
        actions: Vec<RecoveryAction>,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        for action in actions {
            match action {
                RecoveryAction::ReadPoolConfiguration => {
                    effects.push(ProductionSessionEffect::ReadPoolConfiguration);
                }
                RecoveryAction::ConnectPool(pool) => {
                    effects.push(ProductionSessionEffect::ConnectPool(pool));
                }
                RecoveryAction::BlockSubmissions => {
                    for runtime in [&mut self.primary, &mut self.fallback]
                        .into_iter()
                        .flatten()
                    {
                        runtime
                            .runtime
                            .block_work_submission("production_session_safe_stop");
                    }
                    effects.push(ProductionSessionEffect::BlockSubmissions);
                }
                RecoveryAction::InvalidateWorkAndSubmissions => {
                    for pool in [ProductionPool::Primary, ProductionPool::Fallback] {
                        if self.pool_runtime(pool).is_none() {
                            continue;
                        }
                        let replacement_generation = self.allocate_generation();
                        if let Some(runtime) = self.pool_runtime_mut(pool) {
                            runtime.runtime.invalidate_for_session_replacement();
                            runtime.runtime.rebase_generation(replacement_generation);
                            runtime.requests.clear();
                            runtime.submits.clear();
                            runtime.framer.clear();
                        }
                    }
                    self.bridge.invalidate_session();
                    effects.push(ProductionSessionEffect::InvalidateWorkAndSubmissions);
                }
                RecoveryAction::StopAsicInteraction => {
                    effects.push(ProductionSessionEffect::StopAsicInteraction);
                }
                RecoveryAction::ClosePoolConnection => {
                    if self.primary.take().is_some() {
                        effects.push(ProductionSessionEffect::ClosePoolConnection(
                            ProductionPool::Primary,
                        ));
                    }
                    if self.fallback.take().is_some() {
                        effects.push(ProductionSessionEffect::ClosePoolConnection(
                            ProductionPool::Fallback,
                        ));
                    }
                }
                RecoveryAction::Publish(_) => {}
            }
        }
        Ok(())
    }

    fn drain_runtime_actions(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let actions = self
            .pool_runtime_mut(pool)
            .map(|runtime| runtime.runtime.drain_actions())
            .unwrap_or_default();
        for action in actions {
            match action {
                LiveRuntimeAction::SendClientMessage(message) => {
                    if let Some((request_id, kind)) = runtime_request_kind(&message) {
                        if let Some(runtime) = self.pool_runtime_mut(pool) {
                            runtime
                                .requests
                                .insert(request_id, PendingRequestKind::Runtime(kind));
                        }
                    }
                    effects.push(ProductionSessionEffect::WritePoolLine {
                        pool,
                        line: message.to_json_line()?,
                    });
                }
                LiveRuntimeAction::SendSubmitShare {
                    intent,
                    request_id,
                    message,
                } => {
                    if let Some(runtime) = self.pool_runtime_mut(pool) {
                        runtime
                            .requests
                            .insert(request_id, PendingRequestKind::Submit);
                        runtime.submits.insert(request_id, PendingSubmit { intent });
                    }
                    effects.push(ProductionSessionEffect::WritePoolLine {
                        pool,
                        line: message.to_json_line()?,
                    });
                }
            }
        }
        Ok(())
    }

    fn drive_bridge(
        &mut self,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(pool) = self.recovery.projection().maybe_active_pool else {
            return Ok(());
        };
        if let Some(mask) = self
            .pool_runtime_mut(pool)
            .and_then(|runtime| runtime.runtime.take_pending_version_mask_reload())
        {
            effects.push(ProductionSessionEffect::ApplyVersionMask(VersionMask::new(
                mask.mask,
            )));
        }

        let now = self.bridge_epoch + Duration::from_millis(now_ms);
        match self.bridge.next_step(now) {
            BridgeStep::Dispatch => self.dispatch_next(pool, now, effects)?,
            BridgeStep::Regenerate => {
                let regenerated = self
                    .pool_runtime_mut(pool)
                    .map(|runtime| runtime.runtime.regenerate_work())
                    .transpose();
                if matches!(regenerated, Ok(Some(_))) {
                    self.bridge.note_work_queued();
                    self.dispatch_next(pool, now, effects)?;
                } else if regenerated.is_err() {
                    self.bridge.invalidate_session();
                }
            }
            BridgeStep::Poll { slice_ms } => {
                effects.push(ProductionSessionEffect::PollAsic { slice_ms });
            }
            BridgeStep::Idle => {}
        }
        Ok(())
    }

    fn dispatch_next(
        &mut self,
        pool: ProductionPool,
        now: Instant,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_dispatch = self
            .pool_runtime_mut(pool)
            .map(|runtime| runtime.runtime.production_registry_mut().dispatch_next())
            .transpose();
        match maybe_dispatch {
            Ok(Some(dispatch)) => {
                effects.push(ProductionSessionEffect::DispatchAsic(
                    Bm1366ProductionCommand::SendProductionWork(dispatch.work_payload),
                ));
                self.bridge.note_dispatched(now);
            }
            Ok(None) | Err(StratumV1Error::QueueEmpty) => {}
            Err(error) => return Err(error),
        }
        Ok(())
    }

    fn publish_if_changed(&mut self, effects: &mut Vec<ProductionSessionEffect>) {
        let snapshot = self.snapshot();
        if self.maybe_last_snapshot.as_ref() != Some(&snapshot) {
            self.maybe_last_snapshot = Some(snapshot.clone());
            effects.push(ProductionSessionEffect::Publish(snapshot));
        }
    }

    fn allocate_generation(&mut self) -> PoolSessionGeneration {
        self.generation_cursor = self.generation_cursor.next();
        self.generation_cursor
    }

    fn rebase_runtime_generation(&mut self, pool: ProductionPool) {
        let generation = self.allocate_generation();
        if let Some(runtime) = self.pool_runtime_mut(pool) {
            runtime.runtime.rebase_generation(generation);
        }
    }

    fn snapshot_generation(
        &self,
        maybe_active_pool: Option<ProductionPool>,
    ) -> PoolSessionGeneration {
        maybe_active_pool
            .and_then(|pool| self.pool_runtime(pool))
            .map_or(self.generation_cursor, |runtime| {
                runtime.runtime.production_registry().generation()
            })
    }

    fn clear_pending_submits(&mut self, pool: ProductionPool) {
        let Some(runtime) = self.pool_runtime_mut(pool) else {
            return;
        };
        runtime.submits.clear();
        runtime
            .requests
            .retain(|_, kind| *kind != PendingRequestKind::Submit);
    }

    fn runtime_for_projection(&self, maybe_pool: Option<ProductionPool>) -> Option<&PoolRuntime> {
        maybe_pool
            .and_then(|pool| self.pool_runtime(pool))
            .or(self.primary.as_ref())
            .or(self.fallback.as_ref())
    }

    fn pool_runtime(&self, pool: ProductionPool) -> Option<&PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_ref(),
            ProductionPool::Fallback => self.fallback.as_ref(),
        }
    }

    fn pool_runtime_mut(&mut self, pool: ProductionPool) -> Option<&mut PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_mut(),
            ProductionPool::Fallback => self.fallback.as_mut(),
        }
    }

    fn set_pool_runtime(&mut self, pool: ProductionPool, runtime: Option<PoolRuntime>) {
        match pool {
            ProductionPool::Primary => self.primary = runtime,
            ProductionPool::Fallback => self.fallback = runtime,
        }
    }

    fn take_pool_runtime(&mut self, pool: ProductionPool) -> Option<PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.take(),
            ProductionPool::Fallback => self.fallback.take(),
        }
    }
}

fn runtime_request_kind(
    message: &StratumV1ClientMessage,
) -> Option<(StratumRequestId, RuntimeRequestKind)> {
    match message {
        StratumV1ClientMessage::ConfigureVersionRolling { id, .. } => {
            Some((*id, RuntimeRequestKind::Configure))
        }
        StratumV1ClientMessage::Subscribe { id, .. } => Some((*id, RuntimeRequestKind::Subscribe)),
        StratumV1ClientMessage::Authorize { id, .. } => Some((*id, RuntimeRequestKind::Authorize)),
        StratumV1ClientMessage::SuggestDifficulty { .. }
        | StratumV1ClientMessage::ExtranonceSubscribe { .. }
        | StratumV1ClientMessage::Pong { .. }
        | StratumV1ClientMessage::SendVersion { .. }
        | StratumV1ClientMessage::SubmitShare { .. } => None,
    }
}

#[cfg(test)]
mod tests;
