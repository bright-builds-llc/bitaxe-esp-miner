use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, DeviceSessionEvidence, WorkflowIdentity, OTAWWW_EVIDENCE_SCHEMA};

/// Redacted observations from one live OTAWWW update, interruption and recovery.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct OtawwwObservationEvidence {
    pub update_request_count: u64,
    pub probe_update_response_complete: bool,
    pub probe_finished_status_retained: bool,
    pub probe_version_reported: bool,
    pub probe_version_txt_digest_matches: bool,
    pub probe_index_html_digest_matches: bool,
    pub interrupted_upload_attempt_count: u64,
    pub interrupted_upload_prefix_bytes: u64,
    pub interruption_protocol_error_retained: bool,
    pub interruption_boot_session_unchanged: bool,
    pub interrupted_assets_unavailable_after_restart: bool,
    pub recovery_page_served_after_interruption: bool,
    pub recovery_update_response_complete: bool,
    pub recovery_version_reported: bool,
    pub recovery_version_txt_digest_matches: bool,
    pub recovery_index_html_digest_matches: bool,
    pub build_identity_unchanged: bool,
    pub hostname_unchanged: bool,
    pub settings_digest_unchanged: bool,
}

/// Closed public projection for `OTA-002` hardware-regression evidence.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct OtawwwEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub www_probe_metadata_sha256: String,
    pub package_www_sha256: String,
    pub probe_www_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub otawww: OtawwwObservationEvidence,
    pub probe_restart_session: DeviceSessionEvidence,
    pub interrupted_restart_session: DeviceSessionEvidence,
    pub recovery_restart_session: DeviceSessionEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_flash_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl OtawwwEvidence {
    /// Accepts only a complete, redacted run of every OTAWWW stage.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != OTAWWW_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("OTAWWW evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureOtawwwEvidence
        {
            return Err("OTAWWW workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("OTAWWW source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.www_probe_metadata_sha256.as_str(),
            self.package_www_sha256.as_str(),
            self.probe_www_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("OTAWWW digest is invalid");
            }
        }
        if self.package_www_sha256 == self.probe_www_sha256 {
            return Err("OTAWWW probe image is not distinct from the package image");
        }
        validate_observation(&self.otawww)?;
        for session in [
            &self.probe_restart_session,
            &self.interrupted_restart_session,
            &self.recovery_restart_session,
        ] {
            validate_ready_session(session)?;
        }
        if !self.detector_admitted
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.recovery_flash_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("OTAWWW safety, restoration, or redaction evidence is invalid");
        }
        Ok(())
    }
}

fn validate_observation(otawww: &OtawwwObservationEvidence) -> Result<(), &'static str> {
    let complete = otawww.update_request_count == 3
        && otawww.probe_update_response_complete
        && otawww.probe_finished_status_retained
        && otawww.probe_version_reported
        && otawww.probe_version_txt_digest_matches
        && otawww.probe_index_html_digest_matches
        && otawww.interrupted_upload_attempt_count == 1
        && otawww.interrupted_upload_prefix_bytes > 0
        && otawww.interruption_protocol_error_retained
        && otawww.interruption_boot_session_unchanged
        && otawww.interrupted_assets_unavailable_after_restart
        && otawww.recovery_page_served_after_interruption
        && otawww.recovery_update_response_complete
        && otawww.recovery_version_reported
        && otawww.recovery_version_txt_digest_matches
        && otawww.recovery_index_html_digest_matches
        && otawww.build_identity_unchanged
        && otawww.hostname_unchanged
        && otawww.settings_digest_unchanged;
    if !complete {
        return Err("OTAWWW update, interruption, or recovery observation is incomplete");
    }
    Ok(())
}

fn validate_ready_session(session: &DeviceSessionEvidence) -> Result<(), &'static str> {
    if session.schema_version != "esp-device-session-v1"
        || session.terminal_category != "ready"
        || session.platform_category != "macos"
        || session.board_category != "205"
        || session.request_attempt_count != 1
        || !matches!(
            session.request_outcome.as_str(),
            "response_received" | "response_missing"
        )
        || !session.same_physical_device
        || !session.stable_enumeration
        || !session.reader_armed
        || !session.pre_restart_serial_delivery
        || !session.post_restart_serial_delivery
        || !session.service_loss_observed
        || !session.trusted_origin_preserved
        || !session.application_recovered
        || !session.build_identity_matches
        || !session.boot_session_changed
        || !session.boot_ordinal_advanced_by_one
        || !session.software_reset_observed
        || !session.postcondition_matches
        || !session.cleanup_complete
    {
        return Err("OTAWWW restart device session is incomplete");
    }
    Ok(())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests;
