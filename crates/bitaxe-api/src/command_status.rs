//! Privacy-safe command and display status for autonomous device verification.

use serde::{Deserialize, Serialize};

use crate::BootSessionId;

/// Versioned schema emitted by `/api/system/command-status`.
pub const COMMAND_STATUS_SCHEMA: &str = "bitaxe-command-status-v1";

/// One command whose application is visible through the status interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatusEffect {
    Pause,
    Resume,
    Restart,
    IdentifyEnable { expires_at_uptime_ms: u64 },
    IdentifyDisable,
    BlockFoundDismiss,
}

/// Closed, privacy-safe display frame classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayFrameKind {
    Identify,
    NonIdentify,
}

/// Closed display render-attempt outcome.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayRenderOutcome {
    Rendered,
    Failed,
    Unavailable,
}

/// Current command-visible facts supplied by the firmware runtime owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandStatusFacts<'a> {
    pub mining_paused: bool,
    pub mining_activity: &'a str,
    pub identify_active: bool,
    pub block_found: u64,
    pub block_notification_visible: bool,
}

/// One redaction-safe display attempt correlated to command state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayRenderReceiptWire {
    pub status_revision: u64,
    pub uptime_ms: u64,
    pub frame_kind: DisplayFrameKind,
    pub identify_generation: u64,
    pub outcome: DisplayRenderOutcome,
}

/// Mining command state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MiningCommandStatusWire {
    pub paused: bool,
    pub activity: String,
    pub pause_generation: u64,
    pub resume_generation: u64,
}

/// IDENTIFY command state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyCommandStatusWire {
    pub active: bool,
    pub generation: u64,
    pub maybe_expires_at_uptime_ms: Option<u64>,
}

/// Block-notification command state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockNotificationCommandStatusWire {
    pub visible: bool,
    pub count: u64,
    pub dismiss_generation: u64,
}

/// Restart command state within the current boot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestartCommandStatusWire {
    pub accepted_generation: u64,
}

/// Display proof state. A successful receipt exists only after a framebuffer flush.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayCommandStatusWire {
    pub available: bool,
    pub render_revision: u64,
    pub maybe_last_attempt: Option<DisplayRenderReceiptWire>,
    pub maybe_last_success: Option<DisplayRenderReceiptWire>,
}

/// Complete privacy-safe command status response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandStatusWire {
    pub schema: String,
    pub boot_session: BootSessionId,
    pub uptime_ms: u64,
    pub status_revision: u64,
    pub mining: MiningCommandStatusWire,
    pub restart: RestartCommandStatusWire,
    pub identify: IdentifyCommandStatusWire,
    pub block_notification: BlockNotificationCommandStatusWire,
    pub display: DisplayCommandStatusWire,
}

/// Retained transition emitted after one status mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandStatusTransition {
    pub status_revision: u64,
    pub generation: u64,
    pub kind: &'static str,
    pub outcome: &'static str,
}

impl CommandStatusTransition {
    /// Formats a closed marker suitable for serial and retained logs.
    #[must_use]
    pub fn retained_marker(self, boot_session: BootSessionId) -> String {
        format!(
            "command_status_transition session={boot_session} revision={} command={} generation={} outcome={} redacted=true",
            self.status_revision, self.kind, self.generation, self.outcome
        )
    }
}

/// Boot-scoped command and display correlation owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandStatusTracker {
    status_revision: u64,
    pause_generation: u64,
    resume_generation: u64,
    restart_generation: u64,
    identify_generation: u64,
    maybe_identify_expires_at_uptime_ms: Option<u64>,
    dismiss_generation: u64,
    display_available: bool,
    render_revision: u64,
    maybe_last_attempt: Option<DisplayRenderReceiptWire>,
    maybe_last_success: Option<DisplayRenderReceiptWire>,
}

impl Default for CommandStatusTracker {
    fn default() -> Self {
        Self {
            status_revision: 1,
            pause_generation: 0,
            resume_generation: 0,
            restart_generation: 0,
            identify_generation: 0,
            maybe_identify_expires_at_uptime_ms: None,
            dismiss_generation: 0,
            display_available: false,
            render_revision: 0,
            maybe_last_attempt: None,
            maybe_last_success: None,
        }
    }
}

