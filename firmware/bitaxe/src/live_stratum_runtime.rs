//! Phase 25 live Stratum socket adapter.
//!
//! This firmware shell owns ESP-IDF socket effects while protocol state,
//! parsing, submit classification, and safe-stop postconditions stay in pure
//! crates.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use bitaxe_asic::bm1366::{
    mining_ready::bm1366_job_interval_ms, production::ProductionAsicBlocker,
    result::Bm1366NonceResult,
};
use bitaxe_config::{nvs::StoredValueKind, ultra_205_defaults, NvsSnapshot};
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence,
    mining_preconditions::{
        BoundedObservationEvidence, ProductionMiningPreconditionDecision,
        ProductionMiningPreconditions, ProductionMiningPrerequisite, FAN_OBSERVATION_UNAVAILABLE,
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
    bridge_orchestration::{self, BridgeOrchestrator, BridgeStep},
    live_runtime::{
        BridgeObservationOutcome, LivePoolCredentials, LiveRuntimeAction, LiveRuntimeConfig,
        LiveRuntimeEvent, LiveStratumRuntime,
    },
    messages::{parse_server_message, StratumResponse, StratumV1ServerMessage},
    mining_loop::{
        GuardedMiningLoopInputs, GuardedMiningLoopSource, MiningLoopDecision, MiningLoopGate,
    },
    production_work::{PoolSessionGeneration, ProductionNonceObservation, SubmitIntent},
    state::PoolLifecycleStatus,
    submit_response::{
        classify_maybe_submit_response, classify_submit_response, SubmitClassification,
        SubmitResponseObservation,
    },
    telemetry_projection::RuntimeProjectionSampleSource,
};

use crate::{
    asic_adapter::{self, ProductionAsicExecutor, ProductionReadOutcome},
    log_buffer,
    mining_evidence_mode::MiningEvidenceMode,
    runtime_snapshot, safety_adapter, settings_adapter,
};

const BOARD_205: &str = "205";
const EVIDENCE_ID: &str = "phase25-live-stratum-runtime-safe-stop";
const PHASE27_EVIDENCE_ID: &str = "phase27-live-hardware-bridge-safe-stop";
const PHASE25_MODEL: &str = "ultra";
const PHASE25_VERSION: &str = "205";
const SOCKET_TIMEOUT_MS: u64 = 100;
const READ_BUFFER_BYTES: usize = 512;
const MAX_JSON_LINE_BYTES: usize = 16 * 1024;
const LIVE_SOCKET_PUMP_ITERATIONS: usize = 16;
const PHASE27_LIVE_BRIDGE_PUMP_ITERATIONS: usize = 32;
const PHASE27_BOUNDED_OBSERVATION_WINDOW_MS: u32 = 60_000;
const PHASE27_POOL_WAIT_POLL_MS: u64 = 2_000;
const PHASE27_POOL_WAIT_TIMEOUT_MS: u64 = 120_000;
/// Power-delta probe sampling delay after first dispatch (7 s, inside the
/// locked 5-10 s discrimination window).
const POWER_DELTA_PROBE_DELAY_MS: u64 = 7_000;
const PHASE27_BRIDGE_IDLE: u8 = 0;
const PHASE27_BRIDGE_RUNNING: u8 = 1;
const PHASE27_BRIDGE_COMPLETED: u8 = 2;
const PHASE27_POOL_SETTINGS_CONSUMED_MARKER: &str =
    "phase21_pool_settings_consumed=true source=settings_patch redacted=true";
const STRATUM_PLUS_TCP_PREFIX: &str = concat!("stratum", "+tcp://");
const TCP_PREFIX: &str = "tcp://";
const POOL_SETTINGS_UNAVAILABLE: &str = "pool_settings_unavailable";
const POOL_SETTINGS_INVALID: &str = "pool_settings_invalid";
const PHASE25_SAFE_STOP_REASON: &str = "phase25_safe_stop";
const SAFE_STOP_COMPLETE_MARKER: &str = "phase25_safe_stop_status=complete socket=stopped work_queue=invalidated active_work=invalidated mining=disabled hardware_control=disabled work_submission=disabled post_stop_snapshot=updated";

static PHASE27_BRIDGE_STATE: AtomicU8 = AtomicU8::new(PHASE27_BRIDGE_IDLE);
static PHASE27_POOL_SETTINGS_CONSUMED_EMITTED: AtomicBool = AtomicBool::new(false);

pub fn maybe_start_after_network_setup(network_ready: bool) {
    if MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        return;
    }
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

pub fn schedule_phase27_bridge_after_http_ready(network_ready: bool) {
    if !MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        return;
    }

    let _reason_labels = safe_stop_reason_labels();
    publish_phase25_watchdog_boundaries();

    if !network_ready {
        publish_blocked("network_unavailable");
        publish_safe_stop_without_runtime(SafeStopReason::PrerequisiteFailure);
        PHASE27_BRIDGE_STATE.store(PHASE27_BRIDGE_COMPLETED, Ordering::SeqCst);
        return;
    }

    let _ = thread::Builder::new()
        .name("phase27-bridge".to_owned())
        .spawn(move || phase27_bridge_wait_loop(network_ready));
}

pub fn maybe_refresh_phase27_from_settings() {
    if !MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        return;
    }

    if PHASE27_BRIDGE_STATE.load(Ordering::SeqCst) != PHASE27_BRIDGE_IDLE {
        return;
    }

    maybe_emit_phase27_pool_settings_consumed_marker();
    let _ = try_start_phase27_live_bridge_once(true);
}

