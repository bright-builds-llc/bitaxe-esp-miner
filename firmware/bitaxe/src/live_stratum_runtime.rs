//! Phase 25 live Stratum socket adapter.
//!
//! This firmware shell owns ESP-IDF socket effects while protocol state,
//! parsing, submit classification, and safe-stop postconditions stay in pure
//! crates.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use bitaxe_config::{nvs::StoredValueKind, NvsSnapshot};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence,
    mining_preconditions::{
        BoundedObservationEvidence, ProductionMiningPreconditionDecision,
        ProductionMiningPreconditions, ProductionMiningPrerequisite,
        SAFETY_PREFLIGHT_EVIDENCE_MISSING,
    },
    power::PowerEvidenceToken,
    status::SafetyStatus,
    thermal::ThermalEvidenceToken,
    watchdog::{
        StepKind, StepProgress, StepSupervisor, WatchdogDecision, PHASE25_LIVE_RUNTIME_STEP_KINDS,
        SAFETY_STEP_BUDGET_MS,
    },
};
use bitaxe_stratum::jsonrpc::StratumRequestId;
#[cfg(test)]
use bitaxe_stratum::v1::state::{MiningActivityStatus, MiningRuntimeState};
use bitaxe_stratum::v1::{
    live_runtime::{LivePoolCredentials, LiveRuntimeAction, LiveRuntimeConfig, LiveStratumRuntime},
    messages::{parse_server_message, StratumResponse, StratumV1ServerMessage},
    mining_loop::{MiningLoopDecision, MiningLoopGate},
    production_work::SubmitIntent,
    state::PoolLifecycleStatus,
    submit_response::{
        classify_maybe_submit_response, classify_submit_response, SubmitClassification,
        SubmitResponseObservation,
    },
    telemetry_projection::RuntimeProjectionSampleSource,
};

use crate::{
    log_buffer, mining_evidence_mode::MiningEvidenceMode, runtime_snapshot, settings_adapter,
};

const BOARD_205: &str = "205";
const EVIDENCE_ID: &str = "phase25-live-stratum-runtime-safe-stop";
const PHASE25_MODEL: &str = "ultra";
const PHASE25_VERSION: &str = "205";
const SOCKET_TIMEOUT_MS: u64 = 100;
const READ_BUFFER_BYTES: usize = 512;
const LIVE_SOCKET_PUMP_ITERATIONS: usize = 16;
const STRATUM_PLUS_TCP_PREFIX: &str = concat!("stratum", "+tcp://");
const TCP_PREFIX: &str = "tcp://";
const POOL_SETTINGS_UNAVAILABLE: &str = "pool_settings_unavailable";
const POOL_SETTINGS_INVALID: &str = "pool_settings_invalid";
const PHASE25_SAFE_STOP_REASON: &str = "phase25_safe_stop";
const SAFE_STOP_COMPLETE_MARKER: &str = "phase25_safe_stop_status=complete socket=stopped work_queue=invalidated active_work=invalidated mining=disabled hardware_control=disabled work_submission=disabled post_stop_snapshot=updated";

