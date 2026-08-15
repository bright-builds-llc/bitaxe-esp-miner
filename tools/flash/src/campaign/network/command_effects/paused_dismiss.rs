use std::time::Instant;

use bitaxe_http_transport::StrictHttpClient;
use camino::Utf8Path;

use super::{
    arm_identify_transaction, post_succeeded, CampaignTerminalCategory, CommandEffectsEvidence,
    CommandPhase, HTTP_DEADLINE,
};

pub(super) fn begin_paused_dismissal(
    http: &StrictHttpClient,
    evidence: &mut CommandEffectsEvidence,
) -> Result<CommandPhase, CampaignTerminalCategory> {
    if evidence.pause_request_count != 1
        || evidence.pause_confirmed
        || evidence.resume_request_count != 0
        || evidence.dismiss_request_count != 0
    {
        return Err(CampaignTerminalCategory::OperatorCheckpointInvalid);
    }
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    if !post_succeeded(http.post_block_found_dismiss_once(Instant::now() + HTTP_DEADLINE)) {
        return Err(CampaignTerminalCategory::CommandRequestFailed);
    }
    Ok(CommandPhase::PausedDismiss)
}

pub(super) fn arm_ready_after_paused_dismissal(
    root: &Utf8Path,
    evidence: &CommandEffectsEvidence,
) -> Result<CommandPhase, ()> {
    if evidence.pause_request_count != 1
        || !evidence.pause_confirmed
        || evidence.dismiss_request_count != 1
        || !evidence.dismiss_confirmed
        || !evidence.block_count_preserved
        || evidence.resume_request_count != 0
    {
        return Err(());
    }
    arm_identify_transaction(root, evidence)
}
