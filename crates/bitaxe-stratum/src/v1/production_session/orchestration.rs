use bitaxe_asic::bm1366::{command::VersionMask, production::Bm1366ProductionCommand};

use crate::v1::bridge_orchestration::BridgeStep;
use crate::v1::live_runtime::{LiveRuntimeAction, LiveRuntimeEvent, LiveStratumRuntime};
use crate::v1::messages::{parse_server_message, StratumResponse, StratumV1ServerMessage};
use crate::v1::recovery_policy::{ProductionPool, RecoveryAction};
use crate::v1::submit_response::{classify_submit_response, SubmitResponseObservation};
use crate::StratumV1Error;

use super::runtime::{
    maybe_runtime_request_kind, PendingRequestKind, PendingSubmit, PoolRuntime,
    ProductionMiningSession,
};
use super::types::ProductionSessionEffect;

impl ProductionMiningSession {
    pub(super) fn start_pool_runtime(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_config = self
            .maybe_pool_set
            .as_ref()
            .and_then(|set| set.maybe_configuration(pool))
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

    pub(super) fn apply_transport_bytes(
        &mut self,
        pool: ProductionPool,
        bytes: &[u8],
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let lines = {
            let Some(pool_runtime) = self.maybe_pool_runtime_mut(pool) else {
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
            .maybe_pool_runtime(pool)
            .map(|runtime| runtime.runtime.production_registry().generation());
        let maybe_event = {
            let Some(pool_runtime) = self.maybe_pool_runtime_mut(pool) else {
                return Ok(());
            };
            pool_runtime.runtime.maybe_apply_server_message(message)?
        };
        let generation_after = self
            .maybe_pool_runtime(pool)
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
            .maybe_pool_runtime_mut(pool)
            .and_then(|runtime| runtime.requests.remove(&request_id));
        let Some(kind) = maybe_kind else {
            return Ok(());
        };

        match kind {
            PendingRequestKind::Submit => {
                let maybe_pending = self
                    .maybe_pool_runtime_mut(pool)
                    .and_then(|runtime| runtime.submits.remove(&request_id));
                let Some(pending) = maybe_pending else {
                    return Ok(());
                };
                let current_generation = self
                    .maybe_pool_runtime(pool)
                    .map(|runtime| runtime.runtime.production_registry().generation());
                if current_generation != Some(pending.intent.generation) {
                    return Ok(());
                }
                let classification = classify_submit_response(
                    &pending.intent,
                    request_id,
                    SubmitResponseObservation::Response(response),
                );
                if let Some(runtime) = self.maybe_pool_runtime_mut(pool) {
                    runtime.runtime.record_submit_classification(classification);
                }
            }
            PendingRequestKind::Runtime(kind) => {
                let maybe_event = {
                    let Some(pool_runtime) = self.maybe_pool_runtime_mut(pool) else {
                        return Ok(());
                    };
                    pool_runtime
                        .runtime
                        .maybe_apply_matched_response(kind, response)?
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

    pub(super) fn handle_transport_failure(
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
            if let Some(mut runtime) = self.maybe_take_pool_runtime(pool) {
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
        let Some(mut runtime) = self.maybe_take_pool_runtime(pool) else {
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

    pub(super) fn apply_recovery_actions(
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
                        if self.maybe_pool_runtime(pool).is_none() {
                            continue;
                        }
                        let replacement_generation = self.allocate_generation();
                        if let Some(runtime) = self.maybe_pool_runtime_mut(pool) {
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

    pub(super) fn drain_runtime_actions(
        &mut self,
        pool: ProductionPool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let actions = self
            .maybe_pool_runtime_mut(pool)
            .map(|runtime| runtime.runtime.drain_actions())
            .unwrap_or_default();
        for action in actions {
            match action {
                LiveRuntimeAction::SendClientMessage(message) => {
                    if let Some((request_id, kind)) = maybe_runtime_request_kind(&message) {
                        if let Some(runtime) = self.maybe_pool_runtime_mut(pool) {
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
                    if let Some(runtime) = self.maybe_pool_runtime_mut(pool) {
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
}

impl ProductionMiningSession {
    pub(super) fn drive_bridge(
        &mut self,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(pool) = self.recovery.projection().maybe_active_pool else {
            return Ok(());
        };
        if let Some(mask) = self
            .maybe_pool_runtime_mut(pool)
            .and_then(|runtime| runtime.runtime.maybe_take_pending_version_mask_reload())
        {
            effects.push(ProductionSessionEffect::ApplyVersionMask(VersionMask::new(
                mask.mask,
            )));
        }

        match self.bridge.next_step(now_ms) {
            BridgeStep::Dispatch => self.dispatch_next(pool, now_ms, effects)?,
            BridgeStep::Regenerate => {
                let regenerated = self
                    .maybe_pool_runtime_mut(pool)
                    .map(|runtime| runtime.runtime.regenerate_work())
                    .transpose();
                if matches!(regenerated, Ok(Some(_))) {
                    self.bridge.note_work_queued();
                    self.dispatch_next(pool, now_ms, effects)?;
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
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_dispatch = self
            .maybe_pool_runtime_mut(pool)
            .map(|runtime| runtime.runtime.production_registry_mut().dispatch_next())
            .transpose();
        match maybe_dispatch {
            Ok(Some(dispatch)) => {
                effects.push(ProductionSessionEffect::DispatchAsic(
                    Bm1366ProductionCommand::SendProductionWork(dispatch.work_payload),
                ));
                self.bridge.note_dispatched(now_ms);
            }
            Ok(None) | Err(StratumV1Error::QueueEmpty) => {}
            Err(error) => return Err(error),
        }
        Ok(())
    }
}
