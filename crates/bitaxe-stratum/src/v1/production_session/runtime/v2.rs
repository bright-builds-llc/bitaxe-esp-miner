use super::protocol::{ProtocolRuntime, V2Runtime};
use super::*;
use crate::v1::{
    messages::PoolDifficulty,
    production_work::ProductionNonceObservation,
    state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus, ShareDifficulty},
};
use crate::v2::{
    frame::Frame,
    session::SessionConfig,
    standard::{StandardEvent, StandardSession},
};
use bitaxe_asic::bm1366::{
    production::{Bm1366ProductionCommand, ProductionWorkPayload},
    result::Bm1366ValidJobIds,
    work::Bm1366JobId,
};
fn protocol_error() -> StratumV1Error {
    StratumV1Error::InvalidField {
        field: "v2_protocol",
        reason: "fixed standard contract rejected",
    }
}
impl ProductionMiningSession {
    pub(in crate::v1::production_session) fn start_v2_pool(
        &mut self,
        pool: ProductionPool,
        epoch: ProductionTransportEpoch,
        generation: PoolSessionGeneration,
        config: SessionConfig,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let mut protocol = StandardSession::new(config).map_err(|_| protocol_error())?;
        let frame = protocol.start().map_err(|_| protocol_error())?;
        self.set_pool_runtime(
            pool,
            Some(PoolRuntime {
                transport_epoch: epoch,
                protocol: ProtocolRuntime::V2(Box::new(V2Runtime {
                    session: protocol,
                    generation,
                    state: MiningRuntimeState::default(),
                    queued: false,
                    poll_in_flight: false,
                    dispatched: false,
                })),
            }),
        );
        effects.push(ProductionSessionEffect::WritePoolFrame {
            pool,
            transport_epoch: epoch,
            frame,
        });
        Ok(())
    }
    pub(in crate::v1::production_session) fn apply_v2_frame(
        &mut self,
        pool: ProductionPool,
        frame: Frame,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(runtime) = self.maybe_pool_runtime_mut(pool) else {
            return Ok(());
        };
        let epoch = runtime.transport_epoch;
        let Some(v2) = runtime.protocol.maybe_v2_mut() else {
            return self.handle_transport_failure(pool, now_ms, effects);
        };
        let events = match v2.session.receive(&frame) {
            Ok(events) => events,
            Err(reason) => {
                effects.push(ProductionSessionEffect::RecordV2Failure {
                    message_type: frame.header.message_type,
                    reason,
                });
                return self.begin_terminal_safe_stop(
                    Some(ProductionSessionBlocker::JobTransitionProtocolInconsistent),
                    false,
                    effects,
                );
            }
        };
        effects.push(ProductionSessionEffect::RecordV2Frame { frame });
        for event in events {
            match event {
                StandardEvent::Send(frame) => {
                    effects.push(ProductionSessionEffect::WritePoolFrame {
                        pool,
                        transport_epoch: epoch,
                        frame,
                    })
                }
                StandardEvent::Channel => {
                    if let Some(v) = self
                        .maybe_pool_runtime_mut(pool)
                        .and_then(|r| r.protocol.maybe_v2_mut())
                    {
                        v.state.set_lifecycle(PoolLifecycleStatus::Authorized);
                        v.state
                            .set_pool_difficulty(PoolDifficulty { difficulty: 1024.0 });
                    }
                    let actions = self.recovery.on_connection_result(pool, true, now_ms);
                    self.apply_recovery_actions(actions, effects)?;
                }
                StandardEvent::Work { work, commitment } => {
                    effects.push(ProductionSessionEffect::V2WorkReady { work, commitment });
                    if let Some(v) = self
                        .maybe_pool_runtime_mut(pool)
                        .and_then(|r| r.protocol.maybe_v2_mut())
                    {
                        v.queued = true;
                        v.state.set_lifecycle(PoolLifecycleStatus::Active);
                        v.state.set_mining_activity(MiningActivityStatus::Active);
                        v.state.allow_work_submission();
                    }
                }
                StandardEvent::Accepted(_) => {
                    self.share_counters.accepted = self.share_counters.accepted.saturating_add(1);
                    if let Some(v) = self
                        .maybe_pool_runtime_mut(pool)
                        .and_then(|r| r.protocol.maybe_v2_mut())
                    {
                        v.state.record_accepted_share(ShareDifficulty::new(1024.0));
                    }
                    // Normal signed qualification continues until independent
                    // heartbeat revocation; an ACK grants no stop/renew authority.
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(in crate::v1::production_session) fn drive_v2_bridge(
        &mut self,
        pool: ProductionPool,
        _now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(v2) = self
            .maybe_pool_runtime_mut(pool)
            .and_then(|r| r.protocol.maybe_v2_mut())
        else {
            return Ok(());
        };
        let Some(work) = v2.session.maybe_work() else {
            return Ok(());
        };
        let valid_jobs = Bm1366ValidJobIds::single(work.asic_job_id);
        if v2.queued {
            v2.queued = false;
            effects.push(ProductionSessionEffect::DispatchAsic {
                generation: v2.generation,
                valid_jobs,
                command: Bm1366ProductionCommand::SendProductionWork(ProductionWorkPayload::new(
                    work.asic_job_id,
                    work.fields,
                )),
            });
        } else if v2.dispatched && !v2.poll_in_flight {
            v2.poll_in_flight = true;
            effects.push(ProductionSessionEffect::PollAsic {
                generation: v2.generation,
                valid_jobs,
                slice_ms: 50,
            });
        }
        Ok(())
    }
    pub(in crate::v1::production_session) fn note_v2_dispatched(
        &mut self,
        generation: PoolSessionGeneration,
        job: Bm1366JobId,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(pool) = self.recovery.projection().maybe_active_pool else {
            return Ok(());
        };
        let Some(v2) = self
            .maybe_pool_runtime_mut(pool)
            .and_then(|r| r.protocol.maybe_v2_mut())
        else {
            return Ok(());
        };
        if generation != v2.generation {
            return Ok(());
        }
        if v2.session.dispatched(job).is_err() {
            return self.begin_terminal_safe_stop(
                Some(ProductionSessionBlocker::JobTransitionProtocolInconsistent),
                false,
                effects,
            );
        }
        v2.dispatched = true;
        self.job_transition.note_dispatch(generation);
        self.asic_diagnostics.note_dispatch(generation, now_ms);
        self.drive_v2_bridge(pool, now_ms, effects)
    }
    pub(in crate::v1::production_session) fn handle_v2_nonce(
        &mut self,
        pool: ProductionPool,
        observation: ProductionNonceObservation,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(runtime) = self.maybe_pool_runtime_mut(pool) else {
            return Ok(());
        };
        let epoch = runtime.transport_epoch;
        let Some(v2) = runtime.protocol.maybe_v2_mut() else {
            return Ok(());
        };
        if v2.generation != observation.observed_generation {
            return Ok(());
        }
        v2.poll_in_flight = false;
        match v2.session.nonce(observation.result) {
            Ok(Some((frame, _))) => {
                v2.state.record_qualified_candidate();
                self.share_counters.qualified_candidates =
                    self.share_counters.qualified_candidates.saturating_add(1);
                effects.push(ProductionSessionEffect::WritePoolFrame {
                    pool,
                    transport_epoch: epoch,
                    frame,
                });
            }
            Ok(None) => {}
            Err(_) => {
                return self.begin_terminal_safe_stop(
                    Some(ProductionSessionBlocker::JobTransitionProtocolInconsistent),
                    false,
                    effects,
                )
            }
        }
        self.drive_v2_bridge(pool, now_ms, effects)
    }
}
