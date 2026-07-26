//! Work-result investigation compile-time plan selection.
//!
//! Controlled by `BITAXE_WORK_RESULT_INVESTIGATION` action env at firmware build time.
//! Supports comma-separated modes, e.g. `frequency_ramp,single_dispatch_bounded_read`.

use bitaxe_asic::work_result_investigation::investigation_modes_contain;

const INVESTIGATION_RAW: &str = match option_env!("BITAXE_WORK_RESULT_INVESTIGATION") {
    Some(raw) => raw,
    None => "",
};

pub fn clear_rx_before_production_work() -> bool {
    has_investigation_mode("clear_rx_before_production_work")
}

/// Phase 28.1 A/B control lever: compile-time opt-out that restores the
/// pre-28.1 pump behavior — one dispatch per queued pool work, bounded
/// result read, fail-closed `ResultTimeout` on chip silence.
pub fn single_dispatch_bounded_read_enabled() -> bool {
    has_investigation_mode("single_dispatch_bounded_read")
}

/// Phase 28.1.1.2 A/B: match upstream hashrate-monitor register-read poll
/// cadence (~1 Hz × REGISTER_MAP entries). Off by default; investigation only.
pub fn match_upstream_register_read_poll_enabled() -> bool {
    has_investigation_mode("match_upstream_register_read_poll")
}

/// Phase 28.1.1 accepted-state diagnostic. Off by default and one-shot only.
pub fn accepted_state_snapshot_enabled() -> bool {
    has_investigation_mode("accepted_state_snapshot")
}

/// Phase 28.1.1.3 A/B: continuous result poll uses upstream-like long-block
/// `RESULT_WORK_TIMEOUT_MS` (10000) instead of the 100 ms socket clamp.
/// Off by default; investigation only.
pub fn upstream_like_long_block_receive_enabled() -> bool {
    has_investigation_mode("upstream_like_long_block_receive")
}

fn has_investigation_mode(mode: &str) -> bool {
    investigation_modes_contain(INVESTIGATION_RAW, mode)
}

#[cfg(test)]
mod tests {
    use bitaxe_asic::work_result_investigation::investigation_modes_contain;

    use super::accepted_state_snapshot_enabled;

    #[test]
    fn accepted_state_snapshot_is_absent_from_default_build() {
        // Arrange / Act / Assert
        assert!(!accepted_state_snapshot_enabled());
    }

    #[test]
    fn accepted_state_snapshot_token_is_exact() {
        // Arrange
        let modes = "frequency_ramp,accepted_state_snapshot";

        // Act / Assert
        assert!(investigation_modes_contain(
            modes,
            "accepted_state_snapshot"
        ));
        assert!(!investigation_modes_contain(modes, "accepted_state"));
    }
}
