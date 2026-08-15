use std::array;
use std::sync::{Arc, Mutex};

use bitaxe_api::{ExpectedRuntimeAttestationIdentity, SystemInfoWire};
use bitaxe_http_transport::WebSocketReadFailureKind;
use serde::Serialize;

use super::super::*;
use super::command_evidence::CommandEffectsEvidence;
use super::validation::{
    active_mining_state_valid, advances, regresses, update_gap, validate_active_prerequisites,
    validate_sample, watchdog_valid, window_index, SampleValidationFailure,
};

pub(super) const REQUIRED_WINDOWS: usize = 20;
pub(super) const WINDOW_MILLIS: u64 = 30_000;
pub(super) const MAX_ACTIVE_MARKER_GAP_MILLIS: u64 = 5_000;
pub(super) const TERMINAL_NETWORK_DEADLINE_SECONDS: u64 = 10;

#[derive(Clone)]
pub(super) struct TrustedNetworkTarget {
    pub(super) origin: String,
    pub(super) boot_session: String,
    pub(super) boot_ordinal: u64,
    pub(super) expected: ExpectedRuntimeAttestationIdentity,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SerialWindowEvidence {
    marker_count: u64,
    first_poll_request_count: Option<u64>,
    last_poll_request_count: Option<u64>,
}

impl SerialWindowEvidence {
    pub(super) fn observe(&mut self, poll_request_count: u64) {
        self.marker_count = self.marker_count.saturating_add(1);
        self.first_poll_request_count
            .get_or_insert(poll_request_count);
        self.last_poll_request_count = Some(poll_request_count);
    }

