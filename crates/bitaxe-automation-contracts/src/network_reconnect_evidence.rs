use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, NETWORK_RECONNECT_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkReconnectObservationEvidence {
    pub disconnect_event_count: u64,
    pub fallback_enabled: bool,
    pub first_retry_ordinal: u64,
    pub configured_retry_delay_ms: u64,
    pub observed_retry_delay_ms: u64,
    pub dhcp_recovery_observed: bool,
    pub retry_ordinal_reset: bool,
    pub client_only_restored: bool,
    pub stability_window_ms: u64,
    pub stability_observed: bool,
    pub api_postcondition_matches: bool,
    pub exact_build_identity_matches: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct NetworkReconnectEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_boot_session: bool,
    pub reconnect: NetworkReconnectObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_flash_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl NetworkReconnectEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != NETWORK_RECONNECT_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("network reconnect evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureNetworkReconnectEvidence
        {
            return Err("network reconnect workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("network reconnect source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("network reconnect digest is invalid");
            }
        }
        let reconnect = &self.reconnect;
        if reconnect.disconnect_event_count != 1
            || !reconnect.fallback_enabled
            || reconnect.first_retry_ordinal != 1
            || reconnect.configured_retry_delay_ms != 5_000
            || !(5_000..=15_000).contains(&reconnect.observed_retry_delay_ms)
            || !reconnect.dhcp_recovery_observed
            || !reconnect.retry_ordinal_reset
            || !reconnect.client_only_restored
            || reconnect.stability_window_ms != 15_000
            || !reconnect.stability_observed
            || !reconnect.api_postcondition_matches
            || !reconnect.exact_build_identity_matches
        {
            return Err("network reconnect lifecycle observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_boot_session
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.recovery_flash_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("network reconnect safety, cleanup, or redaction evidence is invalid");
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

    fn evidence() -> NetworkReconnectEvidence {
        NetworkReconnectEvidence {
            schema_version: NETWORK_RECONNECT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: digest('a', 40),
            reference_commit: digest('b', 40),
            package_manifest_sha256: digest('c', 64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureNetworkReconnectEvidence,
                request_sha256: digest('d', 64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_boot_session: true,
            reconnect: NetworkReconnectObservationEvidence {
                disconnect_event_count: 1,
                fallback_enabled: true,
                first_retry_ordinal: 1,
                configured_retry_delay_ms: 5_000,
                observed_retry_delay_ms: 5_001,
                dhcp_recovery_observed: true,
                retry_ordinal_reset: true,
                client_only_restored: true,
                stability_window_ms: 15_000,
                stability_observed: true,
                api_postcondition_matches: true,
                exact_build_identity_matches: true,
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
    fn complete_network_reconnect_evidence_validates() {
        // Arrange / Act / Assert
        assert_eq!(evidence().validate(), Ok(()));
    }

    #[test]
    fn early_retry_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.reconnect.observed_retry_delay_ms = 4_999;

        // Act / Assert
        assert_eq!(
            candidate.validate(),
            Err("network reconnect lifecycle observation is incomplete")
        );
    }
}
