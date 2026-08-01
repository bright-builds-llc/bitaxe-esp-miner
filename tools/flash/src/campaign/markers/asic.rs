use std::collections::VecDeque;

use super::*;

#[derive(Clone, Debug, Default, Serialize)]
pub(in crate::campaign) struct CampaignAsicEventTrace {
    pub(in crate::campaign) observed_event_count: u64,
    pub(in crate::campaign) events_truncated: bool,
    pub(in crate::campaign) first_events: Vec<AsicEventMarker>,
    pub(in crate::campaign) last_events: VecDeque<AsicEventMarker>,
    #[serde(skip)]
    maybe_last_sequence: Option<u64>,
}

impl CampaignAsicEventTrace {
    pub(in crate::campaign) fn observe(&mut self, maybe_event: Option<AsicEventMarker>) {
        let Some(event) = maybe_event else {
            return;
        };
        if self.maybe_last_sequence == Some(event.sequence) {
            return;
        }
        if self
            .maybe_last_sequence
            .is_some_and(|previous| event.sequence > previous.saturating_add(1))
        {
            self.events_truncated = true;
        }
        self.maybe_last_sequence = Some(event.sequence);
        self.observed_event_count = self.observed_event_count.saturating_add(1);
        if self.first_events.len() < 32 {
            self.first_events.push(event);
        } else {
            if self.last_events.len() == 32 {
                self.last_events.pop_front();
            }
            self.last_events.push_back(event);
        }
        if self.observed_event_count > 64 {
            self.events_truncated = true;
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum AsicPollStateMarker {
    Idle,
    InFlight,
    Invalidated,
    Completed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum AsicGenerationRelationMarker {
    PreTransition,
    Replacement,
    Stale,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum AsicEventKindMarker {
    GenerationInvalidated,
    ReplacementDispatched,
    PollRequested,
    PollCompleted,
    NonceEmitted,
    Correlation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum AsicEventOutcomeMarker {
    Invalidated,
    Dispatched,
    Requested,
    Idle,
    RegisterRead,
    DiscardInvalidLength,
    DiscardInvalidPreamble,
    DiscardInvalidCrc,
    DiscardJobLookup,
    DiscardCore,
    DiscardAddressInterval,
    DiscardRegisterResponse,
    DiscardParserInvariant,
    Nonce,
    Correlated,
    BelowTarget,
    Duplicate,
    BlockedWrongSession,
    BlockedJobLookup,
    BlockedWorkStale,
    BlockedTargetMismatch,
    BlockedOther,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::campaign) struct AsicEventMarker {
    pub(in crate::campaign) sequence: u64,
    pub(in crate::campaign) monotonic_offset_ms: u64,
    pub(in crate::campaign) kind: AsicEventKindMarker,
    pub(in crate::campaign) generation_relation: AsicGenerationRelationMarker,
    pub(in crate::campaign) outcome: AsicEventOutcomeMarker,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::campaign) struct AsicDiscardMarker {
    pub(in crate::campaign) invalid_length: u64,
    pub(in crate::campaign) invalid_preamble: u64,
    pub(in crate::campaign) invalid_crc: u64,
    pub(in crate::campaign) job_lookup: u64,
    pub(in crate::campaign) core: u64,
    pub(in crate::campaign) address_interval: u64,
    pub(in crate::campaign) register_response: u64,
    pub(in crate::campaign) parser_invariant: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::campaign) struct AsicBridgeMarker {
    pub(in crate::campaign) poll_request_count: u64,
    pub(in crate::campaign) idle_completion_count: u64,
    pub(in crate::campaign) nonce_completion_count: u64,
    pub(in crate::campaign) register_read_count: u64,
    pub(in crate::campaign) discards: AsicDiscardMarker,
    pub(in crate::campaign) generation_invalidation_count: u64,
    pub(in crate::campaign) stale_completion_count: u64,
    pub(in crate::campaign) post_transition_poll_request_count: u64,
    pub(in crate::campaign) post_transition_completion_count: u64,
    pub(in crate::campaign) post_transition_nonce_emission_count: u64,
    pub(in crate::campaign) post_transition_correlation_count: u64,
    pub(in crate::campaign) blocked_correlation_count: u64,
    pub(in crate::campaign) blocked_correlations: AsicBlockedCorrelationMarker,
    pub(in crate::campaign) changed_block_to_replacement_dispatch_ms: Option<u64>,
    pub(in crate::campaign) changed_block_to_first_poll_ms: Option<u64>,
    pub(in crate::campaign) changed_block_to_first_nonce_ms: Option<u64>,
    pub(in crate::campaign) changed_block_to_first_correlation_ms: Option<u64>,
    pub(in crate::campaign) final_poll_state: AsicPollStateMarker,
    pub(in crate::campaign) latest_event: Option<AsicEventMarker>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::campaign) struct AsicBlockedCorrelationMarker {
    pub(in crate::campaign) wrong_session: u64,
    pub(in crate::campaign) job_lookup: u64,
    pub(in crate::campaign) work_stale: u64,
    pub(in crate::campaign) target_mismatch: u64,
    pub(in crate::campaign) other: u64,
}