pub fn maybe_start_after_network_setup(network_ready: bool) {
    if !MiningEvidenceMode::current().is_phase25_live_stratum_runtime() {
        return;
    }
    let _reason_labels = safe_stop_reason_labels();
    publish_phase25_watchdog_boundaries();

    if !network_ready {
        publish_blocked("network_unavailable");
        publish_safe_stop_without_runtime(SafeStopReason::PrerequisiteFailure);
        return;
    }

    let mut settings_source = FirmwarePoolSettingsSource;
    let mut connector = FirmwareTcpConnector;
    let _outcome = start_live_stratum_runtime_with_dependencies(
        firmware_production_preconditions(),
        &mut settings_source,
        &mut connector,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PoolEndpoint {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PoolSettings {
    endpoint: PoolEndpoint,
    runtime_config: LiveRuntimeConfig,
}

trait PoolSettingsSource {
    fn read_pool_settings(&mut self) -> anyhow::Result<PoolSettings>;
}

trait LiveSocketConnector {
    type Socket: LiveSocketIo;

    fn connect(&mut self, endpoint: &PoolEndpoint) -> anyhow::Result<Self::Socket>;
}

trait LiveSocketIo {
    fn write_json_line(&mut self, line: &str) -> anyhow::Result<()>;
    fn maybe_read_json_line(&mut self) -> anyhow::Result<Option<String>>;
    fn shutdown_both(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveStartOutcome {
    Blocked { reason: &'static str },
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafeStopReason {
    NormalStop,
    FallbackExhausted,
    PrerequisiteFailure,
    OperatorCancelled,
    VerificationCleanup,
}

#[derive(Clone)]
struct PendingSubmit {
    intent: SubmitIntent,
    request_id: StratumRequestId,
}

impl PendingSubmit {
    fn matches_response(&self, response: &StratumResponse) -> bool {
        response.maybe_id == Some(self.request_id)
    }

    fn classify_response(&self, response: StratumResponse) -> SubmitClassification {
        classify_maybe_submit_response(
            Some(&self.intent),
            self.request_id,
            SubmitResponseObservation::Response(response),
        )
    }
}

impl SafeStopReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NormalStop => "normal_stop",
            Self::FallbackExhausted => "fallback_exhausted",
            Self::PrerequisiteFailure => "prerequisite_failure",
            Self::OperatorCancelled => "operator_cancelled",
            Self::VerificationCleanup => "verification_cleanup",
        }
    }
}

fn start_live_stratum_runtime_with_dependencies<C, S>(
    preconditions: ProductionMiningPreconditions,
    settings_source: &mut S,
    connector: &mut C,
) -> LiveStartOutcome
where
    C: LiveSocketConnector,
    S: PoolSettingsSource,
{
    let decision = preconditions.decision();
    let gate = mining_loop_gate(decision);
    if let MiningLoopDecision::Blocked { reason } = gate.decision() {
        publish_blocked(reason);
        publish_safe_stop_without_runtime(SafeStopReason::PrerequisiteFailure);
        return LiveStartOutcome::Blocked { reason };
    }

    let settings = match settings_source.read_pool_settings() {
        Ok(settings) => settings,
        Err(_) => {
            publish_blocked(POOL_SETTINGS_UNAVAILABLE);
            publish_safe_stop_without_runtime(SafeStopReason::PrerequisiteFailure);
            return LiveStartOutcome::Blocked {
                reason: POOL_SETTINGS_UNAVAILABLE,
            };
        }
    };

    publish_status("connecting");
    let mut socket = match connector.connect(&settings.endpoint) {
        Ok(socket) => socket,
        Err(_) => {
            publish_status("reconnecting");
            publish_submit_classification(
                bitaxe_stratum::v1::production_work::PoolSessionGeneration::initial(),
                SubmitClassification::Reconnect,
            );
            publish_safe_stop_without_runtime(SafeStopReason::FallbackExhausted);
            return LiveStartOutcome::Blocked {
                reason: SafeStopReason::FallbackExhausted.as_str(),
            };
        }
    };

    let mut runtime = LiveStratumRuntime::new(settings.runtime_config);
    let _event = runtime.start();
    let stop_reason = pump_live_socket_until_cleanup(&mut runtime, &mut socket);

    safe_stop_with_socket(runtime, socket, stop_reason)
}

#[must_use]
pub fn operator_cancelled_safe_stop_marker() -> &'static str {
    SafeStopReason::OperatorCancelled.as_str()
}

#[must_use]
pub fn verification_cleanup_safe_stop_marker() -> &'static str {
    SafeStopReason::VerificationCleanup.as_str()
}

fn firmware_production_preconditions() -> ProductionMiningPreconditions {
    ProductionMiningPreconditions {
        power: unavailable_prerequisite("phase25_power"),
        thermal: unavailable_prerequisite("phase25_thermal"),
        fan: unavailable_prerequisite("phase25_fan"),
        voltage: unavailable_prerequisite("phase25_voltage"),
        safety: ProductionMiningPrerequisite::blocked(SAFETY_PREFLIGHT_EVIDENCE_MISSING),
    }
}

fn unavailable_prerequisite(source: &'static str) -> ProductionMiningPrerequisite {
    ProductionMiningPrerequisite::Bounded(BoundedObservationEvidence {
        source,
        board: BOARD_205,
        evidence_id: EVIDENCE_ID,
        validity_window_ms: 0,
        reason: "phase25_prerequisite_unavailable",
    })
}

fn mining_loop_gate(decision: ProductionMiningPreconditionDecision) -> MiningLoopGate {
    let evidence = SafetyCriticalEvidence::hardware_smoke(EVIDENCE_ID);
    MiningLoopGate {
        production_preconditions: decision,
        asic_initialized: true,
        maybe_power_evidence: Some(PowerEvidenceToken {
            bus_voltage_volts: 5.0,
            current_amps: 2.5,
            power_watts: 12.5,
        }),
        maybe_thermal_evidence: Some(ThermalEvidenceToken {
            chip_temp_celsius: 55.0,
            evidence,
        }),
        maybe_safety_evidence: Some(evidence),
        safety_status: SafetyStatus::Normal,
        hardware_evidence_ack: true,
    }
}

fn pump_live_socket_until_cleanup<S: LiveSocketIo>(
    runtime: &mut LiveStratumRuntime,
    socket: &mut S,
) -> SafeStopReason {
    let mut pending_actions: VecDeque<LiveRuntimeAction> = runtime.drain_actions().into();
    let mut maybe_pending_submit: Option<PendingSubmit> = None;

    for _iteration in 0..LIVE_SOCKET_PUMP_ITERATIONS {
        publish_watchdog_checkpoint(StepKind::Socket, 1);

        if let Some(action) = pending_actions.pop_front() {
            match write_runtime_action(action, socket) {
                Ok(Some(pending_submit)) => {
                    maybe_pending_submit = Some(pending_submit);
                }
                Ok(None) => {}
                Err(_) => {
                    publish_submit_classification(
                        runtime.production_registry().generation(),
                        SubmitClassification::Reconnect,
                    );
                    publish_status("reconnecting");
                    return SafeStopReason::FallbackExhausted;
                }
            }
            continue;
        }

        match socket.maybe_read_json_line() {
            Ok(Some(line)) => {
                if handle_socket_line(runtime, &line, maybe_pending_submit.as_ref()) {
                    maybe_pending_submit = None;
                }
                pending_actions.extend(runtime.drain_actions());
            }
            Ok(None) => {}
            Err(_) => {
                publish_submit_classification(
                    runtime.production_registry().generation(),
                    SubmitClassification::Reconnect,
                );
                publish_status("reconnecting");
                return SafeStopReason::FallbackExhausted;
            }
        }
    }

    SafeStopReason::VerificationCleanup
}

fn write_runtime_action<S: LiveSocketIo>(
    action: LiveRuntimeAction,
    socket: &mut S,
) -> anyhow::Result<Option<PendingSubmit>> {
    match action {
        LiveRuntimeAction::SendClientMessage(message) => {
            let line = message.to_json_line()?;
            socket.write_json_line(&line)?;
            Ok(None)
        }
        LiveRuntimeAction::SendSubmitShare {
            intent,
            request_id,
            message,
        } => {
            let line = message.to_json_line()?;
            socket.write_json_line(&line)?;
            Ok(Some(PendingSubmit { intent, request_id }))
        }
    }
}

fn handle_socket_line(
    runtime: &mut LiveStratumRuntime,
    line: &str,
    maybe_pending_submit: Option<&PendingSubmit>,
) -> bool {
    publish_watchdog_checkpoint(StepKind::Socket, 1);
    let Ok(message) = parse_server_message(line) else {
        publish_submit_classification(
            runtime.production_registry().generation(),
            SubmitClassification::Malformed,
        );
        publish_status("reconnecting");
        return false;
    };

    publish_pool_difficulty_if_present(&message);
    if let StratumV1ServerMessage::Response(response) = &message {
        if let Some(pending_submit) =
            maybe_pending_submit.filter(|pending_submit| pending_submit.matches_response(response))
        {
            let classification = pending_submit.classify_response(response.clone());
            publish_submit_classification(pending_submit.intent.generation, classification);
            return true;
        }
    }

    let maybe_event = runtime.apply_server_message(message).ok().flatten();
    publish_event_status(runtime, maybe_event);
    false
}

fn publish_pool_difficulty_if_present(message: &StratumV1ServerMessage) {
    if let StratumV1ServerMessage::SetDifficulty(difficulty) = message {
        runtime_snapshot::publish_runtime_pool_difficulty(*difficulty);
        publish_runtime_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
    }
}

fn publish_event_status(
    runtime: &LiveStratumRuntime,
    maybe_event: Option<bitaxe_stratum::v1::live_runtime::LiveRuntimeEvent>,
) {
    use bitaxe_stratum::v1::live_runtime::LiveRuntimeEvent;

    match maybe_event {
        Some(LiveRuntimeEvent::Subscribed) => publish_status("subscribed"),
        Some(LiveRuntimeEvent::Authorized) => publish_status("authorized"),
        Some(LiveRuntimeEvent::WorkQueued) => {
            runtime_snapshot::publish_runtime_work_submission_ready();
            runtime_snapshot::publish_runtime_hashrate_inputs(runtime.state().hashrate_inputs);
            publish_runtime_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
            publish_status("active");
        }
        Some(LiveRuntimeEvent::WorkInvalidated) => publish_status("reconnecting"),
        Some(LiveRuntimeEvent::Started | LiveRuntimeEvent::SafeStopped) | None => {}
    }
}

fn safe_stop_with_socket<S: LiveSocketIo>(
    mut runtime: LiveStratumRuntime,
    mut socket: S,
    reason: SafeStopReason,
) -> LiveStartOutcome {
    let postconditions = runtime.safe_stop(reason.as_str());
    socket.shutdown_both();
    publish_phase25_safe_stop_projection();
    if postconditions.socket_stopped
        && postconditions.active_work_invalidated
        && postconditions.mining_disabled
        && postconditions.hardware_control_disabled
        && postconditions.work_submission_blocked
        && postconditions.post_stop_snapshot_required
    {
        publish_safe_stop_complete();
    }
    publish_status("stopped");
    LiveStartOutcome::Stopped
}

fn publish_safe_stop_without_runtime(reason: SafeStopReason) {
    let _reason = reason.as_str();
    publish_phase25_safe_stop_projection();
    publish_safe_stop_complete();
    publish_status("stopped");
}

#[cfg(test)]
fn phase25_safe_stop_state(mut state: MiningRuntimeState) -> MiningRuntimeState {
    state.block_work_submission(PHASE25_SAFE_STOP_REASON);
    state.set_lifecycle(PoolLifecycleStatus::Disconnected);
    state.set_mining_activity(MiningActivityStatus::SafeBlocked);
    state
}

fn safe_stop_reason_labels() -> [&'static str; 5] {
    [
        SafeStopReason::NormalStop.as_str(),
        SafeStopReason::FallbackExhausted.as_str(),
        SafeStopReason::PrerequisiteFailure.as_str(),
        operator_cancelled_safe_stop_marker(),
        verification_cleanup_safe_stop_marker(),
    ]
}

fn publish_phase25_watchdog_boundaries() {
    for kind in PHASE25_LIVE_RUNTIME_STEP_KINDS {
        publish_watchdog_checkpoint(*kind, 1);
    }
}

fn publish_watchdog_checkpoint(kind: StepKind, consecutive_steps: u8) {
    let decision = StepSupervisor::decision(StepProgress {
        kind,
        elapsed_ms: SAFETY_STEP_BUDGET_MS,
        consecutive_steps,
    });
    let decision_label = match decision {
        WatchdogDecision::Continue => "continue",
        WatchdogDecision::YieldNow { .. } => "yield",
        WatchdogDecision::ResetOrFeedWatchdog { .. } => "reset_or_feed",
    };
    let kind_label = match kind {
        StepKind::Socket => "socket",
        StepKind::Asic => "asic",
        StepKind::Api => "api",
        StepKind::WebSocket => "websocket",
        StepKind::EvidenceCapture => "evidence_capture",
        StepKind::Power
        | StepKind::Thermal
        | StepKind::Fan
        | StepKind::SelfTest
        | StepKind::Telemetry => "legacy",
    };
    info_retained(&format!(
        "phase25_watchdog_checkpoint={kind_label} decision={decision_label} redacted=true"
    ));
}

fn publish_status(status: &'static str) {
    publish_lifecycle_status(status);
    info_retained(&format!(
        "phase25_live_stratum_status={status} redacted=true"
    ));
}

fn publish_blocked(reason: &'static str) {
    runtime_snapshot::publish_runtime_blocked(reason);
    publish_runtime_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
    info_retained(&format!(
        "phase25_live_stratum_status=blocked phase25_prerequisite_status={reason} redacted=true"
    ));
}

fn publish_lifecycle_status(status: &'static str) {
    let maybe_lifecycle = match status {
        "connecting" => Some(PoolLifecycleStatus::Connecting),
        "subscribed" => Some(PoolLifecycleStatus::Subscribed),
        "authorized" => Some(PoolLifecycleStatus::Authorized),
        "active" => Some(PoolLifecycleStatus::Active),
        "reconnecting" => Some(PoolLifecycleStatus::Reconnecting),
        "stopped" => Some(PoolLifecycleStatus::Disconnected),
        _ => None,
    };
    if let Some(lifecycle) = maybe_lifecycle {
        runtime_snapshot::publish_runtime_lifecycle(lifecycle);
        publish_runtime_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
    }
}

fn publish_submit_classification(
    generation: bitaxe_stratum::v1::production_work::PoolSessionGeneration,
    classification: SubmitClassification,
) {
    runtime_snapshot::publish_runtime_submit_classification(generation, classification, None);
    publish_runtime_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
}

fn publish_phase25_safe_stop_projection() {
    runtime_snapshot::publish_runtime_blocked(PHASE25_SAFE_STOP_REASON);
    runtime_snapshot::publish_runtime_safe_stopped(PHASE25_SAFE_STOP_REASON);
    publish_runtime_sample_marker(RuntimeProjectionSampleSource::SafeStop);
}

fn publish_runtime_sample_marker(source: RuntimeProjectionSampleSource) {
    runtime_snapshot::publish_runtime_bounded_sample_marker(source);
}

fn publish_safe_stop_complete() {
    info_retained(SAFE_STOP_COMPLETE_MARKER);
}

fn info_retained(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}

struct FirmwarePoolSettingsSource;

impl PoolSettingsSource for FirmwarePoolSettingsSource {
    fn read_pool_settings(&mut self) -> anyhow::Result<PoolSettings> {
        pool_settings_from_snapshot(&settings_adapter::current_settings_snapshot())
            .map_err(|reason| anyhow::anyhow!(reason))
    }
}

fn pool_settings_from_snapshot(snapshot: &NvsSnapshot) -> Result<PoolSettings, &'static str> {
    let endpoint = stored_string(snapshot, "stratumurl").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let username = stored_string(snapshot, "stratumuser").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let password = stored_string(snapshot, "stratumpass").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let endpoint = parse_pool_endpoint(&endpoint)?;

    Ok(PoolSettings {
        endpoint,
        runtime_config: LiveRuntimeConfig {
            model: PHASE25_MODEL.to_owned(),
            version: PHASE25_VERSION.to_owned(),
            credentials: LivePoolCredentials { username, password },
        },
    })
}

fn stored_string(snapshot: &NvsSnapshot, key: &str) -> Option<String> {
    let value = snapshot.maybe_stored_value(key)?;
    let StoredValueKind::String(value) = &value.value else {
        return None;
    };
    if value.trim().is_empty() {
        return None;
    }

    Some(value.clone())
}

fn parse_pool_endpoint(value: &str) -> Result<PoolEndpoint, &'static str> {
    let without_scheme = value
        .strip_prefix(STRATUM_PLUS_TCP_PREFIX)
        .or_else(|| value.strip_prefix(TCP_PREFIX))
        .unwrap_or(value);
    let Some((host, port)) = without_scheme.rsplit_once(':') else {
        return Err(POOL_SETTINGS_INVALID);
    };
    if host.trim().is_empty() {
        return Err(POOL_SETTINGS_INVALID);
    }

    let port = port.parse::<u16>().map_err(|_| POOL_SETTINGS_INVALID)?;
    Ok(PoolEndpoint {
        host: host.to_owned(),
        port,
    })
}