impl CommandStatusTracker {
    /// Records one applied command and returns its retained transition.
    pub fn record_command(&mut self, effect: CommandStatusEffect) -> CommandStatusTransition {
        self.status_revision = self.status_revision.saturating_add(1);
        let (generation, kind) = match effect {
            CommandStatusEffect::Pause => {
                self.pause_generation = self.pause_generation.saturating_add(1);
                (self.pause_generation, "pause")
            }
            CommandStatusEffect::Resume => {
                self.resume_generation = self.resume_generation.saturating_add(1);
                (self.resume_generation, "resume")
            }
            CommandStatusEffect::Restart => {
                self.restart_generation = self.restart_generation.saturating_add(1);
                (self.restart_generation, "restart")
            }
            CommandStatusEffect::IdentifyEnable {
                expires_at_uptime_ms,
            } => {
                self.identify_generation = self.identify_generation.saturating_add(1);
                self.maybe_identify_expires_at_uptime_ms = Some(expires_at_uptime_ms);
                (self.identify_generation, "identify_enable")
            }
            CommandStatusEffect::IdentifyDisable => {
                self.identify_generation = self.identify_generation.saturating_add(1);
                self.maybe_identify_expires_at_uptime_ms = None;
                (self.identify_generation, "identify_disable")
            }
            CommandStatusEffect::BlockFoundDismiss => {
                self.dismiss_generation = self.dismiss_generation.saturating_add(1);
                (self.dismiss_generation, "block_found_dismiss")
            }
        };
        CommandStatusTransition {
            status_revision: self.status_revision,
            generation,
            kind,
            outcome: "applied",
        }
    }

    /// Records display availability without claiming a rendered frame.
    pub fn record_display_availability(
        &mut self,
        available: bool,
        _uptime_ms: u64,
    ) -> CommandStatusTransition {
        self.display_available = available;
        self.status_revision = self.status_revision.saturating_add(1);
        CommandStatusTransition {
            status_revision: self.status_revision,
            generation: self.identify_generation,
            kind: "display_availability",
            outcome: if available {
                "available"
            } else {
                "unavailable"
            },
        }
    }

    /// Records one framebuffer attempt and returns its retained transition.
    pub fn record_display(
        &mut self,
        frame_kind: DisplayFrameKind,
        outcome: DisplayRenderOutcome,
        uptime_ms: u64,
    ) -> CommandStatusTransition {
        self.status_revision = self.status_revision.saturating_add(1);
        self.render_revision = self.render_revision.saturating_add(1);
        if outcome == DisplayRenderOutcome::Rendered {
            self.display_available = true;
        } else if outcome == DisplayRenderOutcome::Unavailable {
            self.display_available = false;
        }
        let receipt = DisplayRenderReceiptWire {
            status_revision: self.status_revision,
            uptime_ms,
            frame_kind,
            identify_generation: self.identify_generation,
            outcome,
        };
        self.maybe_last_attempt = Some(receipt.clone());
        if outcome == DisplayRenderOutcome::Rendered {
            self.maybe_last_success = Some(receipt);
        }
        CommandStatusTransition {
            status_revision: self.status_revision,
            generation: self.identify_generation,
            kind: match frame_kind {
                DisplayFrameKind::Identify => "display_identify",
                DisplayFrameKind::NonIdentify => "display_non_identify",
            },
            outcome: match outcome {
                DisplayRenderOutcome::Rendered => "rendered",
                DisplayRenderOutcome::Failed => "failed",
                DisplayRenderOutcome::Unavailable => "unavailable",
            },
        }
    }

    /// Advances the observable state revision after a non-command runtime change.
    pub fn record_runtime_change(&mut self) {
        self.status_revision = self.status_revision.saturating_add(1);
    }

