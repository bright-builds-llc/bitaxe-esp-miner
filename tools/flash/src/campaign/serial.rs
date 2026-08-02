use std::collections::VecDeque;

use super::markers::{
    CampaignMarkerAggregate, CampaignStateMarker, CampaignStatusMarker,
    ObservationRequirementsMarker,
};
use super::*;
mod framing;
mod preparation;
use super::markers::CampaignFailureStepMarker;
use framing::{count_invalid_utf8_bytes, find_bytes};
use preparation::{CampaignPreparationOutcome, CampaignPreparationProgress};
const DIAGNOSTICS_SCHEMA: &str = "mining-campaign-serial-diagnostics-v1";
const TRACE_EDGE_CAPACITY: usize = 32;
const MAX_RECORDED_LINE_LENGTH: usize = 4_096;
const MAX_PENDING_LINE_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignSerialOutcomeDetail {
    Clean,
    MarkerMissing,
    MarkerPayloadInvalidUtf8,
    MarkerJsonInvalid,
    MarkerSchemaInvalid,
}

impl CampaignSerialOutcomeDetail {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::MarkerMissing => "marker_missing",
            Self::MarkerPayloadInvalidUtf8 => "marker_payload_invalid_utf8",
            Self::MarkerJsonInvalid => "marker_json_invalid",
            Self::MarkerSchemaInvalid => "marker_schema_invalid",
        }
    }
}

