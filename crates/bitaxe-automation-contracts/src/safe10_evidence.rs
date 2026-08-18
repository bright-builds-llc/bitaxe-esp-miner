use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, SAFE10_EVIDENCE_SCHEMA};

#[derive(Debug, Clone, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Safe10SourceEvidence {
    pub plan_sha256: String,
    pub attempt_plan_sha256: String,
    pub attempt_closure_sha256: String,
    pub campaign_result_sha256: String,
    pub campaign_network_sha256: String,
    pub campaign_observations_sha256: String,
    pub campaign_diagnostics_sha256: String,
    pub current_source_inventory_sha256: String,
    pub attempt_source_inventory_sha256: String,
    pub source_semantics_current: bool,
    pub reference_semantics_current: bool,
    pub attempt_source_compatible: bool,
    pub source_path_count: u8,
    pub production_path_count: u8,
    pub reference_path_count: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Safe10PrerequisiteEvidence {
    pub power_watts_required: bool,
    pub bus_voltage_required: bool,
    pub current_required: bool,
    pub chip_temperature_required: bool,
    pub vr_temperature_required: bool,
    pub fan_rpm_required: bool,
    pub power_watts_fresh: bool,
    pub bus_voltage_fresh: bool,
    pub current_fresh: bool,
    pub chip_temperature_fresh: bool,
    pub vr_temperature_fresh: bool,
    pub fan_rpm_fresh: bool,
    pub fresh_observation_count: u8,
    pub safety_fresh: bool,
    pub readiness_unblocked: bool,
    pub session_running_primary: bool,
    pub hardware_ready: bool,
    pub readiness_safety_fresh: bool,
    pub observation_epoch_advanced: bool,
    pub pending_observation_recovered: bool,
    pub active_ms: u64,
    pub required_window_count: u8,
    pub covered_window_count: u8,
    pub work_renewal_valid: bool,
    pub active_state_valid: bool,
    pub network_safety_valid: bool,
    pub watchdog_valid: bool,
    pub qualified_candidate_observed: bool,
    pub accepted_submit_observed: bool,
    pub terminal_http_valid: bool,
    pub terminal_websocket_valid: bool,
    pub terminal_pool_persisted: bool,
    pub final_terminal_consumed: bool,
    pub serial_finished_observed: bool,
}

