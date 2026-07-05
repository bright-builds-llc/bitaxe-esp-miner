//! Pure Stratum v1 mining job bridge to typed BM1366 work fields.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/mining.c`
//! - `reference/esp-miner/components/stratum/include/mining.h`
//! - `crates/bitaxe-asic/src/bm1366/work.rs`
//! - Parity checklist rows `STR-003` and `STR-006`

use bitaxe_asic::bm1366::{
    result::Bm1366NonceResult,
    work::{Bm1366JobId, Bm1366WorkFields},
};

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;
use crate::v1::coinbase::{double_sha256_hex_parts, extranonce_2_generate, hex_32, merkle_root};
use crate::v1::messages::{
    ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumV1ClientMessage, VersionMask,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MiningWork {
    pub stratum_job_id: String,
    pub asic_job_id: Bm1366JobId,
    pub fields: Bm1366WorkFields,
    pub extranonce2: String,
    pub ntime: u32,
    pub maybe_pool_difficulty: Option<PoolDifficulty>,
    pub clean_jobs: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningWorkBuilder {
    notify: MiningNotify,
    extranonce: ExtranonceAssignment,
    maybe_version_mask: Option<VersionMask>,
    maybe_pool_difficulty: Option<PoolDifficulty>,
    extranonce2_value: u64,
}

impl MiningWorkBuilder {
    pub const fn new(notify: MiningNotify, extranonce: ExtranonceAssignment) -> Self {
        Self {
            notify,
            extranonce,
            maybe_version_mask: None,
            maybe_pool_difficulty: None,
            extranonce2_value: 0,
        }
    }

    pub const fn with_version_mask(mut self, mask: VersionMask) -> Self {
        self.maybe_version_mask = Some(mask);
        self
    }

    pub const fn with_pool_difficulty(mut self, difficulty: PoolDifficulty) -> Self {
        self.maybe_pool_difficulty = Some(difficulty);
        self
    }

    pub const fn with_extranonce2_value(mut self, value: u64) -> Self {
        self.extranonce2_value = value;
        self
    }

    pub fn build(self, asic_job_id: Bm1366JobId) -> Result<MiningWork, StratumV1Error> {
        let extranonce2 = extranonce_2_generate(
            self.extranonce2_value,
            usize::from(self.extranonce.extranonce2_len),
        )?;
        let fields = build_work_fields_with_extranonce2(
            &self.notify,
            &self.extranonce,
            &extranonce2,
            asic_job_id,
            self.maybe_version_mask,
        )?;

        Ok(MiningWork {
            stratum_job_id: self.notify.job_id,
            asic_job_id,
            fields,
            extranonce2,
            ntime: self.notify.ntime,
            maybe_pool_difficulty: self.maybe_pool_difficulty,
            clean_jobs: self.notify.clean_jobs,
        })
    }
}

pub fn build_work_fields(
    notify: &MiningNotify,
    extranonce: &ExtranonceAssignment,
    asic_job_id: Bm1366JobId,
    maybe_version_mask: Option<VersionMask>,
) -> Result<Bm1366WorkFields, StratumV1Error> {
    let extranonce2 = extranonce_2_generate(0, usize::from(extranonce.extranonce2_len))?;

    build_work_fields_with_extranonce2(
        notify,
        extranonce,
        &extranonce2,
        asic_job_id,
        maybe_version_mask,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareSubmission {
    pub job_id: String,
    pub extranonce2: String,
    pub ntime: u32,
    pub nonce: u32,
    pub version_bits: u32,
}

impl ShareSubmission {
    pub fn from_nonce_result(
        work: &MiningWork,
        result: Bm1366NonceResult,
    ) -> Result<Self, StratumV1Error> {
        if result.job_id.lookup_key() != work.asic_job_id.lookup_key() {
            return Err(StratumV1Error::InvalidField {
                field: "job_id",
                reason: "nonce result does not match mining work",
            });
        }

        Ok(Self {
            job_id: work.stratum_job_id.clone(),
            extranonce2: work.extranonce2.clone(),
            ntime: work.ntime,
            nonce: result.nonce,
            version_bits: result.version_bits,
        })
    }

    pub fn to_client_message(
        &self,
        id: StratumRequestId,
        username: &str,
    ) -> StratumV1ClientMessage {
        StratumV1ClientMessage::submit_share(
            id,
            username,
            &self.job_id,
            &self.extranonce2,
            self.ntime,
            self.nonce,
            self.version_bits,
        )
    }
}

fn build_work_fields_with_extranonce2(
    notify: &MiningNotify,
    extranonce: &ExtranonceAssignment,
    extranonce2: &str,
    _asic_job_id: Bm1366JobId,
    maybe_version_mask: Option<VersionMask>,
) -> Result<Bm1366WorkFields, StratumV1Error> {
    if let Some(mask) = maybe_version_mask {
        if mask.mask != 0 {
            return Err(StratumV1Error::InvalidField {
                field: "version_mask",
                reason: "version rolling work generation is not implemented",
            });
        }
    }

    let coinbase_hash = double_sha256_hex_parts(&[
        notify.coinbase_1.as_str(),
        extranonce.extranonce1.as_str(),
        extranonce2,
        notify.coinbase_2.as_str(),
    ])?;
    let branches = parse_merkle_branches(&notify.merkle_branches)?;
    let merkle_root = merkle_root(coinbase_hash, &branches);
    let prev_block_hash = hex_32(&notify.prev_block_hash, "prev_block_hash")?;

    Ok(Bm1366WorkFields {
        starting_nonce: 0_u32.to_le_bytes(),
        nbits: notify.nbits.to_le_bytes(),
        ntime: notify.ntime.to_le_bytes(),
        merkle_root: reverse_32bit_words(merkle_root),
        prev_block_hash: reverse_32bit_words(reverse_endianness_per_word(prev_block_hash)),
        version: notify.version.to_le_bytes(),
    })
}

fn parse_merkle_branches(branches: &[String]) -> Result<Vec<[u8; 32]>, StratumV1Error> {
    let mut parsed = Vec::with_capacity(branches.len());
    for branch in branches {
        parsed.push(hex_32(branch, "merkle_branch")?);
    }

    Ok(parsed)
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

fn reverse_endianness_per_word(mut input: [u8; 32]) -> [u8; 32] {
    for word in input.chunks_exact_mut(4) {
        word.reverse();
    }

    input
}

#[cfg(test)]
mod mining_job_tests {
    use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};

    use super::*;
    use crate::jsonrpc::StratumRequestId;
    use crate::v1::messages::{
        ExtranonceAssignment, MiningNotify, StratumV1ClientMessage, VersionMask,
    };

    #[test]
    fn mining_job_builder_produces_bm1366_work_fields() {
        // Arrange
        let notify = sample_notify();
        let extranonce = ExtranonceAssignment {
            extranonce1: "4de05269".to_owned(),
            extranonce2_len: 4,
        };
        let builder =
            MiningWorkBuilder::new(notify, extranonce).with_version_mask(VersionMask { mask: 0 });

        // Act
        let work = builder
            .build(Bm1366JobId::new(0))
            .expect("mining work should build");

        // Assert
        assert_eq!(work.fields.starting_nonce, [0, 0, 0, 0]);
        assert_eq!(work.fields.nbits, 0x1705_ae3a_u32.to_le_bytes());
        assert_eq!(work.fields.ntime, 0x6470_25b5_u32.to_le_bytes());
        assert_eq!(work.fields.merkle_root.len(), 32);
    }

    #[test]
    fn mining_job_builder_rejects_nonzero_version_mask_until_work_generation_is_implemented() {
        // Arrange
        let builder = MiningWorkBuilder::new(
            sample_notify(),
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .with_version_mask(VersionMask { mask: 0x1fff_e000 });

        // Act
        let result = builder.build(Bm1366JobId::new(0x28));

        // Assert
        assert_eq!(
            result,
            Err(StratumV1Error::InvalidField {
                field: "version_mask",
                reason: "version rolling work generation is not implemented",
            })
        );
    }

    #[test]
    fn mining_job_share_submission_serializes_from_nonce_result() {
        // Arrange
        let work = MiningWorkBuilder::new(
            sample_notify(),
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .build(Bm1366JobId::new(0x28))
        .expect("mining work should build");
        let result = Bm1366NonceResult {
            job_id: Bm1366JobId::new(0x28),
            nonce: 0x1234_5678,
            asic_index: 0,
            core_id: 1,
            small_core_id: 0,
            version_bits: 0x0000_2000,
        };

        // Act
        let message = ShareSubmission::from_nonce_result(&work, result)
            .expect("nonce should match active work")
            .to_client_message(StratumRequestId::new(9), "synthetic-user");

        // Assert
        assert!(matches!(
            message,
            StratumV1ClientMessage::SubmitShare {
                id,
                username,
                job_id,
                extranonce2,
                ntime,
                nonce,
                version_bits,
            } if id.raw() == 9
                && username == "synthetic-user"
                && job_id == "job"
                && extranonce2 == "00000000"
                && ntime == 0x6470_25b5
                && nonce == 0x1234_5678
                && version_bits == 0x0000_2000
        ));
    }

    #[test]
    fn share_submission_debug_redacts_raw_context() {
        // Arrange
        let work = MiningWorkBuilder::new(
            sample_notify(),
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .build(Bm1366JobId::new(0x28))
        .expect("mining work should build");
        let submission = ShareSubmission::from_nonce_result(
            &work,
            Bm1366NonceResult {
                job_id: Bm1366JobId::new(0x28),
                nonce: 0x1234_5678,
                asic_index: 0,
                core_id: 1,
                small_core_id: 0,
                version_bits: 0x0000_2000,
            },
        )
        .expect("nonce should match active work");

        // Act
        let rendered = format!("{submission:?}");

        // Assert
        assert!(rendered.contains("ShareSubmission"));
        assert!(rendered.contains("submit_context"));
        assert!(!rendered.contains("job"));
        assert!(!rendered.contains("00000000"));
        assert!(!rendered.contains("12345678"));
        assert!(!rendered.contains("00002000"));
    }

    fn sample_notify() -> MiningNotify {
        MiningNotify {
            job_id: "job".to_owned(),
            prev_block_hash: "00".repeat(32),
            coinbase_1: "0200000001".to_owned(),
            coinbase_2: "ffffffff".to_owned(),
            merkle_branches: vec!["11".repeat(32)],
            version: 0x2000_0004,
            nbits: 0x1705_ae3a,
            ntime: 0x6470_25b5,
            clean_jobs: false,
        }
    }
}
