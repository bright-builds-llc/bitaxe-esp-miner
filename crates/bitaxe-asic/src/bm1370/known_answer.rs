//! The Bitcoin genesis block as one BM1370 job: a public known answer for checking the work
//! encoding, the result decoding and the share check end to end (phase F3.2).
//!
//! The fields are the genesis header (version 1, an all-zero previous hash, merkle root
//! `4a5e1e4b…`, time `0x495fab29`, bits `0x1d00ffff`) in the job's wire byte order: the
//! 32-bit words of each hash reversed, the scalars little-endian. Its winning nonce is
//! `0x7c2bac1d`.

use crate::asic::WorkFields;

/// The genesis header as BM1370 work fields, starting at nonce zero.
pub const GENESIS_WORK_FIELDS: WorkFields = WorkFields {
    starting_nonce: [0x00, 0x00, 0x00, 0x00],
    nbits: [0xff, 0xff, 0x00, 0x1d],
    ntime: [0x29, 0xab, 0x5f, 0x49],
    merkle_root: [
        0x4b, 0x1e, 0x5e, 0x4a, 0x3a, 0x9f, 0xb8, 0xaa, 0x88, 0x8a, 0x51, 0x32, 0x7f, 0xc8, 0x1b,
        0xc3, 0x67, 0x76, 0x8f, 0x61, 0x7a, 0xc7, 0x2c, 0x3e, 0x7a, 0x7b, 0x12, 0xb2, 0x3b, 0xa3,
        0xed, 0xfd,
    ],
    prev_block_hash: [0; 32],
    version: [0x01, 0x00, 0x00, 0x00],
};

/// The genesis block's nonce, as the header carries it.
pub const GENESIS_NONCE: u32 = 0x7c2b_ac1d;

#[cfg(test)]
mod tests {
    use super::{GENESIS_NONCE, GENESIS_WORK_FIELDS};
    use crate::{
        asic::{AsicFamily, JobId, ValidJobIds, WorkPayload},
        bm1366::{command::Bm1366Command, crc::crc5},
        bm1370::{
            init::{
                frequency_plan, frequency_ramp_commands, Bm1370InitConfig,
                RESET_FREQUENCY_QUARTER_MHZ,
            },
            plan::mining_ready_actions,
            result::{parse_result_frame, Bm1370ParsedResult},
            work::encode_work_frame,
            BM1370_RESULT_FRAME_LEN,
        },
    };

    /// A BM1370 nonce result for `job_id` on small core zero, with its CRC.
    fn result_frame(job_id: JobId, nonce: u32) -> [u8; BM1370_RESULT_FRAME_LEN] {
        let mut frame = [0; BM1370_RESULT_FRAME_LEN];
        frame[..2].copy_from_slice(&[0xaa, 0x55]);
        frame[2..6].copy_from_slice(&nonce.to_le_bytes());
        frame[7] = job_id.raw() << 1;
        for checksum in 0..32 {
            frame[10] = 0x80 | checksum;
            if crc5(&frame[2..]) == 0 {
                return frame;
            }
        }
        panic!("one of 32 checksums completes the CRC");
    }

    #[test]
    fn the_diagnostic_work_command_encodes_the_same_frame_as_the_bm1370_job() {
        // Arrange
        let job_id = AsicFamily::Bm1370.first_job_id();
        let command =
            Bm1366Command::SendDiagnosticWork(WorkPayload::new(job_id, GENESIS_WORK_FIELDS));
        // Act
        let diagnostic = command.frame_bytes().expect("diagnostic frame");
        let job = encode_work_frame(job_id, GENESIS_WORK_FIELDS).expect("job frame");
        // Assert
        assert_eq!(job_id, JobId::new(24));
        assert_eq!(diagnostic.as_slice(), job.as_slice());
    }

    #[test]
    fn job_id_24_round_trips_through_the_result_parser() {
        // Arrange
        let job_id = AsicFamily::Bm1370.first_job_id();
        let frame = result_frame(job_id, GENESIS_NONCE);
        // Act
        let parsed = parse_result_frame(&frame, &ValidJobIds::single(job_id), 256);
        // Assert
        let Ok(Bm1370ParsedResult::JobNonce(result)) = parsed else {
            panic!("expected a nonce result, got {parsed:?}");
        };
        assert_eq!(
            (result.job_id, result.nonce, result.version_bits),
            (job_id, GENESIS_NONCE, 0)
        );
    }

    #[test]
    fn the_300_mhz_frequency_plan_is_valid() {
        // Arrange
        let target_quarter_mhz = 300 * 4;
        let config = Bm1370InitConfig {
            target_frequency_mhz: 300,
            ..Bm1370InitConfig::gamma_single_chip()
        };
        // Act
        let plan = frequency_plan(target_quarter_mhz);
        let ramp = frequency_ramp_commands(RESET_FREQUENCY_QUARTER_MHZ, target_quarter_mhz);
        let actions = mining_ready_actions(config);
        // Assert
        assert!(plan.is_ok(), "{plan:?}");
        assert!(ramp.is_ok_and(|commands| !commands.is_empty()));
        assert!(actions.is_ok_and(|actions| !actions.is_empty()));
    }
}
