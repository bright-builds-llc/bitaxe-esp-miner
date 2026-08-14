use super::command_evidence::CommandEffectsEvidence;
use super::model::{CampaignNetworkEvidence, REQUIRED_WINDOWS};

impl CampaignNetworkEvidence {
    pub(crate) fn fixture_complete() -> Self {
        Self {
            schema: "mining-campaign-network-continuity-v3",
            status: "accepted",
            required_window_count: REQUIRED_WINDOWS,
            covered_window_count: REQUIRED_WINDOWS,
            http_success_count: 40,
            websocket_frame_count: 40,
            websocket_reconnect_count: 0,
            websocket_connect_failure_count: 0,
            websocket_peer_close_count: 0,
            websocket_io_failure_count: 0,
            websocket_protocol_failure_count: 0,
            websocket_capacity_failure_count: 0,
            websocket_other_failure_count: 0,
            recovery_pause_request_count: 0,
            http_startup_transition_count: 0,
            websocket_startup_transition_count: 0,
            http_initial_active_observed: true,
            websocket_initial_active_observed: true,
            maximum_http_gap_ms: 5_000,
            maximum_websocket_gap_ms: 500,
            maximum_active_marker_gap_ms: 1_000,
            same_boot_and_package: true,
            active_state_valid: true,
            safety_valid: true,
            watchdog_valid: true,
            work_renewal_valid: true,
            terminal_http_valid: true,
            terminal_websocket_valid: true,
            terminal_pool_persisted: true,
            command_effects: None,
            maybe_failure: None,
        }
    }

    pub(crate) fn fixture_command_effects() -> Self {
        Self::from_command_effects(
            CommandEffectsEvidence {
                schema: "mining-campaign-command-effects-v4",
                genuine_block_notification_observed: true,
                positive_block_count_observed: true,
                pause_request_count: 1,
                pause_confirmed: true,
                resume_request_count: 1,
                resume_confirmed: true,
                identify_operator_ready_confirmed: true,
                identify_request_count: 2,
                identify_replay_request_count: 0,
                identify_rendered_confirmed: true,
                identify_cleared_confirmed: true,
                identify_terminal_outcome: "none",
                dismiss_request_count: 1,
                dismiss_confirmed: true,
                block_count_preserved: true,
                active_before_pause: true,
                active_after_resume: true,
                same_boot_and_package: true,
                safety_valid: true,
                terminal_http_valid: true,
                terminal_pool_persisted: true,
            },
            0,
            None,
        )
    }
}
