use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, LOG_BUFFER_EVIDENCE_SCHEMA};

const MAX_RETAINED_BODY_BYTES: u64 = 1024 * 1024;
const MAX_RAW_FRAME_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct LogBufferObservationEvidence {
    pub boot_session_sha256: String,
    pub baseline_body_sha256: String,
    pub final_body_sha256: String,
    pub raw_frame_sha256: String,
    pub baseline_bytes: u64,
    pub final_bytes: u64,
    pub raw_frame_bytes: u64,
    pub baseline_marker_count: u64,
    pub final_marker_count: u64,
    pub new_marker_count: u64,
    pub both_download_headers_match: bool,
    pub baseline_is_exact_prefix: bool,
    pub raw_frame_plain_text: bool,
    pub raw_frame_marker_matches: bool,
    pub retained_marker_matches_frame: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct LogBufferEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub log_buffer: LogBufferObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl LogBufferEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != LOG_BUFFER_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("log buffer evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureLogBufferEvidence
        {
            return Err("log buffer workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("log buffer source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.log_buffer.boot_session_sha256.as_str(),
            self.log_buffer.baseline_body_sha256.as_str(),
            self.log_buffer.final_body_sha256.as_str(),
            self.log_buffer.raw_frame_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("log buffer evidence digest is invalid");
            }
        }
        let observation = &self.log_buffer;
        let Some(minimum_final_bytes) = observation
            .baseline_bytes
            .checked_add(observation.raw_frame_bytes)
        else {
            return Err("log buffer observation is incomplete");
        };
        let Some(expected_final_marker_count) = observation.baseline_marker_count.checked_add(1)
        else {
            return Err("log buffer observation is incomplete");
        };
        if observation.baseline_bytes > MAX_RETAINED_BODY_BYTES
            || observation.final_bytes > MAX_RETAINED_BODY_BYTES
            || observation.raw_frame_bytes == 0
            || observation.raw_frame_bytes > MAX_RAW_FRAME_BYTES
            || observation.final_bytes < minimum_final_bytes
            || observation.final_marker_count != expected_final_marker_count
            || observation.new_marker_count != 1
            || !observation.both_download_headers_match
            || !observation.baseline_is_exact_prefix
            || !observation.raw_frame_plain_text
            || !observation.raw_frame_marker_matches
            || !observation.retained_marker_matches_frame
        {
            return Err("log buffer observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("log buffer safety or privacy evidence is invalid");
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
    use super::{LogBufferEvidence, LogBufferObservationEvidence};
    use crate::{AutomationCommand, WorkflowIdentity};

    fn valid_evidence() -> LogBufferEvidence {
        LogBufferEvidence {
            schema_version: "bitaxe-log-buffer-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureLogBufferEvidence,
                request_sha256: "d".repeat(64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            log_buffer: LogBufferObservationEvidence {
                boot_session_sha256: "e".repeat(64),
                baseline_body_sha256: "f".repeat(64),
                final_body_sha256: "1".repeat(64),
                raw_frame_sha256: "2".repeat(64),
                baseline_bytes: 100,
                final_bytes: 150,
                raw_frame_bytes: 32,
                baseline_marker_count: 2,
                final_marker_count: 3,
                new_marker_count: 1,
                both_download_headers_match: true,
                baseline_is_exact_prefix: true,
                raw_frame_plain_text: true,
                raw_frame_marker_matches: true,
                retained_marker_matches_frame: true,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn valid_closed_projection_is_accepted() {
        // Arrange
        let evidence = valid_evidence();

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn missing_new_retained_marker_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.log_buffer.final_marker_count = 2;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("log buffer observation is incomplete"));
    }
}
