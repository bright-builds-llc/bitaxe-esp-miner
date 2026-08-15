use super::*;

#[derive(Clone, Copy)]
pub(super) struct PendingFramingFailure {
    detail: CampaignSerialOutcomeDetail,
    preceded_failure: bool,
}

impl CampaignSerialAnalyzer {
    pub(super) fn record_recoverable_framing_failure(
        &mut self,
        detail: CampaignSerialOutcomeDetail,
        event: CampaignSerialEventKind,
        byte_offset: usize,
        line_length: usize,
    ) {
        self.maybe_pending_framing_failure
            .get_or_insert(PendingFramingFailure {
                detail,
                preceded_failure: self.maybe_failure.is_none(),
            });
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            event,
        );
    }

    pub(super) fn refresh_unrecovered_framing_failure(&mut self) {
        let Some(pending) = self.maybe_pending_framing_failure.take() else {
            return;
        };
        if self.outcome_detail == CampaignSerialOutcomeDetail::Clean {
            self.outcome_detail = pending.detail;
        }
        if pending.preceded_failure {
            self.maybe_failure = Some(CampaignTerminalCategory::MarkerInvalid);
        } else {
            self.maybe_failure
                .get_or_insert(CampaignTerminalCategory::MarkerInvalid);
        }
    }
}
