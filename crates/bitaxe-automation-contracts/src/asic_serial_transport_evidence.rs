use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_SERIAL_TRANSPORT_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicSerialTransportSourceEvidence {
    pub work_send_projection_sha256: String,
    pub work_send_projection_current_commit: String,
    pub work_send_projection_valid: bool,
    pub result_parsing_projection_sha256: String,
    pub result_parsing_projection_current_commit: String,
    pub result_parsing_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicSerialTransportObservationEvidence {
    pub initial_baud: u64,
    pub tx_pin: i64,
    pub rx_pin: i64,
    pub data_bits: u64,
    pub stop_bits: u64,
    pub parity_none: bool,
    pub flow_control_none: bool,
    pub tx_wait_timeout_ms: u64,
    pub rx_buffer_bytes: u64,
    pub read_chunk_max_bytes: u64,
    pub exact_write_required: bool,
    pub absolute_read_deadline: bool,
    pub partial_reads_accumulated: bool,
    pub empty_timeout_is_idle: bool,
    pub partial_timeout_clears_rx: bool,
    pub live_work_tx_observed: bool,
    pub live_result_rx_observed: bool,
    pub accepted_submit_observed: bool,
    pub uart_module_unchanged: bool,
    pub adapter_module_unchanged: bool,
    pub production_tx_span_compatible: bool,
    pub production_rx_span_compatible: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicSerialTransportEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicSerialTransportSourceEvidence,
    pub serial_transport: AsicSerialTransportObservationEvidence,
    pub package_admitted: bool,
    pub runtime_identity: String,
    pub runtime_attestation_status: String,
    pub campaign_terminal_category: String,
    pub submit_outcome: String,
    pub safety_status: String,
    pub mine_on_boot_disabled: bool,
    pub safe_stop_confirmed: bool,
    pub lease_cleanup_confirmed: bool,
    pub usb_cleanup_ready: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl AsicSerialTransportEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_SERIAL_TRANSPORT_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC serial-transport evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicSerialTransportEvidence
        {
            return Err("ASIC serial-transport workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
            self.source.work_send_projection_current_commit.as_str(),
            self.source
                .result_parsing_projection_current_commit
                .as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("ASIC serial-transport source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.work_send_projection_sha256.as_str(),
            self.source.result_parsing_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC serial-transport digest is invalid");
            }
        }
        if !self.source.work_send_projection_valid || !self.source.result_parsing_projection_valid {
            return Err("ASIC serial-transport source evidence is invalid");
        }

        let transport = &self.serial_transport;
        if transport.initial_baud != 115_200
            || transport.tx_pin != 17
            || transport.rx_pin != 18
            || transport.data_bits != 8
            || transport.stop_bits != 1
            || !transport.parity_none
            || !transport.flow_control_none
            || transport.tx_wait_timeout_ms != 1_000
            || transport.rx_buffer_bytes != 2_048
            || transport.read_chunk_max_bytes != 64
            || !transport.exact_write_required
            || !transport.absolute_read_deadline
            || !transport.partial_reads_accumulated
            || !transport.empty_timeout_is_idle
            || !transport.partial_timeout_clears_rx
            || !transport.live_work_tx_observed
            || !transport.live_result_rx_observed
            || !transport.accepted_submit_observed
            || !transport.uart_module_unchanged
            || !transport.adapter_module_unchanged
            || !transport.production_tx_span_compatible
            || !transport.production_rx_span_compatible
        {
            return Err("ASIC serial-transport observation is incomplete");
        }
        if !self.package_admitted
            || self.runtime_identity != "trusted"
            || self.runtime_attestation_status != "trusted"
            || self.campaign_terminal_category != "submit_response_observed"
            || self.submit_outcome != "accepted"
            || self.safety_status != "fresh"
            || !self.mine_on_boot_disabled
            || !self.safe_stop_confirmed
            || !self.lease_cleanup_confirmed
            || !self.usb_cleanup_ready
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("ASIC serial-transport campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicSerialTransportEvidence {
        AsicSerialTransportEvidence {
            schema_version: ASIC_SERIAL_TRANSPORT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicSerialTransportEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicSerialTransportSourceEvidence {
                work_send_projection_sha256: "e".repeat(64),
                work_send_projection_current_commit: "f".repeat(40),
                work_send_projection_valid: true,
                result_parsing_projection_sha256: "0".repeat(64),
                result_parsing_projection_current_commit: "1".repeat(40),
                result_parsing_projection_valid: true,
            },
            serial_transport: AsicSerialTransportObservationEvidence {
                initial_baud: 115_200,
                tx_pin: 17,
                rx_pin: 18,
                data_bits: 8,
                stop_bits: 1,
                parity_none: true,
                flow_control_none: true,
                tx_wait_timeout_ms: 1_000,
                rx_buffer_bytes: 2_048,
                read_chunk_max_bytes: 64,
                exact_write_required: true,
                absolute_read_deadline: true,
                partial_reads_accumulated: true,
                empty_timeout_is_idle: true,
                partial_timeout_clears_rx: true,
                live_work_tx_observed: true,
                live_result_rx_observed: true,
                accepted_submit_observed: true,
                uart_module_unchanged: true,
                adapter_module_unchanged: true,
                production_tx_span_compatible: true,
                production_rx_span_compatible: true,
            },
            package_admitted: true,
            runtime_identity: "trusted".to_owned(),
            runtime_attestation_status: "trusted".to_owned(),
            campaign_terminal_category: "submit_response_observed".to_owned(),
            submit_outcome: "accepted".to_owned(),
            safety_status: "fresh".to_owned(),
            mine_on_boot_disabled: true,
            safe_stop_confirmed: true,
            lease_cleanup_confirmed: true,
            usb_cleanup_ready: true,
            hardware_rerun_used: false,
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
    fn incomplete_transport_observation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.serial_transport.partial_timeout_clears_rx = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC serial-transport observation is incomplete")
        );
    }

    #[test]
    fn hardware_rerun_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.hardware_rerun_used = true;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC serial-transport campaign or cleanup evidence is invalid")
        );
    }
}
