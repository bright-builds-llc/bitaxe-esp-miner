use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub(in crate::campaign) struct CommandEffectsEvidence {
    pub(in crate::campaign) schema: &'static str,
    pub(in crate::campaign) genuine_block_notification_observed: bool,
    pub(in crate::campaign) positive_block_count_observed: bool,
    pub(in crate::campaign) pause_request_count: u8,
    pub(in crate::campaign) pause_confirmed: bool,
    pub(in crate::campaign) resume_request_count: u8,
    pub(in crate::campaign) resume_intent_confirmed: bool,
    pub(in crate::campaign) resume_confirmed: bool,
    pub(in crate::campaign) identify_status_baseline_confirmed: bool,
    pub(in crate::campaign) identify_request_count: u8,
    #[cfg(test)]
    #[serde(skip)]
    pub(in crate::campaign) identify_operator_ready_confirmed: bool,
    #[cfg(test)]
    #[serde(skip)]
    pub(in crate::campaign) identify_replay_request_count: u8,
    #[cfg(test)]
    #[serde(skip)]
    pub(in crate::campaign) identify_rendered_confirmed: bool,
    #[cfg(test)]
    #[serde(skip)]
    pub(in crate::campaign) identify_cleared_confirmed: bool,
    pub(in crate::campaign) identify_render_receipt_confirmed: bool,
    pub(in crate::campaign) identify_clear_receipt_confirmed: bool,
    pub(in crate::campaign) retained_identify_transition_confirmed: bool,
    pub(in crate::campaign) serial_transition_witnesses_confirmed: bool,
    pub(in crate::campaign) websocket_transition_witnesses_confirmed: bool,
    pub(in crate::campaign) identify_terminal_outcome: &'static str,
    pub(in crate::campaign) dismiss_request_count: u8,
    pub(in crate::campaign) dismiss_confirmed: bool,
    pub(in crate::campaign) block_count_preserved: bool,
    pub(in crate::campaign) active_before_pause: bool,
    pub(in crate::campaign) active_after_resume: bool,
    pub(in crate::campaign) recovery_pause_api_confirmed: bool,
    pub(in crate::campaign) recovery_pause_serial_confirmed: bool,
    pub(in crate::campaign) recovery_safe_stop_confirmed: bool,
    pub(in crate::campaign) recovery_terminal_outcome: &'static str,
    pub(in crate::campaign) same_boot_and_package: bool,
    pub(in crate::campaign) safety_valid: bool,
    pub(in crate::campaign) terminal_http_valid: bool,
    pub(in crate::campaign) terminal_pool_persisted: bool,
}

impl CommandEffectsEvidence {
    pub(super) fn new() -> Self {
        Self {
            schema: "mining-campaign-command-effects-v8",
            identify_terminal_outcome: "none",
            recovery_terminal_outcome: "not_required",
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
            && self.resume_intent_confirmed
            && self.resume_confirmed
            && self.identify_status_baseline_confirmed
            && self.identify_request_count == 1
            && self.identify_render_receipt_confirmed
            && self.identify_clear_receipt_confirmed
            && self.retained_identify_transition_confirmed
            && self.identify_terminal_outcome == "none"
            && self.dismiss_request_count == 1
            && self.dismiss_confirmed
            && self.block_count_preserved
            && self.active_before_pause
            && self.active_after_resume
            && !self.recovery_pause_api_confirmed
            && !self.recovery_pause_serial_confirmed
            && !self.recovery_safe_stop_confirmed
            && self.recovery_terminal_outcome == "not_required"
            && self.same_boot_and_package
            && self.safety_valid
            && self.terminal_http_valid
            && self.terminal_pool_persisted
    }
}
