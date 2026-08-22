//! Stratum V2 job conversion and share qualification for BM1366.

use std::fmt;

use bitaxe_asic::bm1366::{
    result::Bm1366NonceResult,
    work::{Bm1366JobId, Bm1366WorkFields},
};

use super::messages::{NewExtendedMiningJob, NewMiningJob, SetNewPrevHash};
use super::StratumV2Error;
use crate::v1::coinbase::{double_sha256, merkle_root};

#[derive(Clone, PartialEq, Eq)]
pub struct V2MiningWork {
    pub channel_id: u32,
    pub job_id: u32,
    pub asic_job_id: Bm1366JobId,
    pub fields: Bm1366WorkFields,
    pub pool_target: [u8; 32],
    pub maybe_extranonce: Option<Vec<u8>>,
    pub clean_jobs: bool,
}

impl fmt::Debug for V2MiningWork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("V2MiningWork")
            .field("channel_id", &self.channel_id)
            .field("asic_job_id", &self.asic_job_id)
            .field("work", &"redacted")
            .finish()
    }
}

impl V2MiningWork {
    pub fn standard(
        job: &NewMiningJob,
        prev_hash: &SetNewPrevHash,
        pool_target: [u8; 32],
        asic_job_id: Bm1366JobId,
    ) -> Result<Self, StratumV2Error> {
        require_same_channel(job.channel_id, prev_hash)?;
        Ok(Self {
            channel_id: job.channel_id,
            job_id: job.job_id,
            asic_job_id,
            fields: Bm1366WorkFields {
                starting_nonce: 0_u32.to_le_bytes(),
                nbits: prev_hash.nbits.to_le_bytes(),
                ntime: job
                    .maybe_min_ntime
                    .unwrap_or(prev_hash.min_ntime)
                    .to_le_bytes(),
                merkle_root: reverse_32bit_words(job.merkle_root),
                prev_block_hash: reverse_32bit_words(prev_hash.prev_hash),
                version: job.version.to_le_bytes(),
            },
            pool_target,
            maybe_extranonce: None,
            clean_jobs: true,
        })
    }

    pub fn extended(
        job: &NewExtendedMiningJob,
        prev_hash: &SetNewPrevHash,
        extranonce_prefix: &[u8],
        extranonce: Vec<u8>,
        pool_target: [u8; 32],
        asic_job_id: Bm1366JobId,
    ) -> Result<Self, StratumV2Error> {
        require_same_channel(job.channel_id, prev_hash)?;
        if extranonce.len() > 32 || extranonce_prefix.len() > 32 {
            return Err(StratumV2Error::InvalidField {
                field: "extranonce",
                reason: "exceeds 32 bytes",
            });
        }
        let mut coinbase = Vec::with_capacity(
            job.coinbase_prefix.len()
                + extranonce_prefix.len()
                + extranonce.len()
                + job.coinbase_suffix.len(),
        );
        coinbase.extend_from_slice(&job.coinbase_prefix);
        coinbase.extend_from_slice(extranonce_prefix);
        coinbase.extend_from_slice(&extranonce);
        coinbase.extend_from_slice(&job.coinbase_suffix);
        let merkle = merkle_root(double_sha256(&coinbase), &job.merkle_path);
        Ok(Self {
            channel_id: job.channel_id,
            job_id: job.job_id,
            asic_job_id,
            fields: Bm1366WorkFields {
                starting_nonce: 0_u32.to_le_bytes(),
                nbits: prev_hash.nbits.to_le_bytes(),
                ntime: job
                    .maybe_min_ntime
                    .unwrap_or(prev_hash.min_ntime)
                    .to_le_bytes(),
                merkle_root: reverse_32bit_words(merkle),
                prev_block_hash: reverse_32bit_words(prev_hash.prev_hash),
                version: job.version.to_le_bytes(),
            },
            pool_target,
            maybe_extranonce: Some(extranonce),
            clean_jobs: true,
        })
    }

    pub fn qualifies(&self, result: Bm1366NonceResult) -> Result<bool, StratumV2Error> {
        if result.job_id.lookup_key() != self.asic_job_id.lookup_key() {
            return Err(StratumV2Error::InvalidField {
                field: "asic_job_id",
                reason: "nonce result does not match work",
            });
        }
        let hash = double_sha256(&self.block_header(result));
        Ok(little_endian_less_or_equal(hash, self.pool_target))
    }

    #[must_use]
    pub fn rolled_version(&self, result: Bm1366NonceResult) -> u32 {
        u32::from_le_bytes(self.fields.version) | result.version_bits
    }

    fn block_header(&self, result: Bm1366NonceResult) -> [u8; 80] {
        let mut header = [0; 80];
        header[0..4].copy_from_slice(&self.rolled_version(result).to_le_bytes());
        header[4..36].copy_from_slice(&reverse_32bit_words(self.fields.prev_block_hash));
        header[36..68].copy_from_slice(&reverse_32bit_words(self.fields.merkle_root));
        header[68..72].copy_from_slice(&self.fields.ntime);
        header[72..76].copy_from_slice(&self.fields.nbits);
        header[76..80].copy_from_slice(&result.nonce.to_le_bytes());
        header
    }
}

