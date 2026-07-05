//! Phase 25 live Stratum socket adapter.
//!
//! This firmware shell owns ESP-IDF socket effects while protocol state,
//! parsing, submit classification, and safe-stop postconditions stay in pure
//! crates.

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
use bitaxe_stratum::v1::{
    live_runtime::{LivePoolCredentials, LiveRuntimeAction, LiveRuntimeConfig, LiveStratumRuntime},
    messages::{parse_server_message, StratumResponse, StratumV1ServerMessage},
    mining_loop::{MiningLoopDecision, MiningLoopGate},
    production_work::SubmitIntent,
    state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus},
    submit_response::{
        classify_maybe_submit_response, classify_submit_response, SubmitClassification,
        SubmitResponseObservation,
    },
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
            publish_safe_stop_without_runtime(SafeStopReason::FallbackExhausted);
            return LiveStartOutcome::Blocked {
                reason: SafeStopReason::FallbackExhausted.as_str(),
            };
        }
    };

    let mut runtime = LiveStratumRuntime::new(settings.runtime_config);
    let _event = runtime.start();
    if write_runtime_actions(&mut runtime, &mut socket).is_err() {
        return safe_stop_with_socket(runtime, socket, SafeStopReason::FallbackExhausted);
    }

    if let Ok(Some(line)) = socket.maybe_read_json_line() {
        handle_socket_line(&mut runtime, &line);
        if write_runtime_actions(&mut runtime, &mut socket).is_err() {
            return safe_stop_with_socket(runtime, socket, SafeStopReason::FallbackExhausted);
        }
    }

    safe_stop_with_socket(runtime, socket, SafeStopReason::NormalStop)
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

fn write_runtime_actions<S: LiveSocketIo>(
    runtime: &mut LiveStratumRuntime,
    socket: &mut S,
) -> anyhow::Result<()> {
    publish_watchdog_checkpoint(StepKind::Socket, 1);
    for action in runtime.drain_actions() {
        match action {
            LiveRuntimeAction::SendClientMessage(message) => {
                let line = message.to_json_line()?;
                socket.write_json_line(&line)?;
            }
        }
    }

    Ok(())
}

fn handle_socket_line(runtime: &mut LiveStratumRuntime, line: &str) {
    publish_watchdog_checkpoint(StepKind::Socket, 1);
    let Ok(message) = parse_server_message(line) else {
        let _classification = classify_maybe_submit_response(
            None,
            StratumRequestId::new(0),
            SubmitResponseObservation::Malformed,
        );
        publish_status("reconnecting");
        return;
    };

    publish_event_status(runtime.apply_server_message(message.clone()).ok().flatten());
    if let StratumV1ServerMessage::Response(response) = message {
        let _classification = classify_maybe_submit_response(
            None,
            StratumRequestId::new(0),
            SubmitResponseObservation::Response(response),
        );
    }
}

fn publish_event_status(maybe_event: Option<bitaxe_stratum::v1::live_runtime::LiveRuntimeEvent>) {
    use bitaxe_stratum::v1::live_runtime::LiveRuntimeEvent;

    match maybe_event {
        Some(LiveRuntimeEvent::Subscribed) => publish_status("subscribed"),
        Some(LiveRuntimeEvent::Authorized) => publish_status("authorized"),
        Some(LiveRuntimeEvent::WorkQueued) => publish_status("active"),
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
    let post_stop_state = phase25_safe_stop_state(runtime.state().clone());
    runtime_snapshot::replace_mining_runtime_state_after_phase25_safe_stop(post_stop_state);
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
    let state = phase25_safe_stop_state(MiningRuntimeState::default());
    runtime_snapshot::replace_mining_runtime_state_after_phase25_safe_stop(state);
    publish_safe_stop_complete();
    publish_status("stopped");
}

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
    info_retained(&format!(
        "phase25_live_stratum_status={status} redacted=true"
    ));
}

fn publish_blocked(reason: &'static str) {
    info_retained(&format!(
        "phase25_live_stratum_status=blocked phase25_prerequisite_status={reason} redacted=true"
    ));
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
    use std::cell::Cell;
    use std::rc::Rc;

    use bitaxe_config::{NvsSnapshot, StoredValue};

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
}
