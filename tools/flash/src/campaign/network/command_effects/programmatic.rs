use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn advance_programmatic_commands(
    http: &StrictHttpClient,
    target: &TrustedNetworkTarget,
    evidence_root: &Utf8Path,
    sample: &SystemInfoWire,
    status: &CommandStatusWire,
    serial: &SharedSerialState,
    websocket: &CommandTransitionWitness,
    websocket_connected: bool,
    now: Instant,
    generations: &mut CommandGenerations,
    progress: CommandProgress<'_>,
) {
    let CommandProgress {
        phase,
        maybe_block_count,
        evidence,
        maybe_failure,
    } = progress;
    if status.schema != COMMAND_STATUS_SCHEMA
        || status.boot_session.to_string() != target.boot_session
    {
        *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
        return;
    }

    match phase {
        CommandPhase::Notification
            if websocket_connected
                && active_mining_state_valid(sample)
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
            generations.pause = status.mining.pause_generation.saturating_add(1);
            evidence.pause_request_count = 1;
            if post_succeeded(http.post_pause_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::ProgrammaticPause(PauseJoinState::new(now));
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::ProgrammaticPause(join) => {
            let transition_confirmed = status.mining.pause_generation == generations.pause;
            match join.observe(
                sample.mining_paused && sample.mining_activity == "paused" && transition_confirmed,
                serial.resumable_pause_safe_stop_confirmed,
                now,
            ) {
                PauseJoinDecision::Wait => {}
                PauseJoinDecision::Ready => {
                    let paused_block_count = sample.block_found;
                    if paused_block_count == 0
                        || evidence.pause_request_count != 1
                        || evidence.dismiss_request_count != 0
                    {
                        *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                        return;
                    }
                    evidence.pause_confirmed = true;
                    evidence.dismiss_request_count = 1;
                    *maybe_block_count = Some(paused_block_count);
                    generations.dismiss = status
                        .block_notification
                        .dismiss_generation
                        .saturating_add(1);
                    if post_succeeded(
                        http.post_block_found_dismiss_once(Instant::now() + HTTP_DEADLINE),
                    ) {
                        *phase = CommandPhase::ProgrammaticDismiss;
                    } else {
                        *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
                    }
                }
                PauseJoinDecision::TimedOut => {
                    *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed)
                }
            }
        }
        CommandPhase::ProgrammaticDismiss if !sample.show_new_block => {
            let preserved = maybe_block_count.is_some_and(|count| sample.block_found == count);
            let witnessed = status.block_notification.dismiss_generation == generations.dismiss;
            if !preserved || !witnessed {
                return;
            }
            evidence.dismiss_confirmed = true;
            evidence.block_count_preserved = true;
            *phase = CommandPhase::ProgrammaticIdentifyStart;
        }
        CommandPhase::ProgrammaticIdentifyStart => {
            if status.identify.active || !status.display.available {
                *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                return;
            }
            evidence.identify_status_baseline_confirmed = true;
            generations.identify = status.identify.generation.saturating_add(1);
            evidence.identify_request_count = 1;
            if post_succeeded(http.post_identify_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::ProgrammaticIdentifyRendered;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::ProgrammaticIdentifyRendered => {
            let serial_marker = serial.command_transitions.identify_generation
                >= generations.identify
                && serial.command_transitions.display_identify_generation >= generations.identify;
            let websocket_marker = websocket.identify_generation >= generations.identify
                && websocket.display_identify_generation >= generations.identify;
            let rendered = status.identify.active
                && status.identify.generation == generations.identify
                && successful_display_receipt(
                    status,
                    DisplayFrameKind::Identify,
                    generations.identify,
                )
                && (serial_marker || websocket_marker);
            if rendered {
                evidence.retained_identify_transition_confirmed = true;
                evidence.identify_render_receipt_confirmed = true;
                *phase = CommandPhase::ProgrammaticIdentifyCleared;
            }
        }
        CommandPhase::ProgrammaticIdentifyCleared => {
            let cleared = !status.identify.active
                && status.identify.generation == generations.identify
                && successful_display_receipt(
                    status,
                    DisplayFrameKind::NonIdentify,
                    generations.identify,
                );
            if !cleared {
                return;
            }
            evidence.identify_clear_receipt_confirmed = true;
            generations.resume = status.mining.resume_generation.saturating_add(1);
            evidence.resume_request_count = 1;
            if post_succeeded(http.post_resume_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::ProgrammaticResumeIntent;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::ProgrammaticResumeIntent
            if !sample.mining_paused && status.mining.resume_generation == generations.resume =>
        {
            evidence.resume_intent_confirmed = true;
            evidence.serial_transition_witnesses_confirmed =
                transition_witnesses_complete(&serial.command_transitions, *generations);
            evidence.websocket_transition_witnesses_confirmed =
                transition_witnesses_complete(websocket, *generations);
            *phase = CommandPhase::ProgrammaticResumeActive;
        }
        CommandPhase::ProgrammaticResumeActive if active_mining_state_valid(sample) => {
            evidence.resume_confirmed = true;
            evidence.active_after_resume = true;
            *phase = CommandPhase::Terminal;
        }
        _ => {}
    }
}

fn successful_display_receipt(
    status: &CommandStatusWire,
    frame_kind: DisplayFrameKind,
    identify_generation: u64,
) -> bool {
    status
        .display
        .maybe_last_success
        .as_ref()
        .is_some_and(|receipt| {
            receipt.frame_kind == frame_kind
                && receipt.identify_generation == identify_generation
                && receipt.outcome == DisplayRenderOutcome::Rendered
        })
}

fn transition_witnesses_complete(
    witness: &CommandTransitionWitness,
    generations: CommandGenerations,
) -> bool {
    witness.pause_generation >= generations.pause
        && witness.dismiss_generation >= generations.dismiss
        && witness.identify_generation >= generations.identify
        && witness.display_identify_generation >= generations.identify
        && witness.display_non_identify_generation >= generations.identify
        && witness.resume_generation >= generations.resume
}
