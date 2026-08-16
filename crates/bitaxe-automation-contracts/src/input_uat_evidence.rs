use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::INPUT_UAT_EVIDENCE_SCHEMA;

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputUatObservationEvidence {
    pub gpio: u8,
    pub active_low: bool,
    pub pull_up_enabled: bool,
    pub sampling_ms: u64,
    pub debounce_ms: u64,
    pub long_press_ms: u64,
    pub checkpoint_published_before_input: bool,
    pub physical_short_click_count: u8,
    pub screen_advance_observed: bool,
    pub long_press_observed: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputUatEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub package_manifest_sha256: String,
    pub plan_sha256: String,
    pub input: InputUatObservationEvidence,
    pub exact_package_flash_completed: bool,
    pub runtime_attestation_trusted: bool,
    pub source_semantics_admitted: bool,
    pub reference_semantics_admitted: bool,
    pub usb_admission_confirmed: bool,
    pub cleanup_complete: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub serial_transcript_retained: bool,
    pub redaction_status: String,
}

impl InputUatEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != INPUT_UAT_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("input UAT evidence schema or board is invalid");
        }
        for commit in [&self.source_commit, &self.reference_commit] {
            if !is_lower_hex(commit, 40) {
                return Err("input UAT source identity is invalid");
            }
        }
        for digest in [
            &self.app_elf_sha256,
            &self.package_manifest_sha256,
            &self.plan_sha256,
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("input UAT digest is invalid");
            }
        }
        let input = &self.input;
        if input.gpio != 0
            || !input.active_low
            || !input.pull_up_enabled
            || input.sampling_ms != 10
            || input.debounce_ms != 30
            || input.long_press_ms != 2_000
        {
            return Err("input UAT fixed input contract is invalid");
        }
        if !input.checkpoint_published_before_input
            || input.physical_short_click_count != 1
            || !input.screen_advance_observed
            || input.long_press_observed
        {
            return Err("input UAT physical observation is incomplete");
        }
        if !self.exact_package_flash_completed
            || !self.runtime_attestation_trusted
            || !self.source_semantics_admitted
            || !self.reference_semantics_admitted
            || !self.usb_admission_confirmed
            || !self.cleanup_complete
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || self.serial_transcript_retained
            || self.redaction_status != "passed"
        {
            return Err("input UAT trust, cleanup, or redaction evidence is invalid");
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

    fn evidence() -> InputUatEvidence {
        InputUatEvidence {
            schema_version: INPUT_UAT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            app_elf_sha256: "c".repeat(64),
            package_manifest_sha256: "d".repeat(64),
            plan_sha256: "e".repeat(64),
            input: InputUatObservationEvidence {
                gpio: 0,
                active_low: true,
                pull_up_enabled: true,
                sampling_ms: 10,
                debounce_ms: 30,
                long_press_ms: 2_000,
                checkpoint_published_before_input: true,
                physical_short_click_count: 1,
                screen_advance_observed: true,
                long_press_observed: false,
            },
            exact_package_flash_completed: true,
            runtime_attestation_trusted: true,
            source_semantics_admitted: true,
            reference_semantics_admitted: true,
            usb_admission_confirmed: true,
            cleanup_complete: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            serial_transcript_retained: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_input_uat_projection_is_accepted() {
        // Arrange
        let candidate = evidence();

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn long_press_observation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.input.long_press_observed = true;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("input UAT physical observation is incomplete"));
    }
}