impl Safe10PrerequisiteEvidence {
    fn complete(self) -> bool {
        self.power_watts_required
            && self.bus_voltage_required
            && self.current_required
            && self.chip_temperature_required
            && !self.vr_temperature_required
            && self.fan_rpm_required
            && self.power_watts_fresh
            && self.bus_voltage_fresh
            && self.current_fresh
            && self.chip_temperature_fresh
            && !self.vr_temperature_fresh
            && self.fan_rpm_fresh
            && self.fresh_observation_count == 5
            && self.safety_fresh
            && self.readiness_unblocked
            && self.session_running_primary
            && self.hardware_ready
            && self.readiness_safety_fresh
            && self.observation_epoch_advanced
            && !self.pending_observation_recovered
            && self.active_ms >= 600_000
            && self.required_window_count == 20
            && self.covered_window_count == 20
            && self.work_renewal_valid
            && self.active_state_valid
            && self.network_safety_valid
            && self.watchdog_valid
            && self.qualified_candidate_observed
            && self.accepted_submit_observed
            && self.terminal_http_valid
            && self.terminal_websocket_valid
            && self.terminal_pool_persisted
            && self.final_terminal_consumed
            && self.serial_finished_observed
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Safe10Evidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u8,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: Safe10SourceEvidence,
    pub prerequisites: Safe10PrerequisiteEvidence,
    pub detector_admitted: bool,
    pub runtime_identity: String,
    pub campaign_stage: String,
    pub campaign_profile: String,
    pub campaign_status: String,
    pub network_status: String,
    pub safe_stop_confirmed: bool,
    pub cleanup_complete: bool,
    pub hardware_rerun_used: bool,
    pub protected_modes_valid: bool,
    pub redaction_status: String,
}

impl Safe10Evidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SAFE10_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 3
            || self.workflow.command != AutomationCommand::ProjectSafe10Evidence
        {
            return Err("SAFE-10 identity is invalid");
        }
        for commit in [
            &self.attempt_source_commit,
            &self.current_source_commit,
            &self.reference_commit,
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("SAFE-10 commit identity is invalid");
            }
        }
        for digest in [
            &self.workflow.request_sha256,
            &self.source.plan_sha256,
            &self.source.attempt_plan_sha256,
            &self.source.attempt_closure_sha256,
            &self.source.campaign_result_sha256,
            &self.source.campaign_network_sha256,
            &self.source.campaign_observations_sha256,
            &self.source.campaign_diagnostics_sha256,
            &self.source.current_source_inventory_sha256,
            &self.source.attempt_source_inventory_sha256,
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("SAFE-10 digest identity is invalid");
            }
        }
        if !self.source.source_semantics_current
            || !self.source.reference_semantics_current
            || !self.source.attempt_source_compatible
            || self.source.source_path_count != 19
            || self.source.production_path_count != 9
            || self.source.reference_path_count != 2
        {
            return Err("SAFE-10 source evidence is incomplete");
        }
        if !self.prerequisites.complete() {
            return Err("SAFE-10 prerequisite quorum is incomplete");
        }
        if !self.detector_admitted
            || self.runtime_identity != "trusted"
            || self.campaign_stage != "live-share"
            || self.campaign_profile != "conservative"
            || self.campaign_status != "accepted"
            || self.network_status != "accepted"
            || !self.safe_stop_confirmed
            || !self.cleanup_complete
            || self.hardware_rerun_used
            || !self.protected_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("SAFE-10 live evidence is incomplete");
        }
        Ok(())
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> Safe10Evidence {
        Safe10Evidence {
            schema_version: SAFE10_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 3,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectSafe10Evidence,
                request_sha256: "d".repeat(64),
            },
            source: Safe10SourceEvidence {
                plan_sha256: "e".repeat(64),
                attempt_plan_sha256: "f".repeat(64),
                attempt_closure_sha256: "1".repeat(64),
                campaign_result_sha256: "2".repeat(64),
                campaign_network_sha256: "3".repeat(64),
                campaign_observations_sha256: "4".repeat(64),
                campaign_diagnostics_sha256: "5".repeat(64),
                current_source_inventory_sha256: "6".repeat(64),
                attempt_source_inventory_sha256: "7".repeat(64),
                source_semantics_current: true,
                reference_semantics_current: true,
                attempt_source_compatible: true,
                source_path_count: 19,
                production_path_count: 9,
                reference_path_count: 2,
            },
            prerequisites: Safe10PrerequisiteEvidence {
                power_watts_required: true,
                bus_voltage_required: true,
                current_required: true,
                chip_temperature_required: true,
                vr_temperature_required: false,
                fan_rpm_required: true,
                power_watts_fresh: true,
                bus_voltage_fresh: true,
                current_fresh: true,
                chip_temperature_fresh: true,
                vr_temperature_fresh: false,
                fan_rpm_fresh: true,
                fresh_observation_count: 5,
                safety_fresh: true,
                readiness_unblocked: true,
                session_running_primary: true,
                hardware_ready: true,
                readiness_safety_fresh: true,
                observation_epoch_advanced: true,
                pending_observation_recovered: false,
                active_ms: 600_000,
                required_window_count: 20,
                covered_window_count: 20,
                work_renewal_valid: true,
                active_state_valid: true,
                network_safety_valid: true,
                watchdog_valid: true,
                qualified_candidate_observed: true,
                accepted_submit_observed: true,
                terminal_http_valid: true,
                terminal_websocket_valid: true,
                terminal_pool_persisted: true,
                final_terminal_consumed: true,
                serial_finished_observed: true,
            },
            detector_admitted: true,
            runtime_identity: "trusted".to_owned(),
            campaign_stage: "live-share".to_owned(),
            campaign_profile: "conservative".to_owned(),
            campaign_status: "accepted".to_owned(),
            network_status: "accepted".to_owned(),
            safe_stop_confirmed: true,
            cleanup_complete: true,
            hardware_rerun_used: false,
            protected_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_live_prerequisite_evidence_passes() {
        // Arrange
        let evidence = evidence();

        // Act / Assert
        assert_eq!(evidence.validate(), Ok(()));
    }

    #[test]
    fn every_required_prerequisite_is_fail_closed() {
        for mutate in [
            |value: &mut Safe10PrerequisiteEvidence| value.power_watts_fresh = false,
            |value: &mut Safe10PrerequisiteEvidence| value.bus_voltage_fresh = false,
            |value: &mut Safe10PrerequisiteEvidence| value.current_fresh = false,
            |value: &mut Safe10PrerequisiteEvidence| value.chip_temperature_fresh = false,
            |value: &mut Safe10PrerequisiteEvidence| value.fan_rpm_fresh = false,
            |value: &mut Safe10PrerequisiteEvidence| value.readiness_unblocked = false,
            |value: &mut Safe10PrerequisiteEvidence| value.hardware_ready = false,
            |value: &mut Safe10PrerequisiteEvidence| value.work_renewal_valid = false,
        ] {
            // Arrange
            let mut evidence = evidence();
            mutate(&mut evidence.prerequisites);

            // Act / Assert
            assert_eq!(
                evidence.validate(),
                Err("SAFE-10 prerequisite quorum is incomplete")
            );
        }
    }

    #[test]
    fn source_drift_and_missing_protected_modes_fail() {
        // Arrange
        let mut source_drift = evidence();
        source_drift.source.attempt_source_compatible = false;
        let mut bad_modes = evidence();
        bad_modes.protected_modes_valid = false;

        // Act / Assert
        assert_eq!(
            source_drift.validate(),
            Err("SAFE-10 source evidence is incomplete")
        );
        assert_eq!(
            bad_modes.validate(),
            Err("SAFE-10 live evidence is incomplete")
        );
    }
}
