use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity};

pub const SCOREBOARD_EVIDENCE_SCHEMA: &str = "bitaxe-scoreboard-evidence-v1";

#[derive(Debug, Clone, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ScoreboardSourceEvidence {
    pub plan_sha256: String,
    pub campaign_result_sha256: String,
    pub campaign_network_sha256: String,
    pub campaign_diagnostics_sha256: String,
    pub source_inventory_sha256: String,
    pub source_semantics_current: bool,
    pub reference_semantics_current: bool,
    pub source_path_count: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ScoreboardObservationEvidence {
    pub fresh_nvs_seed_without_scoreboard_keys: bool,
    pub live_qualified_nonce_observed: bool,
    pub submit_outcome_observed: bool,
    pub entry_count: u8,
    pub exact_wire_shape: bool,
    pub finite_positive_difficulty: bool,
    pub bounded_text_fields: bool,
    pub uppercase_fixed_width_hex: bool,
    pub stable_descending_order: bool,
    pub immediate_repeat_unchanged: bool,
    pub live_spa_route_served: bool,
    pub normal_restart_observed: bool,
    pub boot_session_changed: bool,
    pub boot_ordinal_incremented_once: bool,
    pub software_cpu_reset_observed: bool,
    pub exact_package_after_restart: bool,
    pub boot_mining_disabled: bool,
    pub post_restart_persistence: bool,
    pub post_restart_repeat_unchanged: bool,
}

impl ScoreboardObservationEvidence {
    fn is_complete(self) -> bool {
        self.fresh_nvs_seed_without_scoreboard_keys
            && self.live_qualified_nonce_observed
            && self.submit_outcome_observed
            && (1..=20).contains(&self.entry_count)
            && self.exact_wire_shape
            && self.finite_positive_difficulty
            && self.bounded_text_fields
            && self.uppercase_fixed_width_hex
            && self.stable_descending_order
            && self.immediate_repeat_unchanged
            && self.live_spa_route_served
            && self.normal_restart_observed
            && self.boot_session_changed
            && self.boot_ordinal_incremented_once
            && self.software_cpu_reset_observed
            && self.exact_package_after_restart
            && self.boot_mining_disabled
            && self.post_restart_persistence
            && self.post_restart_repeat_unchanged
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ScoreboardEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u8,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: ScoreboardSourceEvidence,
    pub scoreboard: ScoreboardObservationEvidence,
    pub detector_admitted: bool,
    pub runtime_identity: String,
    pub campaign_profile: String,
    pub campaign_duration_seconds: u64,
    pub campaign_status: String,
    pub safe_stop_confirmed: bool,
    pub cleanup_complete: bool,
    pub hardware_rerun_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl ScoreboardEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SCOREBOARD_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 4
            || self.workflow.command != AutomationCommand::CaptureScoreboardEvidence
        {
            return Err("scoreboard identity is invalid");
        }
        for commit in [&self.source_commit, &self.reference_commit] {
            if !is_commit(commit) {
                return Err("scoreboard commit identity is invalid");
            }
        }
        for digest in [
            &self.package_manifest_sha256,
            &self.workflow.request_sha256,
            &self.source.plan_sha256,
            &self.source.campaign_result_sha256,
            &self.source.campaign_network_sha256,
            &self.source.campaign_diagnostics_sha256,
            &self.source.source_inventory_sha256,
        ] {
            if !is_sha256(digest) {
                return Err("scoreboard digest is invalid");
            }
        }
        if !self.source.source_semantics_current
            || !self.source.reference_semantics_current
            || self.source.source_path_count != 31
        {
            return Err("scoreboard source evidence is incomplete");
        }
        if !self.scoreboard.is_complete() {
            return Err("scoreboard observation quorum is incomplete");
        }
        if !self.detector_admitted
            || self.runtime_identity != "trusted"
            || self.campaign_profile != "conservative"
            || self.campaign_duration_seconds != 600
            || self.campaign_status != "accepted"
            || !self.safe_stop_confirmed
            || !self.cleanup_complete
            || self.hardware_rerun_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("scoreboard campaign evidence is incomplete");
        }
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> ScoreboardEvidence {
        ScoreboardEvidence {
            schema_version: SCOREBOARD_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 4,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureScoreboardEvidence,
                request_sha256: "d".repeat(64),
            },
            source: ScoreboardSourceEvidence {
                plan_sha256: "e".repeat(64),
                campaign_result_sha256: "f".repeat(64),
                campaign_network_sha256: "1".repeat(64),
                campaign_diagnostics_sha256: "2".repeat(64),
                source_inventory_sha256: "3".repeat(64),
                source_semantics_current: true,
                reference_semantics_current: true,
                source_path_count: 31,
            },
            scoreboard: ScoreboardObservationEvidence {
                fresh_nvs_seed_without_scoreboard_keys: true,
                live_qualified_nonce_observed: true,
                submit_outcome_observed: true,
                entry_count: 2,
                exact_wire_shape: true,
                finite_positive_difficulty: true,
                bounded_text_fields: true,
                uppercase_fixed_width_hex: true,
                stable_descending_order: true,
                immediate_repeat_unchanged: true,
                live_spa_route_served: true,
                normal_restart_observed: true,
                boot_session_changed: true,
                boot_ordinal_incremented_once: true,
                software_cpu_reset_observed: true,
                exact_package_after_restart: true,
                boot_mining_disabled: true,
                post_restart_persistence: true,
                post_restart_repeat_unchanged: true,
            },
            detector_admitted: true,
            runtime_identity: "trusted".to_owned(),
            campaign_profile: "conservative".to_owned(),
            campaign_duration_seconds: 600,
            campaign_status: "accepted".to_owned(),
            safe_stop_confirmed: true,
            cleanup_complete: true,
            hardware_rerun_used: false,
            private_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_scoreboard_evidence_passes() {
        // Arrange
        let evidence = evidence();

        // Act / Assert
        assert_eq!(evidence.validate(), Ok(()));
    }

    #[test]
    fn empty_scoreboard_fails() {
        // Arrange
        let mut evidence = evidence();
        evidence.scoreboard.entry_count = 0;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("scoreboard observation quorum is incomplete"));
    }

    #[test]
    fn wrong_attempt_ordinal_fails() {
        // Arrange
        let mut evidence = evidence();
        evidence.attempt_ordinal = 3;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("scoreboard identity is invalid"));
    }
}
