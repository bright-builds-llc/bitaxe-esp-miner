use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub(in crate::campaign) struct CommandEffectsEvidence {
    pub(in crate::campaign) schema: &'static str,
    pub(in crate::campaign) genuine_block_notification_observed: bool,
    pub(in crate::campaign) positive_block_count_observed: bool,
    pub(in crate::campaign) pause_request_count: u8,
    pub(in crate::campaign) pause_confirmed: bool,
    pub(in crate::campaign) resume_request_count: u8,
    pub(in crate::campaign) resume_confirmed: bool,
    pub(in crate::campaign) identify_operator_ready_confirmed: bool,
    pub(in crate::campaign) identify_request_count: u8,
    pub(in crate::campaign) identify_rendered_confirmed: bool,
    pub(in crate::campaign) identify_cleared_confirmed: bool,
    pub(in crate::campaign) identify_terminal_outcome: &'static str,
    pub(in crate::campaign) dismiss_request_count: u8,
    pub(in crate::campaign) dismiss_confirmed: bool,
    pub(in crate::campaign) block_count_preserved: bool,
    pub(in crate::campaign) active_before_pause: bool,
    pub(in crate::campaign) active_after_resume: bool,
    pub(in crate::campaign) same_boot_and_package: bool,
    pub(in crate::campaign) safety_valid: bool,
    pub(in crate::campaign) terminal_http_valid: bool,
    pub(in crate::campaign) terminal_pool_persisted: bool,
}

impl CommandEffectsEvidence {
    pub(super) fn new() -> Self {
        Self {
            schema: "mining-campaign-command-effects-v3",
            identify_terminal_outcome: "none",
            same_boot_and_package: true,
            safety_valid: true,
            ..Self::default()
        }
    }

    pub(super) fn complete(&self) -> bool {
        self.genuine_block_notification_observed
            && self.positive_block_count_observed
            && self.pause_request_count == 1
            && self.pause_confirmed
            && self.resume_request_count == 1
            && self.resume_confirmed
            && self.identify_operator_ready_confirmed
            && self.identify_request_count == 2
            && self.identify_rendered_confirmed
            && self.identify_cleared_confirmed
            && self.identify_terminal_outcome == "none"
            && self.dismiss_request_count == 1
            && self.dismiss_confirmed
            && self.block_count_preserved
            && self.active_before_pause
            && self.active_after_resume
            && self.same_boot_and_package
            && self.safety_valid
            && self.terminal_http_valid
            && self.terminal_pool_persisted
    }
}
