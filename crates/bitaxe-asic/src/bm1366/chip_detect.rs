//! Pure BM1366 chip-detect response validation and adapter follow-up actions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/asic_common.c:count_asic_chips`
//! - parity checklist rows `ASIC-005`, `ASIC-006`, and `ASIC-008`

use crate::Bm1366ProtocolFault;

use super::{
    command::Bm1366AdapterAction,
    crc::crc5,
    observation::{AsicIndex, AsicInitStatus, Bm1366Observation, ChipId},
    result::{BM1366_RECEIVE_PREAMBLE, BM1366_RESULT_FRAME_LEN},
    BM1366_CHIP_ID,
};

pub const CHIP_DETECT_RESPONSE_INVALID: &str = "chip_detect_response_invalid";
pub const CHIP_DETECT_ADAPTER_ERROR: &str = "chip_detect_adapter_error";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366AdapterIoFault {
    AdapterError,
    PartialRead,
    UartWrite,
}

pub fn parse_chip_id_response(bytes: &[u8]) -> Result<Bm1366Observation, Bm1366ProtocolFault> {
    if bytes.len() != BM1366_RESULT_FRAME_LEN {
        return Err(Bm1366ProtocolFault::InvalidLength {
            expected: BM1366_RESULT_FRAME_LEN,
            actual: bytes.len(),
        });
    }

    let actual_preamble = u16::from_be_bytes([bytes[0], bytes[1]]);
    if actual_preamble != BM1366_RECEIVE_PREAMBLE {
        return Err(Bm1366ProtocolFault::BadPreamble {
            expected: BM1366_RECEIVE_PREAMBLE,
            actual: actual_preamble,
        });
    }

    if crc5(&bytes[2..]) != 0 {
        return Err(Bm1366ProtocolFault::BadCrc);
    }

    let chip_id = u16::from_be_bytes([bytes[2], bytes[3]]);
    if chip_id != BM1366_CHIP_ID {
        return Err(Bm1366ProtocolFault::PreflightMissing {
            reason: "unexpected_chip_id",
        });
    }

    Ok(Bm1366Observation::ChipId {
        chip_id: ChipId::new(chip_id),
        asic_index: AsicIndex::new(0),
    })
}

pub fn validate_single_chip_detect_response(
    bytes: &[u8],
    expected_chips: u8,
) -> Result<u8, Bm1366ProtocolFault> {
    parse_chip_id_response(bytes)?;

    let detected_chips = 1;
    if detected_chips != expected_chips {
        return Err(Bm1366ProtocolFault::ChipCountMismatch {
            expected: expected_chips,
            actual: detected_chips,
        });
    }

    Ok(detected_chips)
}

#[must_use]
pub fn chip_detect_response_actions(bytes: &[u8], expected_chips: u8) -> Vec<Bm1366AdapterAction> {
    match validate_single_chip_detect_response(bytes, expected_chips) {
        Ok(chips) => vec![Bm1366AdapterAction::PublishStatus(
            AsicInitStatus::ChipDetectedNoMining { chips },
        )],
        Err(_) => fail_closed_actions(CHIP_DETECT_RESPONSE_INVALID),
    }
}

#[must_use]
pub fn adapter_io_failure_actions(_fault: Bm1366AdapterIoFault) -> Vec<Bm1366AdapterAction> {
    fail_closed_actions(CHIP_DETECT_ADAPTER_ERROR)
}

#[must_use]
pub fn fail_closed_actions(reason: &'static str) -> Vec<Bm1366AdapterAction> {
    vec![
        Bm1366AdapterAction::HoldResetLow,
        Bm1366AdapterAction::PublishStatus(AsicInitStatus::FailClosed { reason }),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        adapter_io_failure_actions, chip_detect_response_actions, fail_closed_actions,
        Bm1366AdapterIoFault, CHIP_DETECT_ADAPTER_ERROR, CHIP_DETECT_RESPONSE_INVALID,
    };
    use crate::bm1366::{
        command::Bm1366AdapterAction, crc::crc5, observation::AsicInitStatus, BM1366_CHIP_ID,
        BM1366_RESULT_FRAME_LEN,
    };

    fn chip_id_response_frame(chip_id: u16) -> Vec<u8> {
        let mut frame = vec![
            0xaa,
            0x55,
            (chip_id >> 8) as u8,
            (chip_id & 0xff) as u8,
            0x70,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        for crc in 0..32 {
            frame[10] = crc;
            if crc5(&frame[2..]) == 0 {
                return frame;
            }
        }

        panic!("chip-id fixture must have a CRC5 residue byte");
    }

    #[test]
    fn exact_length_bad_preamble_fails_closed_without_chip_detected_status() {
        // Arrange
        let mut response = chip_id_response_frame(BM1366_CHIP_ID);
        response[0] = 0x00;
        assert_eq!(response.len(), BM1366_RESULT_FRAME_LEN);

        // Act
        let actions = chip_detect_response_actions(&response, 1);

        // Assert
        assert_eq!(actions, fail_closed_actions(CHIP_DETECT_RESPONSE_INVALID));
        assert!(!actions.iter().any(|action| matches!(
            action,
            Bm1366AdapterAction::PublishStatus(AsicInitStatus::ChipDetectedNoMining { .. })
        )));
    }

    #[test]
    fn valid_chip_id_response_publishes_detected_chip_count() {
        // Arrange
        let response = chip_id_response_frame(BM1366_CHIP_ID);

        // Act
        let actions = chip_detect_response_actions(&response, 1);

        // Assert
        assert_eq!(
            actions,
            vec![Bm1366AdapterAction::PublishStatus(
                AsicInitStatus::ChipDetectedNoMining { chips: 1 }
            )]
        );
    }

    #[test]
    fn adapter_io_failures_hold_reset_low_and_publish_fail_closed() {
        // Arrange
        let faults = [
            Bm1366AdapterIoFault::PartialRead,
            Bm1366AdapterIoFault::UartWrite,
        ];

        // Act
        let observed: Vec<_> = faults.into_iter().map(adapter_io_failure_actions).collect();

        // Assert
        for actions in observed {
            assert_eq!(actions, fail_closed_actions(CHIP_DETECT_ADAPTER_ERROR));
        }
    }
}
