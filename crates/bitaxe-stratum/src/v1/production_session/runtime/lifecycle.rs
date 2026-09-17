use super::*;

impl ProductionMiningSession {
    pub(super) fn handle_wakeup(
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
}
