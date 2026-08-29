mod asic;
mod campaign_state;
mod transport;

use super::asic_diagnostics::AsicBridgeDiagnosticsTracker;
use super::campaign::{
    CampaignExpiration, MiningCampaignLease, MiningCampaignLeaseId, MiningCampaignState,
    MiningCampaignStopCondition, MiningCampaignTiming, MiningHardwareState,
};
use super::job_transition::JobTransitionTracker;
use crate::v1::bridge_orchestration::BridgeOrchestrator;
use crate::v1::production_work::PoolSessionGeneration;
use crate::v1::recovery_policy::{
    ProductionPool, ProductionPoolAvailability, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionPhase, ProductionSessionWakeup, RecoveryAction, RecoveryPolicy,
};
use crate::v1::state::{MiningActivityStatus, PoolLifecycleStatus};
use crate::StratumV1Error;

use super::types::{
    HardwareSafeStopPurpose, ProductionAsicFailure, ProductionPoolSet, ProductionSessionEffect,
    ProductionSessionEvent, ProductionSessionSnapshot, ProductionTransportEpoch,
    ProductionTransportFailure,
};

pub(super) use transport::{
    maybe_runtime_request_kind, PendingRequestKind, PendingSubmit, PoolRuntime,
};

pub struct ProductionMiningSession {
    pub(super) recovery: RecoveryPolicy,
    pub(super) maybe_pool_set: Option<ProductionPoolSet>,
    pub(super) primary: Option<PoolRuntime>,
    pub(super) fallback: Option<PoolRuntime>,
    pub(super) bridge: BridgeOrchestrator,
    pub(super) generation_cursor: PoolSessionGeneration,
    pub(super) transport_epoch_cursor: ProductionTransportEpoch,
    pub(super) maybe_primary_transport_epoch: Option<ProductionTransportEpoch>,
    pub(super) maybe_fallback_transport_epoch: Option<ProductionTransportEpoch>,
    pub(super) last_readiness: ProductionReadiness,
    pub(super) hardware_state: MiningHardwareState,
    pub(super) campaign_state: MiningCampaignState,
    pub(super) maybe_lease: Option<MiningCampaignLease>,
    pub(super) maybe_consumed_lease_id: Option<MiningCampaignLeaseId>,
    pub(super) maybe_prepared_at_ms: Option<u64>,
    pub(super) maybe_activation_started_at_ms: Option<u64>,
    pub(super) maybe_resumable_epoch_started_at_ms: Option<u64>,
    pub(super) resumable_active_ms: u64,
    pub(super) maybe_active_since_ms: Option<u64>,
    pub(super) resumable_pause_pending: bool,
    pub(super) job_transition: JobTransitionTracker,
    pub(super) asic_diagnostics: AsicBridgeDiagnosticsTracker,
    pub(super) terminal_publication_pending: bool,
    pub(super) maybe_retained_mining: Option<crate::v1::state::MiningRuntimeState>,
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
            generation_cursor: PoolSessionGeneration::initial(),
            transport_epoch_cursor: ProductionTransportEpoch::initial(),
            maybe_primary_transport_epoch: None,
            maybe_fallback_transport_epoch: None,
            last_readiness: ProductionReadiness {
                operator_intent: crate::v1::state::MiningOperatorIntent::Run,
                network_ready: false,
                stratum_v1_supported: false,
                safety_prerequisites_fresh: false,
                maybe_campaign_lease: None,
                actuation_qualified: false,
            },
            hardware_state: MiningHardwareState::Unprepared,
            campaign_state: MiningCampaignState::Unavailable,
            maybe_lease: None,
            maybe_consumed_lease_id: None,
            maybe_prepared_at_ms: None,
            maybe_activation_started_at_ms: None,
            maybe_resumable_epoch_started_at_ms: None,
            resumable_active_ms: 0,
            maybe_active_since_ms: None,
            resumable_pause_pending: false,
            job_transition: JobTransitionTracker::default(),
            asic_diagnostics: AsicBridgeDiagnosticsTracker::default(),
            terminal_publication_pending: false,
            maybe_retained_mining: None,
            maybe_last_snapshot: None,
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> ProductionSessionSnapshot {
        let projection = self.recovery.projection();
        let mut mining = self
            .maybe_runtime_for_projection(projection.maybe_active_pool)
            .map(|session| session.runtime.state().clone())
            .or_else(|| self.maybe_retained_mining.clone())
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
                    .maybe_runtime_for_projection(projection.maybe_active_pool)
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
            hardware_state: self.hardware_state,
            campaign_state: self.campaign_state,
            job_transition: self.job_transition.evidence(),
            asic_bridge: self.asic_diagnostics.evidence(),
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
                self.handle_wakeup(wakeup, readiness, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::PoolConfigurationLoaded(maybe_pool_set) => {
                if self.hardware_state != MiningHardwareState::Ready {
                    self.publish_if_changed(&mut effects);
                    return Ok(effects);
                }
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
            ProductionSessionEvent::CampaignLeaseRenewed { lease, now_ms } => {
                if self.maybe_lease.map(MiningCampaignLease::id) != Some(lease.id())
                    || !matches!(
                        self.campaign_state,
                        MiningCampaignState::Armed | MiningCampaignState::Active
                    )
                {
                    self.begin_terminal_safe_stop(
                        Some(ProductionSessionBlocker::CampaignLeaseConsumed),
                        false,
                        &mut effects,
                    )?;
                } else {
                    self.maybe_lease = Some(lease);
                    self.maybe_active_since_ms = Some(now_ms);
                }
            }
            ProductionSessionEvent::CampaignLeaseRevoked => {
                self.begin_terminal_safe_stop(
                    Some(ProductionSessionBlocker::CampaignLeaseConsumed),
                    false,
                    &mut effects,
                )?;
            }
            ProductionSessionEvent::TransportConnected {
                pool,
                transport_epoch,
                now_ms,
            } => {
                if self.maybe_pending_transport_epoch(pool) != Some(transport_epoch) {
                    return Ok(effects);
                }
                self.start_pool_runtime(pool, transport_epoch, &mut effects)?;
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::TransportFailed {
                pool,
                transport_epoch,
                failure,
                now_ms,
            } => match failure {
                ProductionTransportFailure::Connect => {
                    if self.maybe_pending_transport_epoch(pool) != Some(transport_epoch) {
                        return Ok(effects);
                    }
                    self.take_pending_transport_epoch(pool);
                    let actions = self.recovery.on_connection_result(pool, false, now_ms);
                    self.apply_recovery_actions(actions, &mut effects)?;
                }
                ProductionTransportFailure::Read | ProductionTransportFailure::Write => {
                    if !self.transport_epoch_is_active(pool, transport_epoch) {
                        return Ok(effects);
                    }
                    self.handle_transport_failure(pool, now_ms, &mut effects)?;
                }
            },
            ProductionSessionEvent::TransportBytes {
                pool,
                transport_epoch,
                bytes,
                now_ms,
            } => {
                if !self.transport_epoch_is_active(pool, transport_epoch) {
                    return Ok(effects);
                }
                self.apply_transport_bytes(pool, transport_epoch, &bytes, now_ms, &mut effects)?;
                self.note_campaign_active(now_ms);
                self.drive_bridge(now_ms, &mut effects)?;
            }
            ProductionSessionEvent::TransportClosed {
                pool,
                transport_epoch,
                now_ms,
            } => {
                if !self.transport_epoch_is_active(pool, transport_epoch) {
                    return Ok(effects);
                }
                self.handle_transport_failure(pool, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::AsicResult {
                observation,
                now_ms,
            } => {
                self.handle_asic_result(observation, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::AsicPollTimedOut { generation, now_ms } => {
                self.handle_asic_poll_completion(
                    generation,
                    super::asic_diagnostics::AsicPollCompletion::Idle,
                    now_ms,
                    &mut effects,
                )?;
            }
            ProductionSessionEvent::AsicPollCompleted {
                generation,
                completion,
                now_ms,
            } => {
                self.handle_asic_poll_completion(generation, completion, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::AsicInteractionFailed {
                generation,
                failure,
                now_ms: _,
            } => {
                let maybe_active_pool = self.recovery.projection().maybe_active_pool;
                if maybe_active_pool.and_then(|pool| self.current_generation(pool))
                    != Some(generation)
                {
                    return Ok(effects);
                }
                self.begin_terminal_safe_stop(
                    Some(match failure {
                        ProductionAsicFailure::VersionMask => {
                            ProductionSessionBlocker::ProductionAsicVersionMaskUnavailable
                        }
                        ProductionAsicFailure::Dispatch => {
                            ProductionSessionBlocker::ProductionAsicDispatchUnavailable
                        }
                        ProductionAsicFailure::Poll => {
                            ProductionSessionBlocker::ProductionAsicPollUnavailable
                        }
                        ProductionAsicFailure::QueueFull => {
                            ProductionSessionBlocker::ProductionAsicQueueFull
                        }
                        ProductionAsicFailure::WorkerDisconnected => {
                            ProductionSessionBlocker::ProductionAsicWorkerUnavailable
                        }
                    }),
                    false,
                    &mut effects,
                )?;
            }
            ProductionSessionEvent::HardwarePrepared { lease_id, now_ms } => {
                self.handle_hardware_prepared(lease_id, now_ms, &mut effects)?;
            }
            ProductionSessionEvent::HardwarePreparationFailed {
                lease_id,
                failure: _,
                now_ms: _,
            } => {
                if self.maybe_lease.map(MiningCampaignLease::id) == Some(lease_id)
                    && self.hardware_state == MiningHardwareState::Preparing
                {
                    self.begin_terminal_safe_stop(
                        Some(ProductionSessionBlocker::ProductionAsicUnavailable),
                        false,
                        &mut effects,
                    )?;
                }
            }
            ProductionSessionEvent::HardwareSafeStopConfirmed { lease_id, now_ms } => {
                self.confirm_hardware_safe_stop(lease_id, now_ms);
            }
            ProductionSessionEvent::EffectFailed {
                maybe_pool,
                reason: _,
                now_ms,
            } => {
                if let Some(pool) = maybe_pool {
                    self.handle_transport_failure(pool, now_ms, &mut effects)?;
                } else {
                    self.begin_terminal_safe_stop(
                        Some(ProductionSessionBlocker::ProductionAsicUnavailable),
                        false,
                        &mut effects,
                    )?;
                }
            }
        }
        if matches!(
            self.recovery.projection().maybe_blocker,
            Some(
                ProductionSessionBlocker::PoolConfigurationUnavailable
                    | ProductionSessionBlocker::PoolsExhausted
            )
        ) {
            self.begin_hardware_safe_stop_if_needed(&mut effects)?;
        }
        self.publish_if_changed(&mut effects);
        Ok(effects)
    }
}

impl ProductionMiningSession {
    fn publish_if_changed(&mut self, effects: &mut Vec<ProductionSessionEffect>) {
        if self.terminal_publication_pending {
            return;
        }
        let snapshot = self.snapshot();
        if self.maybe_last_snapshot.as_ref() != Some(&snapshot) {
            self.maybe_last_snapshot = Some(snapshot.clone());
            effects.push(ProductionSessionEffect::Publish(Box::new(snapshot)));
        }
    }

    fn handle_wakeup(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        self.last_readiness = readiness;
        let timing = MiningCampaignTiming {
            maybe_prepared_at_ms: self.maybe_prepared_at_ms,
            maybe_activation_started_at_ms: self.maybe_activation_started_at_ms,
            maybe_resumable_epoch_started_at_ms: self.maybe_resumable_epoch_started_at_ms,
            resumable_active_ms: self.resumable_active_ms,
            maybe_active_since_ms: self.maybe_active_since_ms,
        };
        if let Some(expiration) = self
            .maybe_lease
            .and_then(|lease| lease.stop_condition().maybe_expiration(now_ms, timing))
        {
            let blocker = match expiration {
                CampaignExpiration::ActivationTimedOut => {
                    ProductionSessionBlocker::CampaignActivationTimedOut
                }
                CampaignExpiration::LeaseConsumed => {
                    ProductionSessionBlocker::CampaignLeaseConsumed
                }
            };
            return self.begin_terminal_safe_stop(Some(blocker), false, effects);
        }
        if matches!(wakeup, Some(ProductionSessionWakeup::ShutdownRequested)) {
            return self.begin_terminal_safe_stop(None, true, effects);
        }
        if matches!(wakeup, Some(ProductionSessionWakeup::SettingsChanged))
            && matches!(
                self.hardware_state,
                MiningHardwareState::Preparing | MiningHardwareState::Ready
            )
        {
            return self.begin_terminal_safe_stop(
                Some(ProductionSessionBlocker::CampaignLeaseConsumed),
                false,
                effects,
            );
        }
        if let Some(blocker) = readiness.maybe_blocker() {
            let actions = self.recovery.on_wakeup(wakeup, readiness, now_ms);
            self.apply_recovery_actions(actions, effects)?;
            self.resumable_pause_pending = blocker == ProductionSessionBlocker::OperatorPaused
                && self
                    .maybe_lease
                    .is_some_and(|lease| lease.stop_condition().allows_operator_resume())
                || self.is_resumable_reactivation_safety_lapse(blocker);
            self.begin_hardware_safe_stop_if_needed(effects)?;
            return Ok(());
        }

        let Some(lease) = readiness.maybe_campaign_lease else {
            return Ok(());
        };
        if self
            .maybe_consumed_lease_id
            .is_some_and(|consumed| lease.id().raw() <= consumed.raw())
        {
            return self.begin_terminal_safe_stop(
                Some(ProductionSessionBlocker::CampaignLeaseConsumed),
                false,
                effects,
            );
        }
        if let Some(active_lease) = self.maybe_lease {
            if active_lease.id() != lease.id() {
                return self.begin_terminal_safe_stop(
                    Some(ProductionSessionBlocker::CampaignLeaseConsumed),
                    false,
                    effects,
                );
            }
        }

        match self.hardware_state {
            MiningHardwareState::Unprepared | MiningHardwareState::Stopped => {
                self.maybe_lease = Some(lease);
                self.maybe_activation_started_at_ms.get_or_insert(now_ms);
                self.hardware_state = MiningHardwareState::Preparing;
                self.campaign_state = MiningCampaignState::Preparing;
                self.maybe_prepared_at_ms = None;
                self.maybe_active_since_ms = None;
                self.job_transition = JobTransitionTracker::default();
                self.asic_diagnostics = AsicBridgeDiagnosticsTracker::default();
                self.maybe_retained_mining = None;
                effects.push(ProductionSessionEffect::PrepareHardware {
                    lease_id: lease.id(),
                    profile: lease.profile(),
                });
            }
            MiningHardwareState::Ready => {
                let actions = self.recovery.on_wakeup(wakeup, readiness, now_ms);
                self.apply_recovery_actions(actions, effects)?;
                self.note_campaign_active(now_ms);
                self.drive_bridge(now_ms, effects)?;
            }
            MiningHardwareState::Preparing | MiningHardwareState::SafeStopping => {}
        }
        Ok(())
    }

    fn handle_hardware_prepared(
        &mut self,
        lease_id: MiningCampaignLeaseId,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        if self.maybe_lease.map(MiningCampaignLease::id) != Some(lease_id)
            || self.hardware_state != MiningHardwareState::Preparing
        {
            return Ok(());
        }
        self.hardware_state = MiningHardwareState::Ready;
        self.campaign_state = MiningCampaignState::Armed;
        self.maybe_prepared_at_ms = Some(now_ms);
        let actions = self.recovery.on_wakeup(None, self.last_readiness, now_ms);
        self.apply_recovery_actions(actions, effects)?;
        self.note_campaign_active(now_ms);
        self.drive_bridge(now_ms, effects)
    }

    pub(super) fn begin_terminal_safe_stop(
        &mut self,
        maybe_blocker: Option<ProductionSessionBlocker>,
        shutdown_requested: bool,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        // A terminal cause supersedes a pending resumable pause. Otherwise a
        // late hardware confirmation can incorrectly re-arm a consumed lease.
        self.resumable_pause_pending = false;
        let actions = if shutdown_requested {
            self.recovery.on_wakeup(
                Some(ProductionSessionWakeup::ShutdownRequested),
                self.last_readiness,
                0,
            )
        } else if let Some(blocker) = maybe_blocker {
            self.recovery.on_session_blocker(blocker)
        } else {
            Vec::new()
        };
        self.apply_recovery_actions(actions, effects)?;
        if self.hardware_state == MiningHardwareState::Stopped {
            if let Some(lease_id) = self.maybe_lease.map(MiningCampaignLease::id) {
                // A prior resumable-stop confirmation already proves the
                // hardware is stopped, so consume without issuing it again.
                self.finish_terminal_safe_stop(lease_id);
            }
            return Ok(());
        }
        self.begin_hardware_safe_stop_if_needed(effects)
    }

    fn begin_hardware_safe_stop_if_needed(
        &mut self,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        if !matches!(
            self.hardware_state,
            MiningHardwareState::Preparing | MiningHardwareState::Ready
        ) {
            return Ok(());
        }

        for action in [
            RecoveryAction::BlockSubmissions,
            RecoveryAction::InvalidateWorkAndSubmissions,
            RecoveryAction::StopAsicInteraction,
        ] {
            let already_ordered = match action {
                RecoveryAction::BlockSubmissions => effects
                    .iter()
                    .any(|effect| matches!(effect, ProductionSessionEffect::BlockSubmissions)),
                RecoveryAction::InvalidateWorkAndSubmissions => effects.iter().any(|effect| {
                    matches!(
                        effect,
                        ProductionSessionEffect::InvalidateWorkAndSubmissions
                    )
                }),
                RecoveryAction::StopAsicInteraction => effects
                    .iter()
                    .any(|effect| matches!(effect, ProductionSessionEffect::StopAsicInteraction)),
                _ => false,
            };
            if !already_ordered {
                self.apply_recovery_actions(vec![action], effects)?;
            }
        }

        let Some(lease_id) = self.maybe_lease.map(MiningCampaignLease::id) else {
            return Ok(());
        };
        self.hardware_state = MiningHardwareState::SafeStopping;
        self.campaign_state = MiningCampaignState::SafeStopping;
        self.terminal_publication_pending = true;
        let purpose = if self.resumable_pause_pending {
            HardwareSafeStopPurpose::ResumablePause
        } else {
            HardwareSafeStopPurpose::Terminal
        };
        effects.push(ProductionSessionEffect::SafeStopHardware { lease_id, purpose });
        Ok(())
    }

    pub(super) fn stop_after_first_submit_response(
        &mut self,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let stop_after_response = self.maybe_lease.is_some_and(|lease| {
            matches!(
                lease.stop_condition(),
                MiningCampaignStopCondition::FirstSubmitResponse { .. }
            )
        });
        if !stop_after_response {
            return Ok(());
        }
        self.begin_terminal_safe_stop(
            Some(ProductionSessionBlocker::CampaignLeaseConsumed),
            false,
            effects,
        )
    }

    pub(super) fn allocate_generation(&mut self) -> PoolSessionGeneration {
        self.generation_cursor = self.generation_cursor.next();
        self.generation_cursor
    }
}
