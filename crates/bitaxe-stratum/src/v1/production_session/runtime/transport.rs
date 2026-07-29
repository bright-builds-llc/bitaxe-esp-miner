use std::collections::HashMap;

use crate::jsonrpc::StratumRequestId;
use crate::v1::line_framer::StratumLineFramer;
use crate::v1::live_runtime::{LiveStratumRuntime, RuntimeRequestKind};
use crate::v1::messages::StratumV1ClientMessage;
use crate::v1::production_work::{PoolSessionGeneration, SubmitIntent};
use crate::v1::recovery_policy::ProductionPool;

use super::ProductionMiningSession;
use crate::v1::production_session::types::ProductionTransportEpoch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::v1::production_session) enum PendingRequestKind {
    Runtime(RuntimeRequestKind),
    Submit,
}

#[derive(Clone)]
pub(in crate::v1::production_session) struct PendingSubmit {
    pub(in crate::v1::production_session) intent: SubmitIntent,
}

pub(in crate::v1::production_session) struct PoolRuntime {
    pub(in crate::v1::production_session) transport_epoch: ProductionTransportEpoch,
    pub(in crate::v1::production_session) runtime: LiveStratumRuntime,
    pub(in crate::v1::production_session) framer: StratumLineFramer,
    pub(in crate::v1::production_session) requests: HashMap<StratumRequestId, PendingRequestKind>,
    pub(in crate::v1::production_session) submits: HashMap<StratumRequestId, PendingSubmit>,
}

impl PoolRuntime {
    pub(in crate::v1::production_session) fn new(
        transport_epoch: ProductionTransportEpoch,
        runtime: LiveStratumRuntime,
    ) -> Self {
        Self {
            transport_epoch,
            runtime,
            framer: StratumLineFramer::default(),
            requests: HashMap::new(),
            submits: HashMap::new(),
        }
    }
}

impl ProductionMiningSession {
    pub(in crate::v1::production_session) fn allocate_transport_epoch(
        &mut self,
    ) -> ProductionTransportEpoch {
        self.transport_epoch_cursor = self.transport_epoch_cursor.next();
        self.transport_epoch_cursor
    }

    pub(in crate::v1::production_session) fn set_pending_transport_epoch(
        &mut self,
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
    ) {
        match pool {
            ProductionPool::Primary => {
                self.maybe_primary_transport_epoch = Some(transport_epoch);
            }
            ProductionPool::Fallback => {
                self.maybe_fallback_transport_epoch = Some(transport_epoch);
            }
        }
    }

    pub(in crate::v1::production_session) fn maybe_pending_transport_epoch(
        &self,
        pool: ProductionPool,
    ) -> Option<ProductionTransportEpoch> {
        match pool {
            ProductionPool::Primary => self.maybe_primary_transport_epoch,
            ProductionPool::Fallback => self.maybe_fallback_transport_epoch,
        }
    }

    pub(in crate::v1::production_session) fn take_pending_transport_epoch(
        &mut self,
        pool: ProductionPool,
    ) -> Option<ProductionTransportEpoch> {
        match pool {
            ProductionPool::Primary => self.maybe_primary_transport_epoch.take(),
            ProductionPool::Fallback => self.maybe_fallback_transport_epoch.take(),
        }
    }

    pub(in crate::v1::production_session) fn transport_epoch_is_active(
        &self,
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
    ) -> bool {
        self.maybe_pool_runtime(pool)
            .is_some_and(|runtime| runtime.transport_epoch == transport_epoch)
    }

    pub(in crate::v1::production_session) fn current_generation(
        &self,
        pool: ProductionPool,
    ) -> Option<PoolSessionGeneration> {
        self.maybe_pool_runtime(pool)
            .map(|runtime| runtime.runtime.production_registry().generation())
    }

    pub(in crate::v1::production_session) fn rebase_runtime_generation(
        &mut self,
        pool: ProductionPool,
    ) {
        let generation = self.allocate_generation();
        if let Some(runtime) = self.maybe_pool_runtime_mut(pool) {
            runtime.runtime.rebase_generation(generation);
        }
    }

    pub(in crate::v1::production_session) fn snapshot_generation(
        &self,
        maybe_active_pool: Option<ProductionPool>,
    ) -> PoolSessionGeneration {
        maybe_active_pool
            .and_then(|pool| self.maybe_pool_runtime(pool))
            .map_or(self.generation_cursor, |runtime| {
                runtime.runtime.production_registry().generation()
            })
    }

    pub(in crate::v1::production_session) fn clear_pending_submits(
        &mut self,
        pool: ProductionPool,
    ) {
        let Some(runtime) = self.maybe_pool_runtime_mut(pool) else {
            return;
        };
        runtime.submits.clear();
        runtime
            .requests
            .retain(|_, kind| *kind != PendingRequestKind::Submit);
    }

    pub(in crate::v1::production_session) fn maybe_runtime_for_projection(
        &self,
        maybe_pool: Option<ProductionPool>,
    ) -> Option<&PoolRuntime> {
        maybe_pool
            .and_then(|pool| self.maybe_pool_runtime(pool))
            .or(self.primary.as_ref())
            .or(self.fallback.as_ref())
    }

    pub(in crate::v1::production_session) fn maybe_pool_runtime(
        &self,
        pool: ProductionPool,
    ) -> Option<&PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_ref(),
            ProductionPool::Fallback => self.fallback.as_ref(),
        }
    }

    pub(in crate::v1::production_session) fn maybe_pool_runtime_mut(
        &mut self,
        pool: ProductionPool,
    ) -> Option<&mut PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.as_mut(),
            ProductionPool::Fallback => self.fallback.as_mut(),
        }
    }

    pub(in crate::v1::production_session) fn set_pool_runtime(
        &mut self,
        pool: ProductionPool,
        runtime: Option<PoolRuntime>,
    ) {
        match pool {
            ProductionPool::Primary => self.primary = runtime,
            ProductionPool::Fallback => self.fallback = runtime,
        }
    }

    pub(in crate::v1::production_session) fn maybe_take_pool_runtime(
        &mut self,
        pool: ProductionPool,
    ) -> Option<PoolRuntime> {
        match pool {
            ProductionPool::Primary => self.primary.take(),
            ProductionPool::Fallback => self.fallback.take(),
        }
    }
}

pub(in crate::v1::production_session) fn maybe_runtime_request_kind(
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
