//! Atomic command-state projection for the physical screen owner.

use bitaxe_api::IdentifyMode;
use bitaxe_stratum::v1::state::MiningOperatorIntent;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct ScreenCommandProjection {
    pub(super) overheat: bool,
    pub(super) identify_active: bool,
    pub(super) fallback_active: bool,
    pub(super) hashrate_ghs: f64,
    pub(super) maybe_best_difficulty: Option<f64>,
    pub(super) shares_accepted: u64,
    pub(super) shares_rejected: u64,
    pub(super) work_received: u64,
    pub(super) mining_paused: bool,
    pub(super) show_new_block: bool,
}

pub(super) fn collect(now_ms: u64) -> ScreenCommandProjection {
    super::mutate_command_visible_state_with_result(ScreenCommandProjection::default(), |state| {
        ScreenCommandProjection {
            overheat: matches!(
                state.mining.maybe_blocked_reason,
                Some("overheat_safe_stop" | "local_overheat")
            ),
            identify_active: state.identify.mode_at(now_ms) == IdentifyMode::Active,
            fallback_active: state.mining.fallback_active,
            hashrate_ghs: state.mining.hashrate_inputs.current_ghs,
            // The reference screen redraws on the session nonce best.
            maybe_best_difficulty: Some(state.best_difficulty.session as f64),
            shares_accepted: state.mining.counters.accepted,
            shares_rejected: state.mining.counters.rejected,
            work_received: state.work_received,
            mining_paused: state.mining.operator_intent == MiningOperatorIntent::Paused,
            show_new_block: state.block_found.show_new_block,
        }
    })
}
