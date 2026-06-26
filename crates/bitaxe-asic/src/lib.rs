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

    #[test]
    fn bm1366_crc_read_register_zero_matches_reference() {
        // Arrange
        let command_body = [0x52, 0x05, 0x00, 0x00];

        // Act
        let observed = bm1366::crc::crc5(&command_body);

        // Assert
        assert_eq!(observed, 0x0a);
    }

    #[test]
    fn bm1366_packet_command_frame_matches_read_register_fixture() {
        // Arrange
        let data = [0x00, 0x00];

        // Act
        let frame = bm1366::packet::CommandFrame::new(0x52, &data)
            .expect("read-register command frame should be valid");

        // Assert
        assert_eq!(frame.bytes(), [0x55, 0xaa, 0x52, 0x05, 0x00, 0x00, 0x0a]);
    }

    #[test]
    fn bm1366_packet_job_frame_uses_crc16_false() {
        // Arrange
        let header = bm1366::packet::JOB_HEADER_TYPE
            | bm1366::packet::GROUP_SINGLE
            | bm1366::packet::CMD_WRITE;
        let data = [0x01, 0x02, 0x03];

        // Act
        let frame =
            bm1366::packet::JobFrame::new(header, &data).expect("job frame should be valid");
        let bytes = frame.bytes();
        let expected_crc = bm1366::crc::crc16_false(&bytes[2..bytes.len() - 2]);

        // Assert
        assert_eq!(bytes.len(), data.len() + 6);
        assert_eq!(bytes[3], (data.len() + 4) as u8);
        assert_eq!(
            u16::from_be_bytes([bytes[bytes.len() - 2], bytes[bytes.len() - 1]]),
            expected_crc
        );
    }

    #[test]
    fn bm1366_register_maps_reference_result_registers() {
        // Arrange
        let known_registers = [
            (0x4c, bm1366::registers::Bm1366Register::ErrorCount),
            (0x88, bm1366::registers::Bm1366Register::Domain0Count),
            (0x89, bm1366::registers::Bm1366Register::Domain1Count),
            (0x8a, bm1366::registers::Bm1366Register::Domain2Count),
            (0x8b, bm1366::registers::Bm1366Register::Domain3Count),
            (0x8c, bm1366::registers::Bm1366Register::TotalCount),
        ];

        // Act
        let observed: Vec<_> = known_registers
            .iter()
            .map(|(raw, _)| bm1366::registers::Bm1366Register::try_from(*raw))
            .collect();
        let unknown = bm1366::registers::Bm1366Register::try_from(0xff);

        // Assert
        for ((_, expected), observed_register) in known_registers.iter().zip(observed) {
            assert_eq!(observed_register, Ok(*expected));
        }
        assert_eq!(
            unknown,
            Err(Bm1366ProtocolFault::UnknownRegister { register: 0xff })
        );
    }
}
