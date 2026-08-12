use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, PROVISIONING_NETWORK_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProvisioningNetworkObservationEvidence {
    pub host_platform_macos: bool,
    pub single_wifi_interface: bool,
    pub initial_wifi_powered_on: bool,
    pub initial_wifi_unassociated: bool,
    pub baseline_candidate_count: u64,
    pub configuration_candidate_count: u64,
    pub association_observed: bool,
    pub dhcp_observed: bool,
    pub dns_query_count: u64,
    pub wildcard_dns_answer_matches_gateway: bool,
    pub dns_ttl_seconds: u64,
    pub captive_redirect_observed: bool,
    pub captive_redirect_root: bool,
    pub captive_redirect_body_matches: bool,
    pub api_postcondition_matches: bool,
    pub exact_build_identity_matches: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProvisioningNetworkEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub provisioning: ProvisioningNetworkObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub host_network_restored: bool,
    pub device_recovery_complete: bool,
    pub cleanup_complete: bool,
    pub recovery_flash_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl ProvisioningNetworkEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != PROVISIONING_NETWORK_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("provisioning network evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureProvisioningNetworkEvidence
        {
            return Err("provisioning network workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("provisioning network source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("provisioning network digest is invalid");
            }
        }
        let observation = &self.provisioning;
        if !observation.host_platform_macos
            || !observation.single_wifi_interface
            || !observation.initial_wifi_powered_on
            || !observation.initial_wifi_unassociated
            || observation.baseline_candidate_count != 0
            || observation.configuration_candidate_count != 1
            || !observation.association_observed
            || !observation.dhcp_observed
            || observation.dns_query_count != 1
            || !observation.wildcard_dns_answer_matches_gateway
            || observation.dns_ttl_seconds != 300
            || !observation.captive_redirect_observed
            || !observation.captive_redirect_root
            || !observation.captive_redirect_body_matches
            || !observation.api_postcondition_matches
            || !observation.exact_build_identity_matches
        {
            return Err("provisioning network observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.host_network_restored
            || !self.device_recovery_complete
            || !self.cleanup_complete
            || !self.recovery_flash_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("provisioning network recovery, cleanup, or redaction evidence is invalid");
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

    fn digest(character: char, length: usize) -> String {
        std::iter::repeat_n(character, length).collect()
    }

    fn evidence() -> ProvisioningNetworkEvidence {
        ProvisioningNetworkEvidence {
            schema_version: PROVISIONING_NETWORK_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: digest('a', 40),
            reference_commit: digest('b', 40),
            package_manifest_sha256: digest('c', 64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureProvisioningNetworkEvidence,
                request_sha256: digest('d', 64),
            },
            detector_admitted: true,
            boot_observed: true,
            provisioning: ProvisioningNetworkObservationEvidence {
                host_platform_macos: true,
                single_wifi_interface: true,
                initial_wifi_powered_on: true,
                initial_wifi_unassociated: true,
                baseline_candidate_count: 0,
                configuration_candidate_count: 1,
                association_observed: true,
                dhcp_observed: true,
                dns_query_count: 1,
                wildcard_dns_answer_matches_gateway: true,
                dns_ttl_seconds: 300,
                captive_redirect_observed: true,
                captive_redirect_root: true,
                captive_redirect_body_matches: true,
                api_postcondition_matches: true,
                exact_build_identity_matches: true,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            host_network_restored: true,
            device_recovery_complete: true,
            cleanup_complete: true,
            recovery_flash_used: true,
            private_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_provisioning_network_evidence_validates() {
        // Arrange / Act / Assert
        assert_eq!(evidence().validate(), Ok(()));
    }

    #[test]
    fn nonunique_configuration_candidate_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.provisioning.configuration_candidate_count = 2;

        // Act / Assert
        assert_eq!(
            candidate.validate(),
            Err("provisioning network observation is incomplete")
        );
    }
}
