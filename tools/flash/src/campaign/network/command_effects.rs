use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bitaxe_api::{SystemInfoWire, IDENTIFY_DURATION_MS};
use bitaxe_http_transport::{ExchangeObservation, StrictHttpClient};
use camino::Utf8Path;

use super::super::CampaignTerminalCategory;
use super::command_evidence::CommandEffectsEvidence;
use super::model::{CampaignNetworkEvidence, SharedSerialState, TrustedNetworkTarget};
use super::validation::{active_mining_state_valid, validate_identity_and_safety};
use crate::write_private_new_bytes;

mod pause_join;
use pause_join::{PauseJoinDecision, PauseJoinState};
mod identify;
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
const NOTIFICATION_DEADLINE: Duration = Duration::from_secs(600);
const TERMINAL_DEADLINE: Duration = Duration::from_secs(15);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandPhase {
    Notification,
    Pause(PauseJoinState),
    Resume,
    IdentifyReady,
    IdentifyRendered { effect_inactive_at: Instant },
    IdentifyReplayPending { starts_at: Instant },
    IdentifyReplayed { effect_inactive_at: Instant },
    IdentifyObserved { clears_at: Instant },
    IdentifyCleared,
    Dismiss,
    Terminal,
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
    let mut phase = CommandPhase::Notification;
    let mut maybe_block_count = None;
    let mut maybe_terminal_deadline = None;
    let mut recovery_pause_request_count = 0;
    let mut phase_started_at = Instant::now();

    loop {
        let now = Instant::now();
        let serial = shared_snapshot(&shared);
        if maybe_failure.is_none() {
            maybe_failure = serial.maybe_failure;
        }
        evidence.terminal_pool_persisted = serial.terminal_pool_persisted;
        if serial.terminal_consumed && maybe_terminal_deadline.is_none() {
            maybe_terminal_deadline = Some(Instant::now() + TERMINAL_DEADLINE);
        }
        if matches!(phase, CommandPhase::Pause(join) if join.expired(Instant::now())) {
            maybe_failure.get_or_insert(CampaignTerminalCategory::NetworkCorrelationFailed);
        }
        if maybe_failure.is_none() && automated_phase_expired(phase, phase_started_at, now) {
            maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
        }

        if maybe_failure.is_none() {
            match fetch_system_info(&http) {
                Ok(Some(sample)) => {
                    if validate_identity_and_safety(&sample, &target).is_err() {
                        evidence.same_boot_and_package = false;
                        evidence.safety_valid = false;
                        maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                    } else {
                        let prior_phase = phase;
                        advance_commands(
                            &http,
                            &target,
                            evidence_root,
                            &sample,
                            &serial,
                            Instant::now(),
                            CommandProgress {
                                phase: &mut phase,
                                maybe_block_count: &mut maybe_block_count,
                                evidence: &mut evidence,
                                maybe_failure: &mut maybe_failure,
                            },
                        );
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
                Ok(None) => {}
                Err(category) => maybe_failure = Some(category),
            }
        }
        if take_recovery_pause_request(maybe_failure, &mut recovery_pause_request_count) {
            let _result = http.post_pause_once(Instant::now() + HTTP_DEADLINE);
            request_network_stop(&shared);
            break;
        }

        if serial.terminal_consumed && evidence.terminal_http_valid {
            break;
        }
        if maybe_terminal_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            maybe_failure.get_or_insert(CampaignTerminalCategory::TerminalStateUnconfirmed);
            break;
        }
        if serial.serial_finished {
            if !serial.terminal_consumed {
                maybe_failure.get_or_insert(CampaignTerminalCategory::TerminalStateUnconfirmed);
            }
            break;
        }
        std::thread::sleep(POLL_INTERVAL);
    }

    CampaignNetworkEvidence::from_command_effects(
        evidence,
        recovery_pause_request_count,
        maybe_failure,
    )
}

fn automated_phase_expired(phase: CommandPhase, started_at: Instant, now: Instant) -> bool {
    let maybe_limit = match phase {
        CommandPhase::Notification => Some(NOTIFICATION_DEADLINE),
        CommandPhase::Resume | CommandPhase::Dismiss | CommandPhase::Terminal => {
            Some(AUTOMATED_PHASE_DEADLINE)
        }
        CommandPhase::Pause(_)
        | CommandPhase::IdentifyReady
        | CommandPhase::IdentifyRendered { .. }
        | CommandPhase::IdentifyReplayPending { .. }
        | CommandPhase::IdentifyReplayed { .. }
        | CommandPhase::IdentifyObserved { .. }
        | CommandPhase::IdentifyCleared => None,
    };
    maybe_limit.is_some_and(|limit| now.duration_since(started_at) >= limit)
}

fn take_recovery_pause_request(
    maybe_failure: Option<CampaignTerminalCategory>,
    request_count: &mut u64,
) -> bool {
    if maybe_failure.is_none() || *request_count > 0 {
        return false;
    }
    *request_count = 1;
    true
}

fn advance_commands(
    http: &StrictHttpClient,
    target: &TrustedNetworkTarget,
    evidence_root: &Utf8Path,
    sample: &SystemInfoWire,
    serial: &SharedSerialState,
    now: Instant,
    progress: CommandProgress<'_>,
) {
    let CommandProgress {
        phase,
        maybe_block_count,
        evidence,
        maybe_failure,
    } = progress;
    match phase {
        CommandPhase::Notification
            if active_mining_state_valid(sample)
                && sample.show_new_block
                && sample.block_found > 0 =>
        {
            evidence.active_before_pause = true;
            evidence.genuine_block_notification_observed = true;
            evidence.positive_block_count_observed = true;
            *maybe_block_count = Some(sample.block_found);
            if write_reboot_intent(evidence_root, target, sample).is_err() {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
                return;
            }
            evidence.pause_request_count = 1;
            if post_succeeded(http.post_pause_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::Pause(PauseJoinState::new(now));
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::Pause(join) => {
            match join.observe(
                sample.mining_paused && sample.mining_activity == "paused",
                serial.resumable_pause_safe_stop_confirmed,
                now,
            ) {
                PauseJoinDecision::Wait => {}
                PauseJoinDecision::Ready => match arm_ready_after_pause(evidence_root, evidence) {
                    Ok(next_phase) => *phase = next_phase,
                    Err(()) => {
                        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                    }
                },
                PauseJoinDecision::TimedOut => {
                    *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                }
            }
        }
        CommandPhase::Resume if active_mining_state_valid(sample) => {
            evidence.resume_confirmed = true;
            evidence.active_after_resume = true;
            evidence.dismiss_request_count = 1;
            if !post_succeeded(http.post_block_found_dismiss_once(Instant::now() + HTTP_DEADLINE)) {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            } else {
                *phase = CommandPhase::Dismiss;
            }
        }
        CommandPhase::IdentifyReady => match consume_ready_signal(evidence_root, evidence) {
            Ok(CheckpointResponse::Confirmed) => {
                // Ready may wait indefinitely before activation. Keep the
                // safe-stop pause held while the exact 30-second device effect
                // runs and while the operator's bound report is in transit.
                evidence.identify_request_count = 1;
                if !post_succeeded(http.post_identify_once(Instant::now() + HTTP_DEADLINE)) {
                    *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
                } else {
                    let effect_inactive_at =
                        Instant::now() + Duration::from_millis(IDENTIFY_DURATION_MS);
                    if write_required_checkpoint(evidence_root, IdentifyCheckpointKind::Rendered)
                        .is_ok()
                    {
                        *phase = CommandPhase::IdentifyRendered { effect_inactive_at };
                    } else {
                        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
                    }
                }
            }
            Ok(CheckpointResponse::Declined) => {
                evidence.identify_terminal_outcome = "declined";
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointDeclined);
            }
            Ok(CheckpointResponse::Replay) => {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
            }
            Ok(CheckpointResponse::Pending) => {}
            Err(()) => *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid),
        },
        CommandPhase::IdentifyRendered { effect_inactive_at } => {
            match consume_checkpoint_response(evidence_root, IdentifyCheckpointKind::Rendered)
                .and_then(|response| {
                    rendered_checkpoint_action(now, *effect_inactive_at, response, true)
                }) {
                Ok(RenderedCheckpointAction::Wait) => {}
                Ok(RenderedCheckpointAction::Confirmed) => {
                    finish_identify_observation(*effect_inactive_at, phase, evidence, maybe_failure)
                }
                Ok(RenderedCheckpointAction::ReplayAt(starts_at)) => {
                    evidence.identify_replay_request_count = 1;
                    *phase = CommandPhase::IdentifyReplayPending { starts_at };
                }
                Ok(RenderedCheckpointAction::Declined) => {
                    evidence.identify_terminal_outcome = "declined";
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointDeclined);
                }
                Err(()) => {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                }
            }
        }
        CommandPhase::IdentifyReplayPending { starts_at } if now >= *starts_at => {
            if evidence.identify_request_count != 1 || evidence.identify_replay_request_count != 1 {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
            } else {
                if !post_succeeded(http.post_identify_once(Instant::now() + HTTP_DEADLINE)) {
                    *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
                    return;
                }
                evidence.identify_request_count = 2;
                let effect_inactive_at =
                    Instant::now() + Duration::from_millis(IDENTIFY_DURATION_MS);
                if write_required_checkpoint(evidence_root, IdentifyCheckpointKind::Replayed)
                    .is_err()
                {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
                } else {
                    *phase = CommandPhase::IdentifyReplayed { effect_inactive_at };
                }
            }
        }
        CommandPhase::IdentifyReplayed { effect_inactive_at } => {
            match consume_checkpoint_response(evidence_root, IdentifyCheckpointKind::Replayed)
                .and_then(|response| {
                    rendered_checkpoint_action(now, *effect_inactive_at, response, false)
                }) {
                Ok(RenderedCheckpointAction::Wait) => {}
                Ok(RenderedCheckpointAction::Confirmed) => {
                    finish_identify_observation(*effect_inactive_at, phase, evidence, maybe_failure)
                }
                Ok(RenderedCheckpointAction::Declined) => {
                    evidence.identify_terminal_outcome = "declined";
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointDeclined);
                }
                Ok(RenderedCheckpointAction::ReplayAt(_)) | Err(()) => {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                }
            }
        }
        CommandPhase::IdentifyObserved { clears_at } => {
            match arm_cleared_after_natural_expiry(evidence_root, now, *clears_at) {
                Ok(true) => *phase = CommandPhase::IdentifyCleared,
                Ok(false) => {}
                Err(()) => {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                }
            }
        }
        CommandPhase::IdentifyCleared => match consume_cleared_signal(evidence_root, evidence) {
            Ok(CheckpointResponse::Confirmed) => {
                if !post_succeeded(http.post_resume_once(Instant::now() + HTTP_DEADLINE)) {
                    *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
                } else {
                    *phase = CommandPhase::Resume;
                }
            }
            Ok(CheckpointResponse::Declined) => {
                evidence.identify_terminal_outcome = "declined";
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointDeclined);
            }
            Ok(CheckpointResponse::Replay) => {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
            }
            Ok(CheckpointResponse::Pending) => {}
            Err(()) => *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid),
        },
        CommandPhase::Dismiss if !sample.show_new_block => {
            evidence.dismiss_confirmed = true;
            evidence.block_count_preserved =
                maybe_block_count.is_some_and(|count| sample.block_found == count);
            if evidence.block_count_preserved {
                *phase = CommandPhase::Terminal;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
            }
        }
        _ => {}
    }
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

fn fetch_system_info(
    http: &StrictHttpClient,
) -> Result<Option<SystemInfoWire>, CampaignTerminalCategory> {
    let observation = http
        .get_system_info(Instant::now() + HTTP_DEADLINE)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)?;
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| response.status() == 200)
    else {
        return Ok(None);
    };
    serde_json::from_slice(response.body())
        .map(Some)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)
}

fn post_succeeded(result: anyhow::Result<ExchangeObservation>) -> bool {
    result.ok().and_then(|observation| {
        observation
            .maybe_http_response()
            .map(|response| response.status())
    }) == Some(200)
}

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

fn arm_ready_after_pause(
    root: &Utf8Path,
    evidence: &mut CommandEffectsEvidence,
) -> Result<CommandPhase, ()> {
    if evidence.pause_request_count != 1 || evidence.resume_request_count != 0 {
        return Err(());
    }
    evidence.pause_confirmed = true;
    arm_identify_transaction(root, evidence)
}

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
