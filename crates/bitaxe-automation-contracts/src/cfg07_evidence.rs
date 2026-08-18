use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, CFG07_EVIDENCE_SCHEMA};

const EXPECTED_SOURCE_PATHS: u16 = 17;
const EXPECTED_PRODUCTION_PATHS: u16 = 7;
const EXPECTED_REFERENCE_PATHS: u16 = 2;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cfg07SourceEvidence {
    pub plan_sha256: String,
    pub attempt_plan_sha256: String,
    pub attempt_closure_sha256: String,
    pub safe10_projection_sha256: String,
    pub current_source_inventory_sha256: String,
    pub attempt_source_inventory_sha256: String,
    pub source_semantics_current: bool,
    pub reference_semantics_current: bool,
    pub attempt_source_compatible: bool,
    pub source_path_count: u16,
    pub production_path_count: u16,
    pub reference_path_count: u16,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cfg07CredentialEvidence {
    pub runtime_credentials_input: String,
    pub wifi_input_required: bool,
    pub pool_input_required: bool,
    pub inputs_forwarded_to_campaign: bool,
    pub live_mining_credentials_consumed: bool,
    pub accepted_submit_observed: bool,
    pub committed_credential_values: String,
    pub raw_artifacts_committed: String,
    pub credential_contents_read_by_projector: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cfg07Evidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: Cfg07SourceEvidence,
    pub credentials: Cfg07CredentialEvidence,
    pub detector_admitted: bool,
    pub runtime_identity: String,
    pub campaign_stage: String,
    pub campaign_profile: String,
    pub campaign_status: String,
    pub network_status: String,
    pub safe_stop_status: String,
    pub cleanup_complete: bool,
    pub protected_modes_valid: bool,
    pub redaction_status: String,
}

impl Cfg07Evidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != CFG07_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 3
        {
            return Err("CFG-07 evidence schema, board, or attempt is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectCfg07Evidence
        {
            return Err("CFG-07 workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !lower_hex(commit, 40) {
                return Err("CFG-07 source commit identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.plan_sha256.as_str(),
            self.source.attempt_plan_sha256.as_str(),
            self.source.attempt_closure_sha256.as_str(),
            self.source.safe10_projection_sha256.as_str(),
            self.source.current_source_inventory_sha256.as_str(),
            self.source.attempt_source_inventory_sha256.as_str(),
        ] {
            if !lower_hex(digest, 64) {
                return Err("CFG-07 evidence digest is invalid");
            }
        }
        if !self.source.source_semantics_current
            || !self.source.reference_semantics_current
            || !self.source.attempt_source_compatible
            || self.source.source_path_count != EXPECTED_SOURCE_PATHS
            || self.source.production_path_count != EXPECTED_PRODUCTION_PATHS
            || self.source.reference_path_count != EXPECTED_REFERENCE_PATHS
        {
            return Err("CFG-07 source compatibility is invalid");
        }
        if self.credentials.runtime_credentials_input != "local-owner-supplied"
            || !self.credentials.wifi_input_required
            || !self.credentials.pool_input_required
            || !self.credentials.inputs_forwarded_to_campaign
            || !self.credentials.live_mining_credentials_consumed
            || !self.credentials.accepted_submit_observed
            || self.credentials.committed_credential_values != "none"
            || self.credentials.raw_artifacts_committed != "no"
            || self.credentials.credential_contents_read_by_projector
        {
            return Err("CFG-07 runtime credential proof is incomplete");
        }
        if !self.detector_admitted
            || self.runtime_identity != "trusted"
            || self.campaign_stage != "live-share"
            || self.campaign_profile != "conservative"
            || self.campaign_status != "accepted"
            || self.network_status != "accepted"
            || self.safe_stop_status != "complete"
            || !self.cleanup_complete
            || !self.protected_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("CFG-07 live mining or privacy quorum is incomplete");
        }
        Ok(())
    }
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> Cfg07Evidence {
        Cfg07Evidence {
            schema_version: CFG07_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 3,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectCfg07Evidence,
                request_sha256: "d".repeat(64),
            },
            source: Cfg07SourceEvidence {
                plan_sha256: "e".repeat(64),
                attempt_plan_sha256: "f".repeat(64),
                attempt_closure_sha256: "1".repeat(64),
                safe10_projection_sha256: "2".repeat(64),
                current_source_inventory_sha256: "3".repeat(64),
                attempt_source_inventory_sha256: "4".repeat(64),
                source_semantics_current: true,
                reference_semantics_current: true,
                attempt_source_compatible: true,
                source_path_count: EXPECTED_SOURCE_PATHS,
                production_path_count: EXPECTED_PRODUCTION_PATHS,
                reference_path_count: EXPECTED_REFERENCE_PATHS,
            },
            credentials: Cfg07CredentialEvidence {
                runtime_credentials_input: "local-owner-supplied".to_owned(),
                wifi_input_required: true,
                pool_input_required: true,
                inputs_forwarded_to_campaign: true,
                live_mining_credentials_consumed: true,
                accepted_submit_observed: true,
                committed_credential_values: "none".to_owned(),
                raw_artifacts_committed: "no".to_owned(),
                credential_contents_read_by_projector: false,
            },
            detector_admitted: true,
            runtime_identity: "trusted".to_owned(),
            campaign_stage: "live-share".to_owned(),
            campaign_profile: "conservative".to_owned(),
            campaign_status: "accepted".to_owned(),
            network_status: "accepted".to_owned(),
            safe_stop_status: "complete".to_owned(),
            cleanup_complete: true,
            protected_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_public_runtime_credential_evidence_passes() {
        // Arrange
        let value = evidence();

        // Act / Assert
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn credential_and_source_failures_are_closed() {
        // Arrange
        let mut exposed = evidence();
        exposed.credentials.committed_credential_values = "present".to_owned();
        let mut incompatible = evidence();
        incompatible.source.attempt_source_compatible = false;

        // Act / Assert
        assert_eq!(
            exposed.validate(),
            Err("CFG-07 runtime credential proof is incomplete")
        );
        assert_eq!(
            incompatible.validate(),
            Err("CFG-07 source compatibility is invalid")
        );
    }

    #[test]
    fn unknown_fields_fail_deserialization() {
        // Arrange
        let mut value = serde_json::to_value(evidence()).expect("fixture must serialize");
        value["credential_path"] = serde_json::Value::String("forbidden".to_owned());

        // Act
        let result = serde_json::from_value::<Cfg07Evidence>(value);

        // Assert
        assert!(result.is_err());
    }
}
