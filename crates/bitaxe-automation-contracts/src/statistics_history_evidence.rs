use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, STATISTICS_HISTORY_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct StatisticsHistoryObservationEvidence {
    pub original_setting_sha256: String,
    pub enabled_setting_sha256: String,
    pub mutation_request_field_count: u16,
    pub enabled_readback_confirmed: bool,
    pub label_count: u16,
    pub row_width: u16,
    pub sample_count: u16,
    pub interval_count: u16,
    pub minimum_interval_ms: u64,
    pub maximum_interval_ms: u64,
    pub timestamps_strictly_increasing: bool,
    pub finite_numeric_rows: bool,
    pub immediate_repeat_unchanged: bool,
    pub later_producer_growth: bool,
    pub restoration_complete: bool,
    pub zero_setting_clear_status: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct StatisticsHistoryEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub plan_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub statistics_history: StatisticsHistoryObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub recovery_flash_used: bool,
    pub recovery_origin_readmitted: bool,
    pub private_modes_valid: bool,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl StatisticsHistoryEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != STATISTICS_HISTORY_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("statistics history evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureStatisticsHistoryEvidence
        {
            return Err("statistics history workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.plan_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.statistics_history.original_setting_sha256.as_str(),
            self.statistics_history.enabled_setting_sha256.as_str(),
        ] {
            if !is_sha256(digest) {
                return Err("statistics history evidence digest is invalid");
            }
        }
        if !is_commit(&self.source_commit) || !is_commit(&self.reference_commit) {
            return Err("statistics history source identity is invalid");
        }
        let history = &self.statistics_history;
        if history.mutation_request_field_count != 1
            || !history.enabled_readback_confirmed
            || history.label_count != 19
            || history.row_width != 19
            || history.sample_count < 3
            || history.interval_count != history.sample_count - 1
            || !(750..=1_500).contains(&history.minimum_interval_ms)
            || !(history.minimum_interval_ms..=1_500).contains(&history.maximum_interval_ms)
            || !history.timestamps_strictly_increasing
            || !history.finite_numeric_rows
            || !history.immediate_repeat_unchanged
            || !history.later_producer_growth
            || !history.restoration_complete
            || !matches!(
                history.zero_setting_clear_status.as_str(),
                "observed" | "not_applicable"
            )
        {
            return Err("statistics history observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || (self.recovery_flash_used && !self.recovery_origin_readmitted)
            || (!self.recovery_flash_used && self.recovery_origin_readmitted)
            || !self.private_modes_valid
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("statistics history safety or privacy evidence is invalid");
        }
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{StatisticsHistoryEvidence, StatisticsHistoryObservationEvidence};
    use crate::{AutomationCommand, WorkflowIdentity};

    fn valid_evidence() -> StatisticsHistoryEvidence {
        StatisticsHistoryEvidence {
            schema_version: "bitaxe-statistics-history-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            plan_sha256: "d".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureStatisticsHistoryEvidence,
                request_sha256: "e".repeat(64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            statistics_history: StatisticsHistoryObservationEvidence {
                original_setting_sha256: "f".repeat(64),
                enabled_setting_sha256: "1".repeat(64),
                mutation_request_field_count: 1,
                enabled_readback_confirmed: true,
                label_count: 19,
                row_width: 19,
                sample_count: 4,
                interval_count: 3,
                minimum_interval_ms: 998,
                maximum_interval_ms: 1_003,
                timestamps_strictly_increasing: true,
                finite_numeric_rows: true,
                immediate_repeat_unchanged: true,
                later_producer_growth: true,
                restoration_complete: true,
                zero_setting_clear_status: "observed".to_owned(),
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            recovery_flash_used: false,
            recovery_origin_readmitted: false,
            private_modes_valid: true,
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
    fn cadence_outside_the_live_tolerance_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.statistics_history.maximum_interval_ms = 1_501;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("statistics history observation is incomplete"));
    }

    #[test]
    fn recovery_flash_requires_a_readmitted_origin() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.recovery_flash_used = true;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(
            result,
            Err("statistics history safety or privacy evidence is invalid")
        );
    }
}
