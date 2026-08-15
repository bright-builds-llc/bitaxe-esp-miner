use super::*;

#[cfg(test)]
pub(super) fn advance_commands(
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
                PauseJoinDecision::Ready => match begin_paused_dismissal(
                    http,
                    evidence,
                    maybe_block_count,
                    sample.block_found,
                ) {
                    Ok(next_phase) => *phase = next_phase,
                    Err(category) => *maybe_failure = Some(category),
                },
                PauseJoinDecision::TimedOut => {
                    *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                }
            }
        }
        CommandPhase::ResumeIntent if !sample.mining_paused => {
            evidence.resume_intent_confirmed = true;
            *phase = CommandPhase::ResumeActive;
        }
        CommandPhase::ResumeActive if active_mining_state_valid(sample) => {
            evidence.resume_confirmed = true;
            evidence.active_after_resume = true;
            *phase = CommandPhase::Terminal;
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
                    *phase = CommandPhase::ResumeIntent;
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
        CommandPhase::PausedDismiss if !sample.show_new_block => {
            evidence.dismiss_confirmed = true;
            evidence.block_count_preserved =
                maybe_block_count.is_some_and(|count| sample.block_found == count);
            if evidence.block_count_preserved {
                match arm_ready_after_paused_dismissal(evidence_root, evidence) {
                    Ok(next_phase) => *phase = next_phase,
                    Err(()) => {
                        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                    }
                }
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
            }
        }
        _ => {}
    }
}
