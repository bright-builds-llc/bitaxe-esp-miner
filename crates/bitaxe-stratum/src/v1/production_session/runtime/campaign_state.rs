use super::ProductionMiningSession;
use crate::v1::production_session::campaign::{
    MiningCampaignLease, MiningCampaignLeaseId, MiningCampaignState, MiningCampaignStopCondition,
    MiningHardwareState,
};
use crate::v1::recovery_policy::ProductionSessionPhase;

impl ProductionMiningSession {
    pub(super) fn confirm_hardware_safe_stop(&mut self, lease_id: MiningCampaignLeaseId) {
        if self.hardware_state != MiningHardwareState::SafeStopping
            || self.maybe_lease.map(MiningCampaignLease::id) != Some(lease_id)
        {
            return;
        }
        self.hardware_state = MiningHardwareState::Stopped;
        if self.resumable_pause_pending {
            self.resumable_pause_pending = false;
            self.campaign_state = MiningCampaignState::Armed;
            self.maybe_prepared_at_ms = None;
            self.maybe_active_since_ms = None;
            self.terminal_publication_pending = false;
            return;
        }
        self.campaign_state = MiningCampaignState::Consumed;
        self.maybe_consumed_lease_id = Some(lease_id);
        self.maybe_lease = None;
        self.maybe_prepared_at_ms = None;
        self.maybe_activation_started_at_ms = None;
        self.maybe_resumable_epoch_started_at_ms = None;
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
