pub mod bm1366;
pub mod dispatch;
pub mod error;

pub use dispatch::{AsicDispatch, DeferredAsic, DeferredAsicModel, DeferredAsicReason};
pub use error::Bm1366ProtocolFault;

/// ASIC runtime status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicRuntimeStatus {
    /// ASIC behavior is deferred to Phase 3.
    DeferredUntilPhase3,
}

#[cfg(test)]
mod tests {
    use bitaxe_config::catalog::{board_catalog, ultra_205_catalog_entry, VerificationScope};

    use super::{
        bm1366::{
            self, command::Bm1366AdapterAction, command::Bm1366Command, observation::AsicIndex,
            observation::Bm1366Observation, observation::ChipId, packet::CommandFrame,
            packet::CMD_READ, packet::COMMAND_HEADER_TYPE, packet::GROUP_ALL,
            registers::read_register_payload, result::Bm1366ValidJobIds, work::Bm1366JobId,
            work::Bm1366WorkFields, work::Bm1366WorkPayload, BM1366_CHIP_ID,
        },
        dispatch::{dispatch_catalog_entry, AsicDispatch, DeferredAsicModel},
        AsicRuntimeStatus, Bm1366ProtocolFault,
    };

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

    #[test]
    fn dispatch_ultra_205_bm1366_is_active() {
        // Arrange
        let entry = ultra_205_catalog_entry();

        // Act
        let dispatch = dispatch_catalog_entry(entry);

        // Assert
        assert_eq!(entry.board_version(), "205");
        assert_eq!(entry.family(), "Ultra");
        assert_eq!(entry.asic().model(), "BM1366");
        assert_eq!(entry.asic_count(), 1);
        assert_eq!(
            entry.verification_scope(),
            VerificationScope::ActiveUltra205
        );
        assert_eq!(dispatch, AsicDispatch::ActiveBm1366);
    }

    #[test]
    fn dispatch_non_v1_asic_families_are_deferred_without_hardware_scope() {
        // Arrange
        let deferred_models = [
            ("BM1370", DeferredAsicModel::Bm1370),
            ("BM1368", DeferredAsicModel::Bm1368),
            ("BM1397", DeferredAsicModel::Bm1397),
        ];

        // Act / Assert
        for (model, expected_model) in deferred_models {
            let entry = board_catalog()
                .iter()
                .copied()
                .find(|entry| entry.asic().model() == model)
                .expect("catalog should include deferred ASIC family");
            let dispatch = dispatch_catalog_entry(entry);

            let AsicDispatch::Deferred(deferred) = dispatch else {
                panic!("expected deferred dispatch for {model}");
            };
            assert_eq!(deferred.model(), expected_model);
            assert_eq!(deferred.scope(), VerificationScope::NotHardwareVerified);
        }
    }

    #[test]
    fn bm1366_read_chip_id_command_emits_write_frame_action() {
        // Arrange
        let expected_frame = CommandFrame::new(
            COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ,
            read_register_payload(0x00).as_bytes(),
        )
        .expect("read chip-id frame should encode")
        .into_bytes();

        // Act
        let actions = Bm1366Command::ReadChipId
            .adapter_actions()
            .expect("read chip-id action should encode");

        // Assert
        assert_eq!(
            actions,
            vec![Bm1366AdapterAction::WriteFrame(expected_frame)]
        );
    }

    #[test]
    fn bm1366_command_exposes_diagnostic_work_variant() {
        // Arrange
        let fields = Bm1366WorkFields {
            starting_nonce: [0x01, 0x02, 0x03, 0x04],
            nbits: [0x05, 0x06, 0x07, 0x08],
            ntime: [0x09, 0x0a, 0x0b, 0x0c],
            merkle_root: [0x11; 32],
            prev_block_hash: [0x22; 32],
            version: [0x33, 0x34, 0x35, 0x36],
        };
        let payload = Bm1366WorkPayload::new(Bm1366JobId::new(0x28), fields);

        // Act
        let command = Bm1366Command::SendDiagnosticWork(payload);

        // Assert
        assert!(matches!(command, Bm1366Command::SendDiagnosticWork(_)));
    }

    fn chip_id_response_frame(chip_id: u16, core_count: u8, asic_address: u8) -> Vec<u8> {
        let mut frame = vec![
            0xaa,
            0x55,
            (chip_id >> 8) as u8,
            (chip_id & 0xff) as u8,
            core_count,
            asic_address,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        for crc in 0..32 {
            frame[10] = crc;
            if bm1366::crc::crc5(&frame[2..]) == 0 {
                return frame;
            }
        }

        panic!("chip-id fixture must have a CRC5 residue byte");
    }

    fn transcript_result_frame(body: [u8; 8], is_job_response: bool) -> Vec<u8> {
        let response_bit = if is_job_response { 0x80 } else { 0x00 };
        let mut frame = vec![0xaa, 0x55, 0, 0, 0, 0, 0, 0, 0, 0, response_bit];
        frame[2..10].copy_from_slice(&body);

        for crc in 0..32 {
            frame[10] = response_bit | crc;
            if bm1366::crc::crc5(&frame[2..]) == 0 {
                return frame;
            }
        }

        panic!("result fixture must have a CRC5 residue byte");
    }

    #[test]
    fn transcript_exact_chip_id_emits_read_command_actions_and_observation() {
        // Arrange
        let expected_frame = Bm1366Command::ReadChipId
            .frame_bytes()
            .expect("read chip-id frame should encode");
        let transcript = bm1366::transcript::FakeUartTranscript::with_expected_chips(
            vec![
                bm1366::transcript::FakeUartEvent::ExpectWrite(expected_frame.clone()),
                bm1366::transcript::FakeUartEvent::ReadBytes(chip_id_response_frame(
                    BM1366_CHIP_ID,
                    0x70,
                    0x00,
                )),
            ],
            1,
        );

        // Act
        let outcome = transcript.run_read_chip_id();

        // Assert
        assert_eq!(outcome.commands(), &[Bm1366Command::ReadChipId]);
        assert!(outcome
            .actions()
            .contains(&Bm1366AdapterAction::WriteFrame(expected_frame)));
        assert!(outcome
            .actions()
            .contains(&Bm1366AdapterAction::read_result_frame()));
        assert!(outcome.observations().contains(&Bm1366Observation::ChipId {
            chip_id: ChipId::new(BM1366_CHIP_ID),
            asic_index: AsicIndex::new(0),
        }));
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::Complete
        );
    }

