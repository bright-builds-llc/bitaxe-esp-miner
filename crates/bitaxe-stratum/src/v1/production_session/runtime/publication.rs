use super::*;
use crate::v1::recovery_policy::RecoveryProjection;
use crate::v1::state::MiningRuntimeState;

impl ProductionMiningSession {
    /// Projects the mining state a snapshot carries from the recovery policy and the hardware state.
    pub(super) fn project_mining(&self, projection: RecoveryProjection) -> MiningRuntimeState {
        let mut mining = self
            .maybe_runtime_for_projection(projection.maybe_active_pool)
            .map(|session| session.protocol.state().clone())
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
        // Last, so it wins: the ASIC stops within milliseconds of the safe-stop's start, but the terminal state
        // waits for its cooling proof (ADR-0016), so the stop shows as `stopping` meanwhile. A stop that
        // failed shows as blocked with its reason, never as still stopping.
        if self.hardware_state == MiningHardwareState::SafeStopping {
            if self.hardware_safe_stop_failed {
                mining.block_work_submission(HARDWARE_SAFE_STOP_FAILED);
                mining.set_mining_activity(MiningActivityStatus::SafeBlocked);
            } else {
                mining.block_work_submission("hardware_safe_stopping");
                mining.set_mining_activity(MiningActivityStatus::Stopping);
            }
        }
        mining
    }

    pub(super) fn publish_if_changed(&mut self, effects: &mut Vec<ProductionSessionEffect>) {
        if self.terminal_publication_pending {
            return;
        }
        self.publish_now_if_changed(effects);
    }

    /// Publishes a changed snapshot even while the terminal one is held back.
    pub(super) fn publish_now_if_changed(&mut self, effects: &mut Vec<ProductionSessionEffect>) {
        let snapshot = self.snapshot();
        if self.maybe_last_snapshot.as_ref() != Some(&snapshot) {
            self.maybe_last_snapshot = Some(snapshot.clone());
            effects.push(ProductionSessionEffect::Publish(Box::new(snapshot)));
        }
    }
}