impl Default for CampaignSerialOutcomeDetail {
    fn default() -> Self {
        Self::Clean
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CampaignSerialEventKind {
    NonUtf8Line,
    CampaignMarkerAccepted,
    MarkerPayloadInvalidUtf8,
    MarkerJsonInvalid,
    MarkerSchemaInvalid,
    MarkerTruncated,
    RuntimeAttestationCandidate,
    RuntimeAttestationInvalidUtf8,
    PreparationEventAccepted,
    PreparationPayloadInvalidUtf8,
    PreparationJsonInvalid,
    PreparationSchemaInvalid,
    PostTerminalBytesIgnored,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct CampaignSerialEvent {
    sequence: u64,
    byte_offset: u64,
    line_length: u32,
    kind: CampaignSerialEventKind,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct CampaignSerialDiagnostics {
    schema: &'static str,
    observation_started: bool,
    total_bytes: u64,
    line_count: u64,
    complete_line_count: u64,
    trailing_byte_count: u64,
    non_utf8_line_count: u64,
    ignored_invalid_byte_count: u64,
    marker_candidate_count: u64,
    accepted_marker_count: u64,
    marker_invalid_encoding_count: u64,
    marker_invalid_json_count: u64,
    marker_invalid_schema_count: u64,
    marker_truncated_count: u64,
    runtime_attestation_candidate_count: u64,
    runtime_attestation_invalid_encoding_count: u64,
    preparation_candidate_count: u64,
    accepted_preparation_event_count: u64,
    preparation_invalid_encoding_count: u64,
    preparation_invalid_json_count: u64,
    preparation_invalid_schema_count: u64,
    latest_preparation_event: Option<CampaignPreparationProgress>,
    trailing_partial_count: u64,
    post_terminal_ignored_byte_count: u64,
    event_count: u64,
    events_truncated: bool,
    events: Vec<CampaignSerialEvent>,
}

impl CampaignSerialDiagnostics {
    pub(super) fn not_observed() -> Self {
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
            runtime_attestation_invalid_encoding_count: 0,
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
}

#[derive(Debug)]
pub(crate) struct CampaignSerialCapture {
    pub(super) aggregate: CampaignMarkerAggregate,
    #[cfg(test)]
    pub(super) markers: Vec<CampaignStatusMarker>,
    pub(super) diagnostics: CampaignSerialDiagnostics,
    pub(super) outcome_detail: CampaignSerialOutcomeDetail,
    pub(super) maybe_failure: Option<CampaignTerminalCategory>,
    runtime_attestations: RuntimeAttestationAccumulator,
}

impl CampaignSerialCapture {
    pub(super) fn runtime_attestation_status(
        &self,
        expected: &ExpectedRuntimeAttestationIdentity,
    ) -> RuntimeAttestationStatus {
        if self.diagnostics.runtime_attestation_invalid_encoding_count > 0 {
            return RuntimeAttestationStatus::Malformed;
        }
        self.runtime_attestations.status(expected)
    }
}

#[derive(Debug, Default)]
struct BoundedEventTrace {
    total: u64,
    first: Vec<CampaignSerialEvent>,
    last: VecDeque<CampaignSerialEvent>,
}

impl BoundedEventTrace {
    fn push(&mut self, byte_offset: u64, line_length: usize, kind: CampaignSerialEventKind) {
        self.total = self.total.saturating_add(1);
        let event = CampaignSerialEvent {
            sequence: self.total,
            byte_offset,
            line_length: u32::try_from(line_length.min(MAX_RECORDED_LINE_LENGTH))
                .unwrap_or(MAX_RECORDED_LINE_LENGTH as u32),
            kind,
        };
        if self.first.len() < TRACE_EDGE_CAPACITY {
            self.first.push(event);
            return;
        }
        if self.last.len() == TRACE_EDGE_CAPACITY {
            self.last.pop_front();
        }
        self.last.push_back(event);
    }

    fn finish(self) -> (u64, bool, Vec<CampaignSerialEvent>) {
        let mut events = self.first;
        events.extend(self.last);
        let truncated = usize::try_from(self.total).map_or(true, |total| total > events.len());
        (self.total, truncated, events)
    }
}

pub(crate) struct CampaignSerialAnalyzer {
    admission: CampaignAdmission,
    #[cfg(test)]
    observed_bytes: usize,
    processed_bytes: usize,
    pending_line: Vec<u8>,
    aggregate: CampaignMarkerAggregate,
    #[cfg(test)]
    markers: Vec<CampaignStatusMarker>,
    runtime_attestations: RuntimeAttestationAccumulator,
    diagnostics: CampaignSerialDiagnostics,
    outcome_detail: CampaignSerialOutcomeDetail,
    maybe_failure: Option<CampaignTerminalCategory>,
    terminal_boundary_reached: bool,
    trace: BoundedEventTrace,
}

impl CampaignSerialAnalyzer {
    pub(crate) fn new(admission: CampaignAdmission) -> Self {
        Self {
            admission,
            #[cfg(test)]
            observed_bytes: 0,
            processed_bytes: 0,
            pending_line: Vec::new(),
            aggregate: CampaignMarkerAggregate::default(),
            #[cfg(test)]
            markers: Vec::new(),
            runtime_attestations: RuntimeAttestationAccumulator::default(),
            diagnostics: CampaignSerialDiagnostics {
                observation_started: true,
                ..CampaignSerialDiagnostics::not_observed()
            },
            outcome_detail: CampaignSerialOutcomeDetail::Clean,
            maybe_failure: None,
            terminal_boundary_reached: false,
            trace: BoundedEventTrace::default(),
        }
    }

    pub(crate) fn terminal_consumed(&self) -> bool {
        self.aggregate
            .terminal
            .as_ref()
            .is_some_and(|marker| marker.campaign_state == CampaignStateMarker::Consumed)
    }

    pub(crate) fn observe_chunk(&mut self, bytes: &[u8]) -> bool {
        self.diagnostics.total_bytes = self
            .diagnostics
            .total_bytes
            .saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        if self.terminal_boundary_reached {
            self.record_post_terminal_bytes(self.processed_bytes, bytes.len());
            self.processed_bytes = self.processed_bytes.saturating_add(bytes.len());
            return true;
        }
        for (index, chunk) in bytes.chunks(MAX_PENDING_LINE_BYTES).enumerate() {
            self.pending_line.extend_from_slice(chunk);
            self.process_complete_lines();
            self.bound_pending_line();
            if self.terminal_boundary_reached {
                let consumed = index
                    .saturating_add(1)
                    .saturating_mul(MAX_PENDING_LINE_BYTES)
                    .min(bytes.len());
                let remaining = bytes.len().saturating_sub(consumed);
                self.record_post_terminal_bytes(self.processed_bytes, remaining);
                self.processed_bytes = self.processed_bytes.saturating_add(remaining);
                break;
            }
        }
        self.refresh_contract_failure();
        self.should_stop()
    }

    #[cfg(test)]
    pub(crate) fn observe_snapshot(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() < self.observed_bytes {
            self.maybe_failure
                .get_or_insert(CampaignTerminalCategory::ObservationFailed);
            return true;
        }
        let delta = &bytes[self.observed_bytes..];
        self.observed_bytes = bytes.len();
        self.observe_chunk(delta)
    }

    pub(crate) fn finish(mut self) -> CampaignSerialCapture {
        self.process_complete_lines();
        if !self.pending_line.is_empty() {
            let line = std::mem::take(&mut self.pending_line);
            self.diagnostics.trailing_byte_count = u64::try_from(line.len()).unwrap_or(u64::MAX);
            self.process_line(&line, self.processed_bytes, true);
            self.processed_bytes = self.processed_bytes.saturating_add(line.len());
        }
        self.refresh_contract_failure();
        self.refresh_incomplete_preparation_failure();
        if self.aggregate.marker_count == 0
            && self.maybe_failure.is_none()
            && self.outcome_detail == CampaignSerialOutcomeDetail::Clean
        {
            self.outcome_detail = CampaignSerialOutcomeDetail::MarkerMissing;
        }
        let (event_count, events_truncated, events) = self.trace.finish();
        self.diagnostics.event_count = event_count;
        self.diagnostics.events_truncated = events_truncated;
        self.diagnostics.events = events;
        CampaignSerialCapture {
            aggregate: self.aggregate,
            #[cfg(test)]
            markers: self.markers,
            diagnostics: self.diagnostics,
            outcome_detail: self.outcome_detail,
            maybe_failure: self.maybe_failure,
            runtime_attestations: self.runtime_attestations,
        }
    }

    fn bound_pending_line(&mut self) {
        if self.pending_line.len() <= MAX_PENDING_LINE_BYTES {
            return;
        }
        let retained = self.pending_line.split_off(
            self.pending_line
                .len()
                .saturating_sub(MAX_PENDING_LINE_BYTES),
        );
        let discarded = std::mem::replace(&mut self.pending_line, retained);
        let discarded_length = discarded.len();
        self.diagnostics.ignored_invalid_byte_count = self
            .diagnostics
            .ignored_invalid_byte_count
            .saturating_add(count_invalid_utf8_bytes(&discarded));
        self.processed_bytes = self.processed_bytes.saturating_add(discarded_length);
    }

    fn process_complete_lines(&mut self) {
        while let Some(newline) = self.pending_line.iter().position(|byte| *byte == b'\n') {
            let mut line = self.pending_line.drain(..=newline).collect::<Vec<_>>();
            line.pop();
            let byte_offset = self.processed_bytes;
            self.processed_bytes = self
                .processed_bytes
                .saturating_add(line.len())
                .saturating_add(1);
            self.diagnostics.complete_line_count =
                self.diagnostics.complete_line_count.saturating_add(1);
            self.process_line(&line, byte_offset, false);
            if self.accepted_live_terminal_boundary() {
                self.terminal_boundary_reached = true;
                let ignored_offset = self.processed_bytes;
                let ignored_length = self.pending_line.len();
                self.record_post_terminal_bytes(ignored_offset, ignored_length);
                self.processed_bytes = self.processed_bytes.saturating_add(ignored_length);
                self.pending_line.clear();
                break;
            }
        }
    }

    fn accepted_live_terminal_boundary(&self) -> bool {
        self.admission.stage != MiningCampaignStage::Observation
            && self.maybe_failure.is_none()
            && (self.aggregate.assess(self.admission).is_ok()
                || self
                    .aggregate
                    .terminal
                    .as_ref()
                    .is_some_and(|marker| marker.campaign_state == CampaignStateMarker::Consumed))
    }

    fn record_post_terminal_bytes(&mut self, byte_offset: usize, byte_count: usize) {
        if byte_count == 0 {
            return;
        }
        self.diagnostics.post_terminal_ignored_byte_count = self
            .diagnostics
            .post_terminal_ignored_byte_count
            .saturating_add(u64::try_from(byte_count).unwrap_or(u64::MAX));
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            byte_count,
            CampaignSerialEventKind::PostTerminalBytesIgnored,
        );
    }

    fn process_line(&mut self, line: &[u8], byte_offset: usize, trailing: bool) {
        self.diagnostics.line_count = self.diagnostics.line_count.saturating_add(1);
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let marker_index = find_bytes(line, CAMPAIGN_MARKER_PREFIX.as_bytes());
        let attestation_index =
            find_bytes(line, bitaxe_api::RUNTIME_BOOT_ATTESTATION_MARKER.as_bytes());
        let preparation_index = find_bytes(line, CAMPAIGN_PREPARATION_PREFIX.as_bytes());
        let candidate_index = [marker_index, attestation_index, preparation_index]
            .into_iter()
            .flatten()
            .min();
        if std::str::from_utf8(line).is_err() {
            self.diagnostics.non_utf8_line_count =
                self.diagnostics.non_utf8_line_count.saturating_add(1);
            let outside_candidate = candidate_index.map_or(line, |index| &line[..index]);
            self.diagnostics.ignored_invalid_byte_count = self
                .diagnostics
                .ignored_invalid_byte_count
                .saturating_add(count_invalid_utf8_bytes(outside_candidate));
            self.trace.push(
                u64::try_from(byte_offset).unwrap_or(u64::MAX),
                line.len(),
                CampaignSerialEventKind::NonUtf8Line,
            );
        }
        if let Some(index) = attestation_index {
            self.process_runtime_attestation(
                &line[index..],
                byte_offset.saturating_add(index),
                line.len().saturating_sub(index),
            );
        }
        if let Some(index) = preparation_index {
            self.process_preparation_progress(
                &line[index + CAMPAIGN_PREPARATION_PREFIX.len()..],
                byte_offset.saturating_add(index),
                line.len().saturating_sub(index),
            );
        }
        if let Some(index) = marker_index {
            self.process_marker(
                &line[index + CAMPAIGN_MARKER_PREFIX.len()..],
                byte_offset.saturating_add(index),
                line.len().saturating_sub(index),
                trailing,
            );
        }
    }

    fn process_runtime_attestation(
        &mut self,
        candidate: &[u8],
        byte_offset: usize,
        line_length: usize,
    ) {
        self.diagnostics.runtime_attestation_candidate_count = self
            .diagnostics
            .runtime_attestation_candidate_count
            .saturating_add(1);
        let Ok(text) = std::str::from_utf8(candidate) else {
            self.diagnostics.runtime_attestation_invalid_encoding_count = self
                .diagnostics
                .runtime_attestation_invalid_encoding_count
                .saturating_add(1);
            self.trace.push(
                u64::try_from(byte_offset).unwrap_or(u64::MAX),
                line_length,
                CampaignSerialEventKind::RuntimeAttestationInvalidUtf8,
            );
            return;
        };
        self.runtime_attestations.observe_line(text);
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            CampaignSerialEventKind::RuntimeAttestationCandidate,
        );
    }

    fn process_marker(
        &mut self,
        payload: &[u8],
        byte_offset: usize,
        line_length: usize,
        trailing: bool,
    ) {
        self.diagnostics.marker_candidate_count =
            self.diagnostics.marker_candidate_count.saturating_add(1);
        if trailing {
            self.diagnostics.trailing_partial_count =
                self.diagnostics.trailing_partial_count.saturating_add(1);
        }
        let Ok(json) = std::str::from_utf8(payload) else {
            self.record_marker_failure(
                CampaignSerialOutcomeDetail::MarkerPayloadInvalidUtf8,
                CampaignSerialEventKind::MarkerPayloadInvalidUtf8,
                byte_offset,
                line_length,
            );
            self.diagnostics.marker_invalid_encoding_count = self
                .diagnostics
                .marker_invalid_encoding_count
                .saturating_add(1);
            return;
        };
        let marker = match serde_json::from_str::<CampaignStatusMarker>(json) {
            Ok(marker) => marker,
            Err(error) if error.is_eof() => {
                self.diagnostics.marker_truncated_count =
                    self.diagnostics.marker_truncated_count.saturating_add(1);
                self.trace.push(
                    u64::try_from(byte_offset).unwrap_or(u64::MAX),
                    line_length,
                    CampaignSerialEventKind::MarkerTruncated,
                );
                return;
            }
            Err(_) => {
                self.diagnostics.marker_invalid_json_count =
                    self.diagnostics.marker_invalid_json_count.saturating_add(1);
                self.record_marker_failure(
                    CampaignSerialOutcomeDetail::MarkerJsonInvalid,
                    CampaignSerialEventKind::MarkerJsonInvalid,
                    byte_offset,
                    line_length,
                );
                return;
            }
        };
        if marker.schema != CAMPAIGN_MARKER_SCHEMA
            || marker.fresh_observation_count != marker.observation_freshness.fresh_count()
            || marker.observation_requirements != ObservationRequirementsMarker::ULTRA_205
            || !marker.failure.is_valid()
        {
            self.diagnostics.marker_invalid_schema_count = self
                .diagnostics
                .marker_invalid_schema_count
                .saturating_add(1);
            self.record_marker_failure(
                CampaignSerialOutcomeDetail::MarkerSchemaInvalid,
                CampaignSerialEventKind::MarkerSchemaInvalid,
                byte_offset,
                line_length,
            );
            return;
        }
        #[cfg(test)]
        self.markers.push(marker.clone());
        let maybe_failure = self.aggregate.observe(marker, self.admission);
        self.diagnostics.accepted_marker_count =
            self.diagnostics.accepted_marker_count.saturating_add(1);
        if let Some(category) = maybe_failure {
            self.maybe_failure.get_or_insert(category);
        }
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            CampaignSerialEventKind::CampaignMarkerAccepted,
        );
    }

    fn record_marker_failure(
        &mut self,
        detail: CampaignSerialOutcomeDetail,
        event: CampaignSerialEventKind,
        byte_offset: usize,
        line_length: usize,
    ) {
        if self.outcome_detail == CampaignSerialOutcomeDetail::Clean {
            self.outcome_detail = detail;
        }
        self.maybe_failure
            .get_or_insert(CampaignTerminalCategory::MarkerInvalid);
        self.trace.push(
            u64::try_from(byte_offset).unwrap_or(u64::MAX),
            line_length,
            event,
        );
    }

    fn refresh_contract_failure(&mut self) {
        if self.maybe_failure.is_some() {
            return;
        }
        self.maybe_failure = self.aggregate.maybe_failure_category;
    }

    fn refresh_incomplete_preparation_failure(&mut self) {
        if self.admission.stage == MiningCampaignStage::Observation || self.maybe_failure.is_some()
        {
            return;
        }
        let Some(progress) = self.diagnostics.latest_preparation_event.as_ref() else {
            return;
        };
        let preparation_completed = progress.step
            == CampaignFailureStepMarker::RetainProductionUart
            && progress.outcome == CampaignPreparationOutcome::Completed;
        if !preparation_completed {
            self.maybe_failure = Some(CampaignTerminalCategory::HardwarePreparationFailed);
        }
    }

    fn should_stop(&self) -> bool {
        if self.admission.stage == MiningCampaignStage::Observation {
            return false;
        }
        if self.maybe_failure.is_some() {
            return true;
        }
        self.aggregate.assess(self.admission).is_ok()
            || self
                .aggregate
                .terminal
                .as_ref()
                .is_some_and(|marker| marker.campaign_state == CampaignStateMarker::Consumed)
    }
}

#[cfg(test)]
pub(crate) fn analyze_campaign_serial_bytes(
    bytes: &[u8],
    admission: CampaignAdmission,
) -> CampaignSerialCapture {
    let mut analyzer = CampaignSerialAnalyzer::new(admission);
    analyzer.observe_chunk(bytes);
    analyzer.finish()
}
#[cfg(test)]
pub(crate) fn campaign_serial_should_stop(bytes: &[u8], admission: CampaignAdmission) -> bool {
    let mut analyzer = CampaignSerialAnalyzer::new(admission);
    analyzer.observe_chunk(bytes)
}

#[cfg(test)]
#[path = "serial/tests.rs"]
mod tests;