#[must_use]
pub fn target_to_pdiff(target: [u8; 32]) -> u32 {
    const TRUE_DIFFICULTY_ONE: f64 =
        26_959_535_291_011_309_493_156_476_344_723_991_336_010_898_738_574_164_086_137_773_096_960.0;
    const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;
    const TWO_POW_128: f64 = 340_282_366_920_938_463_374_607_431_768_211_456.0;
    const TWO_POW_192: f64 =
        6_277_101_735_386_680_763_835_789_423_207_666_416_102_355_444_464_034_512_896.0;
    let limb = |start| {
        u64::from_le_bytes(
            target[start..start + 8]
                .try_into()
                .expect("fixed target limb must have eight bytes"),
        ) as f64
    };
    let value = limb(24) * TWO_POW_192 + limb(16) * TWO_POW_128 + limb(8) * TWO_POW_64 + limb(0);
    if value == 0.0 {
        return u32::MAX;
    }
    (TRUE_DIFFICULTY_ONE / value).clamp(1.0, f64::from(u32::MAX)) as u32
}

fn require_same_channel(channel_id: u32, prev_hash: &SetNewPrevHash) -> Result<(), StratumV2Error> {
    if channel_id != prev_hash.channel_id {
        return Err(StratumV2Error::InvalidField {
            field: "channel_id",
            reason: "job and previous-hash channels differ",
        });
    }
    Ok(())
}

fn little_endian_less_or_equal(left: [u8; 32], right: [u8; 32]) -> bool {
    for index in (0..32).rev() {
        if left[index] < right[index] {
            return true;
        }
        if left[index] > right[index] {
            return false;
        }
    }
    true
}

fn reverse_32bit_words(input: [u8; 32]) -> [u8; 32] {
    let mut output = [0; 32];
    for index in 0..8 {
        let source_start = (7 - index) * 4;
        let target_start = index * 4;
        output[target_start..target_start + 4]
            .copy_from_slice(&input[source_start..source_start + 4]);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_job_maps_reference_byte_order_into_bm1366_fields() {
        // Arrange
        let job = NewMiningJob {
            channel_id: 1,
            job_id: 2,
            maybe_min_ntime: Some(3),
            version: 4,
            merkle_root: [0x11; 32],
        };
        let prev_hash = previous_hash(1, 2);

        // Act
        let work = V2MiningWork::standard(&job, &prev_hash, [0xff; 32], Bm1366JobId::new(8))
            .expect("work");

        // Assert
        assert_eq!(work.fields.ntime, 3_u32.to_le_bytes());
        assert_eq!(work.fields.nbits, 5_u32.to_le_bytes());
        assert_eq!(work.fields.version, 4_u32.to_le_bytes());
        assert_eq!(work.fields.merkle_root, [0x11; 32]);
        assert_eq!(work.fields.prev_block_hash, [0x22; 32]);
    }

    #[test]
    fn extended_job_uses_big_endian_rollable_extranonce_and_bounded_merkle_path() {
        // Arrange
        let job = NewExtendedMiningJob {
            channel_id: 1,
            job_id: 2,
            maybe_min_ntime: None,
            version: 4,
            version_rolling_allowed: true,
            merkle_path: vec![[0x33; 32]],
            coinbase_prefix: vec![1, 2],
            coinbase_suffix: vec![3, 4],
        };
        let prev_hash = previous_hash(1, 2);

        // Act
        let work = V2MiningWork::extended(
            &job,
            &prev_hash,
            &[5],
            vec![0, 1],
            [0xff; 32],
            Bm1366JobId::new(8),
        )
        .expect("work");

        // Assert
        assert_eq!(work.maybe_extranonce, Some(vec![0, 1]));
        assert_eq!(work.fields.ntime, prev_hash.min_ntime.to_le_bytes());
        assert_ne!(work.fields.merkle_root, [0; 32]);
    }

    #[test]
    fn maximum_target_qualifies_a_matching_nonce_and_zero_target_rejects_it() {
        // Arrange
        let job = NewMiningJob {
            channel_id: 1,
            job_id: 2,
            maybe_min_ntime: Some(3),
            version: 4,
            merkle_root: [0x11; 32],
        };
        let prev_hash = previous_hash(1, 2);
        let result = Bm1366NonceResult {
            job_id: Bm1366JobId::new(8),
            nonce: 6,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            version_bits: 0,
        };
        let maximum = V2MiningWork::standard(&job, &prev_hash, [0xff; 32], Bm1366JobId::new(8))
            .expect("maximum work");
        let zero = V2MiningWork::standard(&job, &prev_hash, [0; 32], Bm1366JobId::new(8))
            .expect("zero work");

        // Act
        let maximum_result = maximum.qualifies(result);
        let zero_result = zero.qualifies(result);

        // Assert
        assert_eq!(maximum_result, Ok(true));
        assert_eq!(zero_result, Ok(false));
    }

    fn previous_hash(channel_id: u32, job_id: u32) -> SetNewPrevHash {
        SetNewPrevHash {
            channel_id,
            job_id,
            prev_hash: [0x22; 32],
            min_ntime: 4,
            nbits: 5,
        }
    }
}
