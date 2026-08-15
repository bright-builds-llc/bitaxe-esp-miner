use bitaxe_http_transport::ExchangeObservation;
use std::time::Instant;

use super::{
    CampaignTerminalCategory, CommandEffectsEvidence, CommandFailureCause, SharedSerialState,
};

pub(super) const fn command_state_failure_cause(
    category: CampaignTerminalCategory,
) -> CommandFailureCause {
    if matches!(category, CampaignTerminalCategory::CommandRequestFailed) {
        CommandFailureCause::CommandRequest
    } else {
        CommandFailureCause::CommandStateMachine
    }
}

pub(super) fn may_reuse_confirmed_safe_stop(
    maybe_failure: Option<CampaignTerminalCategory>,
    evidence: &CommandEffectsEvidence,
    serial: &SharedSerialState,
) -> bool {
    maybe_failure == Some(CampaignTerminalCategory::CommandRequestFailed)
        && evidence.pause_confirmed
        && evidence.resume_request_count == 0
        && serial.resumable_pause_safe_stop_confirmed
}

pub(super) fn post_may_have_applied(result: anyhow::Result<ExchangeObservation>) -> bool {
    let Ok(observation) = result else {
        return false;
    };
    match observation.maybe_http_response() {
        Some(response) => response.status() == 200,
        None => observation.request_progress().is_complete(),
    }
}

pub(super) fn terminal_confirmation_timed_out(
    maybe_deadline: Option<Instant>,
    now: Instant,
) -> bool {
    maybe_deadline.is_some_and(|deadline| now >= deadline)
}

pub(super) const fn serial_ended_before_terminal(serial: &SharedSerialState) -> bool {
    serial.serial_finished && !serial.terminal_consumed
}

pub(super) fn take_recovery_pause_request(
    maybe_failure: Option<CampaignTerminalCategory>,
    request_count: &mut u64,
) -> bool {
    if maybe_failure.is_none() || *request_count > 0 {
        return false;
    }
    *request_count = 1;
    true
}
