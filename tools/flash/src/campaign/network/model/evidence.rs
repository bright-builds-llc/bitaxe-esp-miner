use serde::Serialize;

use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CampaignNetworkEvidence {
    pub(in crate::campaign) schema: &'static str,
    pub(in crate::campaign) status: &'static str,
    pub(in crate::campaign) correlation_failure: &'static str,
    pub(in crate::campaign) required_window_count: usize,
    pub(in crate::campaign) covered_window_count: usize,
    pub(in crate::campaign) http_success_count: u64,
    pub(in crate::campaign) websocket_frame_count: u64,
    pub(in crate::campaign) websocket_reconnect_count: u64,
    pub(in crate::campaign) websocket_connect_failure_count: u64,
    pub(in crate::campaign) websocket_peer_close_count: u64,
    pub(in crate::campaign) websocket_io_failure_count: u64,
    pub(in crate::campaign) websocket_protocol_failure_count: u64,
    pub(in crate::campaign) websocket_capacity_failure_count: u64,
    pub(in crate::campaign) websocket_other_failure_count: u64,
    pub(in crate::campaign) recovery_pause_request_count: u64,
    pub(in crate::campaign) http_startup_transition_count: u64,
    pub(in crate::campaign) websocket_startup_transition_count: u64,
    pub(in crate::campaign) http_initial_active_observed: bool,
    pub(in crate::campaign) websocket_initial_active_observed: bool,
    pub(in crate::campaign) maximum_http_gap_ms: u64,
    pub(in crate::campaign) maximum_websocket_gap_ms: u64,
    pub(in crate::campaign) maximum_active_marker_gap_ms: u64,
    pub(in crate::campaign) same_boot_and_package: bool,
    pub(in crate::campaign) active_state_valid: bool,
    pub(in crate::campaign) safety_valid: bool,
    pub(in crate::campaign) watchdog_valid: bool,
    pub(in crate::campaign) watchdog_failure: &'static str,
    pub(in crate::campaign) watchdog_read_outcome: &'static str,
    pub(in crate::campaign) watchdog_owner_phase: &'static str,
    pub(in crate::campaign) watchdog_owner_subphase: &'static str,
    pub(in crate::campaign) watchdog_wait_state: &'static str,
    pub(in crate::campaign) work_renewal_valid: bool,
    pub(in crate::campaign) terminal_http_valid: bool,
    pub(in crate::campaign) terminal_websocket_valid: bool,
    pub(in crate::campaign) terminal_pool_persisted: bool,
    pub(in crate::campaign) hashrate_monitor: CampaignHashrateEvidence,
    pub(in crate::campaign) command_effects: Option<CommandEffectsEvidence>,
    pub(in crate::campaign) command_failure: Option<CommandFailureDiagnostic>,
    #[serde(skip)]
    pub(in crate::campaign) maybe_failure: Option<CampaignTerminalCategory>,
}

impl CampaignNetworkEvidence {
    pub(crate) fn not_required() -> Self {
        Self::empty("not_required", None)
    }

    pub(in crate::campaign::network) fn from_unobserved(
        shared: &Arc<Mutex<SharedSerialState>>,
    ) -> Self {
        let maybe_failure = shared
            .lock()
            .ok()
            .and_then(|state| state.maybe_failure)
            .or(Some(CampaignTerminalCategory::NetworkTargetUnavailable));
        Self::empty("failed", maybe_failure)
    }

    pub(in crate::campaign::network) fn worker_failed(
        shared: &Arc<Mutex<SharedSerialState>>,
    ) -> Self {
        let maybe_failure = shared
            .lock()
            .ok()
            .and_then(|state| state.maybe_failure)
            .or(Some(CampaignTerminalCategory::NetworkCorrelationFailed));
        Self::empty("failed", maybe_failure)
    }

    fn empty(status: &'static str, maybe_failure: Option<CampaignTerminalCategory>) -> Self {
        Self {
            schema: "mining-campaign-network-continuity-v11",
            status,
            correlation_failure: "not_observed",
            required_window_count: REQUIRED_WINDOWS,
            covered_window_count: 0,
            http_success_count: 0,
            websocket_frame_count: 0,
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
            http_initial_active_observed: false,
            websocket_initial_active_observed: false,
            maximum_http_gap_ms: 0,
            maximum_websocket_gap_ms: 0,
            maximum_active_marker_gap_ms: 0,
            same_boot_and_package: false,
            active_state_valid: false,
            safety_valid: false,
            watchdog_valid: false,
            watchdog_failure: WatchdogFailure::None.label(),
            watchdog_read_outcome: WatchdogReadOutcome::Uninitialized.label(),
            watchdog_owner_phase: WatchdogOwnerPhase::Unavailable.label(),
            watchdog_owner_subphase: WatchdogOwnerSubphase::Unavailable.label(),
            watchdog_wait_state: WatchdogWaitState::NotWaiting.label(),
            work_renewal_valid: false,
            terminal_http_valid: false,
            terminal_websocket_valid: false,
            terminal_pool_persisted: false,
            hashrate_monitor: CampaignHashrateEvidence::empty(),
            command_effects: None,
            command_failure: None,
            maybe_failure,
        }
    }

    pub(in crate::campaign::network) fn from_command_effects(
        evidence: CommandEffectsEvidence,
        recovery_pause_request_count: u64,
        maybe_failure: Option<CampaignTerminalCategory>,
        maybe_command_failure: Option<CommandFailureDiagnostic>,
    ) -> Self {
        let complete = evidence.complete();
        let maybe_failure = maybe_failure
            .or_else(|| (!complete).then_some(CampaignTerminalCategory::NetworkCorrelationFailed));
        let status = if maybe_failure.is_none() && complete {
            "accepted"
        } else {
            "failed"
        };
        Self {
            status,
            required_window_count: 0,
            covered_window_count: 0,
            recovery_pause_request_count,
            same_boot_and_package: evidence.same_boot_and_package,
            active_state_valid: evidence.active_before_pause && evidence.active_after_resume,
            safety_valid: evidence.safety_valid,
            terminal_http_valid: evidence.terminal_http_valid,
            terminal_pool_persisted: evidence.terminal_pool_persisted,
            command_effects: Some(evidence),
            command_failure: maybe_command_failure.or_else(|| {
                (!complete).then_some(CommandFailureDiagnostic::new(
                    CommandFailurePhase::Terminal,
                    CommandFailureCause::QuorumIncomplete,
                ))
            }),
            maybe_failure,
            ..Self::empty(status, maybe_failure)
        }
    }
}