    fn work_renewed(self) -> bool {
        self.marker_count >= 2
            && self
                .first_poll_request_count
                .zip(self.last_poll_request_count)
                .is_some_and(|(first, last)| last > first)
    }
}

#[derive(Debug, Clone)]
pub(super) struct SharedSerialState {
    pub(super) latest_active_ms: u64,
    pub(super) active: bool,
    pub(super) resumable_pause_safe_stop_confirmed: bool,
    pub(super) resumable_pause_safe_stop_observation_count: u64,
    pub(super) terminal_consumed: bool,
    pub(super) terminal_pool_persisted: bool,
    pub(super) serial_finished: bool,
    pub(super) network_stop_requested: bool,
    pub(super) maximum_active_marker_gap_ms: u64,
    pub(super) serial_windows: [SerialWindowEvidence; REQUIRED_WINDOWS],
    pub(super) maybe_failure: Option<CampaignTerminalCategory>,
}

impl Default for SharedSerialState {
    fn default() -> Self {
        Self {
            latest_active_ms: 0,
            active: false,
            resumable_pause_safe_stop_confirmed: false,
            resumable_pause_safe_stop_observation_count: 0,
            terminal_consumed: false,
            terminal_pool_persisted: false,
            serial_finished: false,
            network_stop_requested: false,
            maximum_active_marker_gap_ms: 0,
            serial_windows: array::from_fn(|_| SerialWindowEvidence::default()),
            maybe_failure: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct TransportWindowEvidence {
    sample_count: u64,
    first_watchdog_feed_sequence: Option<u64>,
    last_watchdog_feed_sequence: Option<u64>,
    first_checkpoint_sequence: Option<u64>,
    last_checkpoint_sequence: Option<u64>,
}

impl TransportWindowEvidence {
    fn observe(&mut self, sample: &SystemInfoWire) {
        self.sample_count = self.sample_count.saturating_add(1);
        if let Some(sequence) = sample.runtime_health.maybe_task_watchdog_feed_sequence {
            self.first_watchdog_feed_sequence.get_or_insert(sequence);
            self.last_watchdog_feed_sequence = Some(sequence);
        }
        if let Some(sequence) = sample.runtime_health.maybe_checkpoint_sequence {
            self.first_checkpoint_sequence.get_or_insert(sequence);
            self.last_checkpoint_sequence = Some(sequence);
        }
    }

    fn complete(self) -> bool {
        self.sample_count >= 2
    }

    fn watchdog_advanced(self) -> bool {
        advances(
            self.first_watchdog_feed_sequence,
            self.last_watchdog_feed_sequence,
        ) && advances(
            self.first_checkpoint_sequence,
            self.last_checkpoint_sequence,
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ContinuityWindowEvidence {
    http: TransportWindowEvidence,
    websocket: TransportWindowEvidence,
    serial: SerialWindowEvidence,
    complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CampaignNetworkEvidence {
    pub(in crate::campaign) schema: &'static str,
    pub(in crate::campaign) status: &'static str,
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
    pub(in crate::campaign) work_renewal_valid: bool,
    pub(in crate::campaign) terminal_http_valid: bool,
    pub(in crate::campaign) terminal_websocket_valid: bool,
    pub(in crate::campaign) terminal_pool_persisted: bool,
    pub(in crate::campaign) command_effects: Option<CommandEffectsEvidence>,
    #[serde(skip)]
    pub(in crate::campaign) maybe_failure: Option<CampaignTerminalCategory>,
}

impl CampaignNetworkEvidence {
    pub(crate) fn not_required() -> Self {
        Self::empty("not_required", None)
    }

    pub(super) fn from_unobserved(shared: &Arc<Mutex<SharedSerialState>>) -> Self {
        let maybe_failure = shared
            .lock()
            .ok()
            .and_then(|state| state.maybe_failure)
            .or(Some(CampaignTerminalCategory::NetworkTargetUnavailable));
        Self::empty("failed", maybe_failure)
    }

    pub(super) fn worker_failed(shared: &Arc<Mutex<SharedSerialState>>) -> Self {
        let maybe_failure = shared
            .lock()
            .ok()
            .and_then(|state| state.maybe_failure)
            .or(Some(CampaignTerminalCategory::NetworkCorrelationFailed));
        Self::empty("failed", maybe_failure)
    }

    fn empty(status: &'static str, maybe_failure: Option<CampaignTerminalCategory>) -> Self {
        Self {
            schema: "mining-campaign-network-continuity-v3",
            status,
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
            work_renewal_valid: false,
            terminal_http_valid: false,
            terminal_websocket_valid: false,
            terminal_pool_persisted: false,
            command_effects: None,
            maybe_failure,
        }
    }

    pub(super) fn from_command_effects(
        evidence: CommandEffectsEvidence,
        recovery_pause_request_count: u64,
        maybe_failure: Option<CampaignTerminalCategory>,
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
            recovery_pause_request_count,
            same_boot_and_package: evidence.same_boot_and_package,
            active_state_valid: evidence.active_before_pause && evidence.active_after_resume,
            safety_valid: evidence.safety_valid,
            terminal_http_valid: evidence.terminal_http_valid,
            terminal_pool_persisted: evidence.terminal_pool_persisted,
            command_effects: Some(evidence),
            maybe_failure,
            ..Self::empty(status, maybe_failure)
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum NetworkTransport {
    Http,
    WebSocket,
}

pub(super) struct NetworkAccumulator {
    target: TrustedNetworkTarget,
    windows: [ContinuityWindowEvidence; REQUIRED_WINDOWS],
    pub(super) http_success_count: u64,
    pub(super) websocket_frame_count: u64,
    pub(super) websocket_reconnect_count: u64,
    websocket_connect_failure_count: u64,
    websocket_peer_close_count: u64,
    websocket_io_failure_count: u64,
    websocket_protocol_failure_count: u64,
    websocket_capacity_failure_count: u64,
    websocket_other_failure_count: u64,
    pub(super) recovery_pause_request_count: u64,
    pub(super) maximum_http_gap_ms: u64,
    pub(super) maximum_websocket_gap_ms: u64,
    pub(super) terminal_http_valid: bool,
    pub(super) terminal_websocket_valid: bool,
    pub(super) maybe_failure: Option<CampaignTerminalCategory>,
    same_boot_and_package: bool,
    active_state_valid: bool,
    safety_valid: bool,
    watchdog_valid: bool,
    http_startup_transition_count: u64,
    websocket_startup_transition_count: u64,
    http_initial_active_observed: bool,
    websocket_initial_active_observed: bool,
    last_http_at_ms: Option<u64>,
    last_websocket_at_ms: Option<u64>,
    last_http_revision: Option<u64>,
    last_websocket_revision: Option<u64>,
    last_http_shares: Option<(u64, u64)>,
    last_websocket_shares: Option<(u64, u64)>,
}

impl NetworkAccumulator {
    pub(super) fn new(target: TrustedNetworkTarget) -> Self {
        Self {
            target,
            windows: array::from_fn(|_| ContinuityWindowEvidence::default()),
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
            maximum_http_gap_ms: 0,
            maximum_websocket_gap_ms: 0,
            terminal_http_valid: false,
            terminal_websocket_valid: false,
            maybe_failure: None,
            same_boot_and_package: true,
            active_state_valid: true,
            safety_valid: true,
            watchdog_valid: true,
            http_startup_transition_count: 0,
            websocket_startup_transition_count: 0,
            http_initial_active_observed: false,
            websocket_initial_active_observed: false,
            last_http_at_ms: None,
            last_websocket_at_ms: None,
            last_http_revision: None,
            last_websocket_revision: None,
            last_http_shares: None,
            last_websocket_shares: None,
        }
    }

    pub(super) fn record_active_sample(
        &mut self,
        transport: NetworkTransport,
        active_ms: u64,
        observed_at_ms: u64,
        sample: &SystemInfoWire,
    ) {
        if self.maybe_failure.is_some() {
            return;
        }
        if !watchdog_valid(sample) {
            self.watchdog_valid = false;
            self.fail(CampaignTerminalCategory::WatchdogUnresponsive);
            return;
        }
        if let Err(failure) = validate_active_prerequisites(sample, &self.target) {
            self.record_validation_failure(failure, false);
            return;
        }
        if !active_mining_state_valid(sample) {
            if self.initial_active_observed(transport) {
                self.active_state_valid = false;
                self.fail(CampaignTerminalCategory::NetworkCorrelationFailed);
            } else {
                self.record_startup_transition(transport);
            }
            return;
        }
        self.establish_initial_active(transport);
        let index = window_index(active_ms);
        let revision = sample.operator_snapshot_revision.get();
        let shares = (sample.shares_accepted, sample.shares_rejected);
        match transport {
            NetworkTransport::Http => {
                if sequence_regresses(self.last_http_revision, revision)
                    || regresses(self.last_http_shares, shares)
                {
                    self.fail(CampaignTerminalCategory::NetworkCorrelationFailed);
                    return;
                }
                update_gap(
                    &mut self.maximum_http_gap_ms,
                    &mut self.last_http_at_ms,
                    observed_at_ms,
                );
                self.last_http_revision = Some(revision);
                self.last_http_shares = Some(shares);
                self.http_success_count = self.http_success_count.saturating_add(1);
                self.windows[index].http.observe(sample);
            }
            NetworkTransport::WebSocket => {
                if sequence_regresses(self.last_websocket_revision, revision)
                    || regresses(self.last_websocket_shares, shares)
                {
                    self.fail(CampaignTerminalCategory::NetworkCorrelationFailed);
                    return;
                }
                update_gap(
                    &mut self.maximum_websocket_gap_ms,
                    &mut self.last_websocket_at_ms,
                    observed_at_ms,
                );
                self.last_websocket_revision = Some(revision);
                self.last_websocket_shares = Some(shares);
                self.websocket_frame_count = self.websocket_frame_count.saturating_add(1);
                self.windows[index].websocket.observe(sample);
            }
        }
    }

    pub(super) fn record_terminal_sample(
        &mut self,
        transport: NetworkTransport,
        sample: &SystemInfoWire,
    ) {
        if !watchdog_valid(sample) {
            self.watchdog_valid = false;
            self.fail(CampaignTerminalCategory::WatchdogUnresponsive);
            return;
        }
        if let Err(failure) = validate_sample(sample, &self.target, true) {
            self.record_validation_failure(failure, true);
            return;
        }
        match transport {
            NetworkTransport::Http => self.terminal_http_valid = true,
            NetworkTransport::WebSocket => self.terminal_websocket_valid = true,
        }
    }

    pub(super) fn close_elapsed_windows(&mut self, active_ms: u64, serial: &SharedSerialState) {
        let completed = usize::try_from(active_ms / WINDOW_MILLIS)
            .unwrap_or(REQUIRED_WINDOWS)
            .min(REQUIRED_WINDOWS);
        for index in 0..completed {
            self.windows[index].serial = serial.serial_windows[index];
            let http_complete = self.windows[index].http.complete();
            let websocket_complete = self.windows[index].websocket.complete();
            let watchdog_complete = self.windows[index].http.watchdog_advanced()
                && self.windows[index].websocket.watchdog_advanced();
            let work_complete = self.windows[index].serial.work_renewed();
            self.windows[index].complete =
                http_complete && websocket_complete && watchdog_complete && work_complete;
            if !http_complete {
                self.fail(CampaignTerminalCategory::HttpWindowIncomplete);
                return;
            }
            if !websocket_complete {
                self.fail(CampaignTerminalCategory::WebsocketWindowIncomplete);
                return;
            }
            if !watchdog_complete {
                self.watchdog_valid = false;
                self.fail(CampaignTerminalCategory::WatchdogUnresponsive);
                return;
            }
            if !work_complete {
                self.fail(CampaignTerminalCategory::WorkRenewalMissing);
                return;
            }
        }
    }

    pub(super) fn finish(mut self, serial: &SharedSerialState) -> CampaignNetworkEvidence {
        self.close_elapsed_windows(600_000, serial);
        if self.maybe_failure.is_none()
            && serial.maximum_active_marker_gap_ms > MAX_ACTIVE_MARKER_GAP_MILLIS
        {
            self.fail(CampaignTerminalCategory::MarkerContinuityFailed);
        }
        if self.maybe_failure.is_none() && !serial.terminal_pool_persisted {
            self.fail(CampaignTerminalCategory::PoolPersistenceUnconfirmed);
        }
        if self.maybe_failure.is_none()
            && (!self.terminal_http_valid || !self.terminal_websocket_valid)
        {
            self.fail(CampaignTerminalCategory::TerminalStateUnconfirmed);
        }
        let covered_window_count = self.windows.iter().filter(|window| window.complete).count();
        let accepted = self.maybe_failure.is_none()
            && covered_window_count == REQUIRED_WINDOWS
            && serial.terminal_consumed;
        CampaignNetworkEvidence {
            schema: "mining-campaign-network-continuity-v3",
            status: if accepted { "accepted" } else { "failed" },
            required_window_count: REQUIRED_WINDOWS,
            covered_window_count,
            http_success_count: self.http_success_count,
            websocket_frame_count: self.websocket_frame_count,
            websocket_reconnect_count: self.websocket_reconnect_count,
            websocket_connect_failure_count: self.websocket_connect_failure_count,
            websocket_peer_close_count: self.websocket_peer_close_count,
            websocket_io_failure_count: self.websocket_io_failure_count,
            websocket_protocol_failure_count: self.websocket_protocol_failure_count,
            websocket_capacity_failure_count: self.websocket_capacity_failure_count,
            websocket_other_failure_count: self.websocket_other_failure_count,
            recovery_pause_request_count: self.recovery_pause_request_count,
            http_startup_transition_count: self.http_startup_transition_count,
            websocket_startup_transition_count: self.websocket_startup_transition_count,
            http_initial_active_observed: self.http_initial_active_observed,
            websocket_initial_active_observed: self.websocket_initial_active_observed,
            maximum_http_gap_ms: self.maximum_http_gap_ms,
            maximum_websocket_gap_ms: self.maximum_websocket_gap_ms,
            maximum_active_marker_gap_ms: serial.maximum_active_marker_gap_ms,
            same_boot_and_package: self.same_boot_and_package,
            active_state_valid: self.active_state_valid
                && self.http_initial_active_observed
                && self.websocket_initial_active_observed,
            safety_valid: self.safety_valid,
            watchdog_valid: self.watchdog_valid,
            work_renewal_valid: self
                .windows
                .iter()
                .all(|window| window.serial.work_renewed()),
            terminal_http_valid: self.terminal_http_valid,
            terminal_websocket_valid: self.terminal_websocket_valid,
            terminal_pool_persisted: serial.terminal_pool_persisted,
            command_effects: None,
            maybe_failure: self.maybe_failure,
        }
    }

    pub(super) fn fail(&mut self, category: CampaignTerminalCategory) {
        self.maybe_failure.get_or_insert(category);
    }

    pub(super) fn note_websocket_connect_failure(&mut self) {
        self.websocket_connect_failure_count =
            self.websocket_connect_failure_count.saturating_add(1);
    }

    pub(super) fn note_websocket_peer_close(&mut self) {
        self.websocket_peer_close_count = self.websocket_peer_close_count.saturating_add(1);
    }

    pub(super) fn note_websocket_failure(&mut self, kind: WebSocketReadFailureKind) {
        match kind {
            WebSocketReadFailureKind::Io => {
                self.websocket_io_failure_count = self.websocket_io_failure_count.saturating_add(1);
            }
            WebSocketReadFailureKind::Protocol => {
                self.websocket_protocol_failure_count =
                    self.websocket_protocol_failure_count.saturating_add(1);
            }
            WebSocketReadFailureKind::Capacity => {
                self.websocket_capacity_failure_count =
                    self.websocket_capacity_failure_count.saturating_add(1);
            }
            WebSocketReadFailureKind::Other => {
                self.websocket_other_failure_count =
                    self.websocket_other_failure_count.saturating_add(1);
            }
        }
    }

    pub(super) fn take_recovery_pause_request(&mut self) -> bool {
        if self.maybe_failure.is_none() || self.recovery_pause_request_count > 0 {
            return false;
        }
        self.recovery_pause_request_count = 1;
        true
    }

    fn record_validation_failure(&mut self, failure: SampleValidationFailure, terminal: bool) {
        match failure {
            SampleValidationFailure::Identity => self.same_boot_and_package = false,
            SampleValidationFailure::MiningState if !terminal => self.active_state_valid = false,
            SampleValidationFailure::MiningState => {}
            SampleValidationFailure::Safety => self.safety_valid = false,
        }
        self.fail(if terminal {
            CampaignTerminalCategory::TerminalStateUnconfirmed
        } else {
            CampaignTerminalCategory::NetworkCorrelationFailed
        });
    }

    fn initial_active_observed(&self, transport: NetworkTransport) -> bool {
        match transport {
            NetworkTransport::Http => self.http_initial_active_observed,
            NetworkTransport::WebSocket => self.websocket_initial_active_observed,
        }
    }

    fn record_startup_transition(&mut self, transport: NetworkTransport) {
        match transport {
            NetworkTransport::Http => {
                self.http_startup_transition_count =
                    self.http_startup_transition_count.saturating_add(1);
            }
            NetworkTransport::WebSocket => {
                self.websocket_startup_transition_count =
                    self.websocket_startup_transition_count.saturating_add(1);
            }
        }
    }

    fn establish_initial_active(&mut self, transport: NetworkTransport) {
        match transport {
            NetworkTransport::Http => self.http_initial_active_observed = true,
            NetworkTransport::WebSocket => self.websocket_initial_active_observed = true,
        }
    }
}

fn sequence_regresses(maybe_previous: Option<u64>, current: u64) -> bool {
    maybe_previous.is_some_and(|previous| current < previous)
}
