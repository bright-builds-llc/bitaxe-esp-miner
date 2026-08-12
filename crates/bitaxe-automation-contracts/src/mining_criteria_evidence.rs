use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, MINING_CRITERIA_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct MiningCriteriaSourceEvidence {
    pub phase21_summary_sha256: String,
    pub phase21_summary_valid: bool,
    pub phase21_smoke_sha256: String,
    pub phase21_smoke_valid: bool,
    pub phase21_soak_sha256: String,
    pub phase21_soak_valid: bool,
    pub protocol_coordinator_sha256: String,
    pub protocol_coordinator_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct MiningCriteriaObservationEvidence {
    pub historical_smoke_controlled_no_share: bool,
    pub historical_soak_duration_seconds: u64,
    pub historical_watchdog_passed: bool,
    pub historical_telemetry_observed: bool,
    pub historical_safe_stop_confirmed: bool,
    pub current_duration_seconds: u64,
    pub upstream_default_profile_required: bool,
    pub active_duration_accounting: bool,
    pub full_duration_required: bool,
    pub accepted_share_required: bool,
    pub network_correlation_required: bool,
    pub safe_stop_required: bool,
    pub private_evidence_required: bool,
    pub redaction_required: bool,
    pub source_spans_compatible: bool,
    pub terminal_attempt_reopened: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct MiningCriteriaEvidence {
    pub schema_version: String,
    pub board: u16,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: MiningCriteriaSourceEvidence,
    pub criteria: MiningCriteriaObservationEvidence,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl MiningCriteriaEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != MINING_CRITERIA_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("mining criteria evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectMiningCriteriaEvidence
        {
            return Err("mining criteria workflow identity is invalid");
        }
        for commit in [
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("mining criteria source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.phase21_summary_sha256.as_str(),
            self.source.phase21_smoke_sha256.as_str(),
            self.source.phase21_soak_sha256.as_str(),
            self.source.protocol_coordinator_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("mining criteria digest is invalid");
            }
        }
        if !self.source.phase21_summary_valid
            || !self.source.phase21_smoke_valid
            || !self.source.phase21_soak_valid
            || !self.source.protocol_coordinator_valid
        {
            return Err("mining criteria source evidence is invalid");
        }
        let criteria = &self.criteria;
        if !criteria.historical_smoke_controlled_no_share
            || criteria.historical_soak_duration_seconds != 300
            || !criteria.historical_watchdog_passed
            || !criteria.historical_telemetry_observed
            || !criteria.historical_safe_stop_confirmed
            || criteria.current_duration_seconds != 600
            || !criteria.upstream_default_profile_required
            || !criteria.active_duration_accounting
            || !criteria.full_duration_required
            || !criteria.accepted_share_required
            || !criteria.network_correlation_required
            || !criteria.safe_stop_required
            || !criteria.private_evidence_required
            || !criteria.redaction_required
            || !criteria.source_spans_compatible
            || criteria.terminal_attempt_reopened
        {
            return Err("mining criteria observation is incomplete");
        }
        if self.hardware_rerun_used || self.redaction_status != "passed" {
            return Err("mining criteria effect or redaction boundary is invalid");
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

    fn evidence() -> MiningCriteriaEvidence {
        MiningCriteriaEvidence {
            schema_version: MINING_CRITERIA_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            current_source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectMiningCriteriaEvidence,
                request_sha256: "c".repeat(64),
            },
            source: MiningCriteriaSourceEvidence {
                phase21_summary_sha256: "d".repeat(64),
                phase21_summary_valid: true,
                phase21_smoke_sha256: "e".repeat(64),
                phase21_smoke_valid: true,
                phase21_soak_sha256: "f".repeat(64),
                phase21_soak_valid: true,
                protocol_coordinator_sha256: "1".repeat(64),
                protocol_coordinator_valid: true,
            },
            criteria: MiningCriteriaObservationEvidence {
                historical_smoke_controlled_no_share: true,
                historical_soak_duration_seconds: 300,
                historical_watchdog_passed: true,
                historical_telemetry_observed: true,
                historical_safe_stop_confirmed: true,
                current_duration_seconds: 600,
                upstream_default_profile_required: true,
                active_duration_accounting: true,
                full_duration_required: true,
                accepted_share_required: true,
                network_correlation_required: true,
                safe_stop_required: true,
                private_evidence_required: true,
                redaction_required: true,
                source_spans_compatible: true,
                terminal_attempt_reopened: false,
            },
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
    fn incomplete_current_criteria_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.criteria.full_duration_required = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("mining criteria observation is incomplete"));
    }

    #[test]
    fn hardware_rerun_or_invalid_source_is_rejected() {
        // Arrange
        let mut rerun = evidence();
        rerun.hardware_rerun_used = true;
        let mut invalid_source = evidence();
        invalid_source.source.phase21_soak_valid = false;

        // Act / Assert
        assert_eq!(
            rerun.validate(),
            Err("mining criteria effect or redaction boundary is invalid")
        );
        assert_eq!(
            invalid_source.validate(),
            Err("mining criteria source evidence is invalid")
        );
    }
}
