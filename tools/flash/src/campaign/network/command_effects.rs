use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bitaxe_api::{
    DisplayFrameKind, DisplayRenderOutcome, SystemInfoWire, COMMAND_STATUS_SCHEMA,
    IDENTIFY_DURATION_MS,
};
use bitaxe_http_transport::{PlainWebSocket, StrictHttpClient};
use camino::Utf8Path;

use super::super::CampaignTerminalCategory;
use super::command_evidence::CommandEffectsEvidence;
use super::command_witness::CommandTransitionWitness;
use super::model::{
    CampaignNetworkEvidence, CommandFailureCause, CommandFailureDiagnostic, SharedSerialState,
    TrustedNetworkTarget,
};
use super::validation::{
    active_mining_state_valid, validate_identity, validate_identity_and_safety,
};
use crate::write_private_new_bytes;

mod pause_join;
use pause_join::{PauseJoinDecision, PauseJoinState};
mod failure_diagnostic;
use failure_diagnostic::record_command_failure;
mod programmatic;
use programmatic::advance_programmatic_commands;
#[cfg(test)]
mod legacy;
#[cfg(test)]
use legacy::advance_commands;
#[cfg(test)]
mod paused_dismiss;
#[cfg(test)]
use paused_dismiss::{arm_ready_after_paused_dismissal, begin_paused_dismissal};
mod recovery_join;
use recovery_join::{RecoveryJoinDecision, RecoveryPauseJoinState};
mod witness_continuity;
use witness_continuity::consume_optional_websocket_read;
mod status_reads;
use status_reads::{fetch_command_status, fetch_system_info};
mod request;
use request::{
    command_state_failure_cause, may_reuse_confirmed_safe_stop, post_may_have_applied,
    serial_ended_before_terminal, take_recovery_pause_request, terminal_confirmation_timed_out,
};
mod identify;
#[cfg(test)]
use identify::{
    consume_checkpoint_response, rendered_checkpoint_action, write_required_checkpoint,
    CheckpointResponse, RenderedCheckpointAction,
};
pub(crate) use identify::{
    respond_identify_checkpoint, IdentifyCheckpointKind, IdentifyCheckpointOutcome,
};

const HTTP_DEADLINE: Duration = Duration::from_secs(3);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const AUTOMATED_PHASE_DEADLINE: Duration = Duration::from_secs(15);
const REACTIVATION_DEADLINE: Duration = Duration::from_secs(180);
const NOTIFICATION_DEADLINE: Duration = Duration::from_secs(600);
const TERMINAL_DEADLINE: Duration = Duration::from_secs(15);
const WEBSOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const WEBSOCKET_IO_TIMEOUT: Duration = Duration::from_millis(250);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandPhase {
    Notification,
    #[cfg(test)]
    Pause(PauseJoinState),
    #[cfg(test)]
    ResumeIntent,
    #[cfg(test)]
    ResumeActive,
    #[cfg(test)]
    IdentifyReady,
    #[cfg(test)]
    IdentifyRendered {
        effect_inactive_at: Instant,
    },
    #[cfg(test)]
    IdentifyReplayPending {
        starts_at: Instant,
    },
    #[cfg(test)]
    IdentifyReplayed {
        effect_inactive_at: Instant,
    },
    #[cfg(test)]
    IdentifyObserved {
        clears_at: Instant,
    },
    #[cfg(test)]
    IdentifyCleared,
    #[cfg(test)]
    PausedDismiss,
    ProgrammaticPause(PauseJoinState),
    ProgrammaticDismiss,
    ProgrammaticIdentifyStart,
    ProgrammaticIdentifyRendered,
    ProgrammaticIdentifyCleared,
    ProgrammaticResumeIntent,
    ProgrammaticResumeActive,
    Terminal,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CommandGenerations {
    pause: u64,
    dismiss: u64,
    identify: u64,
    resume: u64,
}

struct CommandProgress<'a> {
    phase: &'a mut CommandPhase,
    maybe_block_count: &'a mut Option<u64>,
    evidence: &'a mut CommandEffectsEvidence,
    maybe_failure: &'a mut Option<CampaignTerminalCategory>,
}

