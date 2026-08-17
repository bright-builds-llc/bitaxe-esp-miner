use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const RELEASE_RECOVERY_EVIDENCE_SCHEMA: &str = "bitaxe-release-recovery-evidence-v1";

#[derive(Debug, Clone, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ReleaseRecoveryEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u8,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub plan_sha256: String,
    pub detector_admitted: bool,
    pub large_erase_completed: bool,
    pub factory_restore_completed: bool,
    pub wifi_seed_restored: bool,
    pub mineonboot_disabled: bool,
    pub runtime_identity_trusted: bool,
    pub spiffs_ready: bool,
    pub passive_safe_state_confirmed: bool,
    pub cleanup_complete: bool,
    pub recovery_flash_used: bool,
    pub redaction_status: String,
}

impl ReleaseRecoveryEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != RELEASE_RECOVERY_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 1
        {
            return Err("release recovery identity is invalid");
        }
        for commit in [&self.source_commit, &self.reference_commit] {
            if !is_lower_hex(commit, 40) {
                return Err("release recovery commit identity is invalid");
            }
        }
        for digest in [&self.package_manifest_sha256, &self.plan_sha256] {
            if !is_lower_hex(digest, 64) {
                return Err("release recovery digest is invalid");
            }
        }
        if !self.detector_admitted
            || !self.large_erase_completed
            || !self.factory_restore_completed
            || !self.wifi_seed_restored
            || !self.mineonboot_disabled
            || !self.runtime_identity_trusted
            || !self.spiffs_ready
            || !self.passive_safe_state_confirmed
            || !self.cleanup_complete
            || self.recovery_flash_used
            || self.redaction_status != "passed"
        {
            return Err("release recovery evidence is incomplete");
        }
        Ok(())
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && value.bytes().any(|byte| byte != b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> ReleaseRecoveryEvidence {
        ReleaseRecoveryEvidence {
            schema_version: RELEASE_RECOVERY_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 1,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            plan_sha256: "d".repeat(64),
            detector_admitted: true,
            large_erase_completed: true,
            factory_restore_completed: true,
            wifi_seed_restored: true,
            mineonboot_disabled: true,
            runtime_identity_trusted: true,
            spiffs_ready: true,
            passive_safe_state_confirmed: true,
            cleanup_complete: true,
            recovery_flash_used: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_release_recovery_evidence_passes() {
        // Arrange
        let evidence = evidence();

        // Act / Assert
        assert_eq!(evidence.validate(), Ok(()));
    }

    #[test]
    fn recovery_flash_cannot_produce_success_evidence() {
        // Arrange
        let mut evidence = evidence();
        evidence.recovery_flash_used = true;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("release recovery evidence is incomplete"));
    }
}
