//! Pure non-OTA AxeOS command response and side-effect planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/screen.c`
//! - `.planning/phases/05-axeos-api-logs-and-telemetry/05-UI-SPEC.md`

use bitaxe_stratum::v1::state::{MiningActivityStatus, MiningRuntimeState, WorkSubmissionGate};
use serde_json::{json, Value};

/// Upstream identify-mode duration from the AxeOS command route.
pub const IDENTIFY_DURATION_MS: u64 = 30_000;

/// Pure command route plan: public response first, firmware effect second.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandPlan {
    /// Public JSON body to send before executing the side effect.
    pub response: Value,
    /// Inert firmware action that the route shell may execute after responding.
    pub effect: CommandEffect,
}

/// Non-OTA firmware effect planned by a command route.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandEffect {
    /// Update only the API-visible mining activity state.
    MiningActivity(MiningActivityEffect),
    /// Schedule restart only after the response is sent.
    RestartAfterResponse,
    /// Toggle display identify mode.
    Identify(IdentifyModeEffect),
    /// Clear the visible block-found notification state.
    BlockFoundDismiss(BlockFoundDismissEffect),
}

/// Mining activity update that cannot carry a work-submission gate mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningActivityEffect {
    /// Next API-visible mining activity.
    pub next_activity: MiningActivityStatus,
}

/// Current identify-mode state at the command boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifyMode {
    /// Identify image is not currently visible.
    Inactive,
    /// Identify image is currently visible.
    Active,
}

/// Time-bounded identify display effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifyModeEffect {
    /// Enable identify mode for the given duration.
    Enable { duration_ms: u64 },
    /// Disable identify mode immediately.
    Disable,
}

/// Block-found notification state owned by the firmware shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockFoundNotificationState {
    /// Last known block height reported by the system module.
    pub block_found: u64,
    /// Whether the AxeOS notification should still be shown.
    pub show_new_block: bool,
}

/// Block-found dismiss effect with the next notification state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockFoundDismissEffect {
    /// State after dismissing the visible notification.
    pub next_state: BlockFoundNotificationState,
}

/// Plans `POST /api/system/pause`.
#[must_use]
pub fn pause_mining_plan() -> CommandPlan {
    CommandPlan {
        response: message_response("Mining paused"),
        effect: CommandEffect::MiningActivity(MiningActivityEffect {
            next_activity: MiningActivityStatus::Paused,
        }),
    }
}

/// Plans `POST /api/system/resume` without changing the work-submission gate.
#[must_use]
pub fn resume_mining_plan(state: &MiningRuntimeState) -> CommandPlan {
    let next_activity = match state.work_submission {
        WorkSubmissionGate::Ready => MiningActivityStatus::Active,
        WorkSubmissionGate::Blocked => MiningActivityStatus::SafeBlocked,
    };

    CommandPlan {
        response: message_response("Mining resumed"),
        effect: CommandEffect::MiningActivity(MiningActivityEffect { next_activity }),
    }
}

/// Plans `POST /api/system/restart`.
#[must_use]
pub fn restart_plan() -> CommandPlan {
    CommandPlan {
        response: message_response("System will restart shortly."),
        effect: CommandEffect::RestartAfterResponse,
    }
}

/// Plans `POST /api/system/identify`.
#[must_use]
pub fn identify_plan(mode: IdentifyMode) -> CommandPlan {
    match mode {
        IdentifyMode::Inactive => CommandPlan {
            response: message_response("The device says \"Hi!\" for 30 seconds."),
            effect: CommandEffect::Identify(IdentifyModeEffect::Enable {
                duration_ms: IDENTIFY_DURATION_MS,
            }),
        },
        IdentifyMode::Active => CommandPlan {
            response: message_response("The device no longer says \"Hi!\"."),
            effect: CommandEffect::Identify(IdentifyModeEffect::Disable),
        },
    }
}

/// Plans `POST /api/system/blockFound/dismiss`.
#[must_use]
pub fn block_found_dismiss_plan(state: BlockFoundNotificationState) -> CommandPlan {
    let next_state = BlockFoundNotificationState {
        block_found: state.block_found,
        show_new_block: false,
    };

    CommandPlan {
        response: json!({
            "blockFound": next_state.block_found,
            "showNewBlock": next_state.show_new_block,
            "message": "Block found notification dismissed",
        }),
        effect: CommandEffect::BlockFoundDismiss(BlockFoundDismissEffect { next_state }),
    }
}

