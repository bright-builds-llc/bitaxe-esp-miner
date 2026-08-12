use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_RESULT_PARSING_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResultParsingSourceEvidence {
    pub work_send_projection_sha256: String,
    pub work_send_projection_current_commit: String,
    pub work_send_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResultParsingObservationEvidence {
    pub result_frame_length_bytes: u64,
    pub strict_length_validation: bool,
    pub preamble_validation: bool,
    pub crc_validation: bool,
    pub job_lookup_validation: bool,
    pub submit_nonce_little_endian: bool,
    pub core_validation: bool,
    pub address_interval_validation: bool,
    pub version_bits_recovered: bool,
    pub known_register_classification: bool,
    pub typed_soft_discard_category_count: u64,
    pub soft_discard_continuation: bool,
    pub live_qualified_result_observed: bool,
    pub accepted_submit_observed: bool,
    pub transcript_path_unchanged: bool,
    pub parser_spans_unchanged: bool,
    pub adapter_nonce_span_unchanged: bool,
    pub worker_nonce_span_unchanged: bool,
    pub correlation_semantics_compatible: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResultParsingEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicResultParsingSourceEvidence,
    pub result_parsing: AsicResultParsingObservationEvidence,
    pub package_admitted: bool,
    pub runtime_identity: String,
    pub runtime_attestation_status: String,
    pub campaign_terminal_category: String,
    pub submit_outcome: String,
    pub safety_status: String,
    pub mine_on_boot_disabled: bool,
    pub safe_stop_confirmed: bool,
    pub lease_cleanup_confirmed: bool,
    pub usb_cleanup_ready: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl AsicResultParsingEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_RESULT_PARSING_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC result-parsing evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicResultParsingEvidence
        {
            return Err("ASIC result-parsing workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
            self.source.work_send_projection_current_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("ASIC result-parsing source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.work_send_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC result-parsing digest is invalid");
            }
        }
        if !self.source.work_send_projection_valid {
            return Err("ASIC result-parsing source evidence is invalid");
        }

        let result = &self.result_parsing;
        if result.result_frame_length_bytes != 11
            || !result.strict_length_validation
            || !result.preamble_validation
            || !result.crc_validation
            || !result.job_lookup_validation
            || !result.submit_nonce_little_endian
            || !result.core_validation
            || !result.address_interval_validation
            || !result.version_bits_recovered
            || !result.known_register_classification
            || result.typed_soft_discard_category_count != 8
            || !result.soft_discard_continuation
            || !result.live_qualified_result_observed
            || !result.accepted_submit_observed
            || !result.transcript_path_unchanged
            || !result.parser_spans_unchanged
            || !result.adapter_nonce_span_unchanged
            || !result.worker_nonce_span_unchanged
            || !result.correlation_semantics_compatible
        {
            return Err("ASIC result-parsing observation is incomplete");
        }
        if !self.package_admitted
            || self.runtime_identity != "trusted"
            || self.runtime_attestation_status != "trusted"
            || self.campaign_terminal_category != "submit_response_observed"
            || self.submit_outcome != "accepted"
            || self.safety_status != "fresh"
            || !self.mine_on_boot_disabled
            || !self.safe_stop_confirmed
            || !self.lease_cleanup_confirmed
            || !self.usb_cleanup_ready
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("ASIC result-parsing campaign or cleanup evidence is invalid");
        }
        Ok(())
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> AsicResultParsingEvidence {
        AsicResultParsingEvidence {
            schema_version: ASIC_RESULT_PARSING_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicResultParsingEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicResultParsingSourceEvidence {
                work_send_projection_sha256: "e".repeat(64),
                work_send_projection_current_commit: "f".repeat(40),
                work_send_projection_valid: true,
            },
            result_parsing: AsicResultParsingObservationEvidence {
                result_frame_length_bytes: 11,
                strict_length_validation: true,
                preamble_validation: true,
                crc_validation: true,
                job_lookup_validation: true,
                submit_nonce_little_endian: true,
                core_validation: true,
                address_interval_validation: true,
                version_bits_recovered: true,
                known_register_classification: true,
                typed_soft_discard_category_count: 8,
                soft_discard_continuation: true,
                live_qualified_result_observed: true,
                accepted_submit_observed: true,
                transcript_path_unchanged: true,
                parser_spans_unchanged: true,
                adapter_nonce_span_unchanged: true,
                worker_nonce_span_unchanged: true,
                correlation_semantics_compatible: true,
            },
            package_admitted: true,
            runtime_identity: "trusted".to_owned(),
            runtime_attestation_status: "trusted".to_owned(),
            campaign_terminal_category: "submit_response_observed".to_owned(),
            submit_outcome: "accepted".to_owned(),
            safety_status: "fresh".to_owned(),
            mine_on_boot_disabled: true,
            safe_stop_confirmed: true,
            lease_cleanup_confirmed: true,
            usb_cleanup_ready: true,
            hardware_rerun_used: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_closed_projection_is_accepted() {
        // Arrange
        let candidate = evidence();

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn incomplete_parser_observation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.result_parsing.crc_validation = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("ASIC result-parsing observation is incomplete"));
    }

    #[test]
    fn hardware_rerun_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.hardware_rerun_used = true;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC result-parsing campaign or cleanup evidence is invalid")
        );
    }
}
