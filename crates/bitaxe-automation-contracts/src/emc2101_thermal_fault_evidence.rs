use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, EMC2101_THERMAL_FAULT_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalFaultSourceEvidence {
    pub plan_sha256: String,
    pub prior_thermal_projection_sha256: String,
    pub restore_projection_sha256: String,
    pub intent_sha256: String,
    pub protected_modes_valid: bool,
    pub production_source_current: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalFaultStimulusEvidence {
    pub kind: String,
    pub injected_sample_count: u16,
    pub real_healthy_baseline: bool,
    pub real_reads_during_injection: bool,
    pub typed_invalid_outcomes: bool,
    pub thermal_reading_invalid_fault: bool,
    pub baseline_marker_observed: bool,
    pub fault_marker_observed: bool,
    pub recovery_marker_observed: bool,
    pub marker_order_exact: bool,
    pub intent_consumed_before_use: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalFaultRestorationEvidence {
    pub ordinary_wifi_seed: bool,
    pub exact_package_identity: bool,
    pub http_fresh_sample: bool,
    pub websocket_fresh_sample: bool,
    pub below_throttle_threshold: bool,
    pub fault_absent: bool,
    pub stimulus_not_replayed: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalFaultEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: Emc2101ThermalFaultSourceEvidence,
    pub stimulus: Emc2101ThermalFaultStimulusEvidence,
    pub restoration: Emc2101ThermalFaultRestorationEvidence,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_used: bool,
    pub redaction_status: String,
}

impl Emc2101ThermalFaultEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != EMC2101_THERMAL_FAULT_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 7
        {
            return Err("EMC2101 thermal fault evidence identity is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureEmc2101ThermalFaultEvidence
        {
            return Err("EMC2101 thermal fault workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("EMC2101 thermal fault source identity is invalid");
            }
        }
        for digest in [
            self.app_elf_sha256.as_str(),
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.source.plan_sha256.as_str(),
            self.source.prior_thermal_projection_sha256.as_str(),
            self.source.restore_projection_sha256.as_str(),
            self.source.intent_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("EMC2101 thermal fault evidence digest is invalid");
            }
        }
        if !self.source.protected_modes_valid || !self.source.production_source_current {
            return Err("EMC2101 thermal fault source evidence is incomplete");
        }
        let stimulus = &self.stimulus;
        if stimulus.kind != "emc2101_invalid_sample"
            || stimulus.injected_sample_count != 5
            || !stimulus.real_healthy_baseline
            || !stimulus.real_reads_during_injection
            || !stimulus.typed_invalid_outcomes
            || !stimulus.thermal_reading_invalid_fault
            || !stimulus.baseline_marker_observed
            || !stimulus.fault_marker_observed
            || !stimulus.recovery_marker_observed
            || !stimulus.marker_order_exact
            || !stimulus.intent_consumed_before_use
        {
            return Err("EMC2101 thermal fault stimulus evidence is incomplete");
        }
        let restoration = &self.restoration;
        if !restoration.ordinary_wifi_seed
            || !restoration.exact_package_identity
            || !restoration.http_fresh_sample
            || !restoration.websocket_fresh_sample
            || !restoration.below_throttle_threshold
            || !restoration.fault_absent
            || !restoration.stimulus_not_replayed
        {
            return Err("EMC2101 thermal fault restoration evidence is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || !self.recovery_used
            || self.redaction_status != "passed"
        {
            return Err("EMC2101 thermal fault safety or privacy evidence is invalid");
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

    fn evidence() -> Emc2101ThermalFaultEvidence {
        Emc2101ThermalFaultEvidence {
            schema_version: EMC2101_THERMAL_FAULT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 7,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            app_elf_sha256: "c".repeat(64),
            package_manifest_sha256: "d".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureEmc2101ThermalFaultEvidence,
                request_sha256: "e".repeat(64),
            },
            source: Emc2101ThermalFaultSourceEvidence {
                plan_sha256: "f".repeat(64),
                prior_thermal_projection_sha256: "1".repeat(64),
                restore_projection_sha256: "2".repeat(64),
                intent_sha256: "3".repeat(64),
                protected_modes_valid: true,
                production_source_current: true,
            },
            stimulus: Emc2101ThermalFaultStimulusEvidence {
                kind: "emc2101_invalid_sample".to_owned(),
                injected_sample_count: 5,
                real_healthy_baseline: true,
                real_reads_during_injection: true,
                typed_invalid_outcomes: true,
                thermal_reading_invalid_fault: true,
                baseline_marker_observed: true,
                fault_marker_observed: true,
                recovery_marker_observed: true,
                marker_order_exact: true,
                intent_consumed_before_use: true,
            },
            restoration: Emc2101ThermalFaultRestorationEvidence {
                ordinary_wifi_seed: true,
                exact_package_identity: true,
                http_fresh_sample: true,
                websocket_fresh_sample: true,
                below_throttle_threshold: true,
                fault_absent: true,
                stimulus_not_replayed: true,
            },
            detector_admitted: true,
            boot_observed: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            recovery_used: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_fault_and_restore_projection_is_accepted() {
        // Arrange
        let candidate = evidence();

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn incomplete_marker_sequence_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.stimulus.recovery_marker_observed = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("EMC2101 thermal fault stimulus evidence is incomplete")
        );
    }
}
