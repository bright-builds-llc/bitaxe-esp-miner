//! Pure Stratum V1 share-target validation for BM1366 results.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/mining.c:test_nonce_value`
//! - `reference/esp-miner/components/stratum/utils.c:le256todouble`
//! - `reference/esp-miner/main/tasks/asic_result_task.c`

use bitaxe_asic::bm1366::result::Bm1366NonceResult;

use super::coinbase::double_sha256;
use super::mining::MiningWork;
use crate::error::StratumV1Error;

const TRUE_DIFFICULTY_ONE: f64 =
    26_959_535_291_011_309_493_156_476_344_723_991_336_010_898_738_574_164_086_137_773_096_960.0;
const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;
const TWO_POW_128: f64 = 340_282_366_920_938_463_463_374_607_431_768_211_456.0;
const TWO_POW_192: f64 =
    6_277_101_735_386_680_763_835_789_423_207_666_416_102_355_444_464_034_512_896.0;

pub(super) fn nonce_meets_pool_target(
    work: &MiningWork,
    result: Bm1366NonceResult,
) -> Result<bool, StratumV1Error> {
    let Some(pool_difficulty) = work.maybe_pool_difficulty else {
        return Err(StratumV1Error::InvalidField {
            field: "pool_difficulty",
            reason: "missing for share validation",
        });
    };
    if !pool_difficulty.difficulty.is_finite() || pool_difficulty.difficulty <= 0.0 {
        return Err(StratumV1Error::InvalidField {
            field: "pool_difficulty",
            reason: "must be finite and positive",
        });
    }

    Ok(nonce_difficulty(work, result) >= pool_difficulty.difficulty)
}

fn nonce_difficulty(work: &MiningWork, result: Bm1366NonceResult) -> f64 {
    let header = reconstructed_header(work, result);
    let hash = double_sha256(&header);
    TRUE_DIFFICULTY_ONE / little_endian_256_to_f64(hash)
}

fn reconstructed_header(work: &MiningWork, result: Bm1366NonceResult) -> [u8; 80] {
    let mut header = [0; 80];
    let base_version = u32::from_le_bytes(work.fields.version);
    let rolled_version = base_version | result.version_bits;
    header[0..4].copy_from_slice(&rolled_version.to_le_bytes());
    header[4..36].copy_from_slice(&reverse_32bit_words(work.fields.prev_block_hash));
    header[36..68].copy_from_slice(&reverse_32bit_words(work.fields.merkle_root));
    header[68..72].copy_from_slice(&work.fields.ntime);
    header[72..76].copy_from_slice(&work.fields.nbits);
    header[76..80].copy_from_slice(&result.nonce.to_le_bytes());
    header
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

fn little_endian_256_to_f64(value: [u8; 32]) -> f64 {
    let limb = |start| {
        u64::from_le_bytes(
            value[start..start + 8]
                .try_into()
                .expect("fixed SHA-256 limb must have eight bytes"),
        ) as f64
    };
    limb(24) * TWO_POW_192 + limb(16) * TWO_POW_128 + limb(8) * TWO_POW_64 + limb(0)
}

#[cfg(test)]
mod tests {
    use bitaxe_asic::bm1366::work::{Bm1366JobId, Bm1366WorkFields};

    use super::*;
    use crate::v1::coinbase::hex_32;
    use crate::v1::messages::PoolDifficulty;

    #[test]
    fn nonce_difficulty_matches_first_reference_vector() {
        // Arrange
        let work = reference_work(
            "d02b10fc0d4711eae1a805af50a8a83312a2215e00017f2b0000000000000000",
            "6d0359c451434605c52a5a9ce074340be47c2c63840731f9edf1db3f26b1cdd9",
            0x646f_f1a9,
        );
        let result = reference_result(0x276e_8947);

        // Act
        let difficulty = nonce_difficulty(&work, result);

        // Assert
        assert_eq!(difficulty as u64, 18);
    }

    #[test]
    fn nonce_difficulty_matches_second_reference_vector() {
        // Arrange
        let work = reference_work(
            "0c859545a3498373a57452fac22eb7113df2a465000543520000000000000000",
            "5bdc1968499c3393873edf8e07a1c3a50a97fc3a9d1a376bbf77087dd63778eb",
            0x6470_25b5,
        );
        let result = reference_result(0x0a02_9ed1);

        // Act
        let difficulty = nonce_difficulty(&work, result);

        // Assert
        assert_eq!(difficulty as u64, 683);
    }

    fn reference_work(prev_hash: &str, merkle: &str, ntime: u32) -> MiningWork {
        let prev_block_hash = hex_32(prev_hash, "reference_prev_hash")
            .expect("reference previous hash should decode");
        let merkle_root =
            hex_32(merkle, "reference_merkle").expect("reference merkle should decode");
        MiningWork {
            stratum_job_id: "reference-job".to_owned(),
            asic_job_id: Bm1366JobId::new(0x28),
            fields: Bm1366WorkFields {
                starting_nonce: 0_u32.to_le_bytes(),
                nbits: 0x1705_ae3a_u32.to_le_bytes(),
                ntime: ntime.to_le_bytes(),
                merkle_root: reverse_32bit_words(merkle_root),
                prev_block_hash: reverse_32bit_words(reverse_word_bytes(prev_block_hash)),
                version: 0x2000_0004_u32.to_le_bytes(),
            },
            extranonce2: "00000000".to_owned(),
            ntime,
            maybe_pool_difficulty: Some(PoolDifficulty { difficulty: 1.0 }),
            clean_jobs: false,
            maybe_version_mask: None,
        }
    }

    fn reference_result(nonce: u32) -> Bm1366NonceResult {
        Bm1366NonceResult {
            job_id: Bm1366JobId::new(0x28),
            nonce,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            version_bits: 0,
        }
    }

    fn reverse_word_bytes(mut input: [u8; 32]) -> [u8; 32] {
        for word in input.chunks_exact_mut(4) {
            word.reverse();
        }
        input
    }
}