    /// Projects one immutable response without exposing frame text or device identity.
    #[must_use]
    pub fn snapshot(
        &self,
        boot_session: BootSessionId,
        uptime_ms: u64,
        facts: CommandStatusFacts<'_>,
    ) -> CommandStatusWire {
        CommandStatusWire {
            schema: COMMAND_STATUS_SCHEMA.to_owned(),
            boot_session,
            uptime_ms,
            status_revision: self.status_revision,
            mining: MiningCommandStatusWire {
                paused: facts.mining_paused,
                activity: facts.mining_activity.to_owned(),
                pause_generation: self.pause_generation,
                resume_generation: self.resume_generation,
            },
            restart: RestartCommandStatusWire {
                accepted_generation: self.restart_generation,
            },
            identify: IdentifyCommandStatusWire {
                active: facts.identify_active,
                generation: self.identify_generation,
                maybe_expires_at_uptime_ms: self.maybe_identify_expires_at_uptime_ms,
            },
            block_notification: BlockNotificationCommandStatusWire {
                visible: facts.block_notification_visible,
                count: facts.block_found,
                dismiss_generation: self.dismiss_generation,
            },
            display: DisplayCommandStatusWire {
                available: self.display_available,
                render_revision: self.render_revision,
                maybe_last_attempt: self.maybe_last_attempt.clone(),
                maybe_last_success: self.maybe_last_success.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(identify_active: bool) -> CommandStatusFacts<'static> {
        CommandStatusFacts {
            mining_paused: true,
            mining_activity: "paused",
            identify_active,
            block_found: 1,
            block_notification_visible: false,
        }
    }

    #[test]
    fn identify_success_is_correlated_only_after_rendered_attempt() {
        // Arrange
        let mut tracker = CommandStatusTracker::default();
        tracker.record_command(CommandStatusEffect::IdentifyEnable {
            expires_at_uptime_ms: 31_000,
        });

        // Act
        tracker.record_display(
            DisplayFrameKind::Identify,
            DisplayRenderOutcome::Rendered,
            1_500,
        );
        let status = tracker.snapshot(BootSessionId::from_words([1, 2, 3, 4]), 1_500, facts(true));

        // Assert
        let receipt = status
            .display
            .maybe_last_success
            .expect("successful framebuffer flush has a receipt");
        assert_eq!(receipt.frame_kind, DisplayFrameKind::Identify);
        assert_eq!(receipt.identify_generation, 1);
        assert_eq!(receipt.outcome, DisplayRenderOutcome::Rendered);
    }

    #[test]
    fn failed_render_does_not_replace_last_success() {
        // Arrange
        let mut tracker = CommandStatusTracker::default();
        tracker.record_display(
            DisplayFrameKind::NonIdentify,
            DisplayRenderOutcome::Rendered,
            500,
        );

        // Act
        tracker.record_display(
            DisplayFrameKind::Identify,
            DisplayRenderOutcome::Failed,
            1_000,
        );
        let status = tracker.snapshot(BootSessionId::from_words([1, 2, 3, 4]), 1_000, facts(true));

        // Assert
        assert_eq!(
            status.display.maybe_last_attempt.expect("attempt").outcome,
            DisplayRenderOutcome::Failed
        );
        assert_eq!(
            status
                .display
                .maybe_last_success
                .expect("success")
                .frame_kind,
            DisplayFrameKind::NonIdentify
        );
    }

    #[test]
    fn non_identify_receipt_closes_the_same_naturally_expired_generation() {
        // Arrange
        let mut tracker = CommandStatusTracker::default();
        tracker.record_command(CommandStatusEffect::IdentifyEnable {
            expires_at_uptime_ms: 1_000,
        });

        // Act
        tracker.record_display(
            DisplayFrameKind::NonIdentify,
            DisplayRenderOutcome::Rendered,
            1_500,
        );
        let status = tracker.snapshot(BootSessionId::from_words([1, 2, 3, 4]), 1_500, facts(false));

        // Assert
        let receipt = status.display.maybe_last_success.expect("clear receipt");
        assert_eq!(receipt.identify_generation, 1);
        assert_eq!(receipt.frame_kind, DisplayFrameKind::NonIdentify);
        assert!(!status.identify.active);
    }

    #[test]
    fn serialized_status_contains_no_private_values() {
        // Arrange
        let status = CommandStatusTracker::default().snapshot(
            BootSessionId::from_words([1, 2, 3, 4]),
            42,
            facts(false),
        );

        // Act
        let json = serde_json::to_string(&status).expect("serialize status");

        // Assert
        for forbidden in ["hostname", "ssid", "pool", "origin", "port", "frameText"] {
            assert!(!json.contains(forbidden));
        }
    }
}
