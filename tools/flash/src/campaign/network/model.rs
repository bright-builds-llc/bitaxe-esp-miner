use std::array;
use std::sync::{Arc, Mutex};

use bitaxe_api::{ExpectedRuntimeAttestationIdentity, SystemInfoWire};
use bitaxe_http_transport::WebSocketReadFailureKind;

use super::super::*;
use super::command_evidence::CommandEffectsEvidence;
use super::command_witness::CommandTransitionWitness;
use super::hashrate::{CampaignHashrateEvidence, HashrateObservationPair};
use super::terminal_settlement::TerminalSettlementDecision;
use super::validation::{
    active_mining_state_valid, regresses, update_gap, validate_active_prerequisites,
    validate_sample, window_index, SampleValidationFailure,
};
use super::watchdog::{
    sample_diagnostic, WatchdogDiagnostic, WatchdogFailure, WatchdogOwnerPhase,
    WatchdogOwnerSubphase, WatchdogReadOutcome, WatchdogWaitState,
};
use super::window::{ContinuityWindowEvidence, SerialWindowEvidence};

mod command_failure;
mod evidence;
pub(in crate::campaign) use command_failure::{
    CommandFailureCause, CommandFailureDiagnostic, CommandFailurePhase,
};
pub(crate) use evidence::CampaignNetworkEvidence;

