use std::collections::HashMap;
use std::time::Instant;

use crate::jsonrpc::StratumRequestId;
use crate::v1::bridge_orchestration::BridgeOrchestrator;
use crate::v1::line_framer::StratumLineFramer;
use crate::v1::live_runtime::{LiveStratumRuntime, RuntimeRequestKind};
use crate::v1::messages::StratumV1ClientMessage;
use crate::v1::production_work::{PoolSessionGeneration, SubmitIntent};
use crate::v1::recovery_policy::{
    ProductionPool, ProductionPoolAvailability, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionPhase, RecoveryPolicy,
};
use crate::v1::state::{MiningActivityStatus, PoolLifecycleStatus};
use crate::StratumV1Error;

use super::types::{
    ProductionPoolSet, ProductionSessionEffect, ProductionSessionEvent, ProductionSessionSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PendingRequestKind {
    Runtime(RuntimeRequestKind),
    Submit,
}

#[derive(Clone)]
pub(super) struct PendingSubmit {
    pub(super) intent: SubmitIntent,
}

pub(super) struct PoolRuntime {
    pub(super) runtime: LiveStratumRuntime,
    pub(super) framer: StratumLineFramer,
    pub(super) requests: HashMap<StratumRequestId, PendingRequestKind>,
    pub(super) submits: HashMap<StratumRequestId, PendingSubmit>,
}

impl PoolRuntime {
    pub(super) fn new(runtime: LiveStratumRuntime) -> Self {
        Self {
            runtime,
            framer: StratumLineFramer::default(),
            requests: HashMap::new(),
            submits: HashMap::new(),
        }
    }
}

pub struct ProductionMiningSession {
    pub(super) recovery: RecoveryPolicy,
    pub(super) maybe_pool_set: Option<ProductionPoolSet>,
    pub(super) primary: Option<PoolRuntime>,
    pub(super) fallback: Option<PoolRuntime>,
    pub(super) bridge: BridgeOrchestrator,
    pub(super) bridge_epoch: Instant,
    pub(super) generation_cursor: PoolSessionGeneration,
    pub(super) last_readiness: ProductionReadiness,
    pub(super) maybe_last_snapshot: Option<ProductionSessionSnapshot>,
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
}

impl ProductionMiningSession {
    fn publish_if_changed(&mut self, effects: &mut Vec<ProductionSessionEffect>) {
        let snapshot = self.snapshot();
        if self.maybe_last_snapshot.as_ref() != Some(&snapshot) {
            self.maybe_last_snapshot = Some(snapshot.clone());
            effects.push(ProductionSessionEffect::Publish(snapshot));
        }
    }

    pub(super) fn allocate_generation(&mut self) -> PoolSessionGeneration {
        self.generation_cursor = self.generation_cursor.next();
        self.generation_cursor
    }

    pub(super) fn rebase_runtime_generation(&mut self, pool: ProductionPool) {
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

    pub(super) fn clear_pending_submits(&mut self, pool: ProductionPool) {
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

    pub(super) fn pool_runtime(&self, pool: ProductionPool) -> Option<&PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_ref(),
            ProductionPool::Fallback => self.fallback.as_ref(),
        }
    }

    pub(super) fn pool_runtime_mut(&mut self, pool: ProductionPool) -> Option<&mut PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_mut(),
            ProductionPool::Fallback => self.fallback.as_mut(),
        }
    }

    pub(super) fn set_pool_runtime(&mut self, pool: ProductionPool, runtime: Option<PoolRuntime>) {
        match pool {
            ProductionPool::Primary => self.primary = runtime,
            ProductionPool::Fallback => self.fallback = runtime,
        }
    }

    pub(super) fn take_pool_runtime(&mut self, pool: ProductionPool) -> Option<PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.take(),
            ProductionPool::Fallback => self.fallback.take(),
        }
    }
}

pub(super) fn runtime_request_kind(
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
