use super::*;

/// Returns the current identify mode used to plan the next identify command.
pub fn identify_mode() -> IdentifyMode {
    command_visible_state()
        .identify
        .mode_at(crate::runtime_uptime::millis())
}

/// Returns the current block-found notification state.
pub fn block_found_notification_state() -> BlockFoundNotificationState {
    command_visible_state().block_found
}

/// Applies current-boot operator intent without deriving mining state.
pub fn apply_mining_operator_intent_command(effect: MiningOperatorIntentEffect) {
    let transition = mutate_command_visible_state_with_result(None, |state| {
        state.requested_operator_intent.apply(effect);
        apply_mining_operator_intent_effect(&mut state.mining, effect);
        Some(
            state
                .command_status
                .record_command(match effect.next_intent {
                    MiningOperatorIntent::Paused => CommandStatusEffect::Pause,
                    MiningOperatorIntent::Run => CommandStatusEffect::Resume,
                }),
        )
    });
    retain_command_status_transition(transition);
}

pub(crate) fn apply_command_effects_run_bootstrap() {
    mutate_command_visible_state(|state| {
        state
            .requested_operator_intent
            .apply_command_effects_run_bootstrap();
    });
}

/// Applies an API-visible identify command effect.
pub fn apply_identify_mode_command(effect: IdentifyModeEffect) {
    let now_ms = crate::runtime_uptime::millis();
    let transition = mutate_command_visible_state_with_result(None, |state| {
        apply_identify_mode_effect(&mut state.identify, effect, now_ms);
        let status_effect = match effect {
            IdentifyModeEffect::Enable { duration_ms } => CommandStatusEffect::IdentifyEnable {
                expires_at_uptime_ms: now_ms.saturating_add(duration_ms),
            },
            IdentifyModeEffect::Disable => CommandStatusEffect::IdentifyDisable,
        };
        Some(state.command_status.record_command(status_effect))
    });
    retain_command_status_transition(transition);
}

/// Result of atomically testing and cancelling identify for a short click.
pub enum ButtonIdentifyCancellation {
    Cancelled,
    Inactive,
    StateUnavailable,
}

/// Atomically cancels identify only when it is active at this instant.
pub fn cancel_identify_if_active_at(now_ms: u64) -> ButtonIdentifyCancellation {
    mutate_command_visible_state_with_result(
        ButtonIdentifyCancellation::StateUnavailable,
        |state| {
            if state.identify.mode_at(now_ms) != IdentifyMode::Active {
                return ButtonIdentifyCancellation::Inactive;
            }
            apply_identify_mode_effect(&mut state.identify, IdentifyModeEffect::Disable, now_ms);
            let transition = state
                .command_status
                .record_command(CommandStatusEffect::IdentifyDisable);
            retain_command_status_transition(Some(transition));
            ButtonIdentifyCancellation::Cancelled
        },
    )
}

/// Applies an API-visible block-found dismiss command effect.
pub fn apply_block_found_dismiss_command(effect: BlockFoundDismissEffect) {
    let transition = mutate_command_visible_state_with_result(None, |state| {
        state.block_found = apply_block_found_dismiss_effect(effect);
        Some(
            state
                .command_status
                .record_command(CommandStatusEffect::BlockFoundDismiss),
        )
    });
    retain_command_status_transition(transition);
}

/// Records one production-qualified network-target nonce.
pub fn record_found_block() {
    mutate_command_visible_state(|state| {
        state.block_found = state.block_found.record_found_block();
        state.command_status.record_runtime_change();
    });
}

/// Records that a restart request was accepted before its deferred effect runs.
pub fn record_restart_command() {
    let transition = mutate_command_visible_state_with_result(None, |state| {
        Some(
            state
                .command_status
                .record_command(CommandStatusEffect::Restart),
        )
    });
    retain_command_status_transition(transition);
}

/// Returns the current privacy-safe command and display status.
pub fn command_status_wire(uptime_ms: u64) -> CommandStatusWire {
    mutate_command_visible_state_with_result(
        CommandStatusTracker::default().snapshot(
            crate::boot_evidence::operator_snapshot_boot_session(),
            uptime_ms,
            CommandStatusFacts {
                mining_paused: false,
                mining_activity: "unavailable",
                identify_active: false,
                block_found: 0,
                block_notification_visible: false,
            },
        ),
        |state| {
            state.command_status.snapshot(
                crate::boot_evidence::operator_snapshot_boot_session(),
                uptime_ms,
                CommandStatusFacts {
                    mining_paused: state.mining.operator_intent == MiningOperatorIntent::Paused,
                    mining_activity: mining_activity_label(state.mining.mining_activity),
                    identify_active: state.identify.mode_at(uptime_ms) == IdentifyMode::Active,
                    block_found: state.block_found.block_found,
                    block_notification_visible: state.block_found.show_new_block,
                },
            )
        },
    )
}

/// Records one runtime-display attempt without exposing framebuffer text.
pub fn record_display_render(
    frame_kind: DisplayFrameKind,
    outcome: DisplayRenderOutcome,
    uptime_ms: u64,
) {
    let transition = mutate_command_visible_state_with_result(None, |state| {
        Some(
            state
                .command_status
                .record_display(frame_kind, outcome, uptime_ms),
        )
    });
    retain_command_status_transition(transition);
}

/// Records whether the runtime display is available without claiming a flush.
pub fn record_display_availability(available: bool, uptime_ms: u64) {
    let transition = mutate_command_visible_state_with_result(None, |state| {
        Some(
            state
                .command_status
                .record_display_availability(available, uptime_ms),
        )
    });
    retain_command_status_transition(transition);
}

fn retain_command_status_transition(maybe_transition: Option<CommandStatusTransition>) {
    let Some(transition) = maybe_transition else {
        return;
    };
    let marker = transition.retained_marker(crate::boot_evidence::operator_snapshot_boot_session());
    log::info!("{marker}");
    crate::log_buffer::append_runtime_log_line(&marker);
}

const fn mining_activity_label(activity: MiningActivityStatus) -> &'static str {
    match activity {
        MiningActivityStatus::Paused => "paused",
        MiningActivityStatus::Active => "active",
        MiningActivityStatus::SafeBlocked => "safe_blocked",
    }
}
