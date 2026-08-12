use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, NETWORK_SCAN_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkScanObservationEvidence {
    pub record_count: u64,
    pub scan_duration_ms: u64,
    pub records_shape_valid: bool,
    pub signal_values_valid: bool,
    pub auth_modes_valid: bool,
    pub exact_build_identity_matches: bool,
    pub same_boot_session: bool,
    pub before_after_connected: bool,
    pub client_only_preserved: bool,
    pub uptime_monotonic: bool,
    pub address_family: String,
    pub address_kind: String,
    pub address_stable: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkScanEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub scan: NetworkScanObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_flash_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl NetworkScanEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != NETWORK_SCAN_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("network scan evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureNetworkScanEvidence
        {
            return Err("network scan workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("network scan source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("network scan digest is invalid");
            }
        }

        let scan = &self.scan;
        if !(1..=20).contains(&scan.record_count)
            || scan.scan_duration_ms == 0
            || scan.scan_duration_ms > 10_000
            || !scan.records_shape_valid
            || !scan.signal_values_valid
            || !scan.auth_modes_valid
            || !scan.exact_build_identity_matches
            || !scan.same_boot_session
            || !scan.before_after_connected
            || !scan.client_only_preserved
            || !scan.uptime_monotonic
            || scan.address_family != "v6"
            || !matches!(
                scan.address_kind.as_str(),
                "link_local" | "unique_local" | "global"
            )
            || !scan.address_stable
        {
            return Err("network scan observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.recovery_flash_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("network scan safety, cleanup, or redaction evidence is invalid");
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

    fn evidence() -> NetworkScanEvidence {
        NetworkScanEvidence {
            schema_version: NETWORK_SCAN_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureNetworkScanEvidence,
                request_sha256: "d".repeat(64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            scan: NetworkScanObservationEvidence {
                record_count: 3,
                scan_duration_ms: 125,
                records_shape_valid: true,
                signal_values_valid: true,
                auth_modes_valid: true,
                exact_build_identity_matches: true,
                same_boot_session: true,
                before_after_connected: true,
                client_only_preserved: true,
                uptime_monotonic: true,
                address_family: "v6".to_owned(),
                address_kind: "link_local".to_owned(),
                address_stable: true,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            recovery_flash_used: false,
            private_modes_valid: true,
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
    fn record_bounds_and_address_kind_are_closed() {
        // Arrange
        let mut empty = evidence();
        empty.scan.record_count = 0;
        let mut unknown_kind = evidence();
        unknown_kind.scan.address_kind = "multicast".to_owned();

        // Act
        let results = [empty.validate(), unknown_kind.validate()];

        // Assert
        assert_eq!(
            results,
            [
                Err("network scan observation is incomplete"),
                Err("network scan observation is incomplete"),
            ]
        );
    }
}