pub(super) const REQUIRED_WINDOWS: usize = 20;
pub(super) const WINDOW_MILLIS: u64 = 30_000;
pub(super) const EVIDENCE_DURATION_MILLIS: u64 = REQUIRED_WINDOWS as u64 * WINDOW_MILLIS;
pub(super) const MAX_ACTIVE_MARKER_GAP_MILLIS: u64 = 5_000;
pub(super) const TERMINAL_NETWORK_DEADLINE_SECONDS: u64 = 10;
#[derive(Clone)]
pub(super) struct TrustedNetworkTarget {
    pub(super) origin: String,
    pub(super) boot_session: String,
    pub(super) boot_ordinal: u64,
    pub(super) expected: ExpectedRuntimeAttestationIdentity,
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
    pub(super) command_transitions: CommandTransitionWitness,
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
            command_transitions: CommandTransitionWitness::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum NetworkTransport {
    Http,
    WebSocket,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NetworkCorrelationFailure {
    None,
    HttpProjectionInvalid,
    WebsocketProjectionInvalid,
    HttpRevisionRegressed,
    WebsocketRevisionRegressed,
    HttpShareCounterRegressed,
    WebsocketShareCounterRegressed,
    ActiveStateLost,
    IdentityInvalid,
    MiningStateInvalid,
    SafetyInvalid,
}

impl NetworkCorrelationFailure {
    const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::HttpProjectionInvalid => "http_projection_invalid",
            Self::WebsocketProjectionInvalid => "websocket_projection_invalid",
            Self::HttpRevisionRegressed => "http_revision_regressed",
            Self::WebsocketRevisionRegressed => "websocket_revision_regressed",
            Self::HttpShareCounterRegressed => "http_share_counter_regressed",
            Self::WebsocketShareCounterRegressed => "websocket_share_counter_regressed",
            Self::ActiveStateLost => "active_state_lost",
            Self::IdentityInvalid => "identity_invalid",
            Self::MiningStateInvalid => "mining_state_invalid",
            Self::SafetyInvalid => "safety_invalid",
        }
    }
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
    pub(super) terminal_settlement: &'static str,
    pub(super) terminal_close_requested: bool,
    pub(super) terminal_consumed_observed: bool,
    pub(super) maybe_failure: Option<CampaignTerminalCategory>,
    correlation_failure: NetworkCorrelationFailure,
    same_boot_and_package: bool,
    active_state_valid: bool,
    safety_valid: bool,
    watchdog_valid: bool,
    pub(super) watchdog_failure: WatchdogFailure,
    watchdog_read_outcome: WatchdogReadOutcome,
    watchdog_owner_phase: WatchdogOwnerPhase,
    watchdog_owner_subphase: WatchdogOwnerSubphase,
    watchdog_wait_state: WatchdogWaitState,
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
    hashrate: HashrateObservationPair,
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
            terminal_settlement: TerminalSettlementDecision::Continue.label(),
            terminal_close_requested: false,
            terminal_consumed_observed: false,
            maybe_failure: None,
            correlation_failure: NetworkCorrelationFailure::None,
            same_boot_and_package: true,
            active_state_valid: true,
            safety_valid: true,
            watchdog_valid: true,
            watchdog_failure: WatchdogFailure::None,
            watchdog_read_outcome: WatchdogReadOutcome::Uninitialized,
            watchdog_owner_phase: WatchdogOwnerPhase::Unavailable,
            watchdog_owner_subphase: WatchdogOwnerSubphase::Unavailable,
            watchdog_wait_state: WatchdogWaitState::NotWaiting,
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
            hashrate: HashrateObservationPair::default(),
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
        if active_ms >= EVIDENCE_DURATION_MILLIS && !active_mining_state_valid(sample) {
            self.record_terminal_sample(transport, sample);
            return;
        }
        if !self.observe_watchdog(sample) {
            return;
        }
        if let Err(failure) = validate_active_prerequisites(sample, &self.target) {
            self.record_validation_failure(failure, false);
            return;
        }
        if !active_mining_state_valid(sample) {
            if self.initial_active_observed(transport) {
                self.active_state_valid = false;
                self.fail_correlation(NetworkCorrelationFailure::ActiveStateLost);
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
                if sequence_regresses(self.last_http_revision, revision) {
                    self.fail_correlation(NetworkCorrelationFailure::HttpRevisionRegressed);
                    return;
                }
                if regresses(self.last_http_shares, shares) {
                    self.fail_correlation(NetworkCorrelationFailure::HttpShareCounterRegressed);
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
                self.hashrate.http.observe_active(active_ms, sample);
            }
            NetworkTransport::WebSocket => {
                if sequence_regresses(self.last_websocket_revision, revision) {
                    self.fail_correlation(NetworkCorrelationFailure::WebsocketRevisionRegressed);
                    return;
                }
                if regresses(self.last_websocket_shares, shares) {
                    self.fail_correlation(
                        NetworkCorrelationFailure::WebsocketShareCounterRegressed,
                    );
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
                self.hashrate.websocket.observe_active(active_ms, sample);
            }
        }
    }

    pub(super) fn record_terminal_sample(
        &mut self,
        transport: NetworkTransport,
        sample: &SystemInfoWire,
    ) {
        if !self.observe_watchdog(sample) {
            return;
        }
        if let Err(failure) = validate_sample(sample, &self.target, true) {
            self.record_validation_failure(failure, true);
            return;
        }
        self.hashrate
            .observe_terminal(matches!(transport, NetworkTransport::Http), sample);
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
            let watchdog_failure = self.windows[index].watchdog_failure();
            let watchdog_complete = watchdog_failure == WatchdogFailure::None;
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
                self.fail_watchdog(WatchdogDiagnostic {
                    read_outcome: self.watchdog_read_outcome,
                    owner_phase: self.watchdog_owner_phase,
                    owner_subphase: self.watchdog_owner_subphase,
                    wait_state: self.watchdog_wait_state,
                    failure: watchdog_failure,
                });
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
            && serial.terminal_consumed
            && serial.serial_finished
            && self.terminal_settlement
                == TerminalSettlementDecision::AcceptAfterSerialClose.label();
        CampaignNetworkEvidence {
            schema: "mining-campaign-network-continuity-v12",
            status: if accepted { "accepted" } else { "failed" },
            correlation_failure: self.correlation_failure.label(),
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
            watchdog_failure: self.watchdog_failure.label(),
            watchdog_read_outcome: self.watchdog_read_outcome.label(),
            watchdog_owner_phase: self.watchdog_owner_phase.label(),
            watchdog_owner_subphase: self.watchdog_owner_subphase.label(),
            watchdog_wait_state: self.watchdog_wait_state.label(),
            work_renewal_valid: self
                .windows
                .iter()
                .all(|window| window.serial.work_renewed()),
            terminal_http_valid: self.terminal_http_valid,
            terminal_websocket_valid: self.terminal_websocket_valid,
            terminal_pool_persisted: serial.terminal_pool_persisted,
            terminal_settlement: self.terminal_settlement,
            terminal_close_requested: self.terminal_close_requested,
            terminal_consumed_observed: self.terminal_consumed_observed,
            final_terminal_consumed: serial.terminal_consumed,
            serial_finished_observed: serial.serial_finished,
            hashrate_monitor: self.hashrate.evidence(),
            command_effects: None,
            command_failure: None,
            maybe_failure: self.maybe_failure,
        }
    }

    pub(super) fn fail(&mut self, category: CampaignTerminalCategory) {
        self.maybe_failure.get_or_insert(category);
    }

    pub(super) fn note_terminal_settlement(&mut self, decision: TerminalSettlementDecision) {
        self.terminal_settlement = decision.label();
        self.terminal_close_requested |=
            matches!(decision, TerminalSettlementDecision::RequestSerialClose);
    }

    pub(super) fn fail_correlation(&mut self, failure: NetworkCorrelationFailure) {
        if self.maybe_failure.is_none() {
            self.correlation_failure = failure;
        }
        self.fail(CampaignTerminalCategory::NetworkCorrelationFailed);
    }

    fn observe_watchdog(&mut self, sample: &SystemInfoWire) -> bool {
        let diagnostic = sample_diagnostic(sample);
        if diagnostic.failure != WatchdogFailure::None {
            self.fail_watchdog(diagnostic);
            return false;
        }
        if self.watchdog_failure == WatchdogFailure::None {
            self.watchdog_read_outcome = diagnostic.read_outcome;
            self.watchdog_owner_phase = diagnostic.owner_phase;
            self.watchdog_owner_subphase = diagnostic.owner_subphase;
            self.watchdog_wait_state = diagnostic.wait_state;
        }
        true
    }

    fn fail_watchdog(&mut self, diagnostic: WatchdogDiagnostic) {
        if self.maybe_failure.is_none() {
            self.maybe_failure = Some(CampaignTerminalCategory::WatchdogUnresponsive);
        }
        if self.maybe_failure == Some(CampaignTerminalCategory::WatchdogUnresponsive) {
            self.watchdog_valid = false;
            if self.watchdog_failure == WatchdogFailure::None {
                self.watchdog_failure = diagnostic.failure;
                self.watchdog_read_outcome = diagnostic.read_outcome;
                self.watchdog_owner_phase = diagnostic.owner_phase;
                self.watchdog_owner_subphase = diagnostic.owner_subphase;
                self.watchdog_wait_state = diagnostic.wait_state;
            }
        }
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
            SampleValidationFailure::Identity => {
                self.same_boot_and_package = false;
                self.correlation_failure = NetworkCorrelationFailure::IdentityInvalid;
            }
            SampleValidationFailure::MiningState if !terminal => {
                self.active_state_valid = false;
                self.correlation_failure = NetworkCorrelationFailure::MiningStateInvalid;
            }
            SampleValidationFailure::MiningState => {}
            SampleValidationFailure::Safety => {
                self.safety_valid = false;
                self.correlation_failure = NetworkCorrelationFailure::SafetyInvalid;
            }
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