pub fn maybe_emit_phase27_pool_settings_consumed_marker() {
    if !MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        return;
    }

    if !pool_settings_present_in_snapshot(&settings_adapter::current_settings_snapshot()) {
        return;
    }

    if PHASE27_POOL_SETTINGS_CONSUMED_EMITTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    info_retained(PHASE27_POOL_SETTINGS_CONSUMED_MARKER);
}

fn pool_settings_present_in_snapshot(snapshot: &NvsSnapshot) -> bool {
    pool_settings_from_snapshot(snapshot).is_ok()
}

fn phase27_bridge_wait_loop(network_ready: bool) {
    thread::sleep(Duration::from_millis(500));
    let deadline = Instant::now() + Duration::from_millis(PHASE27_POOL_WAIT_TIMEOUT_MS);

    while Instant::now() < deadline {
        if try_start_phase27_live_bridge_once(network_ready) {
            return;
        }

        thread::sleep(Duration::from_millis(PHASE27_POOL_WAIT_POLL_MS));
    }

    if PHASE27_BRIDGE_STATE.load(Ordering::SeqCst) == PHASE27_BRIDGE_COMPLETED {
        return;
    }

    publish_blocked("phase27_pool_wait_timeout");
    publish_safe_stop_without_runtime(SafeStopReason::PrerequisiteFailure);
    PHASE27_BRIDGE_STATE.store(PHASE27_BRIDGE_COMPLETED, Ordering::SeqCst);
}

fn try_start_phase27_live_bridge_once(network_ready: bool) -> bool {
    if !network_ready {
        return false;
    }

    if PHASE27_BRIDGE_STATE.load(Ordering::SeqCst) == PHASE27_BRIDGE_COMPLETED {
        return true;
    }

    if PHASE27_BRIDGE_STATE.load(Ordering::SeqCst) == PHASE27_BRIDGE_RUNNING {
        return false;
    }

    let mut settings_source = FirmwarePoolSettingsSource;
    if settings_source.read_pool_settings().is_err() {
        publish_status("waiting_for_pool_settings");
        return false;
    }

    if PHASE27_BRIDGE_STATE
        .compare_exchange(
            PHASE27_BRIDGE_IDLE,
            PHASE27_BRIDGE_RUNNING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .is_err()
    {
        return PHASE27_BRIDGE_STATE.load(Ordering::SeqCst) == PHASE27_BRIDGE_COMPLETED;
    }

    let mut connector = FirmwareTcpConnector;
    let _outcome =
        start_phase27_live_bridge_with_dependencies(&mut settings_source, &mut connector);
    PHASE27_BRIDGE_STATE.store(PHASE27_BRIDGE_COMPLETED, Ordering::SeqCst);
    true
}

fn start_phase27_live_bridge_with_dependencies<C, S>(
    settings_source: &mut S,
    connector: &mut C,
) -> LiveStartOutcome
where
    C: LiveSocketConnector,
    S: PoolSettingsSource,
{
    let decision = firmware_phase27_production_preconditions().decision();
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
            publish_status("waiting_for_pool_settings");
            PHASE27_BRIDGE_STATE.store(PHASE27_BRIDGE_IDLE, Ordering::SeqCst);
            return LiveStartOutcome::Blocked {
                reason: POOL_SETTINGS_UNAVAILABLE,
            };
        }
    };

    info_retained(PHASE27_POOL_SETTINGS_CONSUMED_MARKER);

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

fn firmware_phase27_production_preconditions() -> ProductionMiningPreconditions {
    let snapshot = safety_adapter::phase27_safety_snapshot();
    if !snapshot.bring_up_complete {
        return ProductionMiningPreconditions {
            power: phase27_bounded_prerequisite("phase27_power"),
            thermal: phase27_bounded_prerequisite("phase27_thermal"),
            fan: phase27_bounded_prerequisite("phase27_fan"),
            voltage: phase27_bounded_prerequisite("phase27_voltage"),
            safety: phase27_bounded_prerequisite("phase27_safety"),
        };
    }

    ProductionMiningPreconditions {
        power: snapshot
            .maybe_power
            .map(ProductionMiningPrerequisite::from_power_observation)
            .unwrap_or_else(|| ProductionMiningPrerequisite::blocked("power_sample_unavailable")),
        thermal: phase27_thermal_prerequisite(&snapshot),
        fan: if snapshot.fan_duty_percent > 0 {
            ProductionMiningPrerequisite::Fresh
        } else {
            ProductionMiningPrerequisite::blocked(FAN_OBSERVATION_UNAVAILABLE)
        },
        voltage: ProductionMiningPrerequisite::Fresh,
        safety: ProductionMiningPrerequisite::Fresh,
    }
}

fn phase27_thermal_prerequisite(
    snapshot: &safety_adapter::phase27_bring_up::Phase27SafetySnapshot,
) -> ProductionMiningPrerequisite {
    let Some(observation) = snapshot.maybe_thermal else {
        return ProductionMiningPrerequisite::blocked("thermal_reading_unavailable");
    };

    if observation.reason().is_none() {
        return ProductionMiningPrerequisite::from_thermal_observation(observation);
    }

    phase27_bounded_prerequisite("phase27_thermal")
}

