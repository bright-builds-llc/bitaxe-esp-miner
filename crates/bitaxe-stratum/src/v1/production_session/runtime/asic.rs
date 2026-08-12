use bitaxe_asic::bm1366::production::ProductionAsicBlocker;

use super::ProductionMiningSession;
use crate::v1::live_runtime::BridgeObservationOutcome;
use crate::v1::production_session::asic_diagnostics::{AsicCorrelation, AsicPollCompletion};
use crate::v1::production_session::types::ProductionSessionEffect;
use crate::v1::production_work::{
    NonSubmitReason, PoolSessionGeneration, ProductionNonceObservation,
};
use crate::StratumV1Error;

impl ProductionMiningSession {
    pub(super) fn handle_asic_result(
        &mut self,
        observation: ProductionNonceObservation,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let Some(active_pool) = self.recovery.projection().maybe_active_pool else {
            return Ok(());
        };
        let is_current =
            self.current_generation(active_pool) == Some(observation.observed_generation);
        self.asic_diagnostics
            .note_nonce(observation.observed_generation, now_ms, is_current);
        if !is_current {
            self.job_transition.note_stale_generation_result();
            return Ok(());
        }

        let maybe_receipt = self
            .maybe_pool_runtime_mut(active_pool)
            .map(|pool_runtime| {
                pool_runtime
                    .runtime
                    .apply_bridge_observation_with_receipt(observation)
            })
            .transpose()?;
        let maybe_scoreboard_candidate = maybe_receipt
            .as_ref()
            .and_then(|receipt| receipt.maybe_scoreboard_candidate.clone());
        let found_block = maybe_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.found_block);
        if let Some(receipt) = &maybe_receipt {
            self.asic_diagnostics.note_correlation(
                observation.observed_generation,
                correlation_category(receipt.outcome),
                now_ms,
            );
        }
        if maybe_receipt.as_ref().is_some_and(|receipt| {
            !matches!(receipt.outcome, BridgeObservationOutcome::Blocked { .. })
        }) {
            self.job_transition
                .note_correlated_result(observation.observed_generation);
        }
        if found_block {
            effects.push(ProductionSessionEffect::RecordBlockFound);
        }
        self.drain_runtime_actions(active_pool, effects)?;
        if let Some(candidate) = maybe_scoreboard_candidate {
            effects.push(ProductionSessionEffect::RecordScoreboard { candidate });
        }
        self.bridge.note_result_received();
        self.drive_bridge(now_ms, effects)
    }

    pub(super) fn handle_asic_poll_completion(
        &mut self,
        generation: PoolSessionGeneration,
        completion: AsicPollCompletion,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        let maybe_active_pool = self.recovery.projection().maybe_active_pool;
        let is_current =
            maybe_active_pool.and_then(|pool| self.current_generation(pool)) == Some(generation);
        self.asic_diagnostics
            .note_poll_completion(generation, completion, now_ms, is_current);
        if !is_current {
            return Ok(());
        }
        let _streak = self.bridge.note_poll_timeout();
        self.drive_bridge(now_ms, effects)
    }
}

const fn correlation_category(outcome: BridgeObservationOutcome) -> AsicCorrelation {
    match outcome {
        BridgeObservationOutcome::SubmitQueued => AsicCorrelation::Correlated,
        BridgeObservationOutcome::Ignored {
            reason: NonSubmitReason::BelowPoolTarget,
        } => AsicCorrelation::BelowTarget,
        BridgeObservationOutcome::Ignored {
            reason: NonSubmitReason::DuplicateCandidate,
        } => AsicCorrelation::Duplicate,
        BridgeObservationOutcome::Blocked {
            reason: ProductionAsicBlocker::WrongSession,
        } => AsicCorrelation::BlockedWrongSession,
        BridgeObservationOutcome::Blocked {
            reason: ProductionAsicBlocker::JobUncorrelated,
        } => AsicCorrelation::BlockedJobLookup,
        BridgeObservationOutcome::Blocked {
            reason: ProductionAsicBlocker::WorkStale,
        } => AsicCorrelation::BlockedWorkStale,
        BridgeObservationOutcome::Blocked {
            reason: ProductionAsicBlocker::TargetMismatch,
        } => AsicCorrelation::BlockedTargetMismatch,
        BridgeObservationOutcome::Blocked { .. } => AsicCorrelation::BlockedOther,
    }
}
