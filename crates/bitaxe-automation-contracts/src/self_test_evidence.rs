use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity};

pub const SELF_TEST_EVIDENCE_SCHEMA: &str = "bitaxe-self-test-evidence-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelfTestFailureEvidence {
    pub stable_load_ms: u64,
    pub planned_evaluation_failure: bool,
    pub safe_stop_complete: bool,
    pub failed_state_observed: bool,
    pub cancel_checkpoint_safe: bool,
    pub physical_long_press_observed: bool,
    pub cancellation_receipt_observed: bool,
    pub cancellation_restart_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelfTestPassEvidence {
    pub frequency_mhz: u16,
    pub core_voltage_mv: u16,
    pub difficulty: u32,
    pub warmup_celsius: u16,
    pub target_celsius: u16,
    pub maximum_celsius: u16,
    pub measurement_ms: u64,
    pub total_hashrate_passed: bool,
    pub domain_count: u8,
    pub domain_evaluation_passed: bool,
    pub electrical_checks_passed: bool,
    pub fan_check_passed: bool,
    pub watchdog_advanced: bool,
    pub safe_stop_complete: bool,
    pub pass_receipt_observed: bool,
    pub automatic_restart_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelfTestRestorationEvidence {
    pub settings_snapshot_captured_before_write: bool,
    pub local_credentials_used_in_memory: bool,
    pub settings_restored: bool,
    pub mine_on_boot_disabled: bool,
    pub production_mining_never_started: bool,
    pub pool_traffic_absent: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelfTestEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub package_manifest_sha256: String,
    pub plan_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub psram_available: bool,
    pub failure: SelfTestFailureEvidence,
    pub pass: SelfTestPassEvidence,
    pub restoration: SelfTestRestorationEvidence,
    pub cleanup_complete: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl SelfTestEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SELF_TEST_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 5
            || self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::SelfTestCampaign
            || !self.detector_admitted
            || !self.psram_available
            || self.failure.stable_load_ms < 5_000
            || !self.failure.planned_evaluation_failure
            || !self.failure.safe_stop_complete
            || !self.failure.failed_state_observed
            || !self.failure.cancel_checkpoint_safe
            || !self.failure.physical_long_press_observed
            || !self.failure.cancellation_receipt_observed
            || !self.failure.cancellation_restart_observed
            || self.pass.frequency_mhz != 485
            || self.pass.core_voltage_mv != 1_200
            || self.pass.difficulty != 16
            || self.pass.warmup_celsius != 55
            || self.pass.target_celsius != 65
            || self.pass.maximum_celsius != 70
            || self.pass.measurement_ms < 30_000
            || !self.pass.total_hashrate_passed
            || self.pass.domain_count != 4
            || !self.pass.domain_evaluation_passed
            || !self.pass.electrical_checks_passed
            || !self.pass.fan_check_passed
            || !self.pass.watchdog_advanced
            || !self.pass.safe_stop_complete
            || !self.pass.pass_receipt_observed
            || !self.pass.automatic_restart_observed
            || !self.restoration.settings_snapshot_captured_before_write
            || !self.restoration.local_credentials_used_in_memory
            || !self.restoration.settings_restored
            || !self.restoration.mine_on_boot_disabled
            || !self.restoration.production_mining_never_started
            || !self.restoration.pool_traffic_absent
            || !self.cleanup_complete
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("SELF-001 evidence quorum is incomplete");
        }
        for (value, expected_length) in [
            (&self.source_commit, 40),
            (&self.reference_commit, 40),
            (&self.app_elf_sha256, 64),
            (&self.package_manifest_sha256, 64),
            (&self.plan_sha256, 64),
            (&self.workflow.request_sha256, 64),
        ] {
            if value.len() != expected_length
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err("SELF-001 evidence identity is invalid");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> SelfTestEvidence {
        SelfTestEvidence {
            schema_version: SELF_TEST_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 5,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            app_elf_sha256: "c".repeat(64),
            package_manifest_sha256: "d".repeat(64),
            plan_sha256: "e".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::SelfTestCampaign,
                request_sha256: "f".repeat(64),
            },
            detector_admitted: true,
            psram_available: true,
            failure: SelfTestFailureEvidence {
                stable_load_ms: 5_000,
                planned_evaluation_failure: true,
                safe_stop_complete: true,
                failed_state_observed: true,
                cancel_checkpoint_safe: true,
                physical_long_press_observed: true,
                cancellation_receipt_observed: true,
                cancellation_restart_observed: true,
            },
            pass: SelfTestPassEvidence {
                frequency_mhz: 485,
                core_voltage_mv: 1_200,
                difficulty: 16,
                warmup_celsius: 55,
                target_celsius: 65,
                maximum_celsius: 70,
                measurement_ms: 30_000,
                total_hashrate_passed: true,
                domain_count: 4,
                domain_evaluation_passed: true,
                electrical_checks_passed: true,
                fan_check_passed: true,
                watchdog_advanced: true,
                safe_stop_complete: true,
                pass_receipt_observed: true,
                automatic_restart_observed: true,
            },
            restoration: SelfTestRestorationEvidence {
                settings_snapshot_captured_before_write: true,
                local_credentials_used_in_memory: true,
                settings_restored: true,
                mine_on_boot_disabled: true,
                production_mining_never_started: true,
                pool_traffic_absent: true,
            },
            cleanup_complete: true,
            private_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_evidence_passes_and_each_phase_is_mandatory() {
        // Arrange
        let complete = evidence();
        let mut missing_cancel = evidence();
        missing_cancel.failure.physical_long_press_observed = false;
        let mut missing_restoration = evidence();
        missing_restoration.restoration.settings_restored = false;

        // Act / Assert
        assert_eq!(complete.validate(), Ok(()));
        assert!(missing_cancel.validate().is_err());
        assert!(missing_restoration.validate().is_err());
    }
}