fn phase27_bounded_prerequisite(source: &'static str) -> ProductionMiningPrerequisite {
    ProductionMiningPrerequisite::Bounded(BoundedObservationEvidence {
        source,
        board: BOARD_205,
        evidence_id: PHASE27_EVIDENCE_ID,
        validity_window_ms: PHASE27_BOUNDED_OBSERVATION_WINDOW_MS,
        reason: "bounded_observation_accepted",
    })
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
    let phase27_bridge = MiningEvidenceMode::current().is_phase27_live_hardware_bridge();
    let evidence_id = if phase27_bridge {
        PHASE27_EVIDENCE_ID
    } else {
        EVIDENCE_ID
    };
    let evidence = SafetyCriticalEvidence::hardware_smoke(evidence_id);
    let asic_initialized = if phase27_bridge {
        asic_adapter::production_ready()
    } else {
        true
    };

    let snapshot = safety_adapter::phase27_safety_snapshot();
    let (maybe_power_evidence, maybe_thermal_evidence) =
        if phase27_bridge && snapshot.bring_up_complete {
            (
                snapshot
                    .maybe_power
                    .and_then(PowerEvidenceToken::from_observation),
                phase27_thermal_evidence_token(&snapshot, evidence),
            )
        } else if phase27_bridge {
            (None, None)
        } else {
            (
                Some(PowerEvidenceToken {
                    bus_voltage_volts: 5.0,
                    current_amps: 2.5,
                    power_watts: 12.5,
                }),
                Some(ThermalEvidenceToken {
                    chip_temp_celsius: 55.0,
                    evidence,
                }),
            )
        };

    MiningLoopGate {
        production_preconditions: decision,
        asic_initialized,
        maybe_power_evidence,
        maybe_thermal_evidence,
        maybe_safety_evidence: Some(evidence),
        safety_status: SafetyStatus::Normal,
        hardware_evidence_ack: true,
    }
}

fn phase27_thermal_evidence_token(
    snapshot: &safety_adapter::phase27_bring_up::Phase27SafetySnapshot,
    evidence: SafetyCriticalEvidence,
) -> Option<ThermalEvidenceToken> {
    let observation = snapshot.maybe_thermal?;

    ThermalEvidenceToken::from_observation(observation, evidence)
        .or_else(|| ThermalEvidenceToken::from_phase27_fresh_observation(observation, evidence))
        .or_else(|| {
            if observation.reason().is_some() {
                ThermalEvidenceToken::from_phase27_bounded_evidence(evidence)
            } else {
                None
            }
        })
}