    #[test]
    fn transcript_timeout_returns_protocol_fault_and_fail_closed_status() {
        // Arrange
        let transcript = bm1366::transcript::FakeUartTranscript::new(vec![
            bm1366::transcript::FakeUartEvent::Timeout,
        ]);

        // Act
        let outcome = transcript.run_read_chip_id();

        // Assert
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::FailClosed
        );
        assert!(outcome
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::Timeout { timeout_ms: 1_000 }
            )));
    }

    #[test]
    fn transcript_partial_read_returns_invalid_length_and_clears_rx() {
        // Arrange
        let transcript = bm1366::transcript::FakeUartTranscript::new(vec![
            bm1366::transcript::FakeUartEvent::PartialRead(vec![0xaa, 0x55, 0x13]),
        ]);

        // Act
        let outcome = transcript.run_read_chip_id();

        // Assert
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::FailClosed
        );
        assert!(outcome.actions().contains(&Bm1366AdapterAction::ClearRx));
        assert!(outcome
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::InvalidLength {
                    expected: 11,
                    actual: 3
                }
            )));
    }

    #[test]
    fn transcript_bad_preamble_and_bad_crc_return_faults_without_chip_observation() {
        // Arrange
        let mut bad_preamble = chip_id_response_frame(BM1366_CHIP_ID, 0x70, 0x00);
        bad_preamble[0] = 0x00;
        let mut bad_crc = chip_id_response_frame(BM1366_CHIP_ID, 0x70, 0x00);
        bad_crc[10] ^= 0x01;

        // Act
        let preamble = bm1366::transcript::FakeUartTranscript::new(vec![
            bm1366::transcript::FakeUartEvent::MalformedPreamble(bad_preamble),
        ])
        .run_read_chip_id();
        let crc = bm1366::transcript::FakeUartTranscript::new(vec![
            bm1366::transcript::FakeUartEvent::BadCrc(bad_crc),
        ])
        .run_read_chip_id();

        // Assert
        assert!(preamble.observations().iter().any(|observation| matches!(
            observation,
            Bm1366Observation::ProtocolFault(Bm1366ProtocolFault::BadPreamble { .. })
        )));
        assert!(crc
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::BadCrc
            )));
        assert!(!preamble
            .observations()
            .iter()
            .any(|observation| matches!(observation, Bm1366Observation::ChipId { .. })));
        assert!(!crc
            .observations()
            .iter()
            .any(|observation| matches!(observation, Bm1366Observation::ChipId { .. })));
    }

    #[test]
    fn transcript_chip_count_mismatch_fails_closed() {
        // Arrange
        let expected_frame = Bm1366Command::ReadChipId
            .frame_bytes()
            .expect("read chip-id frame should encode");
        let transcript = bm1366::transcript::FakeUartTranscript::with_expected_chips(
            vec![bm1366::transcript::FakeUartEvent::ExpectWrite(
                expected_frame,
            )],
            1,
        );

        // Act
        let outcome = transcript.run_read_chip_id();

        // Assert
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::FailClosed
        );
        assert!(outcome
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::ChipCountMismatch {
                    expected: 1,
                    actual: 0
                }
            )));
    }

    #[test]
    fn transcript_unknown_register_returns_protocol_fault() {
        // Arrange
        let frame =
            transcript_result_frame([0x01, 0x02, 0x03, 0x04, 0x20, 0xff, 0x00, 0x00], false);
        let transcript = bm1366::transcript::FakeUartTranscript::with_result_context(
            vec![bm1366::transcript::FakeUartEvent::ReadBytes(frame)],
            Bm1366ValidJobIds::empty(),
            16,
        );

        // Act
        let outcome = transcript.run_result_read();

        // Assert
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::FailClosed
        );
        assert!(outcome
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::UnknownRegister { register: 0xff }
            )));
    }

    #[test]
    fn transcript_invalid_job_id_returns_protocol_fault() {
        // Arrange
        let nonce_be = (2_u32 << 25) | (0x20_u32 << 17) | 0x1234;
        let mut body = [0; 8];
        body[0..4].copy_from_slice(&nonce_be.to_be_bytes());
        body[4] = 0x01;
        body[5] = 0x28;
        body[6..8].copy_from_slice(&0x0003_u16.to_be_bytes());
        let transcript = bm1366::transcript::FakeUartTranscript::with_result_context(
            vec![bm1366::transcript::FakeUartEvent::ReadBytes(
                transcript_result_frame(body, true),
            )],
            Bm1366ValidJobIds::empty(),
            16,
        );

        // Act
        let outcome = transcript.run_result_read();

        // Assert
        assert_eq!(
            outcome.status(),
            bm1366::transcript::TranscriptStatus::FailClosed
        );
        assert!(outcome
            .observations()
            .contains(&Bm1366Observation::ProtocolFault(
                Bm1366ProtocolFault::InvalidJobId { job_id: 0x28 }
            )));
    }
}
