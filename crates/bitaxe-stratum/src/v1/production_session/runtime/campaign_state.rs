use super::ProductionMiningSession;
use crate::v1::production_session::campaign::{
    MiningCampaignLease, MiningCampaignLeaseId, MiningCampaignState, MiningCampaignStopCondition,
    MiningHardwareState,
};
use crate::v1::recovery_policy::{ProductionSessionBlocker, ProductionSessionPhase};

impl ProductionMiningSession {
    /// Allocates the next owner-local campaign identity without reusing a
    /// currently active or terminally consumed lease.
    #[must_use]
    pub fn next_campaign_lease_id(&self) -> Option<MiningCampaignLeaseId> {
        let active = self.maybe_lease.map(MiningCampaignLease::id);
        let highest = match (active, self.maybe_consumed_lease_id) {
            (Some(active), Some(consumed)) => active.raw().max(consumed.raw()),
            (Some(active), None) => active.raw(),
            (None, Some(consumed)) => consumed.raw(),
            (None, None) => 0,
        };
        highest
            .checked_add(1)
            .and_then(|next| MiningCampaignLeaseId::new(next).ok())
    }

    pub(super) fn is_resumable_reactivation_safety_lapse(
        &self,
        blocker: ProductionSessionBlocker,
    ) -> bool {
        if blocker != ProductionSessionBlocker::SafetyPrerequisitesStale {
            return false;
        }
        if !self
            .maybe_lease
            .is_some_and(|lease| lease.stop_condition().allows_operator_resume())
        {
            return false;
        }

        // Safety freshness can lapse after hardware preparation but before pool
        // activation. A prior active epoch makes that gap resumable; a current
        // active segment keeps an actual mining-time safety lapse terminal.
        self.maybe_resumable_epoch_started_at_ms.is_some() && self.maybe_active_since_ms.is_none()
    }

    pub(super) fn confirm_hardware_safe_stop(
        &mut self,
        lease_id: MiningCampaignLeaseId,
        now_ms: u64,
    ) {
        if self.hardware_state != MiningHardwareState::SafeStopping
            || self.maybe_lease.map(MiningCampaignLease::id) != Some(lease_id)
        {
            return;
        }
        self.hardware_state = MiningHardwareState::Stopped;
        if self.resumable_pause_pending {
            if let Some(active_since_ms) = self.maybe_active_since_ms.take() {
                self.resumable_active_ms = self
                    .resumable_active_ms
                    .saturating_add(now_ms.saturating_sub(active_since_ms));
            }
            self.resumable_pause_pending = false;
            self.campaign_state = MiningCampaignState::Armed;
            self.maybe_prepared_at_ms = None;
            self.terminal_publication_pending = false;
            return;
        }
        self.finish_terminal_safe_stop(lease_id);
    }

    pub(super) fn finish_terminal_safe_stop(&mut self, lease_id: MiningCampaignLeaseId) {
        self.maybe_pool_set = None;
        self.primary = None;
        self.fallback = None;
        self.hardware_state = MiningHardwareState::Stopped;
        self.campaign_state = MiningCampaignState::Consumed;
        self.maybe_consumed_lease_id = Some(lease_id);
        self.maybe_lease = None;
        self.maybe_prepared_at_ms = None;
        self.maybe_activation_started_at_ms = None;
        self.maybe_resumable_epoch_started_at_ms = None;
        self.resumable_active_ms = 0;
        self.maybe_active_since_ms = None;
        self.terminal_publication_pending = false;
    }

    pub(super) fn note_campaign_active(&mut self, now_ms: u64) {
        if matches!(
            self.recovery.projection().phase,
            ProductionSessionPhase::RunningPrimary | ProductionSessionPhase::RunningFallback
        ) && self.hardware_state == MiningHardwareState::Ready
        {
            self.campaign_state = MiningCampaignState::Active;
            self.maybe_active_since_ms.get_or_insert(now_ms);
            if self.maybe_lease.is_some_and(|lease| {
                matches!(
                    lease.stop_condition(),
                    MiningCampaignStopCondition::ResumableActiveEpoch { .. }
                )
            }) {
                self.maybe_resumable_epoch_started_at_ms
                    .get_or_insert(now_ms);
            }
        }
    }
}