fn pump_live_socket_until_cleanup<S: LiveSocketIo>(
    runtime: &mut LiveStratumRuntime,
    socket: &mut S,
) -> SafeStopReason {
    let phase27_bridge = MiningEvidenceMode::current().is_phase27_live_hardware_bridge();
    let mut pending_actions: VecDeque<LiveRuntimeAction> = runtime.drain_actions().into();
    let mut maybe_pending_submit: Option<PendingSubmit> = None;
    let mut asic_bridge = AsicBridgeState::new_for_phase27();

    let base_max_iterations = if phase27_bridge {
        PHASE27_LIVE_BRIDGE_PUMP_ITERATIONS
    } else {
        LIVE_SOCKET_PUMP_ITERATIONS
    };
    let mut iteration = 0usize;

    while iteration < base_max_iterations
        || (phase27_bridge && asic_bridge.should_continue_result_read())
    {
        publish_watchdog_checkpoint(StepKind::Socket, 1);

        if phase27_bridge {
            asic_bridge.maybe_arm_continuous_listener();
            maybe_log_power_delta(&mut asic_bridge);
        }

        if phase27_bridge && asic_bridge.needs_step() {
            publish_watchdog_checkpoint(StepKind::Asic, 1);
            match run_asic_bridge_step(runtime, &mut asic_bridge) {
                AsicBridgeStepOutcome::Actions(actions) => {
                    pending_actions.extend(actions);
                }
                AsicBridgeStepOutcome::Blocked { reason } => {
                    publish_asic_bridge_blocked(reason);
                    return SafeStopReason::PrerequisiteFailure;
                }
                AsicBridgeStepOutcome::Correlated
                | AsicBridgeStepOutcome::UartProof
                | AsicBridgeStepOutcome::NoOp => {}
            }
        }

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
                match handle_socket_line(runtime, &line, maybe_pending_submit.as_ref()) {
                    SocketLineOutcome::SubmitClassified => {
                        maybe_pending_submit = None;
                    }
                    SocketLineOutcome::WorkQueued => {
                        asic_bridge.note_work_queued();
                    }
                    SocketLineOutcome::SessionInvalidated => {
                        asic_bridge.note_session_invalidated();
                    }
                    SocketLineOutcome::None => {}
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

        // Upstream parity: mining continues after a classified share — no
        // first-share exit (reference/esp-miner/main/tasks/asic_result_task.c
        // never leaves its loop on success). Safe-stop, socket errors, and
        // cleanup remain the only exits.
        iteration += 1;
    }

    SafeStopReason::VerificationCleanup
}

/// Thin firmware wrapper around the pure [`BridgeOrchestrator`].
///
/// All step priority and cadence decisions live in
/// `bitaxe_stratum::v1::bridge_orchestration`; this wrapper keeps only the
/// shell-facing state the orchestrator does not own.
#[derive(Debug)]
struct AsicBridgeState {
    orchestrator: BridgeOrchestrator,
    /// Continuous listener + regeneration cadence are the phase27 bridge
    /// defaults. The `single_dispatch_bounded_read` control mode sets this
    /// false to restore the pre-28.1 bounded-read fail-closed behavior for
    /// A/B control runs.
    continuous: bool,
    listener_armed: bool,
    last_dispatch_generation: Option<PoolSessionGeneration>,
    /// Bounded-read state; used only when `continuous == false`.
    awaiting_bounded_read: bool,
    maybe_bounded_read_deadline: Option<Instant>,
    /// Once-per-session guard for the post-dispatch register-read probe.
    post_dispatch_probe_sent: bool,
    /// INA260 sample taken immediately before the first dispatch of a session.
    maybe_power_baseline_mw: Option<u32>,
    /// When the first successful production dispatch of a session happened.
    maybe_first_dispatch_at: Option<Instant>,
    /// Once-per-session guard for the power-delta marker.
    power_delta_logged: bool,
}

impl Default for AsicBridgeState {
    fn default() -> Self {
        Self::new_for_phase27()
    }
}

impl AsicBridgeState {
    fn new_for_phase27() -> Self {
        Self {
            // Ultra 205 carries a single BM1366, so the regeneration cadence
            // is bm1366_job_interval_ms(1) == 2000 ms (upstream parity).
            orchestrator: BridgeOrchestrator::new(bm1366_job_interval_ms(1)),
            continuous: !asic_adapter::single_dispatch_bounded_read_enabled(),
            listener_armed: false,
            last_dispatch_generation: None,
            awaiting_bounded_read: false,
            maybe_bounded_read_deadline: None,
            post_dispatch_probe_sent: false,
            maybe_power_baseline_mw: None,
            maybe_first_dispatch_at: None,
            power_delta_logged: false,
        }
    }

    fn needs_step(&self) -> bool {
        if !self.continuous && self.awaiting_bounded_read {
            return true;
        }

        self.orchestrator.next_step(Instant::now()) != BridgeStep::Idle
    }

    fn note_work_queued(&mut self) {
        self.orchestrator.note_work_queued();
    }

    fn note_session_invalidated(&mut self) {
        self.orchestrator.invalidate_session();
        self.last_dispatch_generation = None;
        self.clear_bounded_read();
        self.post_dispatch_probe_sent = false;
        self.maybe_power_baseline_mw = None;
        self.maybe_first_dispatch_at = None;
        self.power_delta_logged = false;
    }

    fn maybe_arm_continuous_listener(&mut self) {
        if !self.continuous || self.listener_armed {
            return;
        }

        if !asic_adapter::production_ready() {
            return;
        }

        self.listener_armed = true;
        self.orchestrator.note_listener_armed();
        log::info!("h4_continuous_result=listener_armed");
    }

    fn should_continue_result_read(&self) -> bool {
        if self.continuous {
            return self.listener_armed;
        }

        self.awaiting_bounded_read
            && self.maybe_bounded_read_deadline.is_some_and(|deadline| {
                Instant::now() < deadline + Duration::from_millis(SOCKET_TIMEOUT_MS)
            })
    }

    fn clear_bounded_read(&mut self) {
        self.awaiting_bounded_read = false;
        self.maybe_bounded_read_deadline = None;
    }
}

enum AsicBridgeStepOutcome {
    NoOp,
    Actions(Vec<LiveRuntimeAction>),
    Correlated,
    UartProof,
    Blocked { reason: ProductionAsicBlocker },
}

fn run_asic_bridge_step(
    runtime: &mut LiveStratumRuntime,
    bridge: &mut AsicBridgeState,
) -> AsicBridgeStepOutcome {
    if !bridge.continuous && bridge.awaiting_bounded_read {
        return read_bounded_asic_result(runtime, bridge);
    }

    match bridge.orchestrator.next_step(Instant::now()) {
        BridgeStep::Dispatch => dispatch_production_work(runtime, bridge),
        // Control mode (single_dispatch_bounded_read): the pre-28.1 pump
        // never regenerated work — only fresh pool work triggers a dispatch.
        BridgeStep::Regenerate if !bridge.continuous => AsicBridgeStepOutcome::NoOp,
        BridgeStep::Regenerate => match runtime.regenerate_work() {
            Ok(counter) => {
                info_retained(&bridge_orchestration::job_redispatched_marker(counter));
                dispatch_production_work(runtime, bridge)
            }
            // No held notify: regeneration never fabricates work.
            Err(_) => AsicBridgeStepOutcome::NoOp,
        },
        BridgeStep::Poll { slice_ms } => poll_continuous_asic_result(runtime, bridge, slice_ms),
        BridgeStep::Idle => AsicBridgeStepOutcome::NoOp,
    }
}

fn dispatch_production_work(
    runtime: &mut LiveStratumRuntime,
    bridge: &mut AsicBridgeState,
) -> AsicBridgeStepOutcome {
    let gate = mining_loop_gate(firmware_phase27_production_preconditions().decision());
    if gate.decision() != MiningLoopDecision::Ready {
        let MiningLoopDecision::Blocked { reason } = gate.decision() else {
            return AsicBridgeStepOutcome::NoOp;
        };
        if let Some(blocker) = production_asic_blocker_from_reason(reason) {
            return AsicBridgeStepOutcome::Blocked { reason: blocker };
        }
        publish_asic_bridge_blocked(ProductionAsicBlocker::PrerequisiteBlocked);
        return AsicBridgeStepOutcome::Blocked {
            reason: ProductionAsicBlocker::PrerequisiteBlocked,
        };
    }

    let inputs = GuardedMiningLoopInputs {
        gate,
        pool_defaults: ultra_205_defaults(),
        source: GuardedMiningLoopSource::Notify,
        production_registry: runtime.production_registry().clone(),
        runtime_state: runtime.state().clone(),
        maybe_nonce_observation: None,
    };
    let plan = match inputs.plan() {
        Ok(plan) => plan,
        Err(_) => {
            publish_asic_bridge_blocked(ProductionAsicBlocker::WorkStale);
            return AsicBridgeStepOutcome::Blocked {
                reason: ProductionAsicBlocker::WorkStale,
            };
        }
    };

    *runtime.production_registry_mut() = plan.production_registry;
    let Some(dispatch) = plan.maybe_dispatch else {
        // Nothing dispatchable: clear pending/cadence state so the pump
        // idles until fresh pool work arrives.
        bridge.orchestrator.invalidate_session();
        return AsicBridgeStepOutcome::NoOp;
    };
    let Some(command) = dispatch.maybe_production_command else {
        bridge.orchestrator.invalidate_session();
        return AsicBridgeStepOutcome::NoOp;
    };

    let generation = runtime.production_registry().generation();
    bridge.last_dispatch_generation = Some(generation);
    let valid_jobs = runtime.production_registry().valid_jobs();
    sample_power_baseline_before_first_dispatch(bridge);
    let mut executor = ProductionAsicExecutor::new();
    match executor.execute(command, valid_jobs) {
        Ok(_) => {
            bridge.orchestrator.note_dispatched(Instant::now());
            if bridge.maybe_first_dispatch_at.is_none() {
                bridge.maybe_first_dispatch_at = Some(Instant::now());
            }
            maybe_send_post_dispatch_register_probe(bridge);
            if !bridge.continuous {
                bridge.awaiting_bounded_read = true;
                bridge.maybe_bounded_read_deadline = Some(
                    Instant::now()
                        + Duration::from_millis(u64::from(asic_adapter::RESULT_WORK_TIMEOUT_MS)),
                );
            }
            AsicBridgeStepOutcome::NoOp
        }
        Err(blocker) => AsicBridgeStepOutcome::Blocked { reason: blocker },
    }
}

/// Post-dispatch register-read probe: one TX-only `ReadChipId` frame after
/// the FIRST successful production dispatch of a socket session. Does not
/// block for the response — the continuous poll path already classifies a
/// register-read reply (`asic_production_trace=register_read_parsed`);
/// absence of that marker in a capture IS the silent outcome. Diagnostic
/// only; TX failure logs and continues, never blocking dispatch or mining.
fn maybe_send_post_dispatch_register_probe(bridge: &mut AsicBridgeState) {
    if bridge.post_dispatch_probe_sent {
        return;
    }
    bridge.post_dispatch_probe_sent = true;
    if asic_adapter::probe_register_read_tx() {
        info_retained("asic_probe=register_read_tx stage=post_dispatch");
    } else {
        log::warn!("asic_probe=register_read_tx_error stage=post_dispatch");
    }
}

/// INA260 baseline sample immediately before the FIRST production dispatch
/// of a socket session. Numeric-only marker; sampling failure is silent and
/// surfaces later as `asic_probe=power_delta unavailable=true`.
fn sample_power_baseline_before_first_dispatch(bridge: &mut AsicBridgeState) {
    if bridge.maybe_first_dispatch_at.is_some() {
        return;
    }
    bridge.maybe_power_baseline_mw = safety_adapter::power_probe::sample_power_mw();
    if let Some(baseline_mw) = bridge.maybe_power_baseline_mw {
        info_retained(&format!("asic_probe=power_baseline mw={baseline_mw}"));
    }
}

/// Power-delta discrimination marker: once per session, ~7 s after the first
/// dispatch, sample INA260 again and log the numeric-only delta. Hashing at
/// 485 MHz shows a ~10 W class jump; flat means cores are not hashing. The
/// delta magnitude is diagnostic data, never a pass/fail gate.
fn maybe_log_power_delta(bridge: &mut AsicBridgeState) {
    if bridge.power_delta_logged {
        return;
    }
    let Some(first_dispatch_at) = bridge.maybe_first_dispatch_at else {
        return;
    };
    if Instant::now() < first_dispatch_at + Duration::from_millis(POWER_DELTA_PROBE_DELAY_MS) {
        return;
    }

    bridge.power_delta_logged = true;
    let maybe_after_mw = safety_adapter::power_probe::sample_power_mw();
    match (bridge.maybe_power_baseline_mw, maybe_after_mw) {
        (Some(baseline_mw), Some(after_mw)) => {
            let delta_mw = i64::from(after_mw) - i64::from(baseline_mw);
            info_retained(&format!(
                "asic_probe=power_delta baseline_mw={baseline_mw} after_mw={after_mw} delta_mw={delta_mw}"
            ));
        }
        _ => {
            info_retained("asic_probe=power_delta unavailable=true");
        }
    }
}

fn poll_continuous_asic_result(
    runtime: &mut LiveStratumRuntime,
    bridge: &mut AsicBridgeState,
    slice_ms: u32,
) -> AsicBridgeStepOutcome {
    let generation = bridge
        .last_dispatch_generation
        .unwrap_or_else(|| runtime.production_registry().generation());
    let poll_timeout_ms = u64::from(slice_ms).min(SOCKET_TIMEOUT_MS).max(1) as u32;
    let valid_jobs = runtime.production_registry().valid_jobs();
    let mut executor = ProductionAsicExecutor::new();
    match executor.try_read_production_result(valid_jobs, poll_timeout_ms) {
        Ok(ProductionReadOutcome::Pending) => {
            // Upstream parity: a result timeout is continue-with-telemetry,
            // never fatal (reference/esp-miner/main/tasks/asic_result_task.c:24-36).
            log::info!("h4_continuous_result=timeout_continue");
            let streak = bridge.orchestrator.note_poll_timeout();
            info_retained(&bridge_orchestration::timeout_streak_marker(streak));
            AsicBridgeStepOutcome::NoOp
        }
        Ok(ProductionReadOutcome::RegisterReadProof) => {
            bridge.orchestrator.note_result_received();
            AsicBridgeStepOutcome::UartProof
        }
        Ok(ProductionReadOutcome::JobNonce(result)) => {
            bridge.orchestrator.note_result_received();
            correlate_job_nonce(runtime, generation, result)
        }
        Err(blocker) => {
            log::info!(
                "h4_continuous_result=read_error_continue reason={}",
                blocker.as_str()
            );
            AsicBridgeStepOutcome::NoOp
        }
    }
}

fn correlate_job_nonce(
    runtime: &mut LiveStratumRuntime,
    generation: PoolSessionGeneration,
    result: Bm1366NonceResult,
) -> AsicBridgeStepOutcome {
    let observation = ProductionNonceObservation {
        observed_generation: generation,
        result,
    };
    match runtime.apply_bridge_observation(observation) {
        Ok(BridgeObservationOutcome::SubmitQueued) => {
            asic_adapter::publish_production_asic_status(
                bitaxe_asic::bm1366::production::ProductionAsicStatus::ResultCorrelated,
            );
            AsicBridgeStepOutcome::Actions(runtime.drain_actions())
        }
        Ok(BridgeObservationOutcome::Blocked { reason }) => {
            publish_asic_bridge_blocked(reason);
            AsicBridgeStepOutcome::Blocked { reason }
        }
        Err(_) => {
            publish_asic_bridge_blocked(ProductionAsicBlocker::JobUncorrelated);
            AsicBridgeStepOutcome::Blocked {
                reason: ProductionAsicBlocker::JobUncorrelated,
            }
        }
    }
}

/// Bounded single-read rollback path, reachable only when
/// `continuous == false` (the Plan 28.1-04 `single_dispatch_bounded_read`
/// control mode). The default continuous path never fail-closes on timeout.
fn read_bounded_asic_result(
    runtime: &mut LiveStratumRuntime,
    bridge: &mut AsicBridgeState,
) -> AsicBridgeStepOutcome {
    let generation = bridge
        .last_dispatch_generation
        .unwrap_or_else(|| runtime.production_registry().generation());

    let deadline = bridge.maybe_bounded_read_deadline.unwrap_or_else(|| {
        let deadline =
            Instant::now() + Duration::from_millis(u64::from(asic_adapter::RESULT_WORK_TIMEOUT_MS));
        bridge.maybe_bounded_read_deadline = Some(deadline);
        deadline
    });
    if Instant::now() >= deadline {
        bridge.clear_bounded_read();
        return AsicBridgeStepOutcome::Blocked {
            reason: ProductionAsicBlocker::ResultTimeout,
        };
    }

    let remaining_ms = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(u128::from(SOCKET_TIMEOUT_MS)) as u32;
    let poll_timeout_ms = remaining_ms.max(1);
    let valid_jobs = runtime.production_registry().valid_jobs();
    let mut executor = ProductionAsicExecutor::new();
    match executor.try_read_production_result(valid_jobs, poll_timeout_ms) {
        Ok(ProductionReadOutcome::Pending) => AsicBridgeStepOutcome::NoOp,
        Ok(ProductionReadOutcome::RegisterReadProof) => {
            bridge.clear_bounded_read();
            AsicBridgeStepOutcome::UartProof
        }
        Ok(ProductionReadOutcome::JobNonce(result)) => {
            bridge.clear_bounded_read();
            correlate_job_nonce(runtime, generation, result)
        }
        Err(blocker) => {
            bridge.clear_bounded_read();
            AsicBridgeStepOutcome::Blocked { reason: blocker }
        }
    }
}

fn publish_asic_bridge_blocked(reason: ProductionAsicBlocker) {
    asic_adapter::publish_production_asic_blocked_status(reason);
}

fn production_asic_blocker_from_reason(reason: &'static str) -> Option<ProductionAsicBlocker> {
    ProductionAsicBlocker::ALL
        .into_iter()
        .find(|blocker| blocker.as_str() == reason)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SocketLineOutcome {
    None,
    SubmitClassified,
    WorkQueued,
    SessionInvalidated,
}

fn handle_socket_line(
    runtime: &mut LiveStratumRuntime,
    line: &str,
    maybe_pending_submit: Option<&PendingSubmit>,
) -> SocketLineOutcome {
    publish_watchdog_checkpoint(StepKind::Socket, 1);
    let Ok(message) = parse_server_message(line) else {
        publish_submit_classification(
            runtime.production_registry().generation(),
            SubmitClassification::Malformed,
        );
        publish_status("reconnecting");
        return SocketLineOutcome::None;
    };

    publish_pool_difficulty_if_present(&message);
    if let StratumV1ServerMessage::Response(response) = &message {
        if let Some(pending_submit) =
            maybe_pending_submit.filter(|pending_submit| pending_submit.matches_response(response))
        {
            let classification = pending_submit.classify_response(response.clone());
            publish_submit_classification(pending_submit.intent.generation, classification);
            publish_share_submission_status(classification);
            return SocketLineOutcome::SubmitClassified;
        }
    }

    let maybe_event = runtime.apply_server_message(message).ok().flatten();
    publish_event_status(runtime, maybe_event);
    if maybe_event == Some(LiveRuntimeEvent::WorkQueued) {
        return SocketLineOutcome::WorkQueued;
    }
    if maybe_event == Some(LiveRuntimeEvent::WorkInvalidated) {
        return SocketLineOutcome::SessionInvalidated;
    }
    SocketLineOutcome::None
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

fn publish_share_submission_status(classification: SubmitClassification) {
    let label = match classification {
        SubmitClassification::Accepted => "accepted",
        SubmitClassification::Rejected { .. } => "rejected",
        SubmitClassification::Blocked { .. }
        | SubmitClassification::Malformed
        | SubmitClassification::Reconnect
        | SubmitClassification::Timeout
        | SubmitClassification::NoObservedShare
        | SubmitClassification::Stopped => "blocked_safe_prerequisite",
    };
    info_retained(&format!("share_submission_status={label} redacted=true"));
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
    let url = stored_string(snapshot, "stratumurl").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let username = stored_string(snapshot, "stratumuser").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let password = stored_string(snapshot, "stratumpass").ok_or(POOL_SETTINGS_UNAVAILABLE)?;
    let endpoint = pool_endpoint_from_snapshot(snapshot, &url)?;

    Ok(PoolSettings {
        endpoint,
        runtime_config: LiveRuntimeConfig {
            model: PHASE25_MODEL.to_owned(),
            version: PHASE25_VERSION.to_owned(),
            credentials: LivePoolCredentials { username, password },
        },
    })
}

fn pool_endpoint_from_snapshot(
    snapshot: &NvsSnapshot,
    url: &str,
) -> Result<PoolEndpoint, &'static str> {
    if let Ok(endpoint) = parse_pool_endpoint(url) {
        return Ok(endpoint);
    }

    let host = pool_host_from_url(url)?;
    let port = stored_u16(snapshot, "stratumport").ok_or(POOL_SETTINGS_INVALID)?;
    Ok(PoolEndpoint { host, port })
}

fn pool_host_from_url(url: &str) -> Result<String, &'static str> {
    let without_scheme = url
        .strip_prefix(STRATUM_PLUS_TCP_PREFIX)
        .or_else(|| url.strip_prefix(TCP_PREFIX))
        .unwrap_or(url);
    let host = without_scheme
        .split(':')
        .next()
        .unwrap_or(without_scheme)
        .trim();
    if host.is_empty() {
        return Err(POOL_SETTINGS_INVALID);
    }

    Ok(host.to_owned())
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

fn stored_u16(snapshot: &NvsSnapshot, key: &str) -> Option<u16> {
    let value = snapshot.maybe_stored_value(key)?;
    let StoredValueKind::U16(value) = value.value else {
        return None;
    };

    Some(value)
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
        Ok(FirmwareTcpSocket {
            stream,
            read_buffer: Vec::new(),
        })
    }
}

struct FirmwareTcpSocket {
    stream: TcpStream,
    read_buffer: Vec<u8>,
}

impl LiveSocketIo for FirmwareTcpSocket {
    fn write_json_line(&mut self, line: &str) -> anyhow::Result<()> {
        self.stream.write_all(line.as_bytes())?;
        Ok(())
    }

    fn maybe_read_json_line(&mut self) -> anyhow::Result<Option<String>> {
        if let Some(line) = maybe_pop_json_line(&mut self.read_buffer)? {
            return Ok(Some(line));
        }

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
            anyhow::bail!("stratum socket closed");
        }

        self.read_buffer.extend_from_slice(&buffer[..bytes_read]);
        maybe_pop_json_line(&mut self.read_buffer)
    }

    fn shutdown_both(&mut self) {
        let _result = self.stream.shutdown(Shutdown::Both);
    }
}