pub(super) fn observe_command_effects(
    target: TrustedNetworkTarget,
    shared: Arc<Mutex<SharedSerialState>>,
    evidence_root: &Utf8Path,
) -> CampaignNetworkEvidence {
    let Ok(http) = StrictHttpClient::new(&target.origin) else {
        request_network_stop(&shared);
        return CampaignNetworkEvidence::from_unobserved(&shared);
    };
    let mut evidence = CommandEffectsEvidence::new();
    let mut maybe_failure = None;
    let mut maybe_failure_diagnostic = None;
    let mut phase = CommandPhase::Notification;
    let mut maybe_block_count = None;
    let mut maybe_terminal_deadline = None;
    let mut recovery_pause_request_count = 0;
    let mut maybe_recovery_join = None;
    let mut phase_started_at = Instant::now();
    let mut generations = CommandGenerations::default();
    let mut maybe_log_websocket = None;
    let mut websocket_transitions = CommandTransitionWitness::default();
    let mut websocket_pending = Vec::new();

    loop {
        let now = Instant::now();
        let serial = shared_snapshot(&shared);
        if let Some(category) = serial.maybe_failure {
            record_command_failure(
                &mut maybe_failure,
                &mut maybe_failure_diagnostic,
                phase,
                CommandFailureCause::SerialWitness,
                category,
            );
        }
        evidence.terminal_pool_persisted = serial.terminal_pool_persisted;
        if serial.terminal_consumed && maybe_terminal_deadline.is_none() {
            maybe_terminal_deadline = Some(Instant::now() + TERMINAL_DEADLINE);
        }
        if matches!(phase, CommandPhase::ProgrammaticPause(join) if join.expired(Instant::now())) {
            record_command_failure(
                &mut maybe_failure,
                &mut maybe_failure_diagnostic,
                phase,
                CommandFailureCause::PhaseDeadline,
                CampaignTerminalCategory::NetworkCorrelationFailed,
            );
        }
        if maybe_failure.is_none() {
            if let Some(category) = automated_phase_failure(phase, phase_started_at, now) {
                record_command_failure(
                    &mut maybe_failure,
                    &mut maybe_failure_diagnostic,
                    phase,
                    CommandFailureCause::PhaseDeadline,
                    category,
                );
            }
        }

        if maybe_failure.is_none() {
            if maybe_log_websocket.is_none() {
                // WebSocket and receive-only USB are independent witnesses.
                // Connection availability is therefore bounded by the phase
                // deadline, while malformed witness data still fails closed.
                if let Ok(socket) = PlainWebSocket::connect(
                    &target.origin,
                    "/api/ws",
                    WEBSOCKET_CONNECT_TIMEOUT,
                    WEBSOCKET_IO_TIMEOUT,
                ) {
                    maybe_log_websocket = Some(socket);
                }
            }
            if let Some(websocket) = maybe_log_websocket.as_mut() {
                match consume_optional_websocket_read(
                    websocket.read(),
                    &target.boot_session,
                    &mut websocket_pending,
                    &mut websocket_transitions,
                ) {
                    Ok(true) => {}
                    Ok(false) => {
                        maybe_log_websocket = None;
                    }
                    Err(()) => {
                        record_command_failure(
                            &mut maybe_failure,
                            &mut maybe_failure_diagnostic,
                            phase,
                            CommandFailureCause::WebsocketWitness,
                            CampaignTerminalCategory::NetworkCorrelationFailed,
                        );
                    }
                }
            }
        }

        if maybe_failure.is_none() {
            let system_info = fetch_system_info(&http);
            let command_status = fetch_command_status(&http);
            match (system_info, command_status) {
                (Ok(Some(sample)), Ok(Some(status))) => {
                    if validate_command_sample(phase, &sample, &target).is_err() {
                        evidence.same_boot_and_package = false;
                        evidence.safety_valid = false;
                        record_command_failure(
                            &mut maybe_failure,
                            &mut maybe_failure_diagnostic,
                            phase,
                            CommandFailureCause::HttpSampleValidation,
                            CampaignTerminalCategory::NetworkCorrelationFailed,
                        );
                    } else {
                        let prior_phase = phase;
                        advance_programmatic_commands(
                            &http,
                            &target,
                            evidence_root,
                            &sample,
                            &status,
                            &serial,
                            &websocket_transitions,
                            maybe_log_websocket.is_some(),
                            Instant::now(),
                            &mut generations,
                            CommandProgress {
                                phase: &mut phase,
                                maybe_block_count: &mut maybe_block_count,
                                evidence: &mut evidence,
                                maybe_failure: &mut maybe_failure,
                            },
                        );
                        if maybe_failure.is_some() && maybe_failure_diagnostic.is_none() {
                            let cause = maybe_failure.map_or(
                                CommandFailureCause::CommandStateMachine,
                                command_state_failure_cause,
                            );
                            maybe_failure_diagnostic = Some(CommandFailureDiagnostic::new(
                                prior_phase.diagnostic_phase(),
                                cause,
                            ));
                        }
                        if phase != prior_phase {
                            phase_started_at = now;
                        }
                        if serial.terminal_consumed
                            && phase == CommandPhase::Terminal
                            && sample.mining_paused
                            && sample.mining_activity == "paused"
                            && !sample.start_mining_on_boot
                        {
                            evidence.terminal_http_valid = true;
                        }
                    }
                }
                (Err(category), _) => record_command_failure(
                    &mut maybe_failure,
                    &mut maybe_failure_diagnostic,
                    phase,
                    CommandFailureCause::HttpSystemInfo,
                    category,
                ),
                (_, Err(category)) => record_command_failure(
                    &mut maybe_failure,
                    &mut maybe_failure_diagnostic,
                    phase,
                    CommandFailureCause::HttpCommandStatus,
                    category,
                ),
                (Ok(None), _) | (_, Ok(None)) => {}
            }
        }
        if may_reuse_confirmed_safe_stop(maybe_failure, &evidence, &serial) {
            evidence.recovery_pause_api_confirmed = true;
            evidence.recovery_pause_serial_confirmed = true;
            evidence.recovery_safe_stop_confirmed = true;
            evidence.recovery_terminal_outcome = "already_confirmed";
            request_network_stop(&shared);
            break;
        }
        if take_recovery_pause_request(maybe_failure, &mut recovery_pause_request_count) {
            // Sample before the blocking POST so a safe-stop marker emitted
            // while the request is in flight still counts as post-request.
            let pre_request_serial_observation_count =
                serial.resumable_pause_safe_stop_observation_count;
            evidence.recovery_terminal_outcome = "pending";
            if post_may_have_applied(http.post_pause_once(Instant::now() + HTTP_DEADLINE)) {
                maybe_recovery_join = Some(RecoveryPauseJoinState::new(
                    Instant::now(),
                    pre_request_serial_observation_count,
                ));
            } else {
                evidence.recovery_terminal_outcome = "request_failed";
                request_network_stop(&shared);
                break;
            }
        }

        if let Some(join) = maybe_recovery_join.as_mut() {
            let api_pause_confirmed = match fetch_system_info(&http) {
                Ok(Some(sample)) if validate_identity(&sample, &target).is_ok() => {
                    sample.mining_paused
                        && sample.mining_activity == "paused"
                        && !sample.start_mining_on_boot
                }
                Ok(Some(_)) => {
                    evidence.same_boot_and_package = false;
                    evidence.safety_valid = false;
                    false
                }
                Ok(None) | Err(_) => false,
            };
            let decision = join.observe(
                api_pause_confirmed,
                serial.resumable_pause_safe_stop_observation_count,
                Instant::now(),
            );
            evidence.recovery_pause_api_confirmed = join.api_pause_confirmed();
            evidence.recovery_pause_serial_confirmed = join.serial_safe_stop_confirmed();
            match decision {
                RecoveryJoinDecision::Wait if serial.serial_finished => {
                    evidence.recovery_terminal_outcome = "serial_finished";
                    request_network_stop(&shared);
                    break;
                }
                RecoveryJoinDecision::Wait => {
                    std::thread::sleep(POLL_INTERVAL);
                    continue;
                }
                RecoveryJoinDecision::Ready => {
                    evidence.recovery_safe_stop_confirmed = true;
                    evidence.recovery_terminal_outcome = "confirmed";
                    request_network_stop(&shared);
                    break;
                }
                RecoveryJoinDecision::TimedOut => {
                    evidence.recovery_terminal_outcome = "timed_out";
                    request_network_stop(&shared);
                    break;
                }
            }
        }

        if serial.terminal_consumed && evidence.terminal_http_valid {
            break;
        }
        if terminal_confirmation_timed_out(maybe_terminal_deadline, Instant::now()) {
            record_command_failure(
                &mut maybe_failure,
                &mut maybe_failure_diagnostic,
                phase,
                CommandFailureCause::TerminalDeadline,
                CampaignTerminalCategory::TerminalStateUnconfirmed,
            );
            break;
        }
        if serial_ended_before_terminal(&serial) {
            record_command_failure(
                &mut maybe_failure,
                &mut maybe_failure_diagnostic,
                phase,
                CommandFailureCause::SerialEnded,
                CampaignTerminalCategory::TerminalStateUnconfirmed,
            );
            break;
        }
        std::thread::sleep(POLL_INTERVAL);
    }

    if let Some(websocket) = maybe_log_websocket.as_mut() {
        websocket.close();
    }

    CampaignNetworkEvidence::from_command_effects(
        evidence,
        recovery_pause_request_count,
        maybe_failure,
        maybe_failure_diagnostic,
    )
}

