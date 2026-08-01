use bitaxe_asic::bm1366::result::Bm1366ResultDiscardReason;
use serde::Serialize;

use crate::v1::production_work::PoolSessionGeneration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AsicDiscardEvidence {
    pub invalid_length: u64,
    pub invalid_preamble: u64,
    pub invalid_crc: u64,
    pub job_lookup: u64,
    pub core: u64,
    pub address_interval: u64,
    pub register_response: u64,
    pub parser_invariant: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AsicBlockedCorrelationEvidence {
    pub wrong_session: u64,
    pub job_lookup: u64,
    pub work_stale: u64,
    pub target_mismatch: u64,
    pub other: u64,
}

impl AsicDiscardEvidence {
    fn note(&mut self, reason: Bm1366ResultDiscardReason) {
        let counter = match reason {
            Bm1366ResultDiscardReason::InvalidLength => &mut self.invalid_length,
            Bm1366ResultDiscardReason::InvalidPreamble => &mut self.invalid_preamble,
            Bm1366ResultDiscardReason::InvalidCrc => &mut self.invalid_crc,
            Bm1366ResultDiscardReason::JobLookup => &mut self.job_lookup,
            Bm1366ResultDiscardReason::Core => &mut self.core,
            Bm1366ResultDiscardReason::AddressInterval => &mut self.address_interval,
            Bm1366ResultDiscardReason::RegisterResponse => &mut self.register_response,
            Bm1366ResultDiscardReason::ParserInvariant => &mut self.parser_invariant,
        };
        *counter = counter.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsicPollState {
    Idle,
    InFlight,
    Invalidated,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsicGenerationRelation {
    PreTransition,
    Replacement,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsicDiagnosticEventKind {
    GenerationInvalidated,
    ReplacementDispatched,
    PollRequested,
    PollCompleted,
    NonceEmitted,
    Correlation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsicDiagnosticOutcome {
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

impl From<Bm1366ResultDiscardReason> for AsicDiagnosticOutcome {
    fn from(reason: Bm1366ResultDiscardReason) -> Self {
        match reason {
            Bm1366ResultDiscardReason::InvalidLength => Self::DiscardInvalidLength,
            Bm1366ResultDiscardReason::InvalidPreamble => Self::DiscardInvalidPreamble,
            Bm1366ResultDiscardReason::InvalidCrc => Self::DiscardInvalidCrc,
            Bm1366ResultDiscardReason::JobLookup => Self::DiscardJobLookup,
            Bm1366ResultDiscardReason::Core => Self::DiscardCore,
            Bm1366ResultDiscardReason::AddressInterval => Self::DiscardAddressInterval,
            Bm1366ResultDiscardReason::RegisterResponse => Self::DiscardRegisterResponse,
            Bm1366ResultDiscardReason::ParserInvariant => Self::DiscardParserInvariant,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AsicDiagnosticEvent {
    pub sequence: u64,
    pub monotonic_offset_ms: u64,
    pub kind: AsicDiagnosticEventKind,
    pub generation_relation: AsicGenerationRelation,
    pub outcome: AsicDiagnosticOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AsicBridgeEvidence {
    pub poll_request_count: u64,
    pub idle_completion_count: u64,
    pub nonce_completion_count: u64,
    pub register_read_count: u64,
    pub discards: AsicDiscardEvidence,
    pub generation_invalidation_count: u64,
    pub stale_completion_count: u64,
    pub post_transition_poll_request_count: u64,
    pub post_transition_completion_count: u64,
    pub post_transition_nonce_emission_count: u64,
    pub post_transition_correlation_count: u64,
    pub blocked_correlation_count: u64,
    pub blocked_correlations: AsicBlockedCorrelationEvidence,
    pub changed_block_to_replacement_dispatch_ms: Option<u64>,
    pub changed_block_to_first_poll_ms: Option<u64>,
    pub changed_block_to_first_nonce_ms: Option<u64>,
    pub changed_block_to_first_correlation_ms: Option<u64>,
    pub final_poll_state: AsicPollState,
    pub latest_event: Option<AsicDiagnosticEvent>,
}

impl Default for AsicBridgeEvidence {
    fn default() -> Self {
        Self {
            poll_request_count: 0,
            idle_completion_count: 0,
            nonce_completion_count: 0,
            register_read_count: 0,
            discards: AsicDiscardEvidence::default(),
            generation_invalidation_count: 0,
            stale_completion_count: 0,
            post_transition_poll_request_count: 0,
            post_transition_completion_count: 0,
            post_transition_nonce_emission_count: 0,
            post_transition_correlation_count: 0,
            blocked_correlation_count: 0,
            blocked_correlations: AsicBlockedCorrelationEvidence::default(),
            changed_block_to_replacement_dispatch_ms: None,
            changed_block_to_first_poll_ms: None,
            changed_block_to_first_nonce_ms: None,
            changed_block_to_first_correlation_ms: None,
            final_poll_state: AsicPollState::Idle,
            latest_event: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicPollCompletion {
    Idle,
    RegisterRead,
    Discarded(Bm1366ResultDiscardReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicCorrelation {
    Correlated,
    BelowTarget,
    Duplicate,
    BlockedWrongSession,
    BlockedJobLookup,
    BlockedWorkStale,
    BlockedTargetMismatch,
    BlockedOther,
}

#[derive(Debug, Default)]
pub(super) struct AsicBridgeDiagnosticsTracker {
    evidence: AsicBridgeEvidence,
    maybe_transition_generation: Option<PoolSessionGeneration>,
    maybe_transition_started_ms: Option<u64>,
    event_sequence: u64,
}

impl AsicBridgeDiagnosticsTracker {
    pub(super) const fn evidence(&self) -> AsicBridgeEvidence {
        self.evidence
    }

    pub(super) fn note_session_invalidation(&mut self) {
        self.evidence.final_poll_state = AsicPollState::Invalidated;
    }

    pub(super) fn note_generation_invalidation(
        &mut self,
        generation: PoolSessionGeneration,
        now_ms: u64,
        previous_block_changed: bool,
    ) {
        self.evidence.generation_invalidation_count = self
            .evidence
            .generation_invalidation_count
            .saturating_add(1);
        self.evidence.final_poll_state = AsicPollState::Invalidated;
        if previous_block_changed {
            self.maybe_transition_started_ms = Some(now_ms);
            self.evidence.changed_block_to_replacement_dispatch_ms = None;
            self.evidence.changed_block_to_first_poll_ms = None;
            self.evidence.changed_block_to_first_nonce_ms = None;
            self.evidence.changed_block_to_first_correlation_ms = None;
        }
        if previous_block_changed || self.maybe_transition_generation.is_some() {
            self.maybe_transition_generation = Some(generation);
            self.record(
                now_ms,
                generation,
                AsicDiagnosticEventKind::GenerationInvalidated,
                AsicDiagnosticOutcome::Invalidated,
            );
        }
    }

    pub(super) fn note_dispatch(&mut self, generation: PoolSessionGeneration, now_ms: u64) {
        if self.relation(generation) == AsicGenerationRelation::Replacement {
            Self::set_first_delay(
                &mut self.evidence.changed_block_to_replacement_dispatch_ms,
                self.maybe_transition_started_ms,
                now_ms,
            );
            self.record(
                now_ms,
                generation,
                AsicDiagnosticEventKind::ReplacementDispatched,
                AsicDiagnosticOutcome::Dispatched,
            );
        }
    }

    pub(super) fn note_poll_requested(&mut self, generation: PoolSessionGeneration, now_ms: u64) {
        self.evidence.poll_request_count = self.evidence.poll_request_count.saturating_add(1);
        self.evidence.final_poll_state = AsicPollState::InFlight;
        if self.relation(generation) == AsicGenerationRelation::Replacement {
            self.evidence.post_transition_poll_request_count = self
                .evidence
                .post_transition_poll_request_count
                .saturating_add(1);
            Self::set_first_delay(
                &mut self.evidence.changed_block_to_first_poll_ms,
                self.maybe_transition_started_ms,
                now_ms,
            );
            self.record(
                now_ms,
                generation,
                AsicDiagnosticEventKind::PollRequested,
                AsicDiagnosticOutcome::Requested,
            );
        }
    }

    pub(super) fn note_poll_completion(
        &mut self,
        generation: PoolSessionGeneration,
        completion: AsicPollCompletion,
        now_ms: u64,
        is_current: bool,
    ) {
        let relation = self.relation(generation);
        if !is_current {
            self.evidence.stale_completion_count =
                self.evidence.stale_completion_count.saturating_add(1);
            self.record_with_relation(
                now_ms,
                AsicGenerationRelation::Stale,
                AsicDiagnosticEventKind::PollCompleted,
                completion.outcome(),
            );
            return;
        }
        self.evidence.final_poll_state = AsicPollState::Completed;
        match completion {
            AsicPollCompletion::Idle => {
                self.evidence.idle_completion_count =
                    self.evidence.idle_completion_count.saturating_add(1);
            }
            AsicPollCompletion::RegisterRead => {
                self.evidence.register_read_count =
                    self.evidence.register_read_count.saturating_add(1);
            }
            AsicPollCompletion::Discarded(reason) => self.evidence.discards.note(reason),
        }
        if relation == AsicGenerationRelation::Replacement {
            self.evidence.post_transition_completion_count = self
                .evidence
                .post_transition_completion_count
                .saturating_add(1);
            self.record(
                now_ms,
                generation,
                AsicDiagnosticEventKind::PollCompleted,
                completion.outcome(),
            );
        }
    }

    pub(super) fn note_nonce(
        &mut self,
        generation: PoolSessionGeneration,
        now_ms: u64,
        is_current: bool,
    ) {
        if !is_current {
            self.evidence.stale_completion_count =
                self.evidence.stale_completion_count.saturating_add(1);
            self.record_with_relation(
                now_ms,
                AsicGenerationRelation::Stale,
                AsicDiagnosticEventKind::NonceEmitted,
                AsicDiagnosticOutcome::Nonce,
            );
            return;
        }
        self.evidence.nonce_completion_count =
            self.evidence.nonce_completion_count.saturating_add(1);
        self.evidence.final_poll_state = AsicPollState::Completed;
        if self.relation(generation) == AsicGenerationRelation::Replacement {
            self.evidence.post_transition_nonce_emission_count = self
                .evidence
                .post_transition_nonce_emission_count
                .saturating_add(1);
            Self::set_first_delay(
                &mut self.evidence.changed_block_to_first_nonce_ms,
                self.maybe_transition_started_ms,
                now_ms,
            );
            self.record(
                now_ms,
                generation,
                AsicDiagnosticEventKind::NonceEmitted,
                AsicDiagnosticOutcome::Nonce,
            );
        }
    }

    pub(super) fn note_correlation(
        &mut self,
        generation: PoolSessionGeneration,
        outcome: AsicCorrelation,
        now_ms: u64,
    ) {
        if self.relation(generation) != AsicGenerationRelation::Replacement {
            return;
        }
        if outcome.is_correlated() {
            self.evidence.post_transition_correlation_count = self
                .evidence
                .post_transition_correlation_count
                .saturating_add(1);
            Self::set_first_delay(
                &mut self.evidence.changed_block_to_first_correlation_ms,
                self.maybe_transition_started_ms,
                now_ms,
            );
        } else if outcome.is_blocked() {
            self.evidence.blocked_correlation_count =
                self.evidence.blocked_correlation_count.saturating_add(1);
            let counter = match outcome {
                AsicCorrelation::BlockedWrongSession => {
                    &mut self.evidence.blocked_correlations.wrong_session
                }
                AsicCorrelation::BlockedJobLookup => {
                    &mut self.evidence.blocked_correlations.job_lookup
                }
                AsicCorrelation::BlockedWorkStale => {
                    &mut self.evidence.blocked_correlations.work_stale
                }
                AsicCorrelation::BlockedTargetMismatch => {
                    &mut self.evidence.blocked_correlations.target_mismatch
                }
                AsicCorrelation::BlockedOther => &mut self.evidence.blocked_correlations.other,
                AsicCorrelation::Correlated
                | AsicCorrelation::BelowTarget
                | AsicCorrelation::Duplicate => unreachable!("blocked correlation"),
            };
            *counter = counter.saturating_add(1);
        }
        self.record(
            now_ms,
            generation,
            AsicDiagnosticEventKind::Correlation,
            outcome.outcome(),
        );
    }

    fn relation(&self, generation: PoolSessionGeneration) -> AsicGenerationRelation {
        match self.maybe_transition_generation {
            None => AsicGenerationRelation::PreTransition,
            Some(current) if current == generation => AsicGenerationRelation::Replacement,
            Some(_) => AsicGenerationRelation::Stale,
        }
    }

    fn record(
        &mut self,
        now_ms: u64,
        generation: PoolSessionGeneration,
        kind: AsicDiagnosticEventKind,
        outcome: AsicDiagnosticOutcome,
    ) {
        let relation = self.relation(generation);
        self.record_with_relation(now_ms, relation, kind, outcome);
    }

    fn record_with_relation(
        &mut self,
        now_ms: u64,
        generation_relation: AsicGenerationRelation,
        kind: AsicDiagnosticEventKind,
        outcome: AsicDiagnosticOutcome,
    ) {
        let Some(started_ms) = self.maybe_transition_started_ms else {
            return;
        };
        self.event_sequence = self.event_sequence.saturating_add(1);
        self.evidence.latest_event = Some(AsicDiagnosticEvent {
            sequence: self.event_sequence,
            monotonic_offset_ms: now_ms.saturating_sub(started_ms),
            kind,
            generation_relation,
            outcome,
        });
    }

    fn set_first_delay(target: &mut Option<u64>, maybe_started_ms: Option<u64>, now_ms: u64) {
        if target.is_none() {
            *target = maybe_started_ms.map(|started_ms| now_ms.saturating_sub(started_ms));
        }
    }
}

impl AsicPollCompletion {
    const fn outcome(self) -> AsicDiagnosticOutcome {
        match self {
            Self::Idle => AsicDiagnosticOutcome::Idle,
            Self::RegisterRead => AsicDiagnosticOutcome::RegisterRead,
            Self::Discarded(reason) => match reason {
                Bm1366ResultDiscardReason::InvalidLength => {
                    AsicDiagnosticOutcome::DiscardInvalidLength
                }
                Bm1366ResultDiscardReason::InvalidPreamble => {
                    AsicDiagnosticOutcome::DiscardInvalidPreamble
                }
                Bm1366ResultDiscardReason::InvalidCrc => AsicDiagnosticOutcome::DiscardInvalidCrc,
                Bm1366ResultDiscardReason::JobLookup => AsicDiagnosticOutcome::DiscardJobLookup,
                Bm1366ResultDiscardReason::Core => AsicDiagnosticOutcome::DiscardCore,
                Bm1366ResultDiscardReason::AddressInterval => {
                    AsicDiagnosticOutcome::DiscardAddressInterval
                }
                Bm1366ResultDiscardReason::RegisterResponse => {
                    AsicDiagnosticOutcome::DiscardRegisterResponse
                }
                Bm1366ResultDiscardReason::ParserInvariant => {
                    AsicDiagnosticOutcome::DiscardParserInvariant
                }
            },
        }
    }
}

impl AsicCorrelation {
    const fn is_correlated(self) -> bool {
        matches!(self, Self::Correlated | Self::BelowTarget | Self::Duplicate)
    }

    const fn is_blocked(self) -> bool {
        matches!(
            self,
            Self::BlockedWrongSession
                | Self::BlockedJobLookup
                | Self::BlockedWorkStale
                | Self::BlockedTargetMismatch
                | Self::BlockedOther
        )
    }

    const fn outcome(self) -> AsicDiagnosticOutcome {
        match self {
            Self::Correlated => AsicDiagnosticOutcome::Correlated,
            Self::BelowTarget => AsicDiagnosticOutcome::BelowTarget,
            Self::Duplicate => AsicDiagnosticOutcome::Duplicate,
            Self::BlockedWrongSession => AsicDiagnosticOutcome::BlockedWrongSession,
            Self::BlockedJobLookup => AsicDiagnosticOutcome::BlockedJobLookup,
            Self::BlockedWorkStale => AsicDiagnosticOutcome::BlockedWorkStale,
            Self::BlockedTargetMismatch => AsicDiagnosticOutcome::BlockedTargetMismatch,
            Self::BlockedOther => AsicDiagnosticOutcome::BlockedOther,
        }
    }
}