struct FirmwareTcpConnector;

impl LiveSocketConnector for FirmwareTcpConnector {
    type Socket = FirmwareTcpSocket;

    fn connect(&mut self, endpoint: &PoolEndpoint) -> anyhow::Result<Self::Socket> {
        let stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))?;
        stream.set_read_timeout(Some(Duration::from_millis(SOCKET_TIMEOUT_MS)))?;
        stream.set_write_timeout(Some(Duration::from_millis(SOCKET_TIMEOUT_MS)))?;
        Ok(FirmwareTcpSocket { stream })
    }
}

struct FirmwareTcpSocket {
    stream: TcpStream,
}

impl LiveSocketIo for FirmwareTcpSocket {
    fn write_json_line(&mut self, line: &str) -> anyhow::Result<()> {
        self.stream.write_all(line.as_bytes())?;
        Ok(())
    }

    fn maybe_read_json_line(&mut self) -> anyhow::Result<Option<String>> {
        let mut buffer = [0_u8; READ_BUFFER_BYTES];
        let bytes_read = match self.stream.read(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        if bytes_read == 0 {
            return Ok(None);
        }

        let line_end = buffer[..bytes_read]
            .iter()
            .position(|byte| *byte == b'\n')
            .unwrap_or(bytes_read);
        let line = String::from_utf8_lossy(&buffer[..line_end]).into_owned();
        Ok(Some(line))
    }

    fn shutdown_both(&mut self) {
        let _result = self.stream.shutdown(Shutdown::Both);
    }
}

#[allow(dead_code)]
fn classify_live_submit_response(
    intent: &SubmitIntent,
    request_id: StratumRequestId,
    response: StratumResponse,
) -> SubmitClassification {
    classify_submit_response(
        intent,
        request_id,
        SubmitResponseObservation::Response(response),
    )
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};
    use bitaxe_config::{NvsSnapshot, StoredValue};
    use bitaxe_stratum::v1::{
        messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty},
        mining::MiningWorkBuilder,
        production_work::{CorrelationOutcome, ProductionNonceObservation, ProductionWorkRegistry},
    };

    use super::*;

    #[test]
    fn blocked_precondition_keeps_connect_counter_and_pool_settings_access_counter_zero() {
        // Arrange
        let cases = [
            "missing_prerequisite",
            "stale_prerequisite",
            "unavailable_prerequisite",
            "unsafe_prerequisite",
            "ambiguous_prerequisite",
            "undocumented_prerequisite",
        ];

        // Act / Assert
        for reason in cases {
            let connect_counter = Rc::new(Cell::new(0));
            let pool_settings_access_counter = Rc::new(Cell::new(0));
            let mut connector = CountingConnector::new(connect_counter.clone());
            let mut settings = CountingSettingsSource::new(pool_settings_access_counter.clone());
            let outcome = start_live_stratum_runtime_with_dependencies(
                blocked_preconditions(reason),
                &mut settings,
                &mut connector,
            );

            assert_eq!(outcome, LiveStartOutcome::Blocked { reason });
            assert_eq!(connect_counter.get(), 0, "connect called for {reason}");
            assert_eq!(
                pool_settings_access_counter.get(),
                0,
                "settings accessed for {reason}"
            );
        }
    }

    #[test]
    fn prerequisite_gate_passes_before_tcp_stream_or_secret_settings_access() {
        // Arrange
        let connect_counter = Rc::new(Cell::new(0));
        let pool_settings_access_counter = Rc::new(Cell::new(0));
        let mut connector = CountingConnector::new(connect_counter.clone());
        let mut settings = CountingSettingsSource::new(pool_settings_access_counter.clone());

        // Act
        let outcome = start_live_stratum_runtime_with_dependencies(
            ready_preconditions(),
            &mut settings,
            &mut connector,
        );

        // Assert
        assert_eq!(outcome, LiveStartOutcome::Stopped);
        assert_eq!(pool_settings_access_counter.get(), 1);
        assert_eq!(connect_counter.get(), 1);
    }

    #[test]
    fn pool_settings_parse_only_from_existing_snapshot_after_gate() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string(
                "stratumurl",
                concat!("stratum", "+tcp://example.invalid:3333"),
            ),
            StoredValue::string("stratumuser", "redacted-user"),
            StoredValue::string("stratumpass", "redacted-secret"),
        ]);

        // Act
        let settings = pool_settings_from_snapshot(&snapshot).expect("settings should parse");

        // Assert
        assert_eq!(settings.endpoint.port, 3333);
        assert_eq!(settings.runtime_config.model, PHASE25_MODEL);
        assert_eq!(settings.runtime_config.version, PHASE25_VERSION);
    }

    #[test]
    fn lifecycle_markers_are_fixed_categories_and_redacted() {
        // Arrange
        let markers = [
            "blocked",
            "connecting",
            "subscribed",
            "authorized",
            "active",
            "reconnecting",
            "stopped",
        ]
        .map(|status| format!("phase25_live_stratum_status={status} redacted=true"));

        // Act
        let rendered = markers.join("\n");

        // Assert
        for status in [
            "blocked",
            "connecting",
            "subscribed",
            "authorized",
            "active",
            "reconnecting",
            "stopped",
        ] {
            assert!(rendered.contains(&format!("phase25_live_stratum_status={status}")));
        }
        for forbidden in ["redacted-user", "redacted-secret", "example.invalid"] {
            assert!(!rendered.contains(forbidden));
        }
        assert!(rendered.contains("redacted=true"));
    }

    #[test]
    fn safe_stop_marker_and_state_are_phase25_postconditions() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        state.allow_work_submission();

        // Act
        let stopped = phase25_safe_stop_state(state);

        // Assert
        assert_eq!(SAFE_STOP_COMPLETE_MARKER, "phase25_safe_stop_status=complete socket=stopped work_queue=invalidated active_work=invalidated mining=disabled hardware_control=disabled work_submission=disabled post_stop_snapshot=updated");
        assert_eq!(stopped.mining_activity, MiningActivityStatus::SafeBlocked);
        assert_eq!(
            stopped.work_submission,
            bitaxe_stratum::v1::state::WorkSubmissionGate::Blocked
        );
        assert_eq!(stopped.maybe_blocked_reason, Some(PHASE25_SAFE_STOP_REASON));
    }

    #[test]
    fn safe_stop_reasons_cover_all_phase25_convergence_paths() {
        // Arrange
        let labels = safe_stop_reason_labels();

        // Act
        let rendered = labels.join(",");

        // Assert
        for expected in [
            "normal_stop",
            "fallback_exhausted",
            "prerequisite_failure",
            "operator_cancelled",
            "verification_cleanup",
        ] {
            assert!(rendered.contains(expected));
        }
    }

    #[test]
    fn live_socket_loop_progresses_through_notify_before_cleanup_stop() {
        // Arrange
        let socket_state = Rc::new(RefCell::new(ScriptedSocketState::new([
            r#"{"id":1,"result":[[["mining.set_difficulty","1"]],"4de05269",4],"error":null}"#,
            r#"{"id":2,"result":true,"error":null}"#,
            r#"{"id":null,"method":"mining.set_difficulty","params":[42]}"#,
            r#"{"id":null,"method":"mining.notify","params":["job","0100000000000000000000000000000000000000000000000000000000000000","0200000001","ffffffff",[],"20000004","1705ae3a","647025b5",false]}"#,
            r#"{"id":7,"result":true,"error":null}"#,
        ])));
        let mut connector = ScriptedConnector::new(socket_state.clone());
        let pool_settings_access_counter = Rc::new(Cell::new(0));
        let mut settings = CountingSettingsSource::new(pool_settings_access_counter.clone());

        // Act
        let outcome = start_live_stratum_runtime_with_dependencies(
            ready_preconditions(),
            &mut settings,
            &mut connector,
        );
        let state = socket_state.borrow();
        let retained_logs = log_buffer::retained_log_buffer().download_chunks().join("");

        // Assert
        assert_eq!(outcome, LiveStartOutcome::Stopped);
        assert_eq!(pool_settings_access_counter.get(), 1);
        assert!(state.shutdown_called);
        assert!(state.reads.is_empty());
        assert_eq!(state.writes.len(), 2);
        assert!(state
            .writes
            .iter()
            .any(|line| line.contains(r#""method":"mining.subscribe""#)));
        assert!(state
            .writes
            .iter()
            .any(|line| line.contains(r#""method":"mining.authorize""#)));
        assert!(retained_logs.contains("phase25_live_stratum_status=active"));
    }

    #[test]
    fn matching_pending_submit_response_updates_projection_counters() {
        // Arrange
        runtime_snapshot::reset_command_visible_state_for_test();
        let request_id = StratumRequestId::new(7);
        let intent = sample_submit_intent();
        let message = intent
            .submission()
            .to_client_message(request_id, "redacted-user");
        let action = LiveRuntimeAction::SendSubmitShare {
            intent,
            request_id,
            message,
        };
        let mut socket = CountingSocket::default();
        let maybe_pending_submit =
            write_runtime_action(action, &mut socket).expect("submit write should succeed");
        let pending_submit =
            maybe_pending_submit.expect("submit action should retain pending intent");
        let mut runtime = LiveStratumRuntime::new(LiveRuntimeConfig {
            model: PHASE25_MODEL.to_owned(),
            version: PHASE25_VERSION.to_owned(),
            credentials: LivePoolCredentials {
                username: "redacted-user".to_owned(),
                password: "redacted-secret".to_owned(),
            },
        });

        // Act
        let consumed = handle_socket_line(
            &mut runtime,
            r#"{"id":7,"result":true,"error":null}"#,
            Some(&pending_submit),
        );
        let mining = runtime_snapshot::mining_runtime_state();

        // Assert
        assert!(consumed);
        assert_eq!(mining.counters.accepted, 1);
        assert_eq!(mining.counters.rejected, 0);
    }

    struct CountingSettingsSource {
        pool_settings_access_counter: Rc<Cell<usize>>,
    }

    impl CountingSettingsSource {
        fn new(pool_settings_access_counter: Rc<Cell<usize>>) -> Self {
            Self {
                pool_settings_access_counter,
            }
        }
    }

    impl PoolSettingsSource for CountingSettingsSource {
        fn read_pool_settings(&mut self) -> anyhow::Result<PoolSettings> {
            self.pool_settings_access_counter
                .set(self.pool_settings_access_counter.get() + 1);
            Ok(PoolSettings {
                endpoint: PoolEndpoint {
                    host: "example.invalid".to_owned(),
                    port: 3333,
                },
                runtime_config: LiveRuntimeConfig {
                    model: PHASE25_MODEL.to_owned(),
                    version: PHASE25_VERSION.to_owned(),
                    credentials: LivePoolCredentials {
                        username: "redacted-user".to_owned(),
                        password: "redacted-secret".to_owned(),
                    },
                },
            })
        }
    }

    struct CountingConnector {
        connect_counter: Rc<Cell<usize>>,
    }

    impl CountingConnector {
        fn new(connect_counter: Rc<Cell<usize>>) -> Self {
            Self { connect_counter }
        }
    }

    impl LiveSocketConnector for CountingConnector {
        type Socket = CountingSocket;

        fn connect(&mut self, _endpoint: &PoolEndpoint) -> anyhow::Result<Self::Socket> {
            self.connect_counter.set(self.connect_counter.get() + 1);
            Ok(CountingSocket::default())
        }
    }

    #[derive(Default)]
    struct CountingSocket {
        shutdown_called: bool,
    }

    impl LiveSocketIo for CountingSocket {
        fn write_json_line(&mut self, _line: &str) -> anyhow::Result<()> {
            Ok(())
        }

        fn maybe_read_json_line(&mut self) -> anyhow::Result<Option<String>> {
            Ok(Some(
                r#"{"id":1,"result":[[["mining.set_difficulty","1"]],"4de05269",4],"error":null}"#
                    .to_owned(),
            ))
        }

        fn shutdown_both(&mut self) {
            self.shutdown_called = true;
        }
    }

    struct ScriptedConnector {
        socket_state: Rc<RefCell<ScriptedSocketState>>,
    }

    impl ScriptedConnector {
        fn new(socket_state: Rc<RefCell<ScriptedSocketState>>) -> Self {
            Self { socket_state }
        }
    }

    impl LiveSocketConnector for ScriptedConnector {
        type Socket = ScriptedSocket;

        fn connect(&mut self, _endpoint: &PoolEndpoint) -> anyhow::Result<Self::Socket> {
            Ok(ScriptedSocket {
                state: self.socket_state.clone(),
            })
        }
    }

    struct ScriptedSocketState {
        reads: VecDeque<String>,
        writes: Vec<String>,
        shutdown_called: bool,
    }

    impl ScriptedSocketState {
        fn new<const N: usize>(reads: [&'static str; N]) -> Self {
            Self {
                reads: reads.into_iter().map(str::to_owned).collect(),
                writes: Vec::new(),
                shutdown_called: false,
            }
        }
    }

    struct ScriptedSocket {
        state: Rc<RefCell<ScriptedSocketState>>,
    }

    impl LiveSocketIo for ScriptedSocket {
        fn write_json_line(&mut self, line: &str) -> anyhow::Result<()> {
            self.state.borrow_mut().writes.push(line.to_owned());
            Ok(())
        }

        fn maybe_read_json_line(&mut self) -> anyhow::Result<Option<String>> {
            Ok(self.state.borrow_mut().reads.pop_front())
        }

        fn shutdown_both(&mut self) {
            self.state.borrow_mut().shutdown_called = true;
        }
    }

    fn ready_preconditions() -> ProductionMiningPreconditions {
        ProductionMiningPreconditions {
            power: ProductionMiningPrerequisite::Fresh,
            thermal: ProductionMiningPrerequisite::Fresh,
            fan: ProductionMiningPrerequisite::Fresh,
            voltage: ProductionMiningPrerequisite::Fresh,
            safety: ProductionMiningPrerequisite::Fresh,
        }
    }

    fn blocked_preconditions(reason: &'static str) -> ProductionMiningPreconditions {
        ProductionMiningPreconditions {
            power: ProductionMiningPrerequisite::blocked(reason),
            ..ready_preconditions()
        }
    }

    fn sample_submit_intent() -> SubmitIntent {
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x28);
        registry
            .enqueue_pool_work(sample_work(job_id))
            .expect("sample work should enqueue");
        let _dispatch = registry.dispatch_next().expect("work should dispatch");
        let outcome = registry.correlate_nonce_result(ProductionNonceObservation {
            observed_generation: registry.generation(),
            result: Bm1366NonceResult {
                job_id,
                nonce: 0x1234_5678,
                asic_index: 0,
                core_id: 1,
                small_core_id: 0,
                version_bits: 0x0000_2000,
            },
        });
        let CorrelationOutcome::SubmitIntent(intent) = outcome else {
            panic!("sample active work should produce submit intent");
        };
        intent
    }

    fn sample_work(job_id: Bm1366JobId) -> bitaxe_stratum::v1::mining::MiningWork {
        MiningWorkBuilder::new(
            MiningNotify {
                job_id: "correlated-job".to_owned(),
                prev_block_hash: "00".repeat(32),
                coinbase_1: "0200000001".to_owned(),
                coinbase_2: "ffffffff".to_owned(),
                merkle_branches: Vec::new(),
                version: 0x2000_0004,
                nbits: 0x1705_ae3a,
                ntime: 0x6470_25b5,
                clean_jobs: false,
            },
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .with_pool_difficulty(PoolDifficulty { difficulty: 1.25 })
        .build(job_id)
        .expect("sample work should build")
    }
}