fn maybe_pop_json_line(read_buffer: &mut Vec<u8>) -> anyhow::Result<Option<String>> {
    let Some(line_end) = read_buffer.iter().position(|byte| *byte == b'\n') else {
        if read_buffer.len() > MAX_JSON_LINE_BYTES {
            anyhow::bail!("stratum JSON line exceeded maximum length");
        }
        return Ok(None);
    };

    if line_end > MAX_JSON_LINE_BYTES {
        anyhow::bail!("stratum JSON line exceeded maximum length");
    }

    let remainder = read_buffer.split_off(line_end + 1);
    let mut line_bytes = std::mem::replace(read_buffer, remainder);
    line_bytes.truncate(line_end);
    if line_bytes.ends_with(b"\r") {
        let _removed = line_bytes.pop();
    }

    String::from_utf8(line_bytes)
        .map(Some)
        .map_err(|_| anyhow::anyhow!("stratum JSON line was not valid UTF-8"))
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
    fn pool_settings_snapshot_is_complete_when_required_keys_exist() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string("stratumurl", "public-pool.io"),
            StoredValue::string("stratumuser", "redacted-user"),
            StoredValue::string("stratumpass", "redacted-secret"),
            StoredValue::u16("stratumport", 3333),
        ]);

        // Act / Assert
        assert!(pool_settings_from_snapshot(&snapshot).is_ok());
    }

    #[test]
    fn phase27_pool_settings_consumed_marker_is_redacted() {
        // Assert
        assert_eq!(
            PHASE27_POOL_SETTINGS_CONSUMED_MARKER,
            "phase21_pool_settings_consumed=true source=settings_patch redacted=true"
        );
    }

    #[test]
    fn pool_settings_use_stratumport_when_url_has_no_port() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string("stratumurl", "public-pool.io"),
            StoredValue::string("stratumuser", "redacted-user"),
            StoredValue::string("stratumpass", "redacted-secret"),
            StoredValue::u16("stratumport", 3333),
        ]);

        // Act
        let settings = pool_settings_from_snapshot(&snapshot).expect("settings should parse");

        // Assert
        assert_eq!(settings.endpoint.host, "public-pool.io");
        assert_eq!(settings.endpoint.port, 3333);
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
    fn socket_read_error_publishes_reconnect_before_fallback_stop() {
        // Arrange
        runtime_snapshot::reset_command_visible_state_for_test();
        let mut runtime = LiveStratumRuntime::new(LiveRuntimeConfig {
            model: PHASE25_MODEL.to_owned(),
            version: PHASE25_VERSION.to_owned(),
            credentials: LivePoolCredentials {
                username: "redacted-user".to_owned(),
                password: "redacted-secret".to_owned(),
            },
        });
        let _event = runtime.start();
        let socket_state = Rc::new(RefCell::new(ScriptedSocketState::new_with_reads([
            ScriptedRead::Error("stratum socket closed"),
        ])));
        let mut socket = ScriptedSocket {
            state: socket_state.clone(),
        };

        // Act
        let stop_reason = pump_live_socket_until_cleanup(&mut runtime, &mut socket);
        let mining = runtime_snapshot::mining_runtime_state();
        let retained_logs = log_buffer::retained_log_buffer().download_chunks().join("");

        // Assert
        assert_eq!(stop_reason, SafeStopReason::FallbackExhausted);
        assert_eq!(mining.lifecycle, PoolLifecycleStatus::Reconnecting);
        assert_eq!(mining.counters.accepted, 0);
        assert_eq!(mining.counters.rejected, 0);
        assert!(socket_state.borrow().reads.is_empty());
        assert!(retained_logs.contains("phase25_live_stratum_status=reconnecting"));
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

    #[test]
    fn fragmented_json_line_waits_for_newline_before_parsing() {
        // Arrange
        let mut read_buffer = br#"{"id":1"#.to_vec();

        // Act
        let maybe_first_read =
            maybe_pop_json_line(&mut read_buffer).expect("partial line should not fail");
        read_buffer.extend_from_slice(br#","result":true,"error":null}"#);
        let maybe_second_read =
            maybe_pop_json_line(&mut read_buffer).expect("unterminated line should not fail");
        read_buffer.extend_from_slice(b"\n");
        let maybe_complete_line =
            maybe_pop_json_line(&mut read_buffer).expect("complete line should parse");

        // Assert
        assert_eq!(maybe_first_read, None);
        assert_eq!(maybe_second_read, None);
        assert_eq!(
            maybe_complete_line,
            Some(r#"{"id":1,"result":true,"error":null}"#.to_owned())
        );
        assert!(read_buffer.is_empty());
    }

    #[test]
    fn coalesced_server_messages_preserve_remainder_for_next_read() {
        // Arrange
        let mut read_buffer = br#"{"id":1,"result":true,"error":null}
{"id":2,"result":false,"error":null}
"#
        .to_vec();

        // Act
        let first_line = maybe_pop_json_line(&mut read_buffer)
            .expect("first line should parse")
            .expect("first line should be present");
        let second_line = maybe_pop_json_line(&mut read_buffer)
            .expect("second line should parse")
            .expect("second line should be present");

        // Assert
        assert_eq!(first_line, r#"{"id":1,"result":true,"error":null}"#);
        assert_eq!(second_line, r#"{"id":2,"result":false,"error":null}"#);
        assert!(read_buffer.is_empty());
    }

    #[test]
    fn coalesced_submit_response_updates_projection_and_preserves_next_message() {
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
        let mut read_buffer = br#"{"id":7,"result":true,"error":null}
{"id":null,"method":"mining.set_difficulty","params":[42]}
"#
        .to_vec();

        // Act
        let submit_response = maybe_pop_json_line(&mut read_buffer)
            .expect("submit response should parse")
            .expect("submit response should be present");
        let consumed = handle_socket_line(&mut runtime, &submit_response, Some(&pending_submit));
        let next_message = maybe_pop_json_line(&mut read_buffer)
            .expect("next message should parse")
            .expect("next message should be present");
        let mining = runtime_snapshot::mining_runtime_state();

        // Assert
        assert!(consumed);
        assert_eq!(mining.counters.accepted, 1);
        assert_eq!(mining.counters.rejected, 0);
        assert!(next_message.contains(r#""method":"mining.set_difficulty""#));
        assert!(read_buffer.is_empty());
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
        reads: VecDeque<ScriptedRead>,
        writes: Vec<String>,
        shutdown_called: bool,
    }

    impl ScriptedSocketState {
        fn new<const N: usize>(reads: [&'static str; N]) -> Self {
            Self::new_with_reads(reads.map(ScriptedRead::Line))
        }

        fn new_with_reads<const N: usize>(reads: [ScriptedRead; N]) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                writes: Vec::new(),
                shutdown_called: false,
            }
        }
    }

    enum ScriptedRead {
        Line(&'static str),
        Error(&'static str),
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
            let Some(read) = self.state.borrow_mut().reads.pop_front() else {
                return Ok(None);
            };

            match read {
                ScriptedRead::Line(line) => Ok(Some(line.to_owned())),
                ScriptedRead::Error(message) => anyhow::bail!(message),
            }
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
