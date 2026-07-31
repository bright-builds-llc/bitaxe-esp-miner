use serde::{Deserialize, Serialize};

use super::super::markers::CampaignFailureStepMarker;
use super::super::{CampaignTerminalCategory, CAMPAIGN_PREPARATION_SCHEMA};
use super::{CampaignSerialAnalyzer, CampaignSerialEventKind};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignPreparationOutcome {
    Started,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CampaignPreparationProgress {
    pub(super) schema: String,
    pub(super) step: CampaignFailureStepMarker,
    pub(super) outcome: CampaignPreparationOutcome,
}

pub(super) enum PreparationProgressParseError {
    Encoding,
    Json,
    Schema,
}

fn parse_preparation_progress(
    payload: &[u8],
) -> Result<CampaignPreparationProgress, PreparationProgressParseError> {
    let json = std::str::from_utf8(payload).map_err(|_| PreparationProgressParseError::Encoding)?;
    let progress = serde_json::from_str::<CampaignPreparationProgress>(json)
        .map_err(|_| PreparationProgressParseError::Json)?;
    if progress.schema != CAMPAIGN_PREPARATION_SCHEMA || !progress.step.is_preparation() {
        return Err(PreparationProgressParseError::Schema);
    }
    Ok(progress)
}

impl CampaignSerialAnalyzer {
    pub(super) fn process_preparation_progress(
        &mut self,
        payload: &[u8],
        byte_offset: usize,
        line_length: usize,
    ) {
        self.diagnostics.preparation_candidate_count = self
            .diagnostics
            .preparation_candidate_count
            .saturating_add(1);
        let progress = match parse_preparation_progress(payload) {
            Ok(progress) => progress,
            Err(PreparationProgressParseError::Encoding) => {
                self.diagnostics.preparation_invalid_encoding_count = self
                    .diagnostics
                    .preparation_invalid_encoding_count
                    .saturating_add(1);
                return self.record_preparation_failure(
                    byte_offset,
                    line_length,
                    CampaignSerialEventKind::PreparationPayloadInvalidUtf8,
                );
            }
            Err(PreparationProgressParseError::Json) => {
                self.diagnostics.preparation_invalid_json_count = self
                    .diagnostics
                    .preparation_invalid_json_count
                    .saturating_add(1);
                return self.record_preparation_failure(
                    byte_offset,
                    line_length,
                    CampaignSerialEventKind::PreparationJsonInvalid,
                );
            }
            Err(PreparationProgressParseError::Schema) => {
                self.diagnostics.preparation_invalid_schema_count = self
                    .diagnostics
                    .preparation_invalid_schema_count
                    .saturating_add(1);
                return self.record_preparation_failure(
                    byte_offset,
                    line_length,
                    CampaignSerialEventKind::PreparationSchemaInvalid,
                );
            }
        };
        self.diagnostics.accepted_preparation_event_count = self
            .diagnostics
            .accepted_preparation_event_count
            .saturating_add(1);
        self.diagnostics.latest_preparation_event = Some(progress);
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            CampaignSerialEventKind::PreparationEventAccepted,
        );
    }

    fn record_preparation_failure(
        &mut self,
        byte_offset: usize,
        line_length: usize,
        event: CampaignSerialEventKind,
    ) {
        self.maybe_failure
            .get_or_insert(CampaignTerminalCategory::ObservationFailed);
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            event,
        );
    }
}
