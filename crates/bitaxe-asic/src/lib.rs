pub mod bm1366;
pub mod error;

pub use error::Bm1366ProtocolFault;

/// ASIC runtime status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicRuntimeStatus {
    /// ASIC behavior is deferred to Phase 3.
    DeferredUntilPhase3,
}

#[cfg(test)]
mod tests {
    use super::{bm1366, AsicRuntimeStatus, Bm1366ProtocolFault};

    #[test]
    fn asic_runtime_status_defers_active_behavior_until_phase_3() {
        // Arrange
        let status = AsicRuntimeStatus::DeferredUntilPhase3;

        // Act
        let observed = status;

        // Assert
        assert_eq!(observed, AsicRuntimeStatus::DeferredUntilPhase3);
    }

    #[test]
    fn bm1366_contract_exposes_chip_id() {
        // Arrange
        let expected = 0x1366;

        // Act
        let observed = bm1366::BM1366_CHIP_ID;

        // Assert
        assert_eq!(observed, expected);
    }

    #[test]
    fn bm1366_contract_exposes_result_frame_length() {
        // Arrange
        let expected = 11;

        // Act
        let observed = bm1366::BM1366_RESULT_FRAME_LEN;

        // Assert
        assert_eq!(observed, expected);
    }

    #[test]
    fn bm1366_contract_bad_crc_error_mentions_crc() {
        // Arrange
        let fault = Bm1366ProtocolFault::BadCrc;

        // Act
        let rendered = fault.to_string();

        // Assert
        assert!(rendered.contains("bad BM1366 CRC"));
    }
}
