use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity};

pub const HASHRATE_MONITOR_EVIDENCE_SCHEMA: &str = "bitaxe-hashrate-monitor-evidence-v1";

#[derive(Debug, Clone, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct HashrateMonitorSourceEvidence {
    pub plan_sha256: String,
    pub campaign_result_sha256: String,
    pub campaign_network_sha256: String,
    pub source_semantics_current: bool,
    pub reference_semantics_current: bool,
    pub source_path_count: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct HashrateTransportQuorum {
    pub active_sample_count: u64,
    pub positive_coherent_count: u64,
    pub distinct_positive_count: u64,
    pub warm_rolling_window_count: u64,
    pub terminal_zero_confirmed: bool,
}

impl HashrateTransportQuorum {
    fn is_complete(self) -> bool {
        self.active_sample_count >= 2
            && self.positive_coherent_count >= 2
            && self.distinct_positive_count >= 2
            && self.warm_rolling_window_count >= 2
            && self.terminal_zero_confirmed
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct HashrateMonitorQuorum {
    pub monitor_cadence_ms: u64,
    pub asic_count: u8,
    pub domain_count: u8,
    pub required_window_count: u8,
    pub covered_window_count: u8,
    pub http: HashrateTransportQuorum,
    pub websocket: HashrateTransportQuorum,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct HashrateMonitorEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u8,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: HashrateMonitorSourceEvidence,
    pub hashrate: HashrateMonitorQuorum,
    pub detector_admitted: bool,
    pub runtime_identity: String,
    pub campaign_profile: String,
    pub campaign_duration_seconds: u64,
    pub network_status: String,
    pub mining_state: String,
    pub safe_stop_confirmed: bool,
    pub cleanup_complete: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl HashrateMonitorEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != HASHRATE_MONITOR_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 7
            || self.workflow.command != AutomationCommand::CaptureHashrateMonitorEvidence
        {
            return Err("hashrate monitor identity is invalid");
        }
        for commit in [&self.source_commit, &self.reference_commit] {
            if !is_commit(commit) {
                return Err("hashrate monitor commit identity is invalid");
            }
        }
        for digest in [
            &self.package_manifest_sha256,
            &self.workflow.request_sha256,
            &self.source.plan_sha256,
            &self.source.campaign_result_sha256,
            &self.source.campaign_network_sha256,
        ] {
            if !is_sha256(digest) {
                return Err("hashrate monitor digest is invalid");
            }
        }
        if !self.source.source_semantics_current
            || !self.source.reference_semantics_current
            || self.source.source_path_count != 10
        {
            return Err("hashrate monitor source evidence is incomplete");
        }
        if self.hashrate.monitor_cadence_ms != 1_000
            || self.hashrate.asic_count != 1
            || self.hashrate.domain_count != 4
            || self.hashrate.required_window_count != 20
            || self.hashrate.covered_window_count != 20
            || !self.hashrate.http.is_complete()
            || !self.hashrate.websocket.is_complete()
        {
            return Err("hashrate monitor observation quorum is incomplete");
        }
        if !self.detector_admitted
            || self.runtime_identity != "trusted"
            || self.campaign_profile != "conservative"
            || self.campaign_duration_seconds != 600
            || self.network_status != "accepted"
            || self.mining_state != "active_then_paused"
            || !self.safe_stop_confirmed
            || !self.cleanup_complete
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("hashrate monitor campaign evidence is incomplete");
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

    fn evidence() -> HashrateMonitorEvidence {
        let transport = HashrateTransportQuorum {
            active_sample_count: 2,
            positive_coherent_count: 2,
            distinct_positive_count: 2,
            warm_rolling_window_count: 2,
            terminal_zero_confirmed: true,
        };
        HashrateMonitorEvidence {
            schema_version: HASHRATE_MONITOR_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 7,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureHashrateMonitorEvidence,
                request_sha256: "d".repeat(64),
            },
            source: HashrateMonitorSourceEvidence {
                plan_sha256: "e".repeat(64),
                campaign_result_sha256: "f".repeat(64),
                campaign_network_sha256: "1".repeat(64),
                source_semantics_current: true,
                reference_semantics_current: true,
                source_path_count: 10,
            },
            hashrate: HashrateMonitorQuorum {
                monitor_cadence_ms: 1_000,
                asic_count: 1,
                domain_count: 4,
                required_window_count: 20,
                covered_window_count: 20,
                http: transport,
                websocket: transport,
            },
            detector_admitted: true,
            runtime_identity: "trusted".to_owned(),
            campaign_profile: "conservative".to_owned(),
            campaign_duration_seconds: 600,
            network_status: "accepted".to_owned(),
            mining_state: "active_then_paused".to_owned(),
            safe_stop_confirmed: true,
            cleanup_complete: true,
            hardware_rerun_used: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_evidence_passes() {
        // Arrange
        let evidence = evidence();

        // Act / Assert
        assert_eq!(evidence.validate(), Ok(()));
    }

    #[test]
    fn missing_distinct_observations_fail() {
        // Arrange
        let mut evidence = evidence();
        evidence.hashrate.websocket.distinct_positive_count = 1;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(
            result,
            Err("hashrate monitor observation quorum is incomplete")
        );
    }

    #[test]
    fn sha256_length_commit_identity_fails() {
        // Arrange
        let mut evidence = evidence();
        evidence.source_commit = "a".repeat(64);

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("hashrate monitor commit identity is invalid"));
    }

    #[test]
    fn previous_attempt_ordinal_fails() {
        // Arrange
        let mut evidence = evidence();
        evidence.attempt_ordinal = 6;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("hashrate monitor identity is invalid"));
    }
}
