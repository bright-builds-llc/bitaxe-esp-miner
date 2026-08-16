use super::command_evidence::CommandEffectsEvidence;
use super::hashrate::{CampaignHashrateEvidence, HashrateTransportEvidence};
use super::model::{CampaignNetworkEvidence, REQUIRED_WINDOWS};

impl CampaignNetworkEvidence {
    pub(crate) fn fixture_complete() -> Self {
        Self {
            schema: "mining-campaign-network-continuity-v4",
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
            hashrate_monitor: CampaignHashrateEvidence {
                monitor_cadence_ms: 1_000,
                asic_count: 1,
                domain_count: 4,
                http: complete_hashrate_transport(),
                websocket: complete_hashrate_transport(),
            },
            command_effects: None,
            command_failure: None,
            maybe_failure: None,
        }
    }

    pub(crate) fn fixture_command_effects() -> Self {
        Self::from_command_effects(
            CommandEffectsEvidence {
                schema: "mining-campaign-command-effects-v8",
                genuine_block_notification_observed: true,
                positive_block_count_observed: true,
                pause_request_count: 1,
                pause_confirmed: true,
                resume_request_count: 1,
                resume_intent_confirmed: true,
                resume_confirmed: true,
                identify_status_baseline_confirmed: true,
                identify_operator_ready_confirmed: true,
                identify_request_count: 1,
                identify_replay_request_count: 0,
                identify_rendered_confirmed: true,
                identify_cleared_confirmed: true,
                identify_render_receipt_confirmed: true,
                identify_clear_receipt_confirmed: true,
                retained_identify_transition_confirmed: true,
                serial_transition_witnesses_confirmed: true,
                websocket_transition_witnesses_confirmed: true,
                identify_terminal_outcome: "none",
                dismiss_request_count: 1,
                dismiss_confirmed: true,
                block_count_preserved: true,
                active_before_pause: true,
                active_after_resume: true,
                recovery_pause_api_confirmed: false,
                recovery_pause_serial_confirmed: false,
                recovery_safe_stop_confirmed: false,
                recovery_terminal_outcome: "not_required",
                same_boot_and_package: true,
                safety_valid: true,
                terminal_http_valid: true,
                terminal_pool_persisted: true,
            },
            0,
            None,
            None,
        )
    }
}

fn complete_hashrate_transport() -> HashrateTransportEvidence {
    HashrateTransportEvidence {
        active_sample_count: 40,
        positive_coherent_count: 40,
        distinct_positive_count: 2,
        warm_rolling_window_count: 36,
        terminal_zero_confirmed: true,
    }
}