/// Applies a pure mining-activity command effect without touching work submission.
pub fn apply_mining_activity_effect(state: &mut MiningRuntimeState, effect: MiningActivityEffect) {
    state.set_mining_activity(effect.next_activity);
}

/// Applies a pure block-found dismiss effect.
#[must_use]
pub const fn apply_block_found_dismiss_effect(
    effect: BlockFoundDismissEffect,
) -> BlockFoundNotificationState {
    effect.next_state
}

fn message_response(message: &'static str) -> Value {
    json!({ "message": message })
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::state::{MiningActivityStatus, MiningRuntimeState, WorkSubmissionGate};
    use serde::Deserialize;
    use serde_json::Value;

    use crate::commands::{
        apply_block_found_dismiss_effect, apply_mining_activity_effect, block_found_dismiss_plan,
        identify_plan, pause_mining_plan, restart_plan, resume_mining_plan,
        BlockFoundNotificationState, CommandEffect, IdentifyMode, IdentifyModeEffect,
        IDENTIFY_DURATION_MS,
    };

    #[derive(Debug, Deserialize)]
    struct CommandResponseFixtures {
        pause: Value,
        resume: Value,
        restart: Value,
        identify_on: Value,
        identify_off: Value,
        block_found_dismiss: Value,
    }

    fn fixtures() -> CommandResponseFixtures {
        serde_json::from_str(include_str!("../fixtures/api/command-responses.json"))
            .expect("command response fixture should parse")
    }

    #[test]
    fn pause_plan_returns_message_and_visible_paused_state_without_work_ready() {
        // Arrange
        let fixtures = fixtures();
        let mut state = MiningRuntimeState {
            work_submission: WorkSubmissionGate::Blocked,
            ..Default::default()
        };

        // Act
        let plan = pause_mining_plan();
        if let CommandEffect::MiningActivity(effect) = plan.effect {
            apply_mining_activity_effect(&mut state, effect);
        }

        // Assert
        assert_eq!(plan.response, fixtures.pause);
        assert_eq!(state.mining_activity, MiningActivityStatus::Paused);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
    }

    #[test]
    fn resume_plan_returns_message_and_preserves_existing_gate_status() {
        // Arrange
        let fixtures = fixtures();
        let mut state = MiningRuntimeState {
            mining_activity: MiningActivityStatus::Paused,
            work_submission: WorkSubmissionGate::Blocked,
            ..Default::default()
        };

        // Act
        let plan = resume_mining_plan(&state);
        if let CommandEffect::MiningActivity(effect) = plan.effect {
            apply_mining_activity_effect(&mut state, effect);
        }

        // Assert
        assert_eq!(plan.response, fixtures.resume);
        assert_eq!(state.mining_activity, MiningActivityStatus::SafeBlocked);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
    }

    #[test]
    fn restart_plan_returns_response_before_after_response_effect() {
        // Arrange
        let fixtures = fixtures();

        // Act
        let plan = restart_plan();

        // Assert
        assert_eq!(plan.response, fixtures.restart);
        assert_eq!(plan.effect, CommandEffect::RestartAfterResponse);
    }

    #[test]
    fn identify_plan_toggles_on_for_exact_upstream_duration_and_off_message() {
        // Arrange
        let fixtures = fixtures();

        // Act
        let on_plan = identify_plan(IdentifyMode::Inactive);
        let off_plan = identify_plan(IdentifyMode::Active);

        // Assert
        assert_eq!(on_plan.response, fixtures.identify_on);
        assert_eq!(
            on_plan.effect,
            CommandEffect::Identify(IdentifyModeEffect::Enable {
                duration_ms: IDENTIFY_DURATION_MS,
            })
        );
        assert_eq!(off_plan.response, fixtures.identify_off);
        assert_eq!(
            off_plan.effect,
            CommandEffect::Identify(IdentifyModeEffect::Disable)
        );
    }

    #[test]
    fn block_found_dismiss_clears_show_new_block_and_preserves_block_found() {
        // Arrange
        let fixtures = fixtures();
        let state = BlockFoundNotificationState {
            block_found: 840_000,
            show_new_block: true,
        };

        // Act
        let plan = block_found_dismiss_plan(state);

        // Assert
        assert_eq!(plan.response, fixtures.block_found_dismiss);
        assert_eq!(
            plan.effect,
            CommandEffect::BlockFoundDismiss(crate::commands::BlockFoundDismissEffect {
                next_state: BlockFoundNotificationState {
                    block_found: 840_000,
                    show_new_block: false,
                },
            })
        );
    }

    #[test]
    fn pause_effect_only_changes_visible_activity_for_active_mining_state() {
        // Arrange
        let mut state = MiningRuntimeState {
            mining_activity: MiningActivityStatus::Active,
            work_submission: WorkSubmissionGate::Ready,
            ..Default::default()
        };

        // Act
        let plan = pause_mining_plan();
        let CommandEffect::MiningActivity(effect) = plan.effect else {
            panic!("pause should plan a mining activity effect");
        };
        apply_mining_activity_effect(&mut state, effect);

        // Assert
        assert_eq!(state.mining_activity, MiningActivityStatus::Paused);
        assert_eq!(state.work_submission, WorkSubmissionGate::Ready);
    }

    #[test]
    fn pause_effect_only_changes_visible_activity_for_safe_blocked_state() {
        // Arrange
        let mut state = MiningRuntimeState {
            mining_activity: MiningActivityStatus::SafeBlocked,
            work_submission: WorkSubmissionGate::Blocked,
            ..Default::default()
        };

        // Act
        let plan = pause_mining_plan();
        let CommandEffect::MiningActivity(effect) = plan.effect else {
            panic!("pause should plan a mining activity effect");
        };
        apply_mining_activity_effect(&mut state, effect);

        // Assert
        assert_eq!(state.mining_activity, MiningActivityStatus::Paused);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
    }

    #[test]
    fn resume_effect_restores_active_only_when_prior_gate_was_ready() {
        // Arrange
        let mut state = MiningRuntimeState {
            mining_activity: MiningActivityStatus::Paused,
            work_submission: WorkSubmissionGate::Ready,
            ..Default::default()
        };

        // Act
        let plan = resume_mining_plan(&state);
        let CommandEffect::MiningActivity(effect) = plan.effect else {
            panic!("resume should plan a mining activity effect");
        };
        apply_mining_activity_effect(&mut state, effect);

        // Assert
        assert_eq!(state.mining_activity, MiningActivityStatus::Active);
        assert_eq!(state.work_submission, WorkSubmissionGate::Ready);
    }

    #[test]
    fn resume_effect_restores_safe_blocked_when_prior_gate_was_blocked() {
        // Arrange
        let mut state = MiningRuntimeState {
            mining_activity: MiningActivityStatus::Paused,
            work_submission: WorkSubmissionGate::Blocked,
            ..Default::default()
        };

        // Act
        let plan = resume_mining_plan(&state);
        let CommandEffect::MiningActivity(effect) = plan.effect else {
            panic!("resume should plan a mining activity effect");
        };
        apply_mining_activity_effect(&mut state, effect);

        // Assert
        assert_eq!(state.mining_activity, MiningActivityStatus::SafeBlocked);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
    }

    #[test]
    fn block_found_dismiss_effect_is_idempotent_for_repeated_requests() {
        // Arrange
        let initial = BlockFoundNotificationState {
            block_found: 840_000,
            show_new_block: true,
        };
        let first_plan = block_found_dismiss_plan(initial);
        let CommandEffect::BlockFoundDismiss(first_effect) = first_plan.effect else {
            panic!("block-found dismiss should plan a block-found effect");
        };

        // Act
        let first_state = apply_block_found_dismiss_effect(first_effect);
        let second_plan = block_found_dismiss_plan(first_state);
        let CommandEffect::BlockFoundDismiss(second_effect) = second_plan.effect else {
            panic!("block-found dismiss should plan a block-found effect");
        };
        let second_state = apply_block_found_dismiss_effect(second_effect);

        // Assert
        assert_eq!(first_state, second_state);
        assert_eq!(
            second_state,
            BlockFoundNotificationState {
                block_found: 840_000,
                show_new_block: false,
            }
        );
    }
}
