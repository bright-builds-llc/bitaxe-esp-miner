use super::preparation::CampaignPreparationProgress;
use super::*;

const DIAGNOSTICS_SCHEMA: &str = "mining-campaign-serial-diagnostics-v4";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignSerialEventKind {
    NonUtf8Line,
    CampaignMarkerAccepted,
    MarkerPayloadInvalidUtf8,
    MarkerJsonInvalid,
    MarkerSchemaInvalid,
    MarkerTruncated,
    RuntimeAttestationCandidate,
    RuntimeAttestationInvalidUtf8,
    RuntimeAttestationLookalike,
    PanicSignatureObserved,
    PreparationEventAccepted,
    PreparationPayloadInvalidUtf8,
    PreparationJsonInvalid,
    PreparationSchemaInvalid,
    PostTerminalBytesIgnored,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct CampaignSerialEvent {
    pub(super) sequence: u64,
    pub(super) byte_offset: u64,
    pub(super) line_length: u32,
    pub(super) kind: CampaignSerialEventKind,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(in crate::campaign) struct RuntimeAttestationParseFailureCountsEvidence {
    pub(super) missing_marker: u64,
    pub(super) malformed_token: u64,
    pub(super) duplicate_field: u64,
    pub(super) unknown_field: u64,
    pub(super) missing_field: u64,
    pub(super) invalid_field: u64,
    pub(super) incomplete_readiness: u64,
}

impl From<bitaxe_api::RuntimeAttestationParseFailureCounts>
    for RuntimeAttestationParseFailureCountsEvidence
{
    fn from(counts: bitaxe_api::RuntimeAttestationParseFailureCounts) -> Self {
        use bitaxe_api::RuntimeAttestationParseFailure as Failure;

        Self {
            missing_marker: counts.count(Failure::MissingMarker),
            malformed_token: counts.count(Failure::MalformedToken),
            duplicate_field: counts.count(Failure::DuplicateField),
            unknown_field: counts.count(Failure::UnknownField),
            missing_field: counts.count(Failure::MissingField),
            invalid_field: counts.count(Failure::InvalidField),
            incomplete_readiness: counts.count(Failure::IncompleteReadiness),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(in crate::campaign) struct CampaignSerialDiagnostics {
    pub(super) schema: &'static str,
    pub(super) observation_started: bool,
    pub(super) total_bytes: u64,
    pub(super) line_count: u64,
    pub(super) complete_line_count: u64,
    pub(super) trailing_byte_count: u64,
    pub(super) non_utf8_line_count: u64,
    pub(super) ignored_invalid_byte_count: u64,
    pub(super) marker_candidate_count: u64,
    pub(super) accepted_marker_count: u64,
    pub(super) marker_invalid_encoding_count: u64,
    pub(super) marker_invalid_json_count: u64,
    pub(super) marker_invalid_schema_count: u64,
    pub(super) marker_truncated_count: u64,
    pub(super) runtime_attestation_candidate_count: u64,
    pub(super) runtime_attestation_lookalike_count: u64,
    pub(super) runtime_attestation_invalid_encoding_count: u64,
    pub(super) runtime_attestation_parse_failure: &'static str,
    pub(super) runtime_attestation_mixed_reset_reason: &'static str,
    pub(super) runtime_attestation_parse_failure_counts:
        RuntimeAttestationParseFailureCountsEvidence,
    pub(super) panic_signature: &'static str,
    pub(super) panic_task_family: &'static str,
    pub(super) panic_signature_count: u64,
    pub(super) preparation_candidate_count: u64,
    pub(super) accepted_preparation_event_count: u64,
    pub(super) preparation_invalid_encoding_count: u64,
    pub(super) preparation_invalid_json_count: u64,
    pub(super) preparation_invalid_schema_count: u64,
    pub(super) latest_preparation_event: Option<CampaignPreparationProgress>,
    pub(super) trailing_partial_count: u64,
    pub(super) post_terminal_ignored_byte_count: u64,
    pub(super) event_count: u64,
    pub(super) events_truncated: bool,
    pub(super) events: Vec<CampaignSerialEvent>,
}

impl CampaignSerialDiagnostics {
    pub(in crate::campaign) fn not_observed() -> Self {
        Self {
            schema: DIAGNOSTICS_SCHEMA,
            observation_started: false,
            total_bytes: 0,
            line_count: 0,
            complete_line_count: 0,
            trailing_byte_count: 0,
            non_utf8_line_count: 0,
            ignored_invalid_byte_count: 0,
            marker_candidate_count: 0,
            accepted_marker_count: 0,
            marker_invalid_encoding_count: 0,
            marker_invalid_json_count: 0,
            marker_invalid_schema_count: 0,
            marker_truncated_count: 0,
            runtime_attestation_candidate_count: 0,
            runtime_attestation_lookalike_count: 0,
            runtime_attestation_invalid_encoding_count: 0,
            runtime_attestation_parse_failure: "not_observed",
            runtime_attestation_mixed_reset_reason: "not_observed",
            runtime_attestation_parse_failure_counts:
                RuntimeAttestationParseFailureCountsEvidence::default(),
            panic_signature: "not_observed",
            panic_task_family: "not_observed",
            panic_signature_count: 0,
            preparation_candidate_count: 0,
            accepted_preparation_event_count: 0,
            preparation_invalid_encoding_count: 0,
            preparation_invalid_json_count: 0,
            preparation_invalid_schema_count: 0,
            latest_preparation_event: None,
            trailing_partial_count: 0,
            post_terminal_ignored_byte_count: 0,
            event_count: 0,
            events_truncated: false,
            events: Vec::new(),
        }
    }

    pub(super) fn observed() -> Self {
        Self {
            observation_started: true,
            runtime_attestation_parse_failure: "none",
            runtime_attestation_mixed_reset_reason: "none",
            panic_signature: "none",
            panic_task_family: "none",
            ..Self::not_observed()
        }
    }

    pub(in crate::campaign) const fn runtime_attestation_parse_failure(&self) -> &'static str {
        self.runtime_attestation_parse_failure
    }

    pub(in crate::campaign) const fn runtime_attestation_parse_failure_counts(
        &self,
    ) -> &RuntimeAttestationParseFailureCountsEvidence {
        &self.runtime_attestation_parse_failure_counts
    }
}
