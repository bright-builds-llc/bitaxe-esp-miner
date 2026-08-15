use std::time::Instant;

use bitaxe_http_transport::StrictHttpClient;
use camino::Utf8Path;

use super::{
    arm_identify_transaction, post_may_have_applied, CampaignTerminalCategory,
    CommandEffectsEvidence, CommandPhase, HTTP_DEADLINE,
};

pub(super) fn begin_paused_dismissal(
    http: &StrictHttpClient,
    evidence: &mut CommandEffectsEvidence,
    maybe_block_count: &mut Option<u64>,
    paused_block_count: u64,
) -> Result<CommandPhase, CampaignTerminalCategory> {
    if evidence.pause_request_count != 1
        || evidence.pause_confirmed
        || evidence.resume_request_count != 0
        || evidence.dismiss_request_count != 0
    {
        return Err(CampaignTerminalCategory::OperatorCheckpointInvalid);
    }
    if paused_block_count == 0 {
        return Err(CampaignTerminalCategory::NetworkCorrelationFailed);
    }
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    // The notification observation proves provenance, but an in-flight result
    // may settle while pause converges. Preservation therefore starts from the
    // last paused sample immediately before the one dismissal request.
    *maybe_block_count = Some(paused_block_count);
    if !post_may_have_applied(http.post_block_found_dismiss_once(Instant::now() + HTTP_DEADLINE)) {
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
