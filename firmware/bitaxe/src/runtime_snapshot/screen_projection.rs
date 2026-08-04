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
            overheat: state.mining.maybe_blocked_reason == Some("overheat_safe_stop"),
            identify_active: state.identify.mode_at(now_ms) == IdentifyMode::Active,
            fallback_active: state.mining.fallback_active,
            hashrate_ghs: state.mining.hashrate_inputs.current_ghs,
            maybe_best_difficulty: state
                .mining
                .counters
                .maybe_best_difficulty
                .map(|difficulty| difficulty.raw()),
            shares_accepted: state.mining.counters.accepted,
            shares_rejected: state.mining.counters.rejected,
            work_received: state.work_received,
            mining_paused: state.mining.operator_intent == MiningOperatorIntent::Paused,
            show_new_block: state.block_found.show_new_block,
        }
    })
}