fn validate_command_sample(
    phase: CommandPhase,
    sample: &SystemInfoWire,
    target: &TrustedNetworkTarget,
) -> Result<(), super::validation::SampleValidationFailure> {
    let active_safety_required =
        phase == CommandPhase::Notification || active_mining_state_valid(sample);
    if active_safety_required {
        return validate_identity_and_safety(sample, target);
    }
    validate_identity(sample, target)
}

fn automated_phase_failure(
    phase: CommandPhase,
    started_at: Instant,
    now: Instant,
) -> Option<CampaignTerminalCategory> {
    let (maybe_limit, category) = match phase {
        CommandPhase::Notification => (
            Some(NOTIFICATION_DEADLINE),
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        #[cfg(test)]
        CommandPhase::ResumeIntent => (
            Some(AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::ResumeIntentUnconfirmed,
        ),
        #[cfg(test)]
        CommandPhase::ResumeActive => (
            Some(REACTIVATION_DEADLINE),
            CampaignTerminalCategory::ResumeReactivationTimedOut,
        ),
        // Command completion does not consume the firmware lease. The serial
        // owner must remain free to accumulate the rest of its admitted active
        // duration before publishing the terminal marker. The outer child
        // deadline bounds that wait, and TERMINAL_DEADLINE separately bounds
        // HTTP confirmation after the marker is consumed.
        CommandPhase::Terminal => return None,
        #[cfg(test)]
        CommandPhase::PausedDismiss => (
            Some(AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        #[cfg(test)]
        CommandPhase::Pause(_)
        | CommandPhase::IdentifyReady
        | CommandPhase::IdentifyRendered { .. }
        | CommandPhase::IdentifyReplayPending { .. }
        | CommandPhase::IdentifyReplayed { .. }
        | CommandPhase::IdentifyObserved { .. }
        | CommandPhase::IdentifyCleared => {
            (None, CampaignTerminalCategory::NetworkCorrelationFailed)
        }
        CommandPhase::ProgrammaticPause(_) | CommandPhase::ProgrammaticDismiss => (
            Some(AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        CommandPhase::ProgrammaticIdentifyStart | CommandPhase::ProgrammaticIdentifyRendered => (
            Some(AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        CommandPhase::ProgrammaticIdentifyCleared => (
            Some(Duration::from_millis(IDENTIFY_DURATION_MS) + AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        CommandPhase::ProgrammaticResumeIntent => (
            Some(AUTOMATED_PHASE_DEADLINE),
            CampaignTerminalCategory::ResumeIntentUnconfirmed,
        ),
        CommandPhase::ProgrammaticResumeActive => (
            Some(REACTIVATION_DEADLINE),
            CampaignTerminalCategory::ResumeReactivationTimedOut,
        ),
    };
    maybe_limit
        .is_some_and(|limit| now.duration_since(started_at) >= limit)
        .then_some(category)
}

fn write_reboot_intent(
    root: &Utf8Path,
    target: &TrustedNetworkTarget,
    sample: &SystemInfoWire,
) -> anyhow::Result<()> {
    let intent = serde_json::json!({
        "schema_version": "esp-device-session-reboot-intent-v1",
        "board_category": "205",
        "trusted_origin": target.origin,
        "baseline": {
            "boot_session": target.boot_session,
            "boot_ordinal": target.boot_ordinal,
            "source_commit": target.expected.firmware_commit,
            "reference_commit": target.expected.reference_commit,
            "app_elf_sha256": target.expected.app_elf_sha256,
        },
        "expected_postcondition": {
            "hostname_sha256": crate::sha256_bytes(sample.hostname.as_bytes()),
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&intent)?;
    bytes.push(b'\n');
    write_private_new_bytes(
        &root.join("command-effects-reboot-intent.private.json"),
        &bytes,
    )
}

#[cfg(test)]
fn finish_identify_observation(
    clears_at: Instant,
    phase: &mut CommandPhase,
    evidence: &mut CommandEffectsEvidence,
    maybe_failure: &mut Option<CampaignTerminalCategory>,
) {
    let Some(expected_before_clear) = evidence.identify_replay_request_count.checked_add(1) else {
        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
        return;
    };
    if evidence.identify_request_count != expected_before_clear {
        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
        return;
    }
    evidence.identify_rendered_confirmed = true;
    *phase = CommandPhase::IdentifyObserved { clears_at };
}

#[cfg(test)]
fn arm_cleared_after_natural_expiry(
    root: &Utf8Path,
    now: Instant,
    clears_at: Instant,
) -> Result<bool, ()> {
    if now < clears_at {
        return Ok(false);
    }
    write_required_checkpoint(root, IdentifyCheckpointKind::Cleared).map_err(|_| ())?;
    Ok(true)
}

#[cfg(test)]
fn arm_identify_transaction(
    root: &Utf8Path,
    evidence: &CommandEffectsEvidence,
) -> Result<CommandPhase, ()> {
    if evidence.identify_request_count != 0 || evidence.identify_operator_ready_confirmed {
        return Err(());
    }
    write_required_checkpoint(root, IdentifyCheckpointKind::Ready).map_err(|_| ())?;
    Ok(CommandPhase::IdentifyReady)
}

#[cfg(test)]
fn consume_ready_signal(
    root: &Utf8Path,
    evidence: &mut CommandEffectsEvidence,
) -> Result<CheckpointResponse, ()> {
    if !evidence.pause_confirmed
        || evidence.resume_request_count != 0
        || evidence.identify_request_count != 0
    {
        return Err(());
    }
    let response = consume_checkpoint_response(root, IdentifyCheckpointKind::Ready)?;
    if response != CheckpointResponse::Confirmed {
        return Ok(response);
    }
    evidence.identify_operator_ready_confirmed = true;
    Ok(CheckpointResponse::Confirmed)
}

#[cfg(test)]
fn consume_cleared_signal(
    root: &Utf8Path,
    evidence: &mut CommandEffectsEvidence,
) -> Result<CheckpointResponse, ()> {
    if !evidence.pause_confirmed
        || !evidence.identify_operator_ready_confirmed
        || !evidence.identify_rendered_confirmed
        || evidence.identify_replay_request_count > 1
        || evidence.identify_request_count != 1 + evidence.identify_replay_request_count
        || evidence.resume_request_count != 0
    {
        return Err(());
    }
    let response = consume_checkpoint_response(root, IdentifyCheckpointKind::Cleared)?;
    if response != CheckpointResponse::Confirmed {
        return Ok(response);
    }
    evidence.identify_cleared_confirmed = true;
    evidence.resume_request_count = 1;
    Ok(CheckpointResponse::Confirmed)
}

fn request_network_stop(shared: &Arc<Mutex<SharedSerialState>>) {
    if let Ok(mut state) = shared.lock() {
        state.network_stop_requested = true;
    }
}

fn shared_snapshot(shared: &Arc<Mutex<SharedSerialState>>) -> SharedSerialState {
    shared.lock().map_or_else(
        |_| SharedSerialState {
            serial_finished: true,
            maybe_failure: Some(CampaignTerminalCategory::NetworkCorrelationFailed),
            ..SharedSerialState::default()
        },
        |state| state.clone(),
    )
}

#[cfg(test)]
mod tests;
