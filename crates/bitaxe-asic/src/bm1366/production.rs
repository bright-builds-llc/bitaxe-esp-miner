//! BM1366 production-mode work and status primitives.
//!
//! Production types deliberately sit beside diagnostic BM1366 work so downstream
//! firmware can interpret typed adapter actions without owning packet layout.

use super::{
    command::Bm1366AdapterAction,
    packet::{FrameBytes, JobFrame, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE},
    work::{Bm1366JobId, Bm1366WorkFields, Bm1366WorkPayload},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionWorkPayload {
    job_id: Bm1366JobId,
    payload: Bm1366WorkPayload,
}

impl ProductionWorkPayload {
    #[must_use]
    pub fn new(job_id: Bm1366JobId, fields: Bm1366WorkFields) -> Self {
        Self {
            job_id,
            payload: Bm1366WorkPayload::new(job_id, fields),
        }
    }

    #[must_use]
    pub const fn job_id(&self) -> Bm1366JobId {
        self.job_id
    }

    #[must_use]
    pub const fn payload(&self) -> &Bm1366WorkPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366ProductionCommand {
    SendProductionWork(ProductionWorkPayload),
    ReadProductionResult,
}

impl Bm1366ProductionCommand {
    #[must_use]
    pub fn adapter_actions(self) -> Vec<Bm1366AdapterAction> {
        match self {
            Self::SendProductionWork(payload) => {
                vec![Bm1366AdapterAction::WriteFrame(production_job_frame(
                    &payload,
                ))]
            }
            Self::ReadProductionResult => Vec::new(),
        }
    }
}

fn production_job_frame(payload: &ProductionWorkPayload) -> FrameBytes {
    JobFrame::new(
        JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE,
        payload.payload().bytes(),
    )
    .expect("fixed production work payload length must fit BM1366 job frame")
    .into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionAsicBlocker {
    PrerequisiteBlocked,
    AsicInitFailed,
    UartFailed,
    ResetFailed,
    ResultTimeout,
    ResultMalformed,
    WorkStale,
    JobUncorrelated,
    DuplicateResult,
    WrongSession,
    /// Stored work target context drifted before submit-intent preparation.
    ///
    /// This is a fail-closed context guard, not proof that a nonce satisfies a
    /// pool target. Full nonce-vs-target validation remains outside Phase 24.
    TargetMismatch,
}

impl ProductionAsicBlocker {
    pub const ALL: [Self; 11] = [
        Self::PrerequisiteBlocked,
        Self::AsicInitFailed,
        Self::UartFailed,
        Self::ResetFailed,
        Self::ResultTimeout,
        Self::ResultMalformed,
        Self::WorkStale,
        Self::JobUncorrelated,
        Self::DuplicateResult,
        Self::WrongSession,
        Self::TargetMismatch,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrerequisiteBlocked => "production_prerequisite_blocked",
            Self::AsicInitFailed => "production_asic_init_failed",
            Self::UartFailed => "production_uart_failed",
            Self::ResetFailed => "production_reset_failed",
            Self::ResultTimeout => "production_result_timeout",
            Self::ResultMalformed => "production_result_malformed",
            Self::WorkStale => "production_work_stale",
            Self::JobUncorrelated => "production_job_uncorrelated",
            Self::DuplicateResult => "production_duplicate_result",
            Self::WrongSession => "production_wrong_session",
            Self::TargetMismatch => "production_target_mismatch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionAsicStatus {
    InitializedForProduction,
    WorkDispatched,
    ResultCorrelated,
    FailClosed { reason: ProductionAsicBlocker },
}

impl ProductionAsicStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitializedForProduction => "production_initialized_for_production",
            Self::WorkDispatched => "production_work_dispatched",
            Self::ResultCorrelated => "production_result_correlated",
            Self::FailClosed { .. } => "production_fail_closed",
        }
    }

    #[must_use]
    pub const fn reason(self) -> Option<ProductionAsicBlocker> {
        match self {
            Self::FailClosed { reason } => Some(reason),
            Self::InitializedForProduction | Self::WorkDispatched | Self::ResultCorrelated => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bm1366::{
        command::Bm1366AdapterAction,
        work::{Bm1366JobId, Bm1366WorkFields},
    };

    use super::{
        Bm1366ProductionCommand, ProductionAsicBlocker, ProductionAsicStatus, ProductionWorkPayload,
    };

    fn sample_fields() -> Bm1366WorkFields {
        Bm1366WorkFields {
            starting_nonce: [0x01, 0x02, 0x03, 0x04],
            nbits: [0x05, 0x06, 0x07, 0x08],
            ntime: [0x09, 0x0a, 0x0b, 0x0c],
            merkle_root: [0x11; 32],
            prev_block_hash: [0x22; 32],
            version: [0x33, 0x34, 0x35, 0x36],
        }
    }

    fn is_lower_snake_case(label: &str) -> bool {
        !label.is_empty()
            && label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    }

    fn contains_sensitive_fragment(label: &str) -> bool {
        [
            "frame",
            "pool",
            "endpoint",
            "credential",
            concat!("pass", "word"),
            concat!("tok", "en"),
        ]
        .iter()
        .any(|fragment| label.contains(fragment))
    }

    #[test]
    fn production_command_send_work_emits_write_frame_action() {
        // Arrange
        let payload = ProductionWorkPayload::new(Bm1366JobId::new(0x28), sample_fields());
        let command = Bm1366ProductionCommand::SendProductionWork(payload);

        // Act
        let actions = command.adapter_actions();

        // Assert
        assert!(matches!(
            actions.as_slice(),
            [Bm1366AdapterAction::WriteFrame(_)]
        ));
    }

    #[test]
    fn production_job_frame_matches_redacted_fixture_layout() {
        // Arrange
        let job_id = Bm1366JobId::new(0x28);
        let payload = ProductionWorkPayload::new(job_id, sample_fields());
        let command = Bm1366ProductionCommand::SendProductionWork(payload);

        // Act
        let actions = command.adapter_actions();

        // Assert
        let [Bm1366AdapterAction::WriteFrame(frame)] = actions.as_slice() else {
            panic!("production work should emit one typed frame");
        };
        assert_eq!(frame.as_ref().len(), 88);
        assert_eq!(frame.as_ref()[4], 0x28);
    }

    #[test]
    fn read_result_frame_uses_bounded_work_timeout() {
        use crate::bm1366::result::{BM1366_RESULT_FRAME_LEN, RESULT_WORK_TIMEOUT_MS};

        // Act
        let action = Bm1366AdapterAction::read_result_frame();

        // Assert
        assert_eq!(
            action,
            Bm1366AdapterAction::ReadExact {
                len: BM1366_RESULT_FRAME_LEN,
                timeout_ms: RESULT_WORK_TIMEOUT_MS,
            }
        );
    }

    #[test]
    fn production_command_read_result_defers_to_executor_bounded_read() {
        // Arrange
        let command = Bm1366ProductionCommand::ReadProductionResult;

        // Act
        let actions = command.adapter_actions();

        // Assert
        assert!(actions.is_empty());
    }

    #[test]
    fn production_blocker_labels_are_lower_snake_case_and_redaction_safe() {
        // Arrange
        let blockers = ProductionAsicBlocker::ALL;

        // Act
        let labels: Vec<&str> = blockers.iter().map(|blocker| blocker.as_str()).collect();

        // Assert
        assert_eq!(
            labels,
            vec![
                "production_prerequisite_blocked",
                "production_asic_init_failed",
                "production_uart_failed",
                "production_reset_failed",
                "production_result_timeout",
                "production_result_malformed",
                "production_work_stale",
                "production_job_uncorrelated",
                "production_duplicate_result",
                "production_wrong_session",
                "production_target_mismatch",
            ]
        );
        for label in labels {
            assert!(is_lower_snake_case(label));
            assert!(!contains_sensitive_fragment(label));
        }
    }

    #[test]
    fn production_status_fail_closed_exposes_blocker_reason() {
        // Arrange
        let status = ProductionAsicStatus::FailClosed {
            reason: ProductionAsicBlocker::ResultTimeout,
        };

        // Act
        let label = status.as_str();
        let maybe_reason = status.reason();

        // Assert
        assert_eq!(label, "production_fail_closed");
        assert_eq!(maybe_reason, Some(ProductionAsicBlocker::ResultTimeout));
    }
}
