use super::transport::{PendingRequestKind, PendingSubmit};
use crate::jsonrpc::StratumRequestId;
use crate::v1::{
    line_framer::StratumLineFramer, live_runtime::LiveStratumRuntime,
    production_work::PoolSessionGeneration, state::MiningRuntimeState,
};
use crate::v2::standard::StandardSession;
use std::collections::HashMap;

pub(in crate::v1::production_session) struct V1Runtime {
    pub runtime: LiveStratumRuntime,
    pub framer: StratumLineFramer,
    pub requests: HashMap<StratumRequestId, PendingRequestKind>,
    pub submits: HashMap<StratumRequestId, PendingSubmit>,
}
pub(in crate::v1::production_session) struct V2Runtime {
    pub session: StandardSession,
    pub generation: PoolSessionGeneration,
    pub state: MiningRuntimeState,
    pub queued: bool,
    pub poll_in_flight: bool,
    pub dispatched: bool,
}
pub(in crate::v1::production_session) enum ProtocolRuntime {
    V1(Box<V1Runtime>),
    V2(Box<V2Runtime>),
}
impl ProtocolRuntime {
    pub fn v1(runtime: LiveStratumRuntime) -> Self {
        Self::V1(Box::new(V1Runtime {
            runtime,
            framer: StratumLineFramer::default(),
            requests: HashMap::new(),
            submits: HashMap::new(),
        }))
    }
    pub fn state(&self) -> &MiningRuntimeState {
        match self {
            Self::V1(v) => v.runtime.state(),
            Self::V2(v) => &v.state,
        }
    }
    pub fn generation(&self) -> PoolSessionGeneration {
        match self {
            Self::V1(v) => v.runtime.production_registry().generation(),
            Self::V2(v) => v.generation,
        }
    }
    pub fn rebase(&mut self, generation: PoolSessionGeneration) {
        match self {
            Self::V1(v) => v.runtime.rebase_generation(generation),
            Self::V2(v) => v.generation = generation,
        }
    }
    pub fn invalidate(&mut self) {
        match self {
            Self::V1(v) => {
                v.runtime.invalidate_for_session_replacement();
                v.requests.clear();
                v.submits.clear();
                v.framer.clear();
            }
            Self::V2(v) => {
                v.session.stop();
                v.queued = false;
                v.dispatched = false;
                v.poll_in_flight = false;
                v.state
                    .block_work_submission("production_session_safe_stop");
            }
        }
    }
    pub fn block(&mut self) {
        match self {
            Self::V1(v) => v
                .runtime
                .block_work_submission("production_session_safe_stop"),
            Self::V2(v) => v
                .state
                .block_work_submission("production_session_safe_stop"),
        }
    }
    pub fn maybe_v1(&self) -> Option<&V1Runtime> {
        if let Self::V1(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn maybe_v1_mut(&mut self) -> Option<&mut V1Runtime> {
        if let Self::V1(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn maybe_v2_mut(&mut self) -> Option<&mut V2Runtime> {
        if let Self::V2(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
